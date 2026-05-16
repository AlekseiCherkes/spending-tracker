mod handlers;
mod keyboards;

use std::sync::Arc;

use teloxide::prelude::*;
use teloxide::types::BotCommand;

use crate::dal::Db;
use crate::domain::DraftStore;

pub async fn run(db: Db, drafts: DraftStore) {
    let bot = Bot::from_env();
    let db = Arc::new(db);
    let drafts = Arc::new(drafts);

    log::info!("Starting spending tracker bot...");

    let commands = vec![
        BotCommand::new("recent", "🧾 Последние транзакции"),
        BotCommand::new("accounts", "💼 Счета"),
        BotCommand::new("categories", "📋 Категории"),
        BotCommand::new("currencies", "💱 Валюты"),
        BotCommand::new("users", "👥 Пользователи"),
        BotCommand::new("default_account", "⭐ Сменить счёт по умолчанию"),
    ];
    if let Err(e) = bot.set_my_commands(commands).await {
        log::warn!("Failed to set bot commands: {}", e);
    }

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(handlers::handle_message))
        .branch(Update::filter_callback_query().endpoint(handlers::handle_callback));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![db, drafts])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
