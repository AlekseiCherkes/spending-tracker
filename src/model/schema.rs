use log::*;

const SCHEMA_V1: &str =
"
CREATE TABLE Currency (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE Account (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    displayName TEXT,
    currencyId INTEGER NOT NULL,
    FOREIGN KEY (currencyId) REFERENCES Currency (id) ON DELETE RESTRICT
);

CREATE TABLE User (
    telegramId INTEGER PRIMARY KEY,
    telegramName TEXT NOT NULL,
    displayName TEXT NOT NULL
);

CREATE TABLE ExpenseCategory (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT 1,
    comments TEXT,
    sortingOrder INTEGER
);

CREATE TABLE Expense (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    accountId INTEGER NOT NULL,
    categoryId INTEGER NOT NULL,
    userId INTEGER NOT NULL,
    timestamp INTEGER NOT NULL,
    amount REAL NOT NULL,
    comments TEXT,
    FOREIGN KEY (accountId) REFERENCES Account (id) ON DELETE RESTRICT,
    FOREIGN KEY (categoryId) REFERENCES ExpenseCategory (id) ON DELETE RESTRICT,
    FOREIGN KEY (userId) REFERENCES User (telegramId) ON DELETE RESTRICT
);
";

pub(super) fn init_schema(conn: &rusqlite::Connection) {
    info!("Initialising schema...");
    let version: i32 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();

    info!("Current version: {}", version);

    match version {
        0 => {
            init_schema_v1(&conn);
        }
        1 => {
            // schema is up to date, do nothing
        }
        _ => {
            panic!("Unsupported schema version: {}", version);
        }
    }
}

pub(super) fn init_schema_v1(conn: &rusqlite::Connection) {
    info!("Initializing schema v1");
    conn.execute_batch(SCHEMA_V1).unwrap();
    conn.pragma_update(None, "user_version", 1).unwrap();
}

pub(super) fn fill_test_data(conn: &rusqlite::Connection) {
    conn.execute_batch(super::test_data::TEST_DATA).unwrap()
}
