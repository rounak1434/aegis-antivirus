CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    applied_at_utc TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value_json TEXT NOT NULL,
    updated_at_utc TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS scan_history (
    id TEXT PRIMARY KEY,
    scan_mode TEXT NOT NULL,
    status TEXT NOT NULL,
    started_at_utc TEXT NOT NULL,
    completed_at_utc TEXT,
    files_scanned INTEGER NOT NULL DEFAULT 0,
    threats_found INTEGER NOT NULL DEFAULT 0,
    requested_by TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS threats (
    id TEXT PRIMARY KEY,
    scan_id TEXT,
    threat_name TEXT NOT NULL,
    level TEXT NOT NULL,
    score INTEGER NOT NULL CHECK(score >= 0 AND score <= 100),
    file_path TEXT NOT NULL,
    recommended_action TEXT NOT NULL,
    first_seen_utc TEXT NOT NULL,
    last_seen_utc TEXT NOT NULL,
    FOREIGN KEY(scan_id) REFERENCES scan_history(id)
);

CREATE TABLE IF NOT EXISTS detections (
    id TEXT PRIMARY KEY,
    threat_id TEXT NOT NULL,
    layer TEXT NOT NULL,
    evidence_json TEXT NOT NULL,
    created_at_utc TEXT NOT NULL,
    FOREIGN KEY(threat_id) REFERENCES threats(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS quarantine (
    id TEXT PRIMARY KEY,
    threat_id TEXT,
    original_path TEXT NOT NULL,
    vault_path TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    encrypted_size_bytes INTEGER NOT NULL,
    quarantined_at_utc TEXT NOT NULL,
    restored_at_utc TEXT,
    deleted_at_utc TEXT,
    FOREIGN KEY(threat_id) REFERENCES threats(id)
);

CREATE TABLE IF NOT EXISTS yara_rules (
    id TEXT PRIMARY KEY,
    namespace TEXT NOT NULL,
    rule_name TEXT NOT NULL,
    version TEXT NOT NULL,
    source_hash TEXT NOT NULL,
    enabled INTEGER NOT NULL CHECK(enabled IN (0, 1)),
    added_at_utc TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS notifications (
    id TEXT PRIMARY KEY,
    severity TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    created_at_utc TEXT NOT NULL,
    read_at_utc TEXT
);

CREATE TABLE IF NOT EXISTS update_history (
    id TEXT PRIMARY KEY,
    channel TEXT NOT NULL,
    bundle_version TEXT NOT NULL,
    manifest_hash TEXT NOT NULL,
    status TEXT NOT NULL,
    applied_at_utc TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS audit_log (
    id TEXT PRIMARY KEY,
    actor TEXT NOT NULL,
    action TEXT NOT NULL,
    subject TEXT NOT NULL,
    details_json TEXT NOT NULL,
    created_at_utc TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_threats_score ON threats(score DESC);
CREATE INDEX IF NOT EXISTS idx_detections_threat_id ON detections(threat_id);
CREATE INDEX IF NOT EXISTS idx_quarantine_threat_id ON quarantine(threat_id);
CREATE INDEX IF NOT EXISTS idx_audit_created_at ON audit_log(created_at_utc);
