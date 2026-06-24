//! Quarantine data model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub use aegis_common::ThreatLevel;

/// Status of a quarantined item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuarantineStatus {
    Quarantined,
    Restored,
    Deleted,
}

/// A record describing one quarantined file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuarantineRecord {
    pub id: String,
    pub original_path: String,
    pub quarantine_path: String,
    pub sha256: String,
    pub threat_level: ThreatLevel,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
    /// Original (plaintext) size in bytes.
    pub size: u64,
    pub encrypted: bool,
    pub status: QuarantineStatus,
}

/// An audited action on the vault.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditAction {
    Quarantine,
    Restore,
    Delete,
}

impl AuditAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditAction::Quarantine => "quarantine",
            AuditAction::Restore => "restore",
            AuditAction::Delete => "delete",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum QuarantineError {
    #[error("io error on {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("crypto error: {0}")]
    Crypto(#[from] crate::crypto::CryptoError),
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("quarantine record {0} not found")]
    NotFound(String),
    #[error("record {id} is not in the vault (status: {status})")]
    NotInVault { id: String, status: &'static str },
    #[error("integrity check failed for {id}: expected sha256 {expected}, got {actual}")]
    IntegrityMismatch { id: String, expected: String, actual: String },
    #[error("unsafe restore path: {0}")]
    UnsafePath(String),
    #[error("restore target already exists: {0}")]
    TargetExists(String),
}
