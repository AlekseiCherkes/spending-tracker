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

    pub fn get_all_currencies(&self) -> Vec<Currency> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, currency_code FROM currencies ORDER BY id")
            .unwrap();
        stmt.query_map([], |row| {
            Ok(Currency {
                id: row.get(0)?,
                currency_code: row.get(1)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
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

    pub fn get_spending_by_id(&self, id: i64) -> Option<Spending> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, account_id, amount, category_id, reporter_id, notes, created_at FROM spendings WHERE id = ?1",
            [id],
            |row| {
                Ok(Spending {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    amount: row.get(2)?,
                    category_id: row.get(3)?,
                    reporter_id: row.get(4)?,
                    notes: row.get(5)?,
                    created_at: row.get(6)?,
                })
            },
        )
        .ok()
    }

    pub fn update_spending(
        &self,
        id: i64,
        account_id: i64,
        amount: f64,
        category_id: i64,
        notes: Option<&str>,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE spendings SET account_id = ?1, amount = ?2, category_id = ?3, notes = ?4 WHERE id = ?5",
            rusqlite::params![account_id, amount, category_id, notes, id],
        )?;
        Ok(())
    }

    pub fn delete_spending(&self, id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM spendings WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Returns "YYYY-MM" for the current UTC month (matches the format of stored `created_at`).
    pub fn current_year_month_utc(&self) -> String {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT strftime('%Y-%m', 'now')", [], |r| r.get(0))
            .unwrap_or_else(|_| "1970-01".to_string())
    }

    /// All spendings whose `created_at` starts with `year_month` (e.g. "2026-05"),
    /// ordered chronologically (oldest first). Joined with account/currency/category/user.
    pub fn get_spendings_in_month(&self, year_month: &str) -> Vec<RecentSpending> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("{}%", year_month);
        let mut stmt = conn
            .prepare(
                "SELECT s.id, s.amount, c.currency_code, a.name, a.iban, cat.name, u.name, s.notes, s.created_at
                 FROM spendings s
                 JOIN accounts a ON s.account_id = a.id
                 JOIN currencies c ON a.currency_id = c.id
                 JOIN categories cat ON s.category_id = cat.id
                 JOIN users u ON s.reporter_id = u.id
                 WHERE s.created_at LIKE ?1
                 ORDER BY s.created_at ASC, s.id ASC",
            )
            .unwrap();
        stmt.query_map([pattern], |row| {
            Ok(RecentSpending {
                id: row.get(0)?,
                amount: row.get(1)?,
                currency_code: row.get(2)?,
                account_name: row.get(3)?,
                account_iban: row.get(4)?,
                category_name: row.get(5)?,
                reporter_name: row.get(6)?,
                notes: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn get_recent_spendings(&self, limit: i64) -> Vec<RecentSpending> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT s.id, s.amount, c.currency_code, a.name, a.iban, cat.name, u.name, s.notes, s.created_at
                 FROM spendings s
                 JOIN accounts a ON s.account_id = a.id
                 JOIN currencies c ON a.currency_id = c.id
                 JOIN categories cat ON s.category_id = cat.id
                 JOIN users u ON s.reporter_id = u.id
                 ORDER BY s.id DESC
                 LIMIT ?1",
            )
            .unwrap();
        stmt.query_map([limit], |row| {
            Ok(RecentSpending {
                id: row.get(0)?,
                amount: row.get(1)?,
                currency_code: row.get(2)?,
                account_name: row.get(3)?,
                account_iban: row.get(4)?,
                category_name: row.get(5)?,
                reporter_name: row.get(6)?,
                notes: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
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
    fn test_get_all_currencies() {
        let db = Db::open_in_memory().unwrap();
        let codes: Vec<String> = db
            .get_all_currencies()
            .into_iter()
            .map(|c| c.currency_code)
            .collect();
        assert_eq!(codes, vec!["EUR", "USD", "BYN"]);
    }

    #[test]
    fn test_get_recent_spendings_newest_first_with_join() {
        let db = Db::open_in_memory().unwrap();
        let user = db.get_user_by_telegram_id(1111111111).unwrap();
        let cats = db.get_all_categories();
        let accounts = db.get_all_accounts();

        db.insert_spending(accounts[0].id, 1.0, cats[0].id, user.id, Some("first"))
            .unwrap();
        db.insert_spending(accounts[0].id, 2.0, cats[1].id, user.id, None)
            .unwrap();
        db.insert_spending(accounts[0].id, 3.0, cats[0].id, user.id, Some("third"))
            .unwrap();

        let recent = db.get_recent_spendings(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].amount, 3.0);
        assert_eq!(recent[0].notes.as_deref(), Some("third"));
        assert_eq!(recent[0].reporter_name, "Alex");
        assert_eq!(recent[0].currency_code, "EUR");
        assert_eq!(recent[0].account_name, accounts[0].name);
        assert_eq!(recent[0].category_name, cats[0].name);
        assert_eq!(recent[1].amount, 2.0);
        assert!(recent[1].notes.is_none());
    }

    #[test]
    fn test_get_spendings_in_month_includes_account_iban() {
        let db = Db::open_in_memory().unwrap();
        let user = db.get_user_by_telegram_id(1111111111).unwrap();
        let cats = db.get_all_categories();
        let currency_id = db.get_all_currencies()[0].id;

        // Create an account with a known IBAN to verify it surfaces in the joined query.
        let acc_id = {
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO accounts (name, currency_id, owner_id, iban) VALUES ('Test', ?1, ?2, 'LT00 1234')",
                rusqlite::params![currency_id, user.id],
            )
            .unwrap();
            conn.last_insert_rowid()
        };
        let id = db
            .insert_spending(acc_id, 1.0, cats[0].id, user.id, None)
            .unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "UPDATE spendings SET created_at = '2026-05-10T00:00:00' WHERE id = ?1",
                [id],
            )
            .unwrap();
        }
        let rows = db.get_spendings_in_month("2026-05");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].account_name, "Test");
        assert_eq!(rows[0].account_iban.as_deref(), Some("LT00 1234"));
    }

    #[test]
    fn test_get_spendings_in_month_filters_and_orders() {
        let db = Db::open_in_memory().unwrap();
        let user = db.get_user_by_telegram_id(1111111111).unwrap();
        let cats = db.get_all_categories();
        let accounts = db.get_all_accounts();

        // Insert spendings then rewrite their created_at to known months.
        let id_a = db
            .insert_spending(accounts[0].id, 1.0, cats[0].id, user.id, Some("apr1"))
            .unwrap();
        let id_b = db
            .insert_spending(accounts[0].id, 2.0, cats[0].id, user.id, Some("may1"))
            .unwrap();
        let id_c = db
            .insert_spending(accounts[0].id, 3.0, cats[0].id, user.id, Some("may2"))
            .unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "UPDATE spendings SET created_at = ?1 WHERE id = ?2",
                rusqlite::params!["2026-04-15T10:00:00", id_a],
            )
            .unwrap();
            conn.execute(
                "UPDATE spendings SET created_at = ?1 WHERE id = ?2",
                rusqlite::params!["2026-05-01T08:00:00", id_b],
            )
            .unwrap();
            conn.execute(
                "UPDATE spendings SET created_at = ?1 WHERE id = ?2",
                rusqlite::params!["2026-05-20T22:30:00", id_c],
            )
            .unwrap();
        }

        let may = db.get_spendings_in_month("2026-05");
        assert_eq!(may.len(), 2);
        // Chronological order (oldest first).
        assert_eq!(may[0].notes.as_deref(), Some("may1"));
        assert_eq!(may[1].notes.as_deref(), Some("may2"));

        let apr = db.get_spendings_in_month("2026-04");
        assert_eq!(apr.len(), 1);
        assert_eq!(apr[0].notes.as_deref(), Some("apr1"));

        let mar = db.get_spendings_in_month("2026-03");
        assert!(mar.is_empty());
    }

    #[test]
    fn test_current_year_month_utc_format() {
        let db = Db::open_in_memory().unwrap();
        let ym = db.current_year_month_utc();
        // Format must be "YYYY-MM" so callers can do string filtering against created_at.
        assert_eq!(ym.len(), 7);
        assert_eq!(ym.chars().nth(4), Some('-'));
        assert!(ym[..4].chars().all(|c| c.is_ascii_digit()));
        assert!(ym[5..].chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_get_recent_spendings_empty() {
        let db = Db::open_in_memory().unwrap();
        assert!(db.get_recent_spendings(25).is_empty());
    }

    #[test]
    fn test_get_spending_by_id() {
        let db = Db::open_in_memory().unwrap();
        let user = db.get_user_by_telegram_id(1111111111).unwrap();
        let cats = db.get_all_categories();
        let accounts = db.get_all_accounts();

        let id = db
            .insert_spending(accounts[0].id, 7.25, cats[0].id, user.id, Some("note"))
            .unwrap();
        let s = db.get_spending_by_id(id).unwrap();
        assert_eq!(s.amount, 7.25);
        assert_eq!(s.category_id, cats[0].id);
        assert_eq!(s.account_id, accounts[0].id);
        assert_eq!(s.notes.as_deref(), Some("note"));
        assert!(db.get_spending_by_id(99_999).is_none());
    }

    #[test]
    fn test_update_spending() {
        let db = Db::open_in_memory().unwrap();
        let user = db.get_user_by_telegram_id(1111111111).unwrap();
        let cats = db.get_all_categories();
        let accounts = db.get_all_accounts();

        let id = db
            .insert_spending(accounts[0].id, 5.0, cats[0].id, user.id, Some("old"))
            .unwrap();
        db.update_spending(id, accounts[1].id, 9.99, cats[1].id, None)
            .unwrap();
        let s = db.get_spending_by_id(id).unwrap();
        assert_eq!(s.amount, 9.99);
        assert_eq!(s.account_id, accounts[1].id);
        assert_eq!(s.category_id, cats[1].id);
        assert!(s.notes.is_none());
        // reporter is not changed by update
        assert_eq!(s.reporter_id, user.id);
    }

    #[test]
    fn test_delete_spending() {
        let db = Db::open_in_memory().unwrap();
        let user = db.get_user_by_telegram_id(1111111111).unwrap();
        let cats = db.get_all_categories();
        let accounts = db.get_all_accounts();

        let id = db
            .insert_spending(accounts[0].id, 5.0, cats[0].id, user.id, None)
            .unwrap();
        assert!(db.get_spending_by_id(id).is_some());
        db.delete_spending(id).unwrap();
        assert!(db.get_spending_by_id(id).is_none());
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
