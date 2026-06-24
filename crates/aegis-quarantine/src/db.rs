//! SQLite persistence for quarantine records and the audit trail
//! (tables from `003_quarantine.sql` + the `audit_log` from `001_initial.sql`).

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

use crate::model::{AuditAction, QuarantineError, QuarantineRecord, QuarantineStatus, ThreatLevel};

fn level_to_str(level: &ThreatLevel) -> String {
    serde_json::to_string(level)
        .unwrap_or_else(|_| "\"safe\"".into())
        .trim_matches('"')
        .to_string()
}

fn level_from_str(s: &str) -> ThreatLevel {
    serde_json::from_str(&format!("\"{s}\"")).unwrap_or(ThreatLevel::Safe)
}

fn status_to_str(s: QuarantineStatus) -> &'static str {
    match s {
        QuarantineStatus::Quarantined => "quarantined",
        QuarantineStatus::Restored => "restored",
        QuarantineStatus::Deleted => "deleted",
    }
}

fn status_from_str(s: &str) -> QuarantineStatus {
    match s {
        "restored" => QuarantineStatus::Restored,
        "deleted" => QuarantineStatus::Deleted,
        _ => QuarantineStatus::Quarantined,
    }
}

pub fn insert_record(conn: &Connection, rec: &QuarantineRecord) -> Result<(), QuarantineError> {
    conn.execute(
        "INSERT INTO quarantine_records
            (id, original_path, quarantine_path, sha256, threat_level, reason,
             size_bytes, encrypted, status, quarantined_at_utc)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            rec.id,
            rec.original_path,
            rec.quarantine_path,
            rec.sha256,
            level_to_str(&rec.threat_level),
            rec.reason,
            rec.size as i64,
            rec.encrypted as i64,
            status_to_str(rec.status),
            rec.timestamp.to_rfc3339(),
        ],
    )?;
    Ok(())
}

/// Update status and stamp the matching timestamp column.
pub fn set_status(
    conn: &Connection,
    id: &str,
    status: QuarantineStatus,
) -> Result<(), QuarantineError> {
    let now = Utc::now().to_rfc3339();
    let col = match status {
        QuarantineStatus::Restored => "restored_at_utc",
        QuarantineStatus::Deleted => "deleted_at_utc",
        QuarantineStatus::Quarantined => "quarantined_at_utc",
    };
    let sql = format!("UPDATE quarantine_records SET status = ?1, {col} = ?2 WHERE id = ?3");
    let n = conn.execute(&sql, params![status_to_str(status), now, id])?;
    if n == 0 {
        return Err(QuarantineError::NotFound(id.to_string()));
    }
    Ok(())
}

fn row_to_record(row: &rusqlite::Row) -> rusqlite::Result<QuarantineRecord> {
    let ts: String = row.get(9)?;
    Ok(QuarantineRecord {
        id: row.get(0)?,
        original_path: row.get(1)?,
        quarantine_path: row.get(2)?,
        sha256: row.get(3)?,
        threat_level: level_from_str(&row.get::<_, String>(4)?),
        reason: row.get(5)?,
        size: row.get::<_, i64>(6)? as u64,
        encrypted: row.get::<_, i64>(7)? != 0,
        status: status_from_str(&row.get::<_, String>(8)?),
        timestamp: chrono::DateTime::parse_from_rfc3339(&ts)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
    })
}

const SELECT_COLS: &str = "id, original_path, quarantine_path, sha256, threat_level, reason, \
                           size_bytes, encrypted, status, quarantined_at_utc";

pub fn get_record(conn: &Connection, id: &str) -> Result<Option<QuarantineRecord>, QuarantineError> {
    let sql = format!("SELECT {SELECT_COLS} FROM quarantine_records WHERE id = ?1");
    let rec = conn
        .query_row(&sql, params![id], row_to_record)
        .optional()?;
    Ok(rec)
}

pub fn list_records(conn: &Connection) -> Result<Vec<QuarantineRecord>, QuarantineError> {
    let sql = format!("SELECT {SELECT_COLS} FROM quarantine_records ORDER BY quarantined_at_utc DESC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], row_to_record)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Append an audit-log entry for a vault action.
pub fn write_audit(
    conn: &Connection,
    action: AuditAction,
    actor: &str,
    subject: &str,
    result: &str,
) -> Result<(), QuarantineError> {
    conn.execute(
        "INSERT INTO audit_log (id, actor, action, subject, details_json, created_at_utc)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            uuid::Uuid::new_v4().to_string(),
            actor,
            action.as_str(),
            subject,
            serde_json::json!({ "result": result }).to_string(),
            Utc::now().to_rfc3339(),
        ],
    )?;
    Ok(())
}
