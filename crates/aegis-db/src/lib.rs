use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::Path;
use thiserror::Error;

const INITIAL_MIGRATION: &str = include_str!("../../../migrations/001_initial.sql");
const INITIAL_MIGRATION_VERSION: i64 = 1;
const INITIAL_MIGRATION_NAME: &str = "001_initial";

#[derive(Debug, Error)]
pub enum DbError {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}

pub fn open_database(path: impl AsRef<Path>) -> Result<Connection, DbError> {
    let conn = Connection::open(path)?;
    configure_connection(&conn)?;
    Ok(conn)
}

pub fn open_in_memory_database() -> Result<Connection, DbError> {
    let conn = Connection::open_in_memory()?;
    configure_connection(&conn)?;
    Ok(conn)
}

pub fn apply_migrations(conn: &mut Connection) -> Result<(), DbError> {
    let tx = conn.transaction()?;
    tx.execute_batch(INITIAL_MIGRATION)?;
    tx.execute(
        "INSERT OR IGNORE INTO schema_migrations(version, name, applied_at_utc) VALUES (?1, ?2, ?3)",
        params![INITIAL_MIGRATION_VERSION, INITIAL_MIGRATION_NAME, Utc::now().to_rfc3339()],
    )?;
    tx.commit()?;
    Ok(())
}

fn configure_connection(conn: &Connection) -> Result<(), DbError> {
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_initial_schema() {
        let mut conn = open_in_memory_database().expect("open in-memory database");
        apply_migrations(&mut conn).expect("apply migrations");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_migrations WHERE version = 1", [], |row| row.get(0))
            .expect("read migration row");
        assert_eq!(count, 1);
    }
}
