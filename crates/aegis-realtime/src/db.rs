//! Persistence for real-time events and alerts (migration 005).

use chrono::Utc;
use rusqlite::{params, Connection};

use crate::model::RealtimeAlert;

pub fn record_event(
    conn: &Connection,
    event_type: &str,
    path: Option<&str>,
    process: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO realtime_events (id, event_type, path, process, created_at_utc)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            uuid::Uuid::new_v4().to_string(),
            event_type,
            path,
            process,
            Utc::now().to_rfc3339()
        ],
    )?;
    Ok(())
}

pub fn record_alert(conn: &Connection, alert: &RealtimeAlert) -> rusqlite::Result<()> {
    let level = serde_json::to_string(&alert.threat_level).unwrap_or_else(|_| "\"safe\"".into());
    conn.execute(
        "INSERT INTO realtime_alerts
            (id, detected_at_utc, path, process, threat_level, score, action, detail_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            alert.id,
            alert.timestamp.to_rfc3339(),
            alert.path,
            alert.process,
            level.trim_matches('"'),
            alert.score as i64,
            alert.action.as_str(),
            serde_json::json!({ "reason": alert.reason }).to_string(),
        ],
    )?;
    Ok(())
}
