-- Phase 8 secure-update tables (additive).
-- NOTE: `update_history` already exists from migration 001 (id, channel,
-- bundle_version, manifest_hash, status, applied_at_utc) and is reused for the
-- update action log. This migration adds the installed-component registry.

CREATE TABLE IF NOT EXISTS installed_components (
    component TEXT PRIMARY KEY,     -- 'signature_database' | 'yara_rules' | 'threat_metadata' | 'engine_config'
    version TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    installed_path TEXT NOT NULL,
    installed_at_utc TEXT NOT NULL
);
