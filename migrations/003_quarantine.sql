-- Phase 4 quarantine tables (additive). audit_log already exists from 001 and
-- is reused for quarantine/restore/delete action logging.

CREATE TABLE IF NOT EXISTS quarantine_records (
    id TEXT PRIMARY KEY,
    original_path TEXT NOT NULL,
    quarantine_path TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    threat_level TEXT NOT NULL,
    reason TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    encrypted INTEGER NOT NULL CHECK(encrypted IN (0, 1)),
    status TEXT NOT NULL CHECK(status IN ('quarantined', 'restored', 'deleted')),
    quarantined_at_utc TEXT NOT NULL,
    restored_at_utc TEXT,
    deleted_at_utc TEXT
);
CREATE INDEX IF NOT EXISTS idx_quarantine_status ON quarantine_records(status);
CREATE INDEX IF NOT EXISTS idx_quarantine_sha256 ON quarantine_records(sha256);
