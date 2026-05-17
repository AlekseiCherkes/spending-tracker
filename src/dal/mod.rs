mod models;
mod queries;
#[cfg(test)]
mod seed;

pub use models::*;

use models::{
    row_to_account, row_to_category, row_to_currency, row_to_recent_spending, row_to_spending,
    row_to_user, ACCOUNT_COLS, CATEGORY_COLS, CURRENCY_COLS, RECENT_SPENDING_SELECT, SPENDING_COLS,
    USER_COLS,
};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

/// Map a rusqlite Result to Option, logging unexpected errors at warn level.
/// `QueryReturnedNoRows` is the legitimate "not found" case and is silent so
/// the cron log-grep alert doesn't fire for normal lookups.
fn ok_or_log<T>(op: &str, r: rusqlite::Result<T>) -> Option<T> {
    match r {
        Ok(v) => Some(v),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(e) => {
            log::warn!("DB error in {}: {}", op, e);
            None
        }
    }
}

pub struct Db {
    conn: Arc<Mutex<Connection>>,
}

impl Db {
    pub fn open(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.run_migrations();
        Ok(db)
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.run_migrations();
        {
            let c = db.conn.lock().unwrap();
            seed::seed_if_empty(&c);
        }
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
            conn.execute(
                "INSERT OR REPLACE INTO schema_version (rowid, version) VALUES (1, ?1)",
                [1i64],
            )
            .unwrap();
            log::info!("Migrated to schema version 1");
        }
    }

    pub fn get_user_by_telegram_id(&self, telegram_id: i64) -> Option<User> {
        let conn = self.conn.lock().unwrap();
        ok_or_log(
            "get_user_by_telegram_id",
            conn.query_row(
                &format!("SELECT {USER_COLS} FROM users WHERE telegram_id = ?1"),
                [telegram_id],
                row_to_user,
            ),
        )
    }

    pub fn get_all_users(&self) -> Vec<User> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(&format!("SELECT {USER_COLS} FROM users"))
            .unwrap();
        stmt.query_map([], row_to_user)
            .unwrap()
            .filter_map(|r| ok_or_log("get_all_users row", r))
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
            .prepare(&format!("SELECT {ACCOUNT_COLS} FROM accounts"))
            .unwrap();
        stmt.query_map([], row_to_account)
            .unwrap()
            .filter_map(|r| ok_or_log("get_all_accounts row", r))
            .collect()
    }

    pub fn get_account_by_id(&self, id: i64) -> Option<Account> {
        let conn = self.conn.lock().unwrap();
        ok_or_log(
            "get_account_by_id",
            conn.query_row(
                &format!("SELECT {ACCOUNT_COLS} FROM accounts WHERE id = ?1"),
                [id],
                row_to_account,
            ),
        )
    }

    pub fn get_all_categories(&self) -> Vec<Category> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(&format!(
                "SELECT {CATEGORY_COLS} FROM categories ORDER BY sort_order"
            ))
            .unwrap();
        stmt.query_map([], row_to_category)
            .unwrap()
            .filter_map(|r| ok_or_log("get_all_categories row", r))
            .collect()
    }

    pub fn get_category_by_id(&self, id: i64) -> Option<Category> {
        let conn = self.conn.lock().unwrap();
        ok_or_log(
            "get_category_by_id",
            conn.query_row(
                &format!("SELECT {CATEGORY_COLS} FROM categories WHERE id = ?1"),
                [id],
                row_to_category,
            ),
        )
    }

    pub fn get_all_currencies(&self) -> Vec<Currency> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(&format!(
                "SELECT {CURRENCY_COLS} FROM currencies ORDER BY id"
            ))
            .unwrap();
        stmt.query_map([], row_to_currency)
            .unwrap()
            .filter_map(|r| ok_or_log("get_all_currencies row", r))
            .collect()
    }

    pub fn get_currency_by_id(&self, id: i64) -> Option<Currency> {
        let conn = self.conn.lock().unwrap();
        ok_or_log(
            "get_currency_by_id",
            conn.query_row(
                &format!("SELECT {CURRENCY_COLS} FROM currencies WHERE id = ?1"),
                [id],
                row_to_currency,
            ),
        )
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
        ok_or_log(
            "get_spending_by_id",
            conn.query_row(
                &format!("SELECT {SPENDING_COLS} FROM spendings WHERE id = ?1"),
                [id],
                row_to_spending,
            ),
        )
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

    /// Returns "YYYY-MM" for the current local-time month (matches the format
    /// and timezone of stored `created_at`).
    pub fn current_year_month(&self) -> String {
        let conn = self.conn.lock().unwrap();
        let r = conn.query_row("SELECT strftime('%Y-%m', 'now', 'localtime')", [], |r| {
            r.get(0)
        });
        ok_or_log("current_year_month", r).unwrap_or_else(|| "1970-01".to_string())
    }

    /// All spendings whose local-time `created_at` is in `year_month`
    /// (e.g. "2026-05"), ordered chronologically (oldest first). Joined with
    /// account/currency/category/user.
    pub fn get_spendings_in_month(&self, year_month: &str) -> Vec<RecentSpending> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(&format!(
                "{RECENT_SPENDING_SELECT} \
                 WHERE strftime('%Y-%m', s.created_at, 'localtime') = ?1 \
                 ORDER BY s.created_at ASC, s.id ASC"
            ))
            .unwrap();
        stmt.query_map([year_month], row_to_recent_spending)
            .unwrap()
            .filter_map(|r| ok_or_log("get_spendings_in_month row", r))
            .collect()
    }

    pub fn get_recent_spendings(&self, limit: i64) -> Vec<RecentSpending> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(&format!(
                "{RECENT_SPENDING_SELECT} ORDER BY s.id DESC LIMIT ?1"
            ))
            .unwrap();
        stmt.query_map([limit], row_to_recent_spending)
            .unwrap()
            .filter_map(|r| ok_or_log("get_recent_spendings row", r))
            .collect()
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
        assert_eq!(users[0].name, "Alice");
        assert!(users[0].is_admin);
        assert_eq!(users[1].name, "Bob");
    }

    #[test]
    fn test_get_user_by_telegram_id() {
        let db = Db::open_in_memory().unwrap();
        let user = db.get_user_by_telegram_id(seed::ALICE_TELEGRAM_ID).unwrap();
        assert_eq!(user.name, "Alice");
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
        let user = db.get_user_by_telegram_id(seed::ALICE_TELEGRAM_ID).unwrap();
        assert!(user.default_account_id.is_some());

        let account = db
            .get_account_by_id(user.default_account_id.unwrap())
            .unwrap();
        assert_eq!(account.name, "Account A");
    }

    #[test]
    fn test_insert_spending() {
        let db = Db::open_in_memory().unwrap();
        let user = db.get_user_by_telegram_id(seed::ALICE_TELEGRAM_ID).unwrap();
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
        let user = db.get_user_by_telegram_id(seed::ALICE_TELEGRAM_ID).unwrap();
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
        assert_eq!(recent[0].reporter_name, "Alice");
        assert_eq!(recent[0].currency_code, "EUR");
        assert_eq!(recent[0].account_name, accounts[0].name);
        assert_eq!(recent[0].category_name, cats[0].name);
        assert_eq!(recent[1].amount, 2.0);
        assert!(recent[1].notes.is_none());
    }

    #[test]
    fn test_get_spendings_in_month_includes_account_iban() {
        let db = Db::open_in_memory().unwrap();
        let user = db.get_user_by_telegram_id(seed::ALICE_TELEGRAM_ID).unwrap();
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
        let user = db.get_user_by_telegram_id(seed::ALICE_TELEGRAM_ID).unwrap();
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
    fn test_current_year_month_format() {
        let db = Db::open_in_memory().unwrap();
        let ym = db.current_year_month();
        // Format must be "YYYY-MM" so callers can do string filtering against created_at.
        assert_eq!(ym.len(), 7);
        assert_eq!(ym.chars().nth(4), Some('-'));
        assert!(ym[..4].chars().all(|c| c.is_ascii_digit()));
        assert!(ym[5..].chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_get_spending_by_id() {
        let db = Db::open_in_memory().unwrap();
        let user = db.get_user_by_telegram_id(seed::ALICE_TELEGRAM_ID).unwrap();
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
        let user = db.get_user_by_telegram_id(seed::ALICE_TELEGRAM_ID).unwrap();
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
        let user = db.get_user_by_telegram_id(seed::ALICE_TELEGRAM_ID).unwrap();
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
