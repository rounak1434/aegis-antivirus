//! Service-owned persistence: service events, job history, service state, and
//! reading back detection results for the UI.

use aegis_detect::{ThreatDetection, ThreatEvidence, ThreatLevel};
use chrono::Utc;
use rusqlite::{params, Connection};

use crate::jobs::JobState;
use crate::ServiceError;

pub fn record_event(
    conn: &Connection,
    event_type: &str,
    detail: &serde_json::Value,
) -> Result<(), ServiceError> {
    conn.execute(
        "INSERT INTO service_events (id, event_type, detail_json, created_at_utc)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            uuid::Uuid::new_v4().to_string(),
            event_type,
            detail.to_string(),
            Utc::now().to_rfc3339()
        ],
    )?;
    Ok(())
}

/// Upsert a job's row in `job_history` from its current state.
pub fn upsert_job(conn: &Connection, job: &JobState) -> Result<(), ServiceError> {
    let job_type = serde_json::to_string(&job.job_type)?;
    let status = serde_json::to_string(&job.status)?;
    conn.execute(
        "INSERT INTO job_history
            (id, job_type, status, roots_json, files_scanned, threats_found, error,
             queued_at_utc, started_at_utc, finished_at_utc)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(id) DO UPDATE SET
            status = excluded.status,
            files_scanned = excluded.files_scanned,
            threats_found = excluded.threats_found,
            error = excluded.error,
            started_at_utc = excluded.started_at_utc,
            finished_at_utc = excluded.finished_at_utc",
        params![
            job.id,
            job_type.trim_matches('"'),
            status.trim_matches('"'),
            serde_json::to_string(&job.roots)?,
            job.files_scanned as i64,
            job.threats_found as i64,
            job.error,
            job.queued_at.to_rfc3339(),
            job.started_at.map(|t| t.to_rfc3339()),
            job.finished_at.map(|t| t.to_rfc3339()),
        ],
    )?;
    Ok(())
}

pub fn set_state(conn: &Connection, key: &str, value: &serde_json::Value) -> Result<(), ServiceError> {
    conn.execute(
        "INSERT INTO service_state (key, value_json, updated_at_utc) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json, updated_at_utc = excluded.updated_at_utc",
        params![key, value.to_string(), Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

/// Read detection results back into `ThreatDetection`s (highest score first).
pub fn list_detections(conn: &Connection, limit: usize) -> Result<Vec<ThreatDetection>, ServiceError> {
    let mut stmt = conn.prepare(
        "SELECT id, path, threat_level, score, evidence_json, detected_at_utc
         FROM detection_results ORDER BY score DESC, detected_at_utc DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit as i64], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
        ))
    })?;

    let mut out = Vec::new();
    for row in rows {
        let (id, path, level_s, score, evidence_json, ts) = row?;
        let threat_level: ThreatLevel =
            serde_json::from_str(&format!("\"{level_s}\"")).unwrap_or(ThreatLevel::Safe);
        let evidence: Vec<ThreatEvidence> = serde_json::from_str(&evidence_json).unwrap_or_default();
        let timestamp = chrono::DateTime::parse_from_rfc3339(&ts)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        out.push(ThreatDetection {
            id,
            path,
            threat_level,
            score: score.clamp(0, 100) as u8,
            evidence,
            timestamp,
        });
    }
    Ok(out)
}

pub fn count_detections(conn: &Connection) -> Result<i64, ServiceError> {
    Ok(conn.query_row("SELECT COUNT(*) FROM detection_results", [], |r| r.get(0))?)
}
