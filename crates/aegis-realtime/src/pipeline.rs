//! Event pipeline: turns file/process events into detections (via the verified
//! engines) and applies the protection policy.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use aegis_detect::heuristics as dh;
use aegis_detect::{DetectionEngine, SignatureDatabase, ThreatDetection, ThreatEvidence};
use aegis_quarantine::Vault;
use aegis_scan::{scan, ScanOptions};
use aegis_yara::RuleManager;
use chrono::Utc;

use crate::db;
use crate::model::{FileEvent, ProcessEvent, ProtectionMode, RealtimeAction, RealtimeAlert};
use crate::policy::decide;

const TEMP_MARKERS: &[&str] = &[
    "\\temp\\",
    "\\appdata\\local\\temp",
    "\\windows\\temp",
    "%temp%",
    "/tmp/",
];

/// Runs the RTP pipeline against the shared, verified engines. Cheap to clone
/// (shares the inner `Arc`s) so a monitor thread can hold its own handle.
#[derive(Clone)]
pub struct RealtimeEngine {
    signatures: Arc<Mutex<SignatureDatabase>>,
    yara: Arc<Mutex<RuleManager>>,
    vault: Arc<Mutex<Vault>>,
    db_path: PathBuf,
    mode: Arc<Mutex<ProtectionMode>>,
    events: Arc<AtomicU64>,
    alerts: Arc<AtomicU64>,
}

impl RealtimeEngine {
    pub fn new(
        signatures: Arc<Mutex<SignatureDatabase>>,
        yara: Arc<Mutex<RuleManager>>,
        vault: Arc<Mutex<Vault>>,
        db_path: PathBuf,
        mode: ProtectionMode,
    ) -> Self {
        Self {
            signatures,
            yara,
            vault,
            db_path,
            mode: Arc::new(Mutex::new(mode)),
            events: Arc::new(AtomicU64::new(0)),
            alerts: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn set_mode(&self, mode: ProtectionMode) {
        *self.mode.lock().unwrap() = mode;
    }

    pub fn mode(&self) -> ProtectionMode {
        *self.mode.lock().unwrap()
    }

    pub fn events_processed(&self) -> u64 {
        self.events.load(Ordering::Relaxed)
    }

    pub fn alerts_raised(&self) -> u64 {
        self.alerts.load(Ordering::Relaxed)
    }

    /// File event → scan → detect → policy. Returns an alert if a threat fired.
    pub fn handle_file_event(&self, ev: &FileEvent) -> Option<RealtimeAlert> {
        self.events.fetch_add(1, Ordering::Relaxed);
        if let Ok(conn) = aegis_db::open_database(&self.db_path) {
            let _ = db::record_event(&conn, ev.kind.as_str(), Some(&ev.path), None);
        }

        let scanned = scan_one(&ev.path)?;
        let detection = {
            let sig = self.signatures.lock().unwrap();
            let yara = self.yara.lock().unwrap();
            let yara_ref = if yara.is_compiled() {
                Some(&*yara)
            } else {
                None
            };
            DetectionEngine::new().analyze(&scanned, &sig, yara_ref)?
        };

        Some(self.act(detection, Some(ev.path.clone()), None))
    }

    /// Process event → analyze command line + exe → detect → policy.
    pub fn handle_process_event(&self, ev: &ProcessEvent) -> Option<RealtimeAlert> {
        self.events.fetch_add(1, Ordering::Relaxed);
        if let Ok(conn) = aegis_db::open_database(&self.db_path) {
            let _ = db::record_event(&conn, "process_start", Some(&ev.exe_path), Some(&ev.name));
        }

        let evidence = process_evidence(ev);
        let detection = ThreatDetection::from_evidence(ev.exe_path.clone(), evidence, Utc::now())?;
        Some(self.act(detection, Some(ev.exe_path.clone()), Some(ev.name.clone())))
    }

    /// Apply policy to a detection, optionally quarantining, then persist.
    fn act(
        &self,
        detection: ThreatDetection,
        path: Option<String>,
        process: Option<String>,
    ) -> RealtimeAlert {
        let mode = self.mode();
        let mut action = decide(mode, detection.threat_level);

        if action == RealtimeAction::Quarantined {
            let vault = self.vault.lock().unwrap();
            if vault.quarantine_detection(&detection, "realtime").is_err() {
                action = RealtimeAction::QuarantineFailed;
            }
        }

        let alert = RealtimeAlert {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            path,
            process,
            threat_level: detection.threat_level,
            score: detection.score,
            action,
            reason: detection
                .evidence
                .iter()
                .map(|e| e.label())
                .collect::<Vec<_>>()
                .join(", "),
        };

        self.alerts.fetch_add(1, Ordering::Relaxed);
        if let Ok(conn) = aegis_db::open_database(&self.db_path) {
            let _ = db::record_alert(&conn, &alert);
        }
        alert
    }
}

/// Scan a single file through the verified scanner, returning its `ScannedFile`.
fn scan_one(path: &str) -> Option<aegis_scan::ScannedFile> {
    let opts = ScanOptions {
        roots: vec![PathBuf::from(path)],
        follow_symlinks: false,
        include_hidden: true,
        max_depth: Some(1),
        hash_files: true,
    };
    let report = scan(&opts, Arc::new(AtomicBool::new(false)), |_| {}).ok()?;
    report.files.into_iter().next()
}

/// Heuristic evidence for a process launch (reuses aegis-detect heuristics).
fn process_evidence(ev: &ProcessEvent) -> Vec<ThreatEvidence> {
    let mut evidence = Vec::new();
    let exe_lower = ev.exe_path.to_ascii_lowercase();
    let path = std::path::Path::new(&ev.exe_path);

    if TEMP_MARKERS.iter().any(|m| exe_lower.contains(m)) {
        evidence.push(ThreatEvidence::SuspiciousLocation {
            path: ev.exe_path.clone(),
            reason: "process launched from a temp directory".into(),
        });
    }
    if let Some((file_name, decoy_ext, real_ext)) = dh::double_extension(path) {
        evidence.push(ThreatEvidence::DoubleExtension {
            file_name,
            decoy_ext,
            real_ext,
        });
    }
    if let Some(ext) = dh::suspicious_extension(path) {
        evidence.push(ThreatEvidence::SuspiciousExtension { ext });
    }
    for indicator in dh::powershell_indicators(ev.command_line.as_bytes()) {
        evidence.push(ThreatEvidence::PowerShellIndicator { indicator });
    }
    evidence
}
