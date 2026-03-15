mod bot;
mod dal;
mod domain;

use std::env;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let db_path = env::var("DATABASE_PATH").unwrap_or_else(|_| "spending_tracker.db".to_string());
    let db = dal::Db::open(&db_path).expect("Failed to open database");
    let drafts = domain::DraftStore::new();

    bot::run(db, drafts).await;
}
