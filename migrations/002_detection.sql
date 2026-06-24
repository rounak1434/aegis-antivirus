-- Phase 3 detection-engine tables (additive to 001).

-- Named collections of hash signatures (a feed/bundle).
CREATE TABLE IF NOT EXISTS signature_sets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    kind TEXT NOT NULL,            -- 'hash' | 'yara'
    source TEXT NOT NULL,          -- file path or feed URL
    signature_count INTEGER NOT NULL DEFAULT 0,
    updated_at_utc TEXT NOT NULL
);

-- Individual known-bad hashes belonging to a set.
CREATE TABLE IF NOT EXISTS signatures (
    id TEXT PRIMARY KEY,
    set_id TEXT,
    algo TEXT NOT NULL CHECK(algo IN ('sha256', 'md5')),
    hex TEXT NOT NULL,
    threat_name TEXT,
    added_at_utc TEXT NOT NULL,
    FOREIGN KEY(set_id) REFERENCES signature_sets(id) ON DELETE CASCADE,
    UNIQUE(algo, hex)
);
CREATE INDEX IF NOT EXISTS idx_signatures_hex ON signatures(hex);

-- Explainable detection-engine output (one row per detected file).
CREATE TABLE IF NOT EXISTS detection_results (
    id TEXT PRIMARY KEY,
    path TEXT NOT NULL,
    threat_level TEXT NOT NULL,
    score INTEGER NOT NULL CHECK(score >= 0 AND score <= 100),
    evidence_json TEXT NOT NULL,
    detected_at_utc TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_detection_results_score ON detection_results(score DESC);
CREATE INDEX IF NOT EXISTS idx_detection_results_path ON detection_results(path);

-- Audit trail of scan/detection lifecycle events.
CREATE TABLE IF NOT EXISTS scan_events (
    id TEXT PRIMARY KEY,
    scan_id TEXT,
    event_type TEXT NOT NULL,      -- 'scan_started' | 'file_scanned' | 'detection' | 'scan_completed'
    path TEXT,
    detail_json TEXT NOT NULL,
    created_at_utc TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_scan_events_scan_id ON scan_events(scan_id);
