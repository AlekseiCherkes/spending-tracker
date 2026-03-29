use rusqlite::Connection;
use std::env;

pub fn seed_if_empty(conn: &Connection) {
    let user_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
        .unwrap_or(0);
    if user_count > 0 {
        return;
    }

    log::info!("Seeding database with default data...");

    // Currencies
    for code in &["EUR", "USD", "BYN"] {
        conn.execute(
            "INSERT OR IGNORE INTO currencies (currency_code) VALUES (?1)",
            [code],
        )
        .unwrap();
    }

    // Users
    let alex_tg_id: i64 = env::var("ALEX_TELEGRAM_ID")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1111111111);
    let hanna_tg_id: i64 = env::var("HANNA_TELEGRAM_ID")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2222222222);

    conn.execute(
        "INSERT INTO users (name, telegram_id, is_admin) VALUES ('Alex', ?1, 1)",
        [alex_tg_id],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO users (name, telegram_id, is_admin) VALUES ('Hanna', ?1, 0)",
        [hanna_tg_id],
    )
    .unwrap();

    // Categories
    let categories = [
        "Продукты и хозтовары",
        "Еда вне дома",
        "Кофе и вкусняшки",
        "Развлечения и отдых",
        "Одежда",
        "Здоровье и медицина",
        "Спорт, забота о себе",
        "Образование",
        "Путешествия, туризм",
        "Дети (образование)",
        "Дети (хобби)",
        "Дети (присмотр)",
        "Интернет подписки",
        "Транспорт",
        "Автомобиль",
        "Автомобиль (аренда)",
        "Автомобиль (бензин, паркинг)",
        "Жильё",
        "Жильё (обустройство)",
        "Другое",
    ];
    for (i, name) in categories.iter().enumerate() {
        conn.execute(
            "INSERT OR IGNORE INTO categories (name, sort_order) VALUES (?1, ?2)",
            rusqlite::params![name, i as i64],
        )
        .unwrap();
    }

    // Accounts (all EUR)
    let eur_id: i64 = conn
        .query_row(
            "SELECT id FROM currencies WHERE currency_code = 'EUR'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let alex_id: i64 = conn
        .query_row("SELECT id FROM users WHERE name = 'Alex'", [], |row| {
            row.get(0)
        })
        .unwrap();
    let hanna_id: i64 = conn
        .query_row("SELECT id FROM users WHERE name = 'Hanna'", [], |row| {
            row.get(0)
        })
        .unwrap();

    conn.execute(
        "INSERT INTO accounts (name, currency_id, owner_id, iban) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            "Revolut (Joint)",
            eur_id,
            alex_id,
            "LT00 0000 0000 0000 0000"
        ],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO accounts (name, currency_id, owner_id, iban) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            "Nordea (Spending)",
            eur_id,
            alex_id,
            "FI00 0000 0000 0000 00"
        ],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO accounts (name, currency_id, owner_id, iban) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params!["S-pankki", eur_id, hanna_id, "FI90 1432 3500 6670 50"],
    )
    .unwrap();

    // Set Alex's default account to Revolut (Joint)
    let revolut_id: i64 = conn
        .query_row(
            "SELECT id FROM accounts WHERE name = 'Revolut (Joint)'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    conn.execute(
        "UPDATE users SET default_account_id = ?1 WHERE id = ?2",
        rusqlite::params![revolut_id, alex_id],
    )
    .unwrap();

    log::info!("Seed data inserted successfully.");
}
