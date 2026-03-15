use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::dal::{Account, Category};

pub fn summary_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("Категория", "edit_cat"),
            InlineKeyboardButton::callback("Счёт", "edit_acc"),
        ],
        vec![
            InlineKeyboardButton::callback("Заметка", "edit_note"),
            InlineKeyboardButton::callback("\u{2705} Сохранить", "save"),
        ],
    ])
}

pub fn category_keyboard(categories: &[Category]) -> InlineKeyboardMarkup {
    let rows: Vec<Vec<InlineKeyboardButton>> = categories
        .chunks(2)
        .map(|chunk| {
            chunk
                .iter()
                .map(|c| InlineKeyboardButton::callback(&c.name, format!("cat:{}", c.id)))
                .collect()
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
