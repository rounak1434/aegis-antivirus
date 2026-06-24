//! Persistence for detection results and scan events (tables from migration
//! `002_detection.sql`). Keeps the engine output auditable and reportable.

use chrono::Utc;
use rusqlite::{params, Connection};

use crate::model::ThreatDetection;

#[derive(Debug, thiserror::Error)]
pub enum DetectDbError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Insert one detection into `detection_results`. Returns the detection id.
pub fn persist_detection(conn: &Connection, det: &ThreatDetection) -> Result<String, DetectDbError> {
    let evidence_json = serde_json::to_string(&det.evidence)?;
    let level = serde_json::to_string(&det.threat_level)?;
    let level = level.trim_matches('"');
    conn.execute(
        "INSERT INTO detection_results
            (id, path, threat_level, score, evidence_json, detected_at_utc)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![det.id, det.path, level, det.score, evidence_json, det.timestamp.to_rfc3339()],
    )?;
    Ok(det.id.clone())
}

/// Append a row to `scan_events` for history/audit. Returns the event id.
pub fn record_scan_event(
    conn: &Connection,
    scan_id: Option<&str>,
    event_type: &str,
    path: Option<&str>,
    detail: &serde_json::Value,
) -> Result<String, DetectDbError> {
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO scan_events
            (id, scan_id, event_type, path, detail_json, created_at_utc)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, scan_id, event_type, path, detail.to_string(), Utc::now().to_rfc3339()],
    )?;
    Ok(id)
}

/// Count rows in `detection_results` (helper for reporting/tests).
pub fn detection_count(conn: &Connection) -> Result<i64, DetectDbError> {
    Ok(conn.query_row("SELECT COUNT(*) FROM detection_results", [], |r| r.get(0))?)
}
