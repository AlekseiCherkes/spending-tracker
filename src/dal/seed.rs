//! Test-only fixture data. Not compiled into the production binary —
//! the prod database starts empty and is populated by hand once.

use rusqlite::Connection;

pub const ALICE_TELEGRAM_ID: i64 = 1111111111;
pub const BOB_TELEGRAM_ID: i64 = 2222222222;

pub fn seed_if_empty(conn: &Connection) {
    let user_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
        .unwrap_or(0);
    if user_count > 0 {
        return;
    }

    for code in &["EUR", "USD", "BYN"] {
        conn.execute(
            "INSERT OR IGNORE INTO currencies (currency_code) VALUES (?1)",
            [code],
        )
        .unwrap();
    }

    conn.execute(
        "INSERT INTO users (name, telegram_id, is_admin) VALUES ('Alice', ?1, 1)",
        [ALICE_TELEGRAM_ID],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO users (name, telegram_id, is_admin) VALUES ('Bob', ?1, 0)",
        [BOB_TELEGRAM_ID],
    )
    .unwrap();

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

    let eur_id: i64 = conn
        .query_row(
            "SELECT id FROM currencies WHERE currency_code = 'EUR'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let alice_id: i64 = conn
        .query_row("SELECT id FROM users WHERE name = 'Alice'", [], |row| {
            row.get(0)
        })
        .unwrap();
    let bob_id: i64 = conn
        .query_row("SELECT id FROM users WHERE name = 'Bob'", [], |row| {
            row.get(0)
        })
        .unwrap();

    for (name, owner_id, iban) in [
        ("Account A", alice_id, "TEST00 0000 0000 0000 0001"),
        ("Account B", alice_id, "TEST00 0000 0000 0000 0002"),
        ("Account C", bob_id, "TEST00 0000 0000 0000 0003"),
    ] {
        conn.execute(
            "INSERT INTO accounts (name, currency_id, owner_id, iban) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![name, eur_id, owner_id, iban],
        )
        .unwrap();
    }

    let acc_a_id: i64 = conn
        .query_row(
            "SELECT id FROM accounts WHERE name = 'Account A'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let acc_b_id: i64 = conn
        .query_row(
            "SELECT id FROM accounts WHERE name = 'Account B'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let acc_c_id: i64 = conn
        .query_row(
            "SELECT id FROM accounts WHERE name = 'Account C'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    conn.execute(
        "UPDATE users SET default_account_id = ?1 WHERE id = ?2",
        rusqlite::params![acc_a_id, alice_id],
    )
    .unwrap();

    // Sample spendings. Dated in 2025 so they don't collide with tests that
    // assert on 2026-dated rows. Spread across accounts/categories/reporters.
    let cat_id = |name: &str| -> i64 {
        conn.query_row("SELECT id FROM categories WHERE name = ?1", [name], |row| {
            row.get(0)
        })
        .unwrap()
    };

    let spendings: [(i64, f64, i64, i64, Option<&str>, &str); 10] = [
        (
            acc_a_id,
            12.50,
            cat_id("Продукты и хозтовары"),
            alice_id,
            Some("milk and bread"),
            "2025-01-05T10:00:00",
        ),
        (
            acc_a_id,
            35.00,
            cat_id("Еда вне дома"),
            alice_id,
            Some("lunch"),
            "2025-01-15T13:30:00",
        ),
        (
            acc_b_id,
            4.50,
            cat_id("Кофе и вкусняшки"),
            alice_id,
            None,
            "2025-02-03T09:15:00",
        ),
        (
            acc_c_id,
            89.00,
            cat_id("Одежда"),
            bob_id,
            Some("shoes"),
            "2025-02-14T16:00:00",
        ),
        (
            acc_a_id,
            22.00,
            cat_id("Здоровье и медицина"),
            alice_id,
            Some("pharmacy"),
            "2025-03-20T18:00:00",
        ),
        (
            acc_c_id,
            60.00,
            cat_id("Развлечения и отдых"),
            bob_id,
            Some("cinema"),
            "2025-03-25T20:30:00",
        ),
        (
            acc_b_id,
            100.00,
            cat_id("Жильё"),
            alice_id,
            Some("rent share"),
            "2025-04-01T08:00:00",
        ),
        (
            acc_a_id,
            15.75,
            cat_id("Транспорт"),
            alice_id,
            None,
            "2025-04-15T11:00:00",
        ),
        (
            acc_c_id,
            45.00,
            cat_id("Дети (хобби)"),
            bob_id,
            Some("swimming class"),
            "2025-05-10T14:00:00",
        ),
        (
            acc_a_id,
            8.50,
            cat_id("Кофе и вкусняшки"),
            alice_id,
            Some("coffee"),
            "2025-05-12T08:30:00",
        ),
    ];

    for (account_id, amount, category_id, reporter_id, notes, created_at) in spendings {
        conn.execute(
            "INSERT INTO spendings (account_id, amount, category_id, reporter_id, notes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![account_id, amount, category_id, reporter_id, notes, created_at],
        )
        .unwrap();
    }
}
