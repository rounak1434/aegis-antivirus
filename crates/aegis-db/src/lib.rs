use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::Path;
use thiserror::Error;

/// Ordered migration set: (version, name, sql). Applied in order; each is
/// recorded in `schema_migrations` and skipped if already present.
const MIGRATIONS: &[(i64, &str, &str)] = &[
    (1, "001_initial", include_str!("../../../migrations/001_initial.sql")),
    (2, "002_detection", include_str!("../../../migrations/002_detection.sql")),
    (3, "003_quarantine", include_str!("../../../migrations/003_quarantine.sql")),
    (4, "004_service", include_str!("../../../migrations/004_service.sql")),
    (5, "005_realtime", include_str!("../../../migrations/005_realtime.sql")),
    (6, "006_update", include_str!("../../../migrations/006_update.sql")),
];

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
    tx.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at_utc TEXT NOT NULL
        );",
    )?;
    for (version, name, sql) in MIGRATIONS {
        let already: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
            params![version],
            |row| row.get(0),
        )?;
        if already {
            continue;
        }
        tx.execute_batch(sql)?;
        tx.execute(
            "INSERT OR IGNORE INTO schema_migrations(version, name, applied_at_utc) VALUES (?1, ?2, ?3)",
            params![version, name, Utc::now().to_rfc3339()],
        )?;
    }
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
