//! Real-time protection data model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub use aegis_common::ThreatLevel;

/// Kind of filesystem change observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileEventKind {
    Create,
    Modify,
    Rename,
}

impl FileEventKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileEventKind::Create => "file_create",
            FileEventKind::Modify => "file_modify",
            FileEventKind::Rename => "file_rename",
        }
    }
}

/// A filesystem event handed to the pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEvent {
    pub kind: FileEventKind,
    pub path: String,
}

/// A process-launch event handed to the pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessEvent {
    pub pid: u32,
    pub name: String,
    pub exe_path: String,
    pub command_line: String,
}

/// Protection policy mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtectionMode {
    /// Observe + log only, never alert the user.
    MonitorOnly,
    /// Alert on detections but take no action (default).
    #[default]
    NotifyOnly,
    /// Auto-quarantine high/critical detections, notify otherwise.
    AutoQuarantine,
}

/// Action taken for an alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RealtimeAction {
    Monitored,
    Notified,
    Quarantined,
    QuarantineFailed,
}

impl RealtimeAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            RealtimeAction::Monitored => "monitored",
            RealtimeAction::Notified => "notified",
            RealtimeAction::Quarantined => "quarantined",
            RealtimeAction::QuarantineFailed => "quarantine_failed",
        }
    }
}

/// An alert raised when an event produced a detection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealtimeAlert {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub path: Option<String>,
    pub process: Option<String>,
    pub threat_level: ThreatLevel,
    pub score: u8,
    pub action: RealtimeAction,
    pub reason: String,
}

/// Live status of the RTP subsystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeStatus {
    pub running: bool,
    pub mode: ProtectionMode,
    pub watched_paths: Vec<String>,
    pub events_processed: u64,
    pub alerts_raised: u64,
}
