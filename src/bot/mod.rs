mod handlers;
mod keyboards;

use std::sync::Arc;

use teloxide::prelude::*;

use crate::dal::Db;
use crate::domain::DraftStore;

pub async fn run(db: Db, drafts: DraftStore) {
    let bot = Bot::from_env();
    let db = Arc::new(db);
    let drafts = Arc::new(drafts);

    log::info!("Starting spending tracker bot...");

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
