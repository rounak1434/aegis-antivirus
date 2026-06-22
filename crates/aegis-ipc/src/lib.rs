use aegis_common::{ProtectionStatus, ScanJobId, ScanMode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StartScanCommand {
    pub mode: ScanMode,
    pub roots: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StartScanAccepted {
    pub scan_id: ScanJobId,
    pub accepted_at_utc: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuarantineActionCommand {
    pub quarantine_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "payload")]
pub enum ServiceRequest {
    GetServiceStatus,
    StartScan(StartScanCommand),
    CancelScan { scan_id: ScanJobId },
    ListThreats,
    ListQuarantine,
    QuarantineRestore(QuarantineActionCommand),
    QuarantineDelete(QuarantineActionCommand),
    CheckForUpdates,
    GetSettings,
    UpdateSettings { settings_json: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "payload")]
pub enum ServiceResponse {
    ServiceStatus(ProtectionStatus),
    ScanAccepted(StartScanAccepted),
    Accepted,
    Rejected { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "payload")]
pub enum ServiceEvent {
    ServiceStatusChanged(ProtectionStatus),
    ScanProgress { scan_id: ScanJobId, files_scanned: u64, threats_found: u32 },
    SecurityNotification { severity: String, title: String, body: String },
}
