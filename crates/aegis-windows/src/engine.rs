//! Windows persistence scanner — runs every collector, applies heuristics, and
//! converts suspicious entries into `aegis_detect::ThreatDetection`s.

use aegis_detect::ThreatDetection;
use chrono::Utc;

use crate::heuristics::analyze_entry;
use crate::model::{PersistenceEntry, PersistenceKind};
use crate::{
    browser_extensions, drivers, hosts_file, registry, scheduled_tasks, services, startup,
};

/// The Windows security scanner.
#[derive(Debug, Default)]
pub struct WindowsScanner;

impl WindowsScanner {
    pub fn new() -> Self {
        Self
    }

    /// Analyze one entry; `None` when nothing suspicious is found.
    pub fn analyze(&self, entry: &PersistenceEntry) -> Option<ThreatDetection> {
        let evidence = analyze_entry(entry);
        let artifact = artifact_path(entry);
        ThreatDetection::from_evidence(artifact, evidence, Utc::now())
    }

    /// Analyze a batch of already-collected entries (used by tests + the
    /// service after a custom collection).
    pub fn analyze_entries(&self, entries: &[PersistenceEntry]) -> Vec<ThreatDetection> {
        entries.iter().filter_map(|e| self.analyze(e)).collect()
    }

    /// Collect every persistence source on this machine and analyze it.
    /// On non-Windows hosts the collectors return empty, so this yields `[]`.
    pub fn scan_all(&self) -> Vec<ThreatDetection> {
        self.analyze_entries(&collect_all())
    }
}

/// Run every collector and concatenate the entries (no analysis).
pub fn collect_all() -> Vec<PersistenceEntry> {
    let mut entries = Vec::new();
    entries.extend(startup::collect());
    entries.extend(registry::collect());
    entries.extend(scheduled_tasks::collect());
    entries.extend(services::collect());
    entries.extend(drivers::collect());
    entries.extend(browser_extensions::collect());
    entries.extend(hosts_file::collect());
    entries
}

fn artifact_path(entry: &PersistenceEntry) -> String {
    match entry.kind {
        PersistenceKind::HostsFileModification => entry.name.clone(),
        _ if !entry.command.is_empty() => entry.command.clone(),
        _ => entry.name.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aegis_detect::{ThreatEvidence, ThreatLevel};

    #[test]
    fn suspicious_entry_becomes_detection() {
        let scanner = WindowsScanner::new();
        let entry = PersistenceEntry::new(
            PersistenceKind::RegistryRunKey,
            "Updater",
            "C:\\Users\\a\\AppData\\Local\\Temp\\u.exe",
            "HKCU\\...\\Run",
        );
        let det = scanner.analyze(&entry).expect("detection");
        assert!(det.score >= 30);
        assert!(det
            .evidence
            .iter()
            .any(|e| matches!(e, ThreatEvidence::PersistenceMechanism { .. })));
    }

    #[test]
    fn benign_entry_no_detection() {
        let scanner = WindowsScanner::new();
        let entry = PersistenceEntry::new(
            PersistenceKind::RegistryRunKey,
            "OneDrive",
            "\"C:\\Program Files\\Microsoft OneDrive\\OneDrive.exe\" /background",
            "HKCU\\...\\Run",
        );
        assert!(scanner.analyze(&entry).is_none());
    }

    #[test]
    fn stacked_evidence_raises_level() {
        let scanner = WindowsScanner::new();
        let entry = PersistenceEntry::new(
            PersistenceKind::ScheduledTask,
            "Evil",
            "powershell -w hidden -enc SQBFAFgA C:\\Windows\\Temp\\x.exe",
            "Task Scheduler",
        );
        let det = scanner.analyze(&entry).unwrap();
        // persistence(15) + powershell(25) + temp-exe(20) ⇒ High+
        assert!(matches!(
            det.threat_level,
            ThreatLevel::High | ThreatLevel::Critical
        ));
    }
}
