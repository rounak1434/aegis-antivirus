//! AegisService library — the central orchestrator.
//!
//! Every security engine (scan, detection, quarantine, Windows persistence) is
//! reachable only through [`AegisOrchestrator`]. The UI calls the orchestrator
//! over IPC; it never touches the engine crates directly.

pub mod jobs;
pub mod orchestrator;
pub mod runtime;
pub mod service;

mod db;

pub use jobs::{JobManager, JobState, JobStatus, JobType};
pub use orchestrator::{AegisOrchestrator, ProtectionMode, RealtimeStatus, ServiceConfig};
pub use service::status_service::{ComponentStatus, ServiceHealth};

/// Errors surfaced by the service orchestration layer.
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("database error: {0}")]
    Db(#[from] aegis_db::DbError),
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("scan error: {0}")]
    Scan(#[from] aegis_scan::ScanError),
    #[error("quarantine error: {0}")]
    Quarantine(#[from] aegis_quarantine::QuarantineError),
    #[error("yara error: {0}")]
    Yara(#[from] aegis_yara::YaraError),
    #[error("update error: {0}")]
    Update(#[from] aegis_update::UpdateError),
    #[error("update subsystem not configured")]
    UpdatesNotConfigured,
    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("job {0} not found")]
    JobNotFound(String),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, ServiceError>;
