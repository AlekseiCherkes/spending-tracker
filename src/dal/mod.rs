mod models;
mod queries;
mod seed;

pub use models::*;

use rusqlite::Connection;
use std::sync::{Arc, Mutex};

pub struct Db {
    conn: Arc<Mutex<Connection>>,
}

#[allow(dead_code)]
impl Db {
    pub fn open(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.run_migrations();
        db.seed();
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.run_migrations();
        db.seed();
        Ok(db)
    }

    fn run_migrations(&self) {
        let conn = self.conn.lock().unwrap();
        let version: i64 = conn
            .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
                row.get(0)
            })
            .unwrap_or(0);

        if version < 1 {
            conn.execute_batch(queries::SCHEMA_V1).unwrap();
            conn.execute("INSERT INTO schema_version (version) VALUES (?1)", [1i64])
                .unwrap();
            log::info!("Migrated to schema version 1");
        }
    }

    fn seed(&self) {
        let conn = self.conn.lock().unwrap();
        seed::seed_if_empty(&conn);
    }

    pub fn get_user_by_telegram_id(&self, telegram_id: i64) -> Option<User> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, name, telegram_id, is_admin, default_account_id FROM users WHERE telegram_id = ?1",
            [telegram_id],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    telegram_id: row.get(2)?,
                    is_admin: row.get::<_, i64>(3)? != 0,
                    default_account_id: row.get(4)?,
                })
            },
        )
        .ok()
    }

    pub fn get_all_users(&self) -> Vec<User> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, telegram_id, is_admin, default_account_id FROM users")
            .unwrap();
        stmt.query_map([], |row| {
            Ok(User {
                id: row.get(0)?,
                name: row.get(1)?,
                telegram_id: row.get(2)?,
                is_admin: row.get::<_, i64>(3)? != 0,
                default_account_id: row.get(4)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn update_user_default_account(
        &self,
        user_id: i64,
        account_id: i64,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE users SET default_account_id = ?1 WHERE id = ?2",
            rusqlite::params![account_id, user_id],
        )?;
        Ok(())
    }

    pub fn get_all_accounts(&self) -> Vec<Account> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, currency_id, owner_id, iban FROM accounts")
            .unwrap();
        stmt.query_map([], |row| {
            Ok(Account {
                id: row.get(0)?,
                name: row.get(1)?,
                currency_id: row.get(2)?,
                owner_id: row.get(3)?,
                iban: row.get(4)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn get_account_by_id(&self, id: i64) -> Option<Account> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, name, currency_id, owner_id, iban FROM accounts WHERE id = ?1",
            [id],
            |row| {
                Ok(Account {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    currency_id: row.get(2)?,
                    owner_id: row.get(3)?,
                    iban: row.get(4)?,
                })
            },
        )
        .ok()
    }

    pub fn get_all_categories(&self) -> Vec<Category> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, sort_order FROM categories ORDER BY sort_order")
            .unwrap();
        stmt.query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                sort_order: row.get(2)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn get_category_by_id(&self, id: i64) -> Option<Category> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, name, sort_order FROM categories WHERE id = ?1",
            [id],
            |row| {
                Ok(Category {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    sort_order: row.get(2)?,
                })
            },
        )
        .ok()
    }

    pub fn get_currency_by_id(&self, id: i64) -> Option<Currency> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, currency_code FROM currencies WHERE id = ?1",
            [id],
            |row| {
                Ok(Currency {
                    id: row.get(0)?,
                    currency_code: row.get(1)?,
                })
            },
        )
        .ok()
    }

    pub fn insert_spending(
        &self,
        account_id: i64,
        amount: f64,
        category_id: i64,
        reporter_id: i64,
        notes: Option<&str>,
    ) -> Result<i64, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO spendings (account_id, amount, category_id, reporter_id, notes) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![account_id, amount, category_id, reporter_id, notes],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_spending_created_at(&self, id: i64) -> Option<String> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT created_at FROM spendings WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .ok()
    }
}

impl Clone for Db {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory_and_seed() {
        let db = Db::open_in_memory().unwrap();
        let users = db.get_all_users();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].name, "Alex");
        assert!(users[0].is_admin);
        assert_eq!(users[1].name, "Hanna");
    }

    #[test]
    fn test_get_user_by_telegram_id() {
        let db = Db::open_in_memory().unwrap();
        let user = db.get_user_by_telegram_id(1111111111).unwrap();
        assert_eq!(user.name, "Alex");
        assert!(db.get_user_by_telegram_id(9999999999).is_none());
    }

    #[test]
    fn test_categories_seeded() {
        let db = Db::open_in_memory().unwrap();
        let cats = db.get_all_categories();
        assert_eq!(cats.len(), 20);
        assert_eq!(cats[0].name, "Продукты и хозтовары");
        assert_eq!(cats[19].name, "Другое");
    }

    #[test]
    fn test_accounts_seeded() {
        let db = Db::open_in_memory().unwrap();
        let accounts = db.get_all_accounts();
        assert_eq!(accounts.len(), 3);
    }

    #[test]
    fn test_insert_and_default_account() {
        let db = Db::open_in_memory().unwrap();
        let user = db.get_user_by_telegram_id(1111111111).unwrap();
        assert!(user.default_account_id.is_some());

        let account = db
            .get_account_by_id(user.default_account_id.unwrap())
            .unwrap();
        assert_eq!(account.name, "Revolut (Joint)");
    }

    #[test]
    fn test_insert_spending() {
        let db = Db::open_in_memory().unwrap();
        let user = db.get_user_by_telegram_id(1111111111).unwrap();
        let cats = db.get_all_categories();
        let accounts = db.get_all_accounts();

        let id = db
            .insert_spending(accounts[0].id, 15.50, cats[0].id, user.id, Some("test"))
            .unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_currency() {
        let db = Db::open_in_memory().unwrap();
        let accounts = db.get_all_accounts();
        let currency = db.get_currency_by_id(accounts[0].currency_id).unwrap();
        assert_eq!(currency.currency_code, "EUR");
    }

    #[test]
    fn test_seed_idempotent() {
        let db = Db::open_in_memory().unwrap();
        // Seed runs in open_in_memory, calling it again should be no-op
        {
            let conn = db.conn.lock().unwrap();
            seed::seed_if_empty(&conn);
        }
        assert_eq!(db.get_all_users().len(), 2);
    }
}
