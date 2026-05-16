use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::dal::{Account, Category, RecentSpending, User};

pub fn category_emoji(name: &str) -> &'static str {
    match name {
        "Продукты и хозтовары" => "🛒",
        "Еда вне дома" => "🍽️",
        "Кофе и вкусняшки" => "☕",
        "Развлечения и отдых" => "🎭",
        "Одежда" => "👗",
        "Здоровье и медицина" => "💊",
        "Спорт, забота о себе" => "🏋️",
        "Образование" => "📚",
        "Путешествия, туризм" => "✈️",
        "Дети (образование)" => "🎓",
        "Дети (хобби)" => "🎨",
        "Дети (присмотр)" => "👶",
        "Интернет подписки" => "💻",
        "Транспорт" => "🚌",
        "Автомобиль" => "🚗",
        "Автомобиль (аренда)" => "🔑",
        "Автомобиль (бензин, паркинг)" => "⛽",
        "Жильё" => "🏠",
        "Жильё (обустройство)" => "🔧",
        "Другое" => "📦",
        _ => "",
    }
}

pub fn format_category(name: &str) -> String {
    let emoji = category_emoji(name);
    if emoji.is_empty() {
        name.to_string()
    } else {
        format!("{} {}", emoji, name)
    }
}

pub fn summary_keyboard(
    amount_label: &str,
    category: &str,
    account: &str,
    notes: Option<&str>,
    editing: bool,
) -> InlineKeyboardMarkup {
    let note_label = match notes {
        Some(n) => format!("📝 {}", n),
        None => "📝 Заметка".to_string(),
    };
    let mut rows = vec![
        vec![InlineKeyboardButton::callback(
            format!("💰 {}", amount_label),
            "edit_amount",
        )],
        vec![InlineKeyboardButton::callback(category, "edit_cat")],
        vec![InlineKeyboardButton::callback(account, "edit_acc")],
        vec![InlineKeyboardButton::callback(note_label, "edit_note")],
        vec![InlineKeyboardButton::callback("✅ Сохранить", "save")],
    ];
    if editing {
        rows.push(vec![InlineKeyboardButton::callback("🗑 Удалить", "delete")]);
    }
    rows.push(vec![InlineKeyboardButton::callback("❌ Отмена", "cancel")]);
    InlineKeyboardMarkup::new(rows)
}

pub fn confirm_delete_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("✅ Да, удалить", "confirm_delete"),
        InlineKeyboardButton::callback("❌ Отмена", "cancel_delete"),
    ]])
}

/// Each option: (label, year_month). Renders one button per row; callback is `export:<year_month>`.
pub fn export_months_keyboard(options: &[(String, String)]) -> InlineKeyboardMarkup {
    let rows: Vec<Vec<InlineKeyboardButton>> = options
        .iter()
        .map(|(label, ym)| {
            vec![InlineKeyboardButton::callback(
                label.clone(),
                format!("export:{}", ym),
            )]
        })
        .collect();
    InlineKeyboardMarkup::new(rows)
}

pub fn recent_keyboard(spendings: &[RecentSpending]) -> InlineKeyboardMarkup {
    let buttons: Vec<InlineKeyboardButton> = spendings
        .iter()
        .enumerate()
        .map(|(i, s)| {
            InlineKeyboardButton::callback(format!("{}", i + 1), format!("edit_sp:{}", s.id))
        })
        .collect();
    let rows: Vec<Vec<InlineKeyboardButton>> = buttons.chunks(5).map(|c| c.to_vec()).collect();
    InlineKeyboardMarkup::new(rows)
}

pub fn category_keyboard(categories: &[Category]) -> InlineKeyboardMarkup {
    let rows: Vec<Vec<InlineKeyboardButton>> = categories
        .iter()
        .map(|c| {
            vec![InlineKeyboardButton::callback(
                format_category(&c.name),
                format!("cat:{}", c.id),
            )]
        })
        .collect();
    InlineKeyboardMarkup::new(rows)
}

pub fn account_keyboard(accounts: &[Account], users: &[User]) -> InlineKeyboardMarkup {
    let rows: Vec<Vec<InlineKeyboardButton>> = accounts
        .iter()
        .map(|a| {
            let owner = a
                .owner_id
                .and_then(|id| users.iter().find(|u| u.id == id))
                .map(|u| u.name.as_str());
            let label = match owner {
                Some(name) => format!("{} — {}", a.name, name),
                None => a.name.clone(),
            };
            vec![InlineKeyboardButton::callback(
                label,
                format!("acc:{}", a.id),
            )]
        })
        .collect();
    InlineKeyboardMarkup::new(rows)
}

pub fn default_account_keyboard(
    accounts: &[Account],
    current_default: Option<i64>,
) -> InlineKeyboardMarkup {
    let rows: Vec<Vec<InlineKeyboardButton>> = accounts
        .iter()
        .map(|a| {
            let label = if Some(a.id) == current_default {
                format!("⭐ {}", a.name)
            } else {
                a.name.clone()
            };
            vec![InlineKeyboardButton::callback(
                label,
                format!("setdef:{}", a.id),
            )]
        })
        .collect();
    InlineKeyboardMarkup::new(rows)
}
