use std::sync::Arc;

use teloxide::prelude::*;
use teloxide::types::{MessageId, ParseMode};

use crate::dal::{Account, Category, Currency, Db, User};
use crate::domain::{DraftKey, DraftStore, EditState, SpendingDraft};

use super::keyboards;

#[derive(Debug, PartialEq)]
enum Command {
    Accounts,
    Categories,
    Currencies,
    Users,
}

fn parse_command(text: &str) -> Option<Command> {
    let first = text.split_whitespace().next()?;
    let name = first.split('@').next()?;
    match name {
        "/accounts" => Some(Command::Accounts),
        "/categories" => Some(Command::Categories),
        "/currencies" => Some(Command::Currencies),
        "/users" => Some(Command::Users),
        _ => None,
    }
}

fn format_users(users: &[User]) -> String {
    let mut out = String::from("👥 Пользователи\n\n");
    for u in users {
        if u.is_admin {
            out.push_str(&format!("• {} 👑 admin\n", u.name));
        } else {
            out.push_str(&format!("• {}\n", u.name));
        }
    }
    out.trim_end().to_string()
}

fn format_currencies(currencies: &[Currency]) -> String {
    let mut out = String::from("💱 Валюты\n\n");
    for c in currencies {
        out.push_str(&format!("• {}\n", c.currency_code));
    }
    out.trim_end().to_string()
}

fn format_categories(categories: &[Category]) -> String {
    let mut out = String::from("📋 Категории\n\n");
    for (i, c) in categories.iter().enumerate() {
        out.push_str(&format!(
            "{}. {}\n",
            i + 1,
            keyboards::format_category(&c.name)
        ));
    }
    out.trim_end().to_string()
}

fn format_accounts(accounts: &[Account], users: &[User], currencies: &[Currency]) -> String {
    let code_of = |id: i64| -> &str {
        currencies
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.currency_code.as_str())
            .unwrap_or("?")
    };

    let mut out = String::from("💼 Счета\n\n");

    for u in users {
        let owned: Vec<&Account> = accounts
            .iter()
            .filter(|a| a.owner_id == Some(u.id))
            .collect();
        if owned.is_empty() {
            continue;
        }
        out.push_str(&format!("👤 {}\n", u.name));
        for a in owned {
            let default_mark = if u.default_account_id == Some(a.id) {
                " ⭐ по умолчанию"
            } else {
                ""
            };
            out.push_str(&format!(
                "• {} — {}{}\n",
                a.name,
                code_of(a.currency_id),
                default_mark
            ));
            if let Some(iban) = &a.iban {
                out.push_str(&format!("  IBAN: {}\n", iban));
            }
        }
        out.push('\n');
    }

    let unassigned: Vec<&Account> = accounts.iter().filter(|a| a.owner_id.is_none()).collect();
    if !unassigned.is_empty() {
        out.push_str("❓ Без владельца\n");
        for a in unassigned {
            out.push_str(&format!("• {} — {}\n", a.name, code_of(a.currency_id)));
            if let Some(iban) = &a.iban {
                out.push_str(&format!("  IBAN: {}\n", iban));
            }
        }
    }

    out.trim_end().to_string()
}

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

    if let Some(cmd) = parse_command(&text) {
        let response = match cmd {
            Command::Accounts => format_accounts(
                &db.get_all_accounts(),
                &db.get_all_users(),
                &db.get_all_currencies(),
            ),
            Command::Categories => format_categories(&db.get_all_categories()),
            Command::Currencies => format_currencies(&db.get_all_currencies()),
            Command::Users => format_users(&db.get_all_users()),
        };
        bot.send_message(msg.chat.id, response).await?;
        return Ok(());
    }

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

    // --- parse_command ---

    #[test]
    fn test_parse_known_commands() {
        assert_eq!(parse_command("/accounts"), Some(Command::Accounts));
        assert_eq!(parse_command("/categories"), Some(Command::Categories));
        assert_eq!(parse_command("/currencies"), Some(Command::Currencies));
        assert_eq!(parse_command("/users"), Some(Command::Users));
    }

    #[test]
    fn test_parse_command_with_bot_suffix() {
        assert_eq!(parse_command("/users@MyBot"), Some(Command::Users));
    }

    #[test]
    fn test_parse_command_ignores_trailing_args() {
        assert_eq!(parse_command("/accounts please"), Some(Command::Accounts));
    }

    #[test]
    fn test_parse_command_unknown() {
        assert_eq!(parse_command("/wat"), None);
        assert_eq!(parse_command("hello"), None);
        assert_eq!(parse_command(""), None);
    }

    // --- formatters ---

    #[test]
    fn test_format_users_marks_admin() {
        let users = vec![
            User {
                id: 1,
                name: "Alex".into(),
                telegram_id: 1,
                is_admin: true,
                default_account_id: None,
            },
            User {
                id: 2,
                name: "Hanna".into(),
                telegram_id: 2,
                is_admin: false,
                default_account_id: None,
            },
        ];
        let out = format_users(&users);
        assert!(out.contains("Alex 👑 admin"));
        assert!(out.contains("• Hanna"));
        assert!(!out.contains("Hanna 👑"));
    }

    #[test]
    fn test_format_currencies_lists_codes() {
        let currencies = vec![
            Currency {
                id: 1,
                currency_code: "EUR".into(),
            },
            Currency {
                id: 2,
                currency_code: "USD".into(),
            },
        ];
        let out = format_currencies(&currencies);
        assert!(out.contains("• EUR"));
        assert!(out.contains("• USD"));
    }

    #[test]
    fn test_format_categories_numbered_in_order() {
        let categories = vec![
            Category {
                id: 1,
                name: "Продукты и хозтовары".into(),
                sort_order: 0,
            },
            Category {
                id: 2,
                name: "Другое".into(),
                sort_order: 1,
            },
        ];
        let out = format_categories(&categories);
        let p_idx = out.find("Продукты").unwrap();
        let o_idx = out.find("Другое").unwrap();
        assert!(p_idx < o_idx);
        assert!(out.contains("1. 🛒 Продукты и хозтовары"));
        assert!(out.contains("2. 📦 Другое"));
    }

    #[test]
    fn test_format_accounts_groups_and_marks_default() {
        let users = vec![
            User {
                id: 1,
                name: "Alex".into(),
                telegram_id: 1,
                is_admin: true,
                default_account_id: Some(10),
            },
            User {
                id: 2,
                name: "Hanna".into(),
                telegram_id: 2,
                is_admin: false,
                default_account_id: None,
            },
        ];
        let currencies = vec![Currency {
            id: 1,
            currency_code: "EUR".into(),
        }];
        let accounts = vec![
            Account {
                id: 10,
                name: "Revolut".into(),
                currency_id: 1,
                owner_id: Some(1),
                iban: Some("LT00".into()),
            },
            Account {
                id: 11,
                name: "Nordea".into(),
                currency_id: 1,
                owner_id: Some(1),
                iban: None,
            },
            Account {
                id: 12,
                name: "S-pankki".into(),
                currency_id: 1,
                owner_id: Some(2),
                iban: None,
            },
        ];
        let out = format_accounts(&accounts, &users, &currencies);
        assert!(out.contains("👤 Alex"));
        assert!(out.contains("👤 Hanna"));
        assert!(out.contains("Revolut — EUR ⭐ по умолчанию"));
        assert!(out.contains("Nordea — EUR\n"));
        assert!(!out.contains("Nordea — EUR ⭐"));
        assert!(out.contains("IBAN: LT00"));
        let alex_idx = out.find("Alex").unwrap();
        let hanna_idx = out.find("Hanna").unwrap();
        assert!(alex_idx < hanna_idx);
    }

    #[test]
    fn test_format_accounts_shows_unassigned() {
        let users = vec![];
        let currencies = vec![Currency {
            id: 1,
            currency_code: "EUR".into(),
        }];
        let accounts = vec![Account {
            id: 1,
            name: "Orphan".into(),
            currency_id: 1,
            owner_id: None,
            iban: None,
        }];
        let out = format_accounts(&accounts, &users, &currencies);
        assert!(out.contains("❓ Без владельца"));
        assert!(out.contains("Orphan — EUR"));
    }
}
