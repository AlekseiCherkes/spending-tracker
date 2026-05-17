use rusqlite::Row;

pub struct User {
    pub id: i64,
    pub name: String,
    #[allow(dead_code)]
    pub telegram_id: i64,
    pub is_admin: bool,
    pub default_account_id: Option<i64>,
}

pub struct Account {
    pub id: i64,
    pub name: String,
    pub currency_id: i64,
    pub owner_id: Option<i64>,
    pub iban: Option<String>,
}

pub struct Category {
    pub id: i64,
    pub name: String,
    #[allow(dead_code)]
    pub sort_order: i64,
}

pub struct Currency {
    pub id: i64,
    pub currency_code: String,
}

pub struct Spending {
    pub id: i64,
    pub account_id: i64,
    pub amount: f64,
    pub category_id: i64,
    pub reporter_id: i64,
    pub notes: Option<String>,
    #[allow(dead_code)]
    pub created_at: String,
}

pub struct RecentSpending {
    pub id: i64,
    pub amount: f64,
    pub currency_code: String,
    pub account_name: String,
    pub account_iban: Option<String>,
    pub category_name: String,
    pub reporter_name: String,
    pub notes: Option<String>,
    pub created_at: String,
}

// SELECT-list constants and row→struct mappers. Each `*_COLS` constant must list
// columns in the exact order the matching `row_to_*` function reads them.

pub(super) const USER_COLS: &str = "id, name, telegram_id, is_admin, default_account_id";
pub(super) const ACCOUNT_COLS: &str = "id, name, currency_id, owner_id, iban";
pub(super) const CATEGORY_COLS: &str = "id, name, sort_order";
pub(super) const CURRENCY_COLS: &str = "id, currency_code";
pub(super) const SPENDING_COLS: &str =
    "id, account_id, amount, category_id, reporter_id, notes, created_at";

/// SELECT + FROM + JOINs for `RecentSpending`. Append a WHERE / ORDER BY / LIMIT
/// at the call site.
pub(super) const RECENT_SPENDING_SELECT: &str = "\
    SELECT s.id, s.amount, c.currency_code, a.name, a.iban, cat.name, u.name, s.notes, s.created_at \
    FROM spendings s \
    JOIN accounts a ON s.account_id = a.id \
    JOIN currencies c ON a.currency_id = c.id \
    JOIN categories cat ON s.category_id = cat.id \
    JOIN users u ON s.reporter_id = u.id";

pub(super) fn row_to_user(row: &Row) -> rusqlite::Result<User> {
    Ok(User {
        id: row.get(0)?,
        name: row.get(1)?,
        telegram_id: row.get(2)?,
        is_admin: row.get::<_, i64>(3)? != 0,
        default_account_id: row.get(4)?,
    })
}

pub(super) fn row_to_account(row: &Row) -> rusqlite::Result<Account> {
    Ok(Account {
        id: row.get(0)?,
        name: row.get(1)?,
        currency_id: row.get(2)?,
        owner_id: row.get(3)?,
        iban: row.get(4)?,
    })
}

pub(super) fn row_to_category(row: &Row) -> rusqlite::Result<Category> {
    Ok(Category {
        id: row.get(0)?,
        name: row.get(1)?,
        sort_order: row.get(2)?,
    })
}

pub(super) fn row_to_currency(row: &Row) -> rusqlite::Result<Currency> {
    Ok(Currency {
        id: row.get(0)?,
        currency_code: row.get(1)?,
    })
}

pub(super) fn row_to_spending(row: &Row) -> rusqlite::Result<Spending> {
    Ok(Spending {
        id: row.get(0)?,
        account_id: row.get(1)?,
        amount: row.get(2)?,
        category_id: row.get(3)?,
        reporter_id: row.get(4)?,
        notes: row.get(5)?,
        created_at: row.get(6)?,
    })
}

pub(super) fn row_to_recent_spending(row: &Row) -> rusqlite::Result<RecentSpending> {
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
}
