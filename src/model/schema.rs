use log::*;

pub(crate) fn init_scheme(conn: &rusqlite::Connection) {
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

    conn.execute(
        "CREATE TABLE person (
                id   INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                data BLOB
            )",
        (),
    )
        .unwrap();

    conn.pragma_update(None, "user_version", 1).unwrap();
}
