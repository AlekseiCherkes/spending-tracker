use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::dal::{Account, Category};

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
    category: &str,
    account: &str,
    notes: Option<&str>,
) -> InlineKeyboardMarkup {
    let note_label = match notes {
        Some(n) => format!("📝 {}", n),
        None => "📝 Заметка".to_string(),
    };
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(category, "edit_cat")],
        vec![InlineKeyboardButton::callback(account, "edit_acc")],
        vec![InlineKeyboardButton::callback(note_label, "edit_note")],
        vec![InlineKeyboardButton::callback("✅ Сохранить", "save")],
    ])
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

pub fn account_keyboard(accounts: &[Account]) -> InlineKeyboardMarkup {
    let rows: Vec<Vec<InlineKeyboardButton>> = accounts
        .iter()
        .map(|a| {
            vec![InlineKeyboardButton::callback(
                &a.name,
                format!("acc:{}", a.id),
            )]
        })
        .collect();
    InlineKeyboardMarkup::new(rows)
}
