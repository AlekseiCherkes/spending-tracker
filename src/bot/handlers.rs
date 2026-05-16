use std::sync::Arc;

use teloxide::prelude::*;
use teloxide::types::{InputFile, MessageId, ParseMode};

use crate::dal::{Account, Category, Currency, Db, RecentSpending, User};
use crate::domain::{DraftKey, DraftStore, EditState, SpendingDraft};

use super::keyboards;

#[derive(Debug, PartialEq)]
enum Command {
    Accounts,
    Categories,
    Currencies,
    Users,
    DefaultAccount,
    Recent,
    Export,
}

fn parse_command(text: &str) -> Option<Command> {
    let first = text.split_whitespace().next()?;
    let name = first.split('@').next()?;
    match name {
        "/accounts" => Some(Command::Accounts),
        "/categories" => Some(Command::Categories),
        "/currencies" => Some(Command::Currencies),
        "/users" => Some(Command::Users),
        "/default_account" => Some(Command::DefaultAccount),
        "/recent" => Some(Command::Recent),
        "/export" => Some(Command::Export),
        _ => None,
    }
}

const RECENT_LIMIT: i64 = 25;

const EXPORT_MONTHS: usize = 3;

fn prev_month(year: i32, month: u32) -> (i32, u32) {
    if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    }
}

fn russian_month_name(month: u32) -> &'static str {
    match month {
        1 => "Январь",
        2 => "Февраль",
        3 => "Март",
        4 => "Апрель",
        5 => "Май",
        6 => "Июнь",
        7 => "Июль",
        8 => "Август",
        9 => "Сентябрь",
        10 => "Октябрь",
        11 => "Ноябрь",
        12 => "Декабрь",
        _ => "",
    }
}

fn parse_year_month(year_month: &str) -> Option<(i32, u32)> {
    let (y, m) = year_month.split_once('-')?;
    let year: i32 = y.parse().ok()?;
    let month: u32 = m.parse().ok().filter(|m| (1..=12).contains(m))?;
    Some((year, month))
}

fn format_month_label(year_month: &str, is_current: bool) -> String {
    match parse_year_month(year_month) {
        Some((year, month)) => {
            let name = russian_month_name(month);
            if is_current {
                format!("{} {} (текущий)", name, year)
            } else {
                format!("{} {}", name, year)
            }
        }
        None => year_month.to_string(),
    }
}

/// Returns up to `EXPORT_MONTHS` year-months ending in `current_ym`, newest first.
/// Each entry: (year_month string, is_current).
fn export_month_options(current_ym: &str) -> Vec<(String, bool)> {
    let Some((mut year, mut month)) = parse_year_month(current_ym) else {
        return vec![];
    };
    let mut out = Vec::with_capacity(EXPORT_MONTHS);
    out.push((format!("{:04}-{:02}", year, month), true));
    for _ in 1..EXPORT_MONTHS {
        let (y, m) = prev_month(year, month);
        year = y;
        month = m;
        out.push((format!("{:04}-{:02}", year, month), false));
    }
    out
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn build_csv(items: &[RecentSpending]) -> String {
    let mut out = String::from("Timestamp,Amount,Currency,Category,Account,IBAN,Reporter,Notes\n");
    for s in items {
        let notes = s.notes.as_deref().unwrap_or("");
        let iban = s.account_iban.as_deref().unwrap_or("");
        out.push_str(&format!(
            "{},{:.2},{},{},{},{},{},{}\n",
            csv_escape(&s.created_at),
            s.amount,
            csv_escape(&s.currency_code),
            csv_escape(&s.category_name),
            csv_escape(&s.account_name),
            csv_escape(iban),
            csv_escape(&s.reporter_name),
            csv_escape(notes),
        ));
    }
    out
}

fn format_short_datetime(iso: &str) -> String {
    let (date_part, time_part) = iso.split_once('T').unwrap_or((iso, ""));
    let hm: String = time_part.split(':').take(2).collect::<Vec<_>>().join(":");
    if hm.is_empty() {
        date_part.to_string()
    } else {
        format!("{} {}", date_part, hm)
    }
}

fn format_recent_spendings(items: &[RecentSpending]) -> String {
    if items.is_empty() {
        return "🧾 Транзакций пока нет".to_string();
    }
    let mut out = String::from("🧾 Последние транзакции (нажмите номер, чтобы изменить)\n\n");
    for (i, s) in items.iter().enumerate() {
        let when = format_short_datetime(&s.created_at);
        let cat = keyboards::format_category(&s.category_name);
        out.push_str(&format!(
            "{}. {} — {:.2} {} — {} — {}\n",
            i + 1,
            when,
            s.amount,
            s.currency_code,
            cat,
            s.reporter_name
        ));
        if let Some(note) = s.notes.as_deref().filter(|n| !n.is_empty()) {
            out.push_str(&format!("   📝 {}\n", note));
        }
    }
    out.trim_end().to_string()
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
                    Command::DefaultAccount | Command::Recent | Command::Export => unreachable!(),
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
            let d = build_draft_display(&updated, &db);
            let (chat_id, msg_id) = (ChatId(key.0), MessageId(key.1));
            bot.edit_message_text(chat_id, msg_id, &d.summary_text)
                .parse_mode(ParseMode::Html)
                .reply_markup(keyboards::summary_keyboard(
                    &d.amount_label,
                    &d.category_label,
                    &d.account_label,
                    updated.notes.as_deref(),
                    updated.editing_id.is_some(),
                ))
                .await?;
            return Ok(());
        }
        MessageAction::NoteInput(key) => {
            drafts.update_note(key, text);
            let updated = drafts.get(key).unwrap();
            let d = build_draft_display(&updated, &db);
            let (chat_id, msg_id) = (ChatId(key.0), MessageId(key.1));
            bot.edit_message_text(chat_id, msg_id, &d.summary_text)
                .parse_mode(ParseMode::Html)
                .reply_markup(keyboards::summary_keyboard(
                    &d.amount_label,
                    &d.category_label,
                    &d.account_label,
                    updated.notes.as_deref(),
                    updated.editing_id.is_some(),
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
                .get_spending_created_at(spending_id)
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
            let d = build_draft_display(&updated, &db);
            bot.edit_message_text(chat_id, msg_id, &d.summary_text)
                .parse_mode(ParseMode::Html)
                .reply_markup(keyboards::summary_keyboard(
                    &d.amount_label,
                    &d.category_label,
                    &d.account_label,
                    updated.notes.as_deref(),
                    updated.editing_id.is_some(),
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
                    &d.amount_label,
                    &d.category_label,
                    &d.account_label,
                    updated.notes.as_deref(),
                    updated.editing_id.is_some(),
                ))
                .await?;
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
            let d = build_draft_display(&updated, &db);
            bot.edit_message_text(chat_id, msg_id, &d.summary_text)
                .parse_mode(ParseMode::Html)
                .reply_markup(keyboards::summary_keyboard(
                    &d.amount_label,
                    &d.category_label,
                    &d.account_label,
                    updated.notes.as_deref(),
                    updated.editing_id.is_some(),
                ))
                .await?;
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

    // --- parse_command ---

    #[test]
    fn test_parse_known_commands() {
        assert_eq!(parse_command("/accounts"), Some(Command::Accounts));
        assert_eq!(parse_command("/categories"), Some(Command::Categories));
        assert_eq!(parse_command("/currencies"), Some(Command::Currencies));
        assert_eq!(parse_command("/users"), Some(Command::Users));
        assert_eq!(
            parse_command("/default_account"),
            Some(Command::DefaultAccount)
        );
        assert_eq!(parse_command("/recent"), Some(Command::Recent));
        assert_eq!(parse_command("/export"), Some(Command::Export));
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
    fn test_format_short_datetime_iso() {
        assert_eq!(
            format_short_datetime("2026-05-16T14:30:25"),
            "2026-05-16 14:30"
        );
    }

    #[test]
    fn test_format_short_datetime_no_time() {
        assert_eq!(format_short_datetime("2026-05-16"), "2026-05-16");
    }

    #[test]
    fn test_format_recent_spendings_empty() {
        assert_eq!(format_recent_spendings(&[]), "🧾 Транзакций пока нет");
    }

    // --- export helpers ---

    #[test]
    fn test_prev_month_basic() {
        assert_eq!(prev_month(2026, 5), (2026, 4));
        assert_eq!(prev_month(2026, 2), (2026, 1));
    }

    #[test]
    fn test_prev_month_wraps_year() {
        assert_eq!(prev_month(2026, 1), (2025, 12));
    }

    #[test]
    fn test_parse_year_month_valid() {
        assert_eq!(parse_year_month("2026-05"), Some((2026, 5)));
        assert_eq!(parse_year_month("2026-12"), Some((2026, 12)));
    }

    #[test]
    fn test_parse_year_month_invalid() {
        assert!(parse_year_month("2026-00").is_none());
        assert!(parse_year_month("2026-13").is_none());
        assert!(parse_year_month("2026").is_none());
        assert!(parse_year_month("abc-de").is_none());
    }

    #[test]
    fn test_format_month_label() {
        assert_eq!(format_month_label("2026-05", true), "Май 2026 (текущий)");
        assert_eq!(format_month_label("2026-04", false), "Апрель 2026");
        assert_eq!(format_month_label("2025-12", false), "Декабрь 2025");
        // Falls back to raw input on parse failure.
        assert_eq!(format_month_label("bogus", false), "bogus");
    }

    #[test]
    fn test_export_month_options_three_months_back() {
        let opts = export_month_options("2026-05");
        assert_eq!(opts.len(), 3);
        assert_eq!(opts[0], ("2026-05".to_string(), true));
        assert_eq!(opts[1], ("2026-04".to_string(), false));
        assert_eq!(opts[2], ("2026-03".to_string(), false));
    }

    #[test]
    fn test_export_month_options_crosses_year_boundary() {
        let opts = export_month_options("2026-01");
        assert_eq!(
            opts,
            vec![
                ("2026-01".to_string(), true),
                ("2025-12".to_string(), false),
                ("2025-11".to_string(), false),
            ]
        );
    }

    #[test]
    fn test_csv_escape_plain() {
        assert_eq!(csv_escape("hello"), "hello");
    }

    #[test]
    fn test_csv_escape_with_comma() {
        assert_eq!(csv_escape("a, b"), "\"a, b\"");
    }

    #[test]
    fn test_csv_escape_with_quote() {
        assert_eq!(csv_escape("she said \"hi\""), "\"she said \"\"hi\"\"\"");
    }

    #[test]
    fn test_csv_escape_with_newline() {
        assert_eq!(csv_escape("line1\nline2"), "\"line1\nline2\"");
    }

    #[test]
    fn test_build_csv_header_and_rows() {
        let items = vec![
            RecentSpending {
                id: 1,
                amount: 15.5,
                currency_code: "EUR".into(),
                account_name: "Revolut".into(),
                account_iban: Some("LT00 0000 0001".into()),
                category_name: "Продукты и хозтовары".into(),
                reporter_name: "Alex".into(),
                notes: Some("молоко, хлеб".into()),
                created_at: "2026-05-01T08:00:00".into(),
            },
            RecentSpending {
                id: 2,
                amount: 4.0,
                currency_code: "USD".into(),
                account_name: "Cash".into(),
                account_iban: None,
                category_name: "Кофе и вкусняшки".into(),
                reporter_name: "Hanna".into(),
                notes: None,
                created_at: "2026-05-02T09:10:00".into(),
            },
        ];
        let csv = build_csv(&items);
        let mut lines = csv.lines();
        assert_eq!(
            lines.next(),
            Some("Timestamp,Amount,Currency,Category,Account,IBAN,Reporter,Notes")
        );
        assert_eq!(
            lines.next(),
            Some(
                "2026-05-01T08:00:00,15.50,EUR,Продукты и хозтовары,Revolut,LT00 0000 0001,Alex,\"молоко, хлеб\""
            )
        );
        assert_eq!(
            lines.next(),
            Some("2026-05-02T09:10:00,4.00,USD,Кофе и вкусняшки,Cash,,Hanna,")
        );
        assert_eq!(lines.next(), None);
    }

    #[test]
    fn test_build_csv_empty_only_header() {
        let csv = build_csv(&[]);
        assert_eq!(
            csv,
            "Timestamp,Amount,Currency,Category,Account,IBAN,Reporter,Notes\n"
        );
    }

    #[test]
    fn test_format_recent_spendings_with_and_without_notes() {
        let items = vec![
            RecentSpending {
                id: 2,
                amount: 15.5,
                currency_code: "EUR".into(),
                account_name: "Revolut".into(),
                account_iban: None,
                category_name: "Продукты и хозтовары".into(),
                reporter_name: "Alex".into(),
                notes: Some("молоко".into()),
                created_at: "2026-05-16T14:30:25".into(),
            },
            RecentSpending {
                id: 1,
                amount: 4.0,
                currency_code: "EUR".into(),
                account_name: "Revolut".into(),
                account_iban: None,
                category_name: "Кофе и вкусняшки".into(),
                reporter_name: "Hanna".into(),
                notes: None,
                created_at: "2026-05-15T09:10:00".into(),
            },
        ];
        let out = format_recent_spendings(&items);
        assert!(out.starts_with("🧾 Последние транзакции"));
        assert!(out.contains("1. 2026-05-16 14:30 — 15.50 EUR — 🛒 Продукты и хозтовары — Alex"));
        assert!(out.contains("   📝 молоко"));
        assert!(out.contains("2. 2026-05-15 09:10 — 4.00 EUR — ☕ Кофе и вкусняшки — Hanna"));
        // Order is preserved (newest first as passed in).
        let alex_idx = out.find("Alex").unwrap();
        let hanna_idx = out.find("Hanna").unwrap();
        assert!(alex_idx < hanna_idx);
        // No stray note line for the second entry.
        assert_eq!(out.matches("📝").count(), 1);
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
