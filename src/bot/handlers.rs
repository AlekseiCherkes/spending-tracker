use std::sync::Arc;

use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardMarkup, ParseMode};

use crate::dal::Db;
use crate::domain::{DraftStore, EditState, SpendingDraft};

use super::keyboards;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

fn format_summary(draft: &SpendingDraft, db: &Db) -> String {
    let category = db
        .get_category_by_id(draft.category_id)
        .map(|c| c.name)
        .unwrap_or_else(|| "?".to_string());
    let account = db
        .get_account_by_id(draft.account_id)
        .map(|a| {
            let currency = db
                .get_currency_by_id(a.currency_id)
                .map(|c| c.currency_code)
                .unwrap_or_default();
            format!("{} ({})", a.name, currency)
        })
        .unwrap_or_else(|| "?".to_string());
    let currency_code = db
        .get_account_by_id(draft.account_id)
        .and_then(|a| db.get_currency_by_id(a.currency_id))
        .map(|c| c.currency_code)
        .unwrap_or_default();
    let notes = draft.notes.as_deref().unwrap_or("\u{2014}");

    format!(
        "Сумма: {:.2} {}\nКатегория: {}\nСчёт: {}\nЗаметка: {}",
        draft.amount, currency_code, category, account, notes
    )
}

pub async fn handle_message(
    bot: Bot,
    msg: Message,
    db: Arc<Db>,
    drafts: Arc<DraftStore>,
) -> HandlerResult {
    let text = match msg.text() {
        Some(t) => t.trim().to_string(),
        None => return Ok(()),
    };

    let telegram_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);

    // Whitelist check
    let user = match db.get_user_by_telegram_id(telegram_id) {
        Some(u) => u,
        None => {
            log::warn!("Unknown user telegram_id={}", telegram_id);
            return Ok(());
        }
    };

    // Check if user is entering a note
    if let Some(draft) = drafts.get(telegram_id) {
        if draft.edit_state == EditState::EnteringNote {
            drafts.update_note(telegram_id, text);
            let updated = drafts.get(telegram_id).unwrap();
            let summary = format_summary(&updated, &db);
            bot.send_message(msg.chat.id, summary)
                .parse_mode(ParseMode::Html)
                .reply_markup(keyboards::summary_keyboard())
                .await?;
            return Ok(());
        }
    }

    // Try to parse as number
    let amount: f64 = match text.replace(',', ".").parse() {
        Ok(v) if v > 0.0 => v,
        _ => {
            bot.send_message(
                msg.chat.id,
                "Отправьте сумму (число), чтобы начать запись расхода.",
            )
            .await?;
            return Ok(());
        }
    };

    // Create new draft with defaults
    let default_account_id = user
        .default_account_id
        .unwrap_or_else(|| db.get_all_accounts().first().map(|a| a.id).unwrap_or(1));
    let default_category_id = db.get_all_categories().first().map(|c| c.id).unwrap_or(1);

    let draft = SpendingDraft {
        amount,
        category_id: default_category_id,
        account_id: default_account_id,
        reporter_user_id: user.id,
        notes: None,
        edit_state: EditState::Summary,
    };

    drafts.set(telegram_id, draft.clone());

    let summary = format_summary(&draft, &db);
    bot.send_message(msg.chat.id, summary)
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboards::summary_keyboard())
        .await?;

    Ok(())
}

pub async fn handle_callback(
    bot: Bot,
    q: CallbackQuery,
    db: Arc<Db>,
    drafts: Arc<DraftStore>,
) -> HandlerResult {
    bot.answer_callback_query(q.id.clone()).await?;

    let data = match q.data.as_deref() {
        Some(d) => d,
        None => return Ok(()),
    };

    let telegram_id = q.from.id.0 as i64;
    let (chat_id, msg_id) = match q.message {
        Some(ref m) => (m.chat().id, m.id()),
        None => return Ok(()),
    };

    // Remove inline keyboard from the message that was clicked
    bot.edit_message_reply_markup(chat_id, msg_id)
        .reply_markup(InlineKeyboardMarkup::default())
        .await
        .ok();

    // Whitelist check
    if db.get_user_by_telegram_id(telegram_id).is_none() {
        return Ok(());
    }

    if drafts.get(telegram_id).is_none() {
        bot.send_message(chat_id, "Нет активного черновика. Отправьте сумму.")
            .await?;
        return Ok(());
    }

    match data {
        "edit_cat" => {
            drafts.update_state(telegram_id, EditState::ChoosingCategory);
            let categories = db.get_all_categories();
            bot.send_message(chat_id, "Выберите категорию:")
                .reply_markup(keyboards::category_keyboard(&categories))
                .await?;
        }
        "edit_acc" => {
            drafts.update_state(telegram_id, EditState::ChoosingAccount);
            let accounts = db.get_all_accounts();
            bot.send_message(chat_id, "Выберите счёт:")
                .reply_markup(keyboards::account_keyboard(&accounts))
                .await?;
        }
        "edit_note" => {
            drafts.update_state(telegram_id, EditState::EnteringNote);
            bot.send_message(chat_id, "Введите заметку:").await?;
        }
        "save" => {
            let draft = drafts.remove(telegram_id).unwrap();
            db.insert_spending(
                draft.account_id,
                draft.amount,
                draft.category_id,
                draft.reporter_user_id,
                draft.notes.as_deref(),
            )?;

            let currency_code = db
                .get_account_by_id(draft.account_id)
                .and_then(|a| db.get_currency_by_id(a.currency_id))
                .map(|c| c.currency_code)
                .unwrap_or_default();

            bot.send_message(
                chat_id,
                format!(
                    "\u{2705} Расход {:.2} {} сохранён!",
                    draft.amount, currency_code
                ),
            )
            .await?;
        }
        _ if data.starts_with("cat:") => {
            if let Ok(cat_id) = data[4..].parse::<i64>() {
                drafts.update_category(telegram_id, cat_id);
                let updated = drafts.get(telegram_id).unwrap();
                let summary = format_summary(&updated, &db);
                bot.send_message(chat_id, summary)
                    .parse_mode(ParseMode::Html)
                    .reply_markup(keyboards::summary_keyboard())
                    .await?;
            }
        }
        _ if data.starts_with("acc:") => {
            if let Ok(acc_id) = data[4..].parse::<i64>() {
                drafts.update_account(telegram_id, acc_id);
                let updated = drafts.get(telegram_id).unwrap();
                let summary = format_summary(&updated, &db);
                bot.send_message(chat_id, summary)
                    .parse_mode(ParseMode::Html)
                    .reply_markup(keyboards::summary_keyboard())
                    .await?;
            }
        }
        _ => {}
    }

    Ok(())
}
