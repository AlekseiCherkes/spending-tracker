pub const SCHEMA_V1: &str = "
PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL);

CREATE TABLE IF NOT EXISTS currencies (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    currency_code TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS users (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    name               TEXT NOT NULL,
    telegram_id        INTEGER NOT NULL UNIQUE,
    is_admin           INTEGER NOT NULL DEFAULT 0,
    default_account_id INTEGER,
    FOREIGN KEY (default_account_id) REFERENCES accounts(id)
);

CREATE TABLE IF NOT EXISTS accounts (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    currency_id INTEGER NOT NULL,
    owner_id    INTEGER,
    iban        TEXT,
    FOREIGN KEY (currency_id) REFERENCES currencies(id),
    FOREIGN KEY (owner_id) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS categories (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL UNIQUE,
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS spendings (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id  INTEGER NOT NULL,
    amount      REAL NOT NULL,
    category_id INTEGER NOT NULL,
    reporter_id INTEGER NOT NULL,
    notes       TEXT,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S','now')),
    FOREIGN KEY (account_id) REFERENCES accounts(id),
    FOREIGN KEY (category_id) REFERENCES categories(id),
    FOREIGN KEY (reporter_id) REFERENCES users(id)
);
";
