use std::sync::Arc;

use teloxide::prelude::*;
use teloxide::types::{MessageId, ParseMode};

use crate::dal::Db;
use crate::domain::{DraftKey, DraftStore, EditState, SpendingDraft};

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
        category_label: keyboards::format_category(&category_name),
        account_label: format!("{} ({})", account_name, currency_code),
    }
}

fn draft_key(chat_id: ChatId, msg_id: MessageId) -> DraftKey {
    (chat_id.0, msg_id.0)
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

    // Try to parse as number first
    let parsed_amount: Option<f64> = text
        .replace(',', ".")
        .parse::<f64>()
        .ok()
        .filter(|v| *v > 0.0);

    // If it's NOT a number, check if user is entering a note
    if parsed_amount.is_none() {
        if let Some(key) = drafts.find_by_state(telegram_id, EditState::EnteringNote) {
            drafts.update_note(key, text);
            let updated = drafts.get(key).unwrap();
            let d = build_draft_display(&updated, &db);
            let (chat_id, msg_id) = (ChatId(key.0), MessageId(key.1));
            bot.edit_message_text(chat_id, msg_id, &d.summary_text)
                .parse_mode(ParseMode::Html)
                .reply_markup(keyboards::summary_keyboard(
                    &d.category_label,
                    &d.account_label,
                    updated.notes.as_deref(),
                ))
                .await?;
            return Ok(());
        }

        bot.send_message(
            msg.chat.id,
            "Отправьте сумму (число), чтобы начать запись расхода.",
        )
        .await?;
        return Ok(());
    }

    let amount = parsed_amount.unwrap();

    // If user was entering a note for another draft, cancel that note entry
    drafts.cancel_note_entry(telegram_id);

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
        telegram_id,
        notes: None,
        edit_state: EditState::Summary,
    };

    let d = build_draft_display(&draft, &db);
    // Send summary message first, then store draft keyed by that message
    let sent = bot
        .send_message(msg.chat.id, &d.summary_text)
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboards::summary_keyboard(
            &d.category_label,
            &d.account_label,
            draft.notes.as_deref(),
        ))
        .await?;

    drafts.set(draft_key(sent.chat.id, sent.id), draft);

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

    let key = draft_key(chat_id, msg_id);

    if drafts.get(key).is_none() {
        bot.edit_message_text(chat_id, msg_id, "Нет активного черновика. Отправьте сумму.")
            .await?;
        return Ok(());
    }

    match data {
        "edit_cat" => {
            drafts.update_state(key, EditState::ChoosingCategory);
            let categories = db.get_all_categories();
            bot.edit_message_text(chat_id, msg_id, "Выберите категорию:")
                .reply_markup(keyboards::category_keyboard(&categories))
                .await?;
        }
        "edit_acc" => {
            drafts.update_state(key, EditState::ChoosingAccount);
            let accounts = db.get_all_accounts();
            bot.edit_message_text(chat_id, msg_id, "Выберите счёт:")
                .reply_markup(keyboards::account_keyboard(&accounts))
                .await?;
        }
        "edit_note" => {
            drafts.update_state(key, EditState::EnteringNote);
            bot.edit_message_text(chat_id, msg_id, "Введите заметку:")
                .await?;
        }
        "save" => {
            let draft = drafts.remove(key).unwrap();
            let spending_id = db.insert_spending(
                draft.account_id,
                draft.amount,
                draft.category_id,
                draft.reporter_user_id,
                draft.notes.as_deref(),
            )?;

            let d = build_draft_display(&draft, &db);
            let created_at = db
                .get_spending_created_at(spending_id)
                .unwrap_or_else(|| "—".to_string());
            let mut text = format!(
                "✅ Сохранено!\n\nСумма: {}\n{}\n{}\nДата: {}",
                d.summary_text.trim_start_matches("Сумма: "),
                d.category_label,
                d.account_label,
                created_at,
            );
            if let Some(notes) = &draft.notes {
                text.push_str(&format!("\nЗаметка: {}", notes));
            }

            bot.edit_message_text(chat_id, msg_id, text).await?;
        }
        "cancel" => {
            drafts.remove(key);
            bot.edit_message_text(chat_id, msg_id, "❌ Расход отменён.")
                .await?;
        }
        _ if data.starts_with("cat:") => {
            if let Ok(cat_id) = data[4..].parse::<i64>() {
                drafts.update_category(key, cat_id);
                let updated = drafts.get(key).unwrap();
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
                drafts.update_account(key, acc_id);
                let updated = drafts.get(key).unwrap();
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
