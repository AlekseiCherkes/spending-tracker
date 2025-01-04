mod types;
mod schema;

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

        schema::init_scheme(&conn);

        info!("Model created successfully");

        Model { connection: conn }
    }

}
