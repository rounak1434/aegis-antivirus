-- Phase 6 service-orchestration tables (additive).

-- Lifecycle / orchestration events emitted by AegisService.
CREATE TABLE IF NOT EXISTS service_events (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,      -- 'service_started' | 'scan_queued' | 'scan_completed' | ...
    detail_json TEXT NOT NULL,
    created_at_utc TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_service_events_type ON service_events(event_type);

-- History of background jobs (scans, windows sweeps).
CREATE TABLE IF NOT EXISTS job_history (
    id TEXT PRIMARY KEY,
    job_type TEXT NOT NULL,        -- 'file_scan' | 'windows_scan'
    status TEXT NOT NULL,          -- 'queued' | 'running' | 'completed' | 'cancelled' | 'failed'
    roots_json TEXT NOT NULL,
    files_scanned INTEGER NOT NULL DEFAULT 0,
    threats_found INTEGER NOT NULL DEFAULT 0,
    error TEXT,
    queued_at_utc TEXT NOT NULL,
    started_at_utc TEXT,
    finished_at_utc TEXT
);
CREATE INDEX IF NOT EXISTS idx_job_history_status ON job_history(status);

-- Key/value service state (singleton settings, last-scan markers, etc).
CREATE TABLE IF NOT EXISTS service_state (
    key TEXT PRIMARY KEY,
    value_json TEXT NOT NULL,
    updated_at_utc TEXT NOT NULL
);
