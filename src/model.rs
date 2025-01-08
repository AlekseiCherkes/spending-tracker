mod schema;
mod test_data;

use log::*;
use rusqlite;
use std::path;

pub struct Model {
    connection: rusqlite::Connection,
}

#[derive(Clone)]
pub struct ActiveTransaction {
    pub amount: f32,                              // const
    pub comments: String,                         // const
    pub user_name: String,                        // const
    pub timestamp: chrono::DateTime<chrono::Utc>, // const

    pub account_info: AccountInfo,
    pub category_info: CategoryInfo,
}

#[derive(Clone)]
pub struct AccountInfo {
    pub id: u64,
    pub display_name: String,
}

#[derive(Clone)]
pub struct CategoryInfo {
    pub id: u64,
    pub display_name: String,
}

impl Model {
    pub fn new(in_memory: bool) -> Model {
        info!("Creating model...");

        let conn = if in_memory {
            info!("Use in memory connection");
            rusqlite::Connection::open_in_memory().unwrap()
        } else {
            let path = path::absolute("./spending-tracker.db").unwrap();
            info!("Use file: {}", path.display());
            rusqlite::Connection::open(path).unwrap()
        };

        conn.pragma_update(None, "foreign_keys", "ON").unwrap();

        schema::init_schema(&conn);

        info!("Model created successfully");

        Model { connection: conn }
    }

    //#[cfg(test)]
    pub fn fill_test_data(&self) {
        schema::fill_test_data(&self.connection);
    }

    pub fn make_active_transaction(&self) -> ActiveTransaction {
        ActiveTransaction {
            amount: 123.0,
            comments: String::from("My comments"),
            user_name: String::from("My user"),
            timestamp: chrono::Utc::now(),
            account_info: self.get_accounts().get(0).unwrap().clone(),
            category_info: self.get_categories().get(0).unwrap().clone(),
        }
    }

    pub fn get_accounts(&self) -> Vec<AccountInfo> {
        vec![
            AccountInfo {
                id: 1,
                display_name: String::from("User1"),
            },
            AccountInfo {
                id: 2,
                display_name: String::from("User2"),
            },
            AccountInfo {
                id: 3,
                display_name: String::from("User3"),
            },
        ]
    }

    pub fn get_account_info(&self, id: u64) -> Option<AccountInfo> {
        let accounts = self.get_accounts();
        accounts.into_iter().find(|&a| a.id == id)
    }

    pub fn get_categories(&self) -> Vec<CategoryInfo> {
        vec![
            CategoryInfo {
                id: 1,
                display_name: String::from("Category 1"),
            },
            CategoryInfo {
                id: 2,
                display_name: String::from("Category 2"),
            },
            CategoryInfo {
                id: 3,
                display_name: String::from("Category 3"),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::model::Model;

    #[test]
    fn test_1() {
        let model = Model::new(true);
        model.fill_test_data();

        let accounts = model.get_accounts();
        assert!(accounts.len() > 0);

        let categories = model.get_categories();
        assert!(categories.len() > 0);
    }

    #[test]
    fn test_2() {
        let model = Model::new(true);
        model.fill_test_data();

        let transaction = model.make_active_transaction();
        assert_eq!(transaction.user_name, String::from("My user"));
    }
}
