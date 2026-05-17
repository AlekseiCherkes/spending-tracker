use std::sync::Arc;

use teloxide::prelude::*;
use teloxide::types::{InputFile, MessageId, ParseMode};

use crate::dal::Db;
use crate::domain::{DraftKey, DraftStore, EditState, SpendingDraft};

use super::commands::{parse_command, Command};
use super::export::{build_csv, export_month_options};
use super::format::{
    format_accounts, format_categories, format_currencies, format_month_label,
    format_recent_spendings, format_users, parse_year_month,
};
use super::keyboards;

const RECENT_LIMIT: i64 = 25;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, PartialEq)]
enum MessageAction {
    /// User sent a valid number while no draft awaits an amount — create a new draft.
    NewAmount(f64),
    /// User sent a valid number while a draft is in EnteringAmount — update that draft's amount.
    AmountInput(DraftKey, f64),
    /// User sent text while a draft is in EnteringNote — treat as note for that draft.
    NoteInput(DraftKey),
    /// Nothing actionable — show help.
    ShowHelp,
}

fn classify_input(
    text: &str,
    amount_draft_key: Option<DraftKey>,
    note_draft_key: Option<DraftKey>,
) -> MessageAction {
    let parsed: Option<f64> = text
        .replace(',', ".")
        .parse::<f64>()
        .ok()
        .filter(|v| *v > 0.0);

    match parsed {
        Some(amount) => match amount_draft_key {
            Some(key) => MessageAction::AmountInput(key, amount),
            None => MessageAction::NewAmount(amount),
        },
        None => match note_draft_key {
            Some(key) => MessageAction::NoteInput(key),
            None => MessageAction::ShowHelp,
        },
    }
}

#[derive(Debug, PartialEq)]
enum CallbackAction {
    EditAmount,
    EditCategory,
    EditAccount,
    EditNote,
    Save,
    Cancel,
    Delete,
    ConfirmDelete,
    CancelDelete,
    SelectCategory(i64),
    SelectAccount(i64),
    SetDefaultAccount(i64),
    EditSpending(i64),
    ExportMonth(String),
    Unknown,
}

fn classify_callback(data: &str) -> CallbackAction {
    match data {
        "edit_amount" => CallbackAction::EditAmount,
        "edit_cat" => CallbackAction::EditCategory,
        "edit_acc" => CallbackAction::EditAccount,
        "edit_note" => CallbackAction::EditNote,
        "save" => CallbackAction::Save,
        "cancel" => CallbackAction::Cancel,
        "delete" => CallbackAction::Delete,
        "confirm_delete" => CallbackAction::ConfirmDelete,
        "cancel_delete" => CallbackAction::CancelDelete,
        _ if data.starts_with("cat:") => data[4..]
            .parse()
            .map(CallbackAction::SelectCategory)
            .unwrap_or(CallbackAction::Unknown),
        _ if data.starts_with("acc:") => data[4..]
            .parse()
            .map(CallbackAction::SelectAccount)
            .unwrap_or(CallbackAction::Unknown),
        _ if data.starts_with("setdef:") => data[7..]
            .parse()
            .map(CallbackAction::SetDefaultAccount)
            .unwrap_or(CallbackAction::Unknown),
        _ if data.starts_with("edit_sp:") => data[8..]
            .parse()
            .map(CallbackAction::EditSpending)
            .unwrap_or(CallbackAction::Unknown),
        _ if data.starts_with("export:") => {
            let ym = &data[7..];
            if parse_year_month(ym).is_some() {
                CallbackAction::ExportMonth(ym.to_string())
            } else {
                CallbackAction::Unknown
            }
        }
        _ => CallbackAction::Unknown,
    }
}

struct DraftDisplay {
    summary_text: String,
    amount_label: String,
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

    let amount_label = format!("{:.2} {}", draft.amount, currency_code);
    DraftDisplay {
        summary_text: format!("Сумма: {}", amount_label),
        amount_label,
        category_label: keyboards::format_category(&category_name),
        account_label: format!("{} ({})", account_name, currency_code),
    }
}

fn draft_key(chat_id: ChatId, msg_id: MessageId) -> DraftKey {
    (chat_id.0, msg_id.0)
}

/// Re-renders the draft summary message in place. Shared by every callback / message
/// branch that mutates the draft and needs the UI to reflect the new state.
async fn rerender_draft_summary(
    bot: &Bot,
    db: &Db,
    chat_id: ChatId,
    msg_id: MessageId,
    draft: &SpendingDraft,
) -> HandlerResult {
    let d = build_draft_display(draft, db);
    bot.edit_message_text(chat_id, msg_id, &d.summary_text)
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboards::summary_keyboard(
            &d.amount_label,
            &d.category_label,
            &d.account_label,
            draft.notes.as_deref(),
            draft.editing_id.is_some(),
        ))
        .await?;
    Ok(())
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

    if let Some(cmd) = parse_command(&text) {
        match cmd {
            Command::Start => {
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "Привет, {}!\n\nОтправьте сумму (число), чтобы записать расход. Команды доступны в меню.",
                        user.name
                    ),
                )
                .await?;
                return Ok(());
            }
            Command::DefaultAccount => {
                let accounts = db.get_all_accounts();
                let prompt = match user.default_account_id {
                    Some(id) => {
                        let current = accounts
                            .iter()
                            .find(|a| a.id == id)
                            .map(|a| a.name.as_str())
                            .unwrap_or("?");
                        format!("Текущий счёт по умолчанию: {}\n\nВыберите новый:", current)
                    }
                    None => "Счёт по умолчанию не выбран.\n\nВыберите счёт:".to_string(),
                };
                bot.send_message(msg.chat.id, prompt)
                    .reply_markup(keyboards::default_account_keyboard(
                        &accounts,
                        user.default_account_id,
                    ))
                    .await?;
                return Ok(());
            }
            Command::Recent => {
                let recent = db.get_recent_spendings(RECENT_LIMIT);
                let text = format_recent_spendings(&recent);
                let mut send = bot.send_message(msg.chat.id, text);
                if !recent.is_empty() {
                    send = send.reply_markup(keyboards::recent_keyboard(&recent));
                }
                send.await?;
                return Ok(());
            }
            Command::Export => {
                let current_ym = db.current_year_month_utc();
                let labeled: Vec<(String, String)> = export_month_options(&current_ym)
                    .into_iter()
                    .map(|(ym, is_current)| (format_month_label(&ym, is_current), ym))
                    .collect();
                bot.send_message(msg.chat.id, "📤 Выберите месяц для экспорта:")
                    .reply_markup(keyboards::export_months_keyboard(&labeled))
                    .await?;
                return Ok(());
            }
            other => {
                let response = match other {
                    Command::Accounts => format_accounts(
                        &db.get_all_accounts(),
                        &db.get_all_users(),
                        &db.get_all_currencies(),
                    ),
                    Command::Categories => format_categories(&db.get_all_categories()),
                    Command::Currencies => format_currencies(&db.get_all_currencies()),
                    Command::Users => format_users(&db.get_all_users()),
                    Command::Start
                    | Command::DefaultAccount
                    | Command::Recent
                    | Command::Export => unreachable!(),
                };
                bot.send_message(msg.chat.id, response).await?;
                return Ok(());
            }
        }
    }

    let amount_key = drafts.find_by_state(telegram_id, EditState::EnteringAmount);
    let note_key = drafts.find_by_state(telegram_id, EditState::EnteringNote);
    let action = classify_input(&text, amount_key, note_key);

    match action {
        MessageAction::AmountInput(key, new_amount) => {
            drafts.update_amount(key, new_amount);
            let updated = drafts.get(key).unwrap();
            let (chat_id, msg_id) = (ChatId(key.0), MessageId(key.1));
            rerender_draft_summary(&bot, &db, chat_id, msg_id, &updated).await?;
            return Ok(());
        }
        MessageAction::NoteInput(key) => {
            drafts.update_note(key, text);
            let updated = drafts.get(key).unwrap();
            let (chat_id, msg_id) = (ChatId(key.0), MessageId(key.1));
            rerender_draft_summary(&bot, &db, chat_id, msg_id, &updated).await?;
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
            drafts.cancel_pending_input(telegram_id);
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
        editing_id: None,
    };

    let d = build_draft_display(&draft, &db);
    // Send summary message first, then store draft keyed by that message
    let sent = bot
        .send_message(msg.chat.id, &d.summary_text)
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboards::summary_keyboard(
            &d.amount_label,
            &d.category_label,
            &d.account_label,
            draft.notes.as_deref(),
            false,
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
    let user = match db.get_user_by_telegram_id(telegram_id) {
        Some(u) => u,
        None => return Ok(()),
    };

    let action = classify_callback(data);

    if let CallbackAction::SetDefaultAccount(acc_id) = action {
        db.update_user_default_account(user.id, acc_id)?;
        let name = db
            .get_account_by_id(acc_id)
            .map(|a| a.name)
            .unwrap_or_else(|| "?".to_string());
        bot.edit_message_text(chat_id, msg_id, format!("✅ Счёт по умолчанию: {}", name))
            .await?;
        return Ok(());
    }

    if let CallbackAction::ExportMonth(year_month) = action {
        let items = db.get_spendings_in_month(&year_month);
        if items.is_empty() {
            bot.send_message(
                chat_id,
                format!(
                    "В {} нет транзакций.",
                    format_month_label(&year_month, false)
                ),
            )
            .await?;
            return Ok(());
        }
        let csv = build_csv(&items);
        let file =
            InputFile::memory(csv.into_bytes()).file_name(format!("spendings_{}.csv", year_month));
        bot.send_document(chat_id, file).await?;
        return Ok(());
    }

    if let CallbackAction::EditSpending(spending_id) = action {
        let spending = match db.get_spending_by_id(spending_id) {
            Some(s) => s,
            None => {
                bot.send_message(chat_id, "Транзакция не найдена.").await?;
                return Ok(());
            }
        };
        let draft = SpendingDraft {
            amount: spending.amount,
            category_id: spending.category_id,
            account_id: spending.account_id,
            reporter_user_id: spending.reporter_id,
            telegram_id,
            notes: spending.notes,
            edit_state: EditState::Summary,
            editing_id: Some(spending.id),
        };
        let d = build_draft_display(&draft, &db);
        let sent = bot
            .send_message(chat_id, &d.summary_text)
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboards::summary_keyboard(
                &d.amount_label,
                &d.category_label,
                &d.account_label,
                draft.notes.as_deref(),
                true,
            ))
            .await?;
        drafts.set(draft_key(sent.chat.id, sent.id), draft);
        return Ok(());
    }

    let key = draft_key(chat_id, msg_id);

    if drafts.get(key).is_none() {
        bot.edit_message_text(chat_id, msg_id, "Нет активного черновика. Отправьте сумму.")
            .await?;
        return Ok(());
    }

    match action {
        CallbackAction::EditAmount => {
            drafts.update_state(key, EditState::EnteringAmount);
            bot.edit_message_text(chat_id, msg_id, "Введите новую сумму:")
                .await?;
        }
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
            let users = db.get_all_users();
            bot.edit_message_text(chat_id, msg_id, "Выберите счёт:")
                .reply_markup(keyboards::account_keyboard(&accounts, &users))
                .await?;
        }
        CallbackAction::EditNote => {
            drafts.update_state(key, EditState::EnteringNote);
            bot.edit_message_text(chat_id, msg_id, "Введите заметку:")
                .await?;
        }
        CallbackAction::Save => {
            let draft = drafts.remove(key).unwrap();
            let (header, spending_id) = match draft.editing_id {
                Some(id) => {
                    db.update_spending(
                        id,
                        draft.account_id,
                        draft.amount,
                        draft.category_id,
                        draft.notes.as_deref(),
                    )?;
                    ("✅ Изменено!", id)
                }
                None => {
                    let new_id = db.insert_spending(
                        draft.account_id,
                        draft.amount,
                        draft.category_id,
                        draft.reporter_user_id,
                        draft.notes.as_deref(),
                    )?;
                    ("✅ Сохранено!", new_id)
                }
            };

            let d = build_draft_display(&draft, &db);
            let created_at = db
                .get_spending_by_id(spending_id)
                .map(|s| s.created_at)
                .unwrap_or_else(|| "—".to_string());
            let mut text = format!(
                "{}\n\nСумма: {}\n{}\n{}\nДата: {}",
                header, d.amount_label, d.category_label, d.account_label, created_at,
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
            rerender_draft_summary(&bot, &db, chat_id, msg_id, &updated).await?;
        }
        CallbackAction::SelectAccount(acc_id) => {
            drafts.update_account(key, acc_id);
            let updated = drafts.get(key).unwrap();
            rerender_draft_summary(&bot, &db, chat_id, msg_id, &updated).await?;
        }
        CallbackAction::Delete => {
            drafts.update_state(key, EditState::ConfirmingDelete);
            bot.edit_message_text(chat_id, msg_id, "🗑 Удалить эту транзакцию?")
                .reply_markup(keyboards::confirm_delete_keyboard())
                .await?;
        }
        CallbackAction::ConfirmDelete => {
            let draft = drafts.remove(key).unwrap();
            match draft.editing_id {
                Some(id) => {
                    db.delete_spending(id)?;
                    bot.edit_message_text(chat_id, msg_id, "🗑 Транзакция удалена.")
                        .await?;
                }
                None => {
                    bot.edit_message_text(chat_id, msg_id, "❌ Расход отменён.")
                        .await?;
                }
            }
        }
        CallbackAction::CancelDelete => {
            drafts.update_state(key, EditState::Summary);
            let updated = drafts.get(key).unwrap();
            rerender_draft_summary(&bot, &db, chat_id, msg_id, &updated).await?;
        }
        CallbackAction::SetDefaultAccount(_) => unreachable!(),
        CallbackAction::EditSpending(_) => unreachable!(),
        CallbackAction::ExportMonth(_) => unreachable!(),
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
        assert_eq!(
            classify_input("50", None, None),
            MessageAction::NewAmount(50.0)
        );
    }

    #[test]
    fn test_decimal_amount_with_dot() {
        assert_eq!(
            classify_input("12.5", None, None),
            MessageAction::NewAmount(12.5)
        );
    }

    #[test]
    fn test_decimal_amount_with_comma() {
        assert_eq!(
            classify_input("12,5", None, None),
            MessageAction::NewAmount(12.5)
        );
    }

    #[test]
    fn test_zero_is_not_valid_amount() {
        assert_eq!(classify_input("0", None, None), MessageAction::ShowHelp);
    }

    #[test]
    fn test_negative_is_not_valid_amount() {
        assert_eq!(classify_input("-10", None, None), MessageAction::ShowHelp);
    }

    #[test]
    fn test_text_without_note_draft_shows_help() {
        assert_eq!(classify_input("hello", None, None), MessageAction::ShowHelp);
    }

    #[test]
    fn test_text_with_note_draft_is_note_input() {
        let key: DraftKey = (100, 1);
        assert_eq!(
            classify_input("lunch with team", None, Some(key)),
            MessageAction::NoteInput(key)
        );
    }

    #[test]
    fn test_number_with_amount_entry_is_amount_input() {
        let key: DraftKey = (100, 1);
        assert_eq!(
            classify_input("42", Some(key), None),
            MessageAction::AmountInput(key, 42.0)
        );
    }

    #[test]
    fn test_number_without_amount_entry_creates_new_draft() {
        let note_key: DraftKey = (100, 1);
        assert_eq!(
            classify_input("42", None, Some(note_key)),
            MessageAction::NewAmount(42.0)
        );
    }

    #[test]
    fn test_amount_entry_takes_priority_over_note_entry() {
        let amount_key: DraftKey = (100, 1);
        let note_key: DraftKey = (100, 2);
        assert_eq!(
            classify_input("42", Some(amount_key), Some(note_key)),
            MessageAction::AmountInput(amount_key, 42.0)
        );
    }

    #[test]
    fn test_text_in_amount_entry_only_shows_help() {
        let amount_key: DraftKey = (100, 1);
        assert_eq!(
            classify_input("hello", Some(amount_key), None),
            MessageAction::ShowHelp
        );
    }

    // --- classify_callback ---

    #[test]
    fn test_callback_edit_actions() {
        assert_eq!(classify_callback("edit_amount"), CallbackAction::EditAmount);
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
    fn test_callback_delete_actions() {
        assert_eq!(classify_callback("delete"), CallbackAction::Delete);
        assert_eq!(
            classify_callback("confirm_delete"),
            CallbackAction::ConfirmDelete
        );
        assert_eq!(
            classify_callback("cancel_delete"),
            CallbackAction::CancelDelete
        );
    }

    #[test]
    fn test_callback_edit_spending() {
        assert_eq!(
            classify_callback("edit_sp:42"),
            CallbackAction::EditSpending(42)
        );
        assert_eq!(classify_callback("edit_sp:abc"), CallbackAction::Unknown);
    }

    #[test]
    fn test_callback_export_month() {
        assert_eq!(
            classify_callback("export:2026-05"),
            CallbackAction::ExportMonth("2026-05".to_string())
        );
        assert_eq!(classify_callback("export:2026-13"), CallbackAction::Unknown);
        assert_eq!(classify_callback("export:nope"), CallbackAction::Unknown);
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
    fn test_callback_set_default_account() {
        assert_eq!(
            classify_callback("setdef:7"),
            CallbackAction::SetDefaultAccount(7)
        );
        assert_eq!(classify_callback("setdef:abc"), CallbackAction::Unknown);
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
