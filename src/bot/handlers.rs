use std::sync::Arc;

use teloxide::prelude::*;
use teloxide::types::{MessageId, ParseMode};

use crate::dal::Db;
use crate::domain::{DraftKey, DraftStore, EditState, SpendingDraft};

use super::keyboards;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, PartialEq)]
enum MessageAction {
    /// User sent a valid amount — create a new draft (and cancel any active note entry).
    NewAmount(f64),
    /// User sent text while a draft is in EnteringNote — treat as note for that draft.
    NoteInput(DraftKey),
    /// Not a number and no active note entry — show help.
    ShowHelp,
}

fn classify_input(text: &str, note_draft_key: Option<DraftKey>) -> MessageAction {
    let parsed: Option<f64> = text
        .replace(',', ".")
        .parse::<f64>()
        .ok()
        .filter(|v| *v > 0.0);

    match parsed {
        Some(amount) => MessageAction::NewAmount(amount),
        None => match note_draft_key {
            Some(key) => MessageAction::NoteInput(key),
            None => MessageAction::ShowHelp,
        },
    }
}

#[derive(Debug, PartialEq)]
enum CallbackAction {
    EditCategory,
    EditAccount,
    EditNote,
    Save,
    Cancel,
    SelectCategory(i64),
    SelectAccount(i64),
    Unknown,
}

fn classify_callback(data: &str) -> CallbackAction {
    match data {
        "edit_cat" => CallbackAction::EditCategory,
        "edit_acc" => CallbackAction::EditAccount,
        "edit_note" => CallbackAction::EditNote,
        "save" => CallbackAction::Save,
        "cancel" => CallbackAction::Cancel,
        _ if data.starts_with("cat:") => data[4..]
            .parse()
            .map(CallbackAction::SelectCategory)
            .unwrap_or(CallbackAction::Unknown),
        _ if data.starts_with("acc:") => data[4..]
            .parse()
            .map(CallbackAction::SelectAccount)
            .unwrap_or(CallbackAction::Unknown),
        _ => CallbackAction::Unknown,
    }
}

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

    let note_key = drafts.find_by_state(telegram_id, EditState::EnteringNote);
    let action = classify_input(&text, note_key);

    match action {
        MessageAction::NoteInput(key) => {
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
        MessageAction::ShowHelp => {
            bot.send_message(
                msg.chat.id,
                "Отправьте сумму (число), чтобы начать запись расхода.",
            )
            .await?;
            return Ok(());
        }
        MessageAction::NewAmount(_) => {
            drafts.cancel_note_entry(telegram_id);
        }
    }

    let amount = match action {
        MessageAction::NewAmount(v) => v,
        _ => unreachable!(),
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

    match classify_callback(data) {
        CallbackAction::EditCategory => {
            drafts.update_state(key, EditState::ChoosingCategory);
            let categories = db.get_all_categories();
            bot.edit_message_text(chat_id, msg_id, "Выберите категорию:")
                .reply_markup(keyboards::category_keyboard(&categories))
                .await?;
        }
        CallbackAction::EditAccount => {
            drafts.update_state(key, EditState::ChoosingAccount);
            let accounts = db.get_all_accounts();
            bot.edit_message_text(chat_id, msg_id, "Выберите счёт:")
                .reply_markup(keyboards::account_keyboard(&accounts))
                .await?;
        }
        CallbackAction::EditNote => {
            drafts.update_state(key, EditState::EnteringNote);
            bot.edit_message_text(chat_id, msg_id, "Введите заметку:")
                .await?;
        }
        CallbackAction::Save => {
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
        CallbackAction::Cancel => {
            drafts.remove(key);
            bot.edit_message_text(chat_id, msg_id, "❌ Расход отменён.")
                .await?;
        }
        CallbackAction::SelectCategory(cat_id) => {
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
        CallbackAction::SelectAccount(acc_id) => {
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
        CallbackAction::Unknown => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- classify_input ---

    #[test]
    fn test_integer_amount() {
        assert_eq!(classify_input("50", None), MessageAction::NewAmount(50.0));
    }

    #[test]
    fn test_decimal_amount_with_dot() {
        assert_eq!(classify_input("12.5", None), MessageAction::NewAmount(12.5));
    }

    #[test]
    fn test_decimal_amount_with_comma() {
        assert_eq!(classify_input("12,5", None), MessageAction::NewAmount(12.5));
    }

    #[test]
    fn test_zero_is_not_valid_amount() {
        assert_eq!(classify_input("0", None), MessageAction::ShowHelp);
    }

    #[test]
    fn test_negative_is_not_valid_amount() {
        assert_eq!(classify_input("-10", None), MessageAction::ShowHelp);
    }

    #[test]
    fn test_text_without_note_draft_shows_help() {
        assert_eq!(classify_input("hello", None), MessageAction::ShowHelp);
    }

    #[test]
    fn test_text_with_note_draft_is_note_input() {
        let key: DraftKey = (100, 1);
        assert_eq!(
            classify_input("lunch with team", Some(key)),
            MessageAction::NoteInput(key)
        );
    }

    #[test]
    fn test_number_overrides_note_entry() {
        let key: DraftKey = (100, 1);
        assert_eq!(
            classify_input("42", Some(key)),
            MessageAction::NewAmount(42.0)
        );
    }

    // --- classify_callback ---

    #[test]
    fn test_callback_edit_actions() {
        assert_eq!(classify_callback("edit_cat"), CallbackAction::EditCategory);
        assert_eq!(classify_callback("edit_acc"), CallbackAction::EditAccount);
        assert_eq!(classify_callback("edit_note"), CallbackAction::EditNote);
    }

    #[test]
    fn test_callback_save_cancel() {
        assert_eq!(classify_callback("save"), CallbackAction::Save);
        assert_eq!(classify_callback("cancel"), CallbackAction::Cancel);
    }

    #[test]
    fn test_callback_select_category() {
        assert_eq!(
            classify_callback("cat:5"),
            CallbackAction::SelectCategory(5)
        );
    }

    #[test]
    fn test_callback_select_account() {
        assert_eq!(classify_callback("acc:3"), CallbackAction::SelectAccount(3));
    }

    #[test]
    fn test_callback_invalid_id() {
        assert_eq!(classify_callback("cat:abc"), CallbackAction::Unknown);
    }

    #[test]
    fn test_callback_unknown() {
        assert_eq!(classify_callback("something"), CallbackAction::Unknown);
    }
}
