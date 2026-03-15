#[allow(dead_code)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub telegram_id: i64,
    pub is_admin: bool,
    pub default_account_id: Option<i64>,
}

#[allow(dead_code)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub currency_id: i64,
    pub owner_id: Option<i64>,
    pub iban: Option<String>,
}

#[allow(dead_code)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub sort_order: i64,
}

#[allow(dead_code)]
pub struct Currency {
    pub id: i64,
    pub currency_code: String,
}

#[allow(dead_code)]
pub struct Spending {
    pub id: i64,
    pub account_id: i64,
    pub amount: f64,
    pub category_id: i64,
    pub reporter_id: i64,
    pub notes: Option<String>,
    pub created_at: String,
}
