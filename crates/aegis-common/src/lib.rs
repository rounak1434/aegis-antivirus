use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub const PRODUCT_NAME: &str = "Aegis Antivirus";
pub const SERVICE_NAME: &str = "AegisService";
pub const IPC_ENDPOINT_NAME: &str = r"\\.\pipe\AegisService.Ipc";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceHealth {
    Starting,
    Running,
    Degraded,
    Stopped,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScanMode {
    Quick,
    Full,
    Deep,
    Custom,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThreatLevel {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtectionStatus {
    pub health: ServiceHealth,
    pub real_time_protection: bool,
    pub file_monitor: bool,
    pub process_monitor: bool,
    pub scheduled_scans: bool,
    pub signature_version: Option<String>,
    pub last_service_heartbeat_utc: Option<DateTime<Utc>>,
}

impl ProtectionStatus {
    pub fn service_unavailable() -> Self {
        Self {
            health: ServiceHealth::Stopped,
            real_time_protection: false,
            file_monitor: false,
            process_monitor: false,
            scheduled_scans: false,
            signature_version: None,
            last_service_heartbeat_utc: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanJobId(pub Uuid);

impl ScanJobId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ScanJobId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error)]
pub enum AegisError {
    #[error("AegisService is not available")]
    ServiceUnavailable,
    #[error("invalid command: {0}")]
    InvalidCommand(String),
    #[error("database error: {0}")]
    Database(String),
    #[error("configuration error: {0}")]
    Configuration(String),
}
