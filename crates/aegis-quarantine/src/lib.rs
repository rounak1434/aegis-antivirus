use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuarantineId(pub Uuid);

impl QuarantineId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for QuarantineId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuarantineRecord {
    pub id: QuarantineId,
    pub original_path: String,
    pub vault_path: String,
    pub sha256: String,
    pub encrypted_size_bytes: u64,
    pub quarantined_at_utc: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum QuarantineDecision {
    Restore { id: QuarantineId, reason: String },
    PermanentlyDelete { id: QuarantineId, reason: String },
}
