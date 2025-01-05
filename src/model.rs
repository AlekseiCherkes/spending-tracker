mod schema;
mod test_data;

use log::*;
use rusqlite;
use std::path;

pub struct Model {
    connection: rusqlite::Connection,
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
}

#[cfg(test)]
mod tests {
    use crate::model::Model;

    #[test]
    fn test_1() {
        let model = Model::new(true);
        model.fill_test_data();
        assert_eq!(6, 6);
    }
}
