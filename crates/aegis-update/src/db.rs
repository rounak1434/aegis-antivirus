//! Update persistence: history (reuses `update_history` from migration 001) and
//! the `installed_components` registry (migration 006).

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

use crate::manifest::{UpdateComponent, UpdateManifest};

pub type DbResult<T> = rusqlite::Result<T>;

/// Append an update action to `update_history`.
pub fn record_history(
    conn: &Connection,
    component: UpdateComponent,
    version: &str,
    sha256: &str,
    status: &str,
) -> DbResult<()> {
    conn.execute(
        "INSERT INTO update_history (id, channel, bundle_version, manifest_hash, status, applied_at_utc)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            uuid::Uuid::new_v4().to_string(),
            component.as_str(),
            version,
            sha256,
            status,
            Utc::now().to_rfc3339()
        ],
    )?;
    Ok(())
}

/// Upsert the installed-component registry after a successful install/rollback.
pub fn upsert_installed(
    conn: &Connection,
    manifest: &UpdateManifest,
    installed_path: &str,
) -> DbResult<()> {
    conn.execute(
        "INSERT INTO installed_components (component, version, sha256, installed_path, installed_at_utc)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(component) DO UPDATE SET
            version = excluded.version,
            sha256 = excluded.sha256,
            installed_path = excluded.installed_path,
            installed_at_utc = excluded.installed_at_utc",
        params![
            manifest.component.as_str(),
            manifest.version,
            manifest.sha256.to_ascii_lowercase(),
            installed_path,
            Utc::now().to_rfc3339()
        ],
    )?;
    Ok(())
}

/// Currently installed version of a component, if any.
pub fn installed_version(
    conn: &Connection,
    component: UpdateComponent,
) -> DbResult<Option<String>> {
    conn.query_row(
        "SELECT version FROM installed_components WHERE component = ?1",
        params![component.as_str()],
        |row| row.get::<_, String>(0),
    )
    .optional()
}

/// (component, version) for every installed component.
pub fn list_installed(conn: &Connection) -> DbResult<Vec<(String, String)>> {
    let mut stmt =
        conn.prepare("SELECT component, version FROM installed_components ORDER BY component")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    rows.collect()
}
