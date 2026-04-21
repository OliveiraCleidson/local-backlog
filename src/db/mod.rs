pub mod migrations;
pub mod repo;

use std::path::Path;

use rusqlite::Connection;

use crate::error::BacklogError;

/// Abre uma conexão SQLite no caminho dado, aplica todas as migrations e
/// garante `PRAGMA foreign_keys = ON`.
pub fn open(path: &Path) -> Result<Connection, BacklogError> {
    let mut conn = Connection::open(path)?;
    configure(&mut conn)?;
    migrations::runner().to_latest(&mut conn)?;
    Ok(conn)
}

/// Abre uma conexão em memória com as migrations aplicadas. Usada por testes.
pub fn open_in_memory() -> Result<Connection, BacklogError> {
    let mut conn = Connection::open_in_memory()?;
    configure(&mut conn)?;
    migrations::runner().to_latest(&mut conn)?;
    Ok(conn)
}

fn configure(conn: &mut Connection) -> Result<(), BacklogError> {
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    Ok(())
}
