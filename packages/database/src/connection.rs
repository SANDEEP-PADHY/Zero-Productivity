use rusqlite::{Connection, Result};
use std::path::Path;

pub fn open_database(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    setup_connection(&conn)?;
    Ok(conn)
}

pub fn open_in_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    setup_connection(&conn)?;
    Ok(conn)
}

fn setup_connection(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(())
}
