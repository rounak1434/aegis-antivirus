-- Phase 7 real-time protection tables (additive).

-- Raw monitored events (file + process activity).
CREATE TABLE IF NOT EXISTS realtime_events (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,      -- 'file_create' | 'file_modify' | 'file_rename' | 'process_start'
    path TEXT,
    process TEXT,
    created_at_utc TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_realtime_events_type ON realtime_events(event_type);

-- Alerts raised when an event produced a detection.
CREATE TABLE IF NOT EXISTS realtime_alerts (
    id TEXT PRIMARY KEY,
    detected_at_utc TEXT NOT NULL,
    path TEXT,
    process TEXT,
    threat_level TEXT NOT NULL,
    score INTEGER NOT NULL CHECK(score >= 0 AND score <= 100),
    action TEXT NOT NULL,          -- 'monitored' | 'notified' | 'quarantined' | 'quarantine_failed'
    detail_json TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_realtime_alerts_action ON realtime_alerts(action);
