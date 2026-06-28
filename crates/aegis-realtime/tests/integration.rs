//! RTP pipeline integration tests with mock file + process events.

use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use aegis_quarantine::Vault;
use aegis_realtime::{
    FileEvent, FileEventKind, ProcessEvent, ProtectionMode, RealtimeAction, RealtimeEngine,
    ThreatLevel,
};
use aegis_signatures::{HashAlgo, SignatureDatabase};
use aegis_yara::RuleManager;

struct Fixture {
    engine: RealtimeEngine,
    signatures: Arc<Mutex<SignatureDatabase>>,
    db_path: std::path::PathBuf,
    _data: tempfile::TempDir,
    work: tempfile::TempDir,
}

fn fixture(mode: ProtectionMode) -> Fixture {
    let data = tempfile::tempdir().unwrap();
    let work = tempfile::tempdir().unwrap();
    let db_path = data.path().join("aegis.db");
    let mut conn = aegis_db::open_database(&db_path).unwrap();
    aegis_db::apply_migrations(&mut conn).unwrap();
    let vault_conn = aegis_db::open_database(&db_path).unwrap();
    let vault = Vault::open(data.path().join("quarantine"), vault_conn).unwrap();

    let signatures = Arc::new(Mutex::new(SignatureDatabase::new()));
    let engine = RealtimeEngine::new(
        signatures.clone(),
        Arc::new(Mutex::new(RuleManager::new())),
        Arc::new(Mutex::new(vault)),
        db_path.clone(),
        mode,
    );
    Fixture {
        engine,
        signatures,
        db_path,
        _data: data,
        work,
    }
}

fn sha256_of(path: &Path) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(fs::read(path).unwrap())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[test]
fn malicious_file_event_raises_notify_alert() {
    let fx = fixture(ProtectionMode::NotifyOnly);
    let p = fx.work.path().join("dropper.ps1");
    fs::write(&p, b"powershell -w hidden -enc AAAA; IEX (DownloadString)").unwrap();

    let alert = fx
        .engine
        .handle_file_event(&FileEvent {
            kind: FileEventKind::Create,
            path: p.display().to_string(),
        })
        .expect("alert");
    assert_eq!(alert.action, RealtimeAction::Notified);
    assert!(alert.path.as_deref().unwrap().contains("dropper.ps1"));
    assert!(p.exists(), "notify-only must not remove the file");
    assert_eq!(fx.engine.alerts_raised(), 1);
}

#[test]
fn clean_file_event_no_alert() {
    let fx = fixture(ProtectionMode::NotifyOnly);
    let p = fx.work.path().join("notes.txt");
    fs::write(&p, b"perfectly benign text").unwrap();
    assert!(fx
        .engine
        .handle_file_event(&FileEvent {
            kind: FileEventKind::Create,
            path: p.display().to_string()
        })
        .is_none());
    assert_eq!(fx.engine.alerts_raised(), 0);
}

#[test]
fn auto_quarantine_isolates_known_bad_file() {
    let fx = fixture(ProtectionMode::AutoQuarantine);
    let p = fx.work.path().join("malware.bin");
    fs::write(&p, b"known bad sample").unwrap();
    // Make it a signature hit ⇒ Critical ⇒ auto-quarantine.
    fx.signatures
        .lock()
        .unwrap()
        .insert(HashAlgo::Sha256, &sha256_of(&p));

    let alert = fx
        .engine
        .handle_file_event(&FileEvent {
            kind: FileEventKind::Create,
            path: p.display().to_string(),
        })
        .expect("alert");
    assert_eq!(alert.threat_level, ThreatLevel::Critical);
    assert_eq!(alert.action, RealtimeAction::Quarantined);
    assert!(!p.exists(), "auto-quarantine must remove the original");

    let count: i64 = aegis_db::open_database(&fx.db_path)
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM realtime_alerts WHERE action='quarantined'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn auto_quarantine_only_high_or_critical() {
    let fx = fixture(ProtectionMode::AutoQuarantine);
    let p = fx.work.path().join("susp.ps1");
    fs::write(&p, b"powershell start; some script body").unwrap(); // script(10)+ext ps1(15)=25 Low
    let alert = fx
        .engine
        .handle_file_event(&FileEvent {
            kind: FileEventKind::Create,
            path: p.display().to_string(),
        })
        .expect("alert");
    assert!(matches!(
        alert.threat_level,
        ThreatLevel::Low | ThreatLevel::Medium
    ));
    assert_eq!(alert.action, RealtimeAction::Notified); // below High ⇒ not quarantined
    assert!(p.exists());
}

#[test]
fn process_event_flags_temp_powershell() {
    let fx = fixture(ProtectionMode::NotifyOnly);
    let alert = fx
        .engine
        .handle_process_event(&ProcessEvent {
            pid: 4242,
            name: "powershell.exe".into(),
            exe_path: "C:\\Windows\\Temp\\p.exe".into(),
            command_line: "p.exe -w hidden -enc AAAA IEX DownloadString".into(),
        })
        .expect("alert");
    assert_eq!(alert.action, RealtimeAction::Notified);
    assert!(alert.process.as_deref() == Some("powershell.exe"));
    assert!(alert.score >= 40);
}

#[test]
fn benign_process_no_alert() {
    let fx = fixture(ProtectionMode::NotifyOnly);
    assert!(fx
        .engine
        .handle_process_event(&ProcessEvent {
            pid: 1,
            name: "explorer.exe".into(),
            exe_path: "C:\\Windows\\explorer.exe".into(),
            command_line: "explorer.exe".into(),
        })
        .is_none());
}

#[test]
fn monitor_only_logs_without_acting() {
    let fx = fixture(ProtectionMode::MonitorOnly);
    let p = fx.work.path().join("x.ps1");
    fs::write(&p, b"powershell -enc AAAA IEX").unwrap();
    let alert = fx
        .engine
        .handle_file_event(&FileEvent {
            kind: FileEventKind::Create,
            path: p.display().to_string(),
        })
        .expect("alert");
    assert_eq!(alert.action, RealtimeAction::Monitored);
    assert!(p.exists());
}
