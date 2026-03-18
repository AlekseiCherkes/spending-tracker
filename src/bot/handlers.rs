use std::sync::Arc;

use teloxide::prelude::*;
use teloxide::types::ParseMode;

use crate::dal::Db;
use crate::domain::{DraftStore, EditState, SpendingDraft};

use super::keyboards;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

struct DraftDisplay {
    summary_text: String,
    category_label: String,
    account_label: String,
}

fn build_draft_display(draft: &SpendingDraft, db: &Db) -> DraftDisplay {
    let category_name = db
        .get_category_by_id(draft.category_id)
        .map(|c| c.name)
        .unwrap_or_else(|| "?".to_string());

    let (account_name, currency_code) = db
        .get_account_by_id(draft.account_id)
        .map(|a| {
            let code = db
                .get_currency_by_id(a.currency_id)
                .map(|c| c.currency_code)
                .unwrap_or_default();
            (a.name, code)
        })
        .unwrap_or_else(|| ("?".to_string(), String::new()));

    DraftDisplay {
        summary_text: format!("Сумма: {:.2} {}", draft.amount, currency_code),
        category_label: category_name,
        account_label: format!("{} ({})", account_name, currency_code),
    }
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
            let d = build_draft_display(&updated, &db);
            bot.send_message(msg.chat.id, &d.summary_text)
                .parse_mode(ParseMode::Html)
                .reply_markup(keyboards::summary_keyboard(
                    &d.category_label,
                    &d.account_label,
                    updated.notes.as_deref(),
                ))
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

    let d = build_draft_display(&draft, &db);
    bot.send_message(msg.chat.id, &d.summary_text)
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboards::summary_keyboard(
            &d.category_label,
            &d.account_label,
            draft.notes.as_deref(),
        ))
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

    // Whitelist check
    if db.get_user_by_telegram_id(telegram_id).is_none() {
        return Ok(());
    }

    if drafts.get(telegram_id).is_none() {
        bot.edit_message_text(chat_id, msg_id, "Нет активного черновика. Отправьте сумму.")
            .await?;
        return Ok(());
    }

    match data {
        "edit_cat" => {
            drafts.update_state(telegram_id, EditState::ChoosingCategory);
            let categories = db.get_all_categories();
            bot.edit_message_text(chat_id, msg_id, "Выберите категорию:")
                .reply_markup(keyboards::category_keyboard(&categories))
                .await?;
        }
        "edit_acc" => {
            drafts.update_state(telegram_id, EditState::ChoosingAccount);
            let accounts = db.get_all_accounts();
            bot.edit_message_text(chat_id, msg_id, "Выберите счёт:")
                .reply_markup(keyboards::account_keyboard(&accounts))
                .await?;
        }
        "edit_note" => {
            drafts.update_state(telegram_id, EditState::EnteringNote);
            bot.edit_message_text(chat_id, msg_id, "Введите заметку:")
                .await?;
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

            bot.edit_message_text(
                chat_id,
                msg_id,
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
                let d = build_draft_display(&updated, &db);
                bot.edit_message_text(chat_id, msg_id, &d.summary_text)
                    .parse_mode(ParseMode::Html)
                    .reply_markup(keyboards::summary_keyboard(
                        &d.category_label,
                        &d.account_label,
                        updated.notes.as_deref(),
                    ))
                    .await?;
            }
        }
        _ if data.starts_with("acc:") => {
            if let Ok(acc_id) = data[4..].parse::<i64>() {
                drafts.update_account(telegram_id, acc_id);
                let updated = drafts.get(telegram_id).unwrap();
                let d = build_draft_display(&updated, &db);
                bot.edit_message_text(chat_id, msg_id, &d.summary_text)
                    .parse_mode(ParseMode::Html)
                    .reply_markup(keyboards::summary_keyboard(
                        &d.category_label,
                        &d.account_label,
                        updated.notes.as_deref(),
                    ))
                    .await?;
            }
        }
        _ => {}
    }

    Ok(())
}
