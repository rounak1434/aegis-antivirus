//! Shared model for Windows persistence entries.

use serde::{Deserialize, Serialize};

/// The kind of persistence mechanism an entry was discovered in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistenceKind {
    StartupEntry,
    RegistryRunKey,
    RegistryRunOnce,
    ScheduledTask,
    ServicePersistence,
    DriverPersistence,
    BrowserExtension,
    HostsFileModification,
}

impl PersistenceKind {
    /// Stable mechanism label used in `ThreatEvidence::PersistenceMechanism`.
    pub fn mechanism(&self) -> &'static str {
        match self {
            PersistenceKind::StartupEntry => "startup_entry",
            PersistenceKind::RegistryRunKey => "registry_run_key",
            PersistenceKind::RegistryRunOnce => "registry_run_once",
            PersistenceKind::ScheduledTask => "scheduled_task",
            PersistenceKind::ServicePersistence => "service_persistence",
            PersistenceKind::DriverPersistence => "driver_persistence",
            PersistenceKind::BrowserExtension => "browser_extension",
            PersistenceKind::HostsFileModification => "hosts_file_modification",
        }
    }
}

/// One discovered persistence entry, normalized across all collectors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistenceEntry {
    pub kind: PersistenceKind,
    /// Entry name (value name, task name, service name, extension id, hostname).
    pub name: String,
    /// Command line / image path / target (for hosts: the mapped IP).
    pub command: String,
    /// Where it was found (hive\key, folder, file path).
    pub location: String,
    /// Authenticode signed, if determinable. `None` = unknown.
    pub signed: Option<bool>,
    /// Free-form extra detail (e.g. "unpacked extension", hosts mapping).
    pub detail: String,
}

impl PersistenceEntry {
    /// Construct an entry with unknown signature and empty detail.
    pub fn new(
        kind: PersistenceKind,
        name: impl Into<String>,
        command: impl Into<String>,
        location: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            name: name.into(),
            command: command.into(),
            location: location.into(),
            signed: None,
            detail: String::new(),
        }
    }

    pub fn with_signed(mut self, signed: Option<bool>) -> Self {
        self.signed = signed;
        self
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = detail.into();
        self
    }
}
