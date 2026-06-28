//! Service-orchestration integration tests: scan → detect → persist, quarantine,
//! Windows scan, health, and job lifecycle through `AegisOrchestrator`.

use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use aegis_service::orchestrator::ScanMode;
use aegis_service::{AegisOrchestrator, ComponentStatus, ServiceConfig};

fn orchestrator() -> (AegisOrchestrator, tempfile::TempDir, tempfile::TempDir) {
    let data = tempfile::tempdir().unwrap();
    let work = tempfile::tempdir().unwrap();
    let orch = AegisOrchestrator::new(&ServiceConfig::new(data.path())).unwrap();
    (orch, data, work)
}

fn wait_terminal(orch: &AegisOrchestrator, id: &str) -> aegis_service::JobState {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let s = orch.get_scan_status(id).expect("job exists");
        if s.status.is_terminal() {
            return s;
        }
        assert!(Instant::now() < deadline, "scan job did not finish in time");
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn scan_detects_and_persists_through_service() {
    let (orch, _data, work) = orchestrator();
    fs::write(work.path().join("clean.txt"), b"benign document").unwrap();
    fs::write(
        work.path().join("update.ps1"),
        b"powershell -w hidden -enc AAAA; IEX (New-Object Net.WebClient).DownloadString('x')",
    )
    .unwrap();

    let id = orch.start_scan(vec![work.path().to_path_buf()], ScanMode::Full).unwrap();
    let final_state = wait_terminal(&orch, &id);
    assert_eq!(final_state.status, aegis_service::JobStatus::Completed);
    assert!(final_state.threats_found >= 1);

    let threats = orch.get_threats().unwrap();
    assert!(threats.iter().any(|t| t.path.contains("update.ps1")));
    assert!(!threats.iter().any(|t| t.path.contains("clean.txt")));
}

#[test]
fn quarantine_restore_delete_through_service() {
    let (orch, _data, work) = orchestrator();
    let target = work.path().join("malware.ps1");
    fs::write(&target, b"powershell -enc AAAA IEX DownloadString").unwrap();

    let id = orch.start_scan(vec![work.path().to_path_buf()], ScanMode::Full).unwrap();
    wait_terminal(&orch, &id);
    let threats = orch.get_threats().unwrap();
    let det = threats.iter().find(|t| t.path.contains("malware.ps1")).expect("detected");

    // Quarantine it (encrypts + removes original).
    let record = orch.quarantine_detection(det, "tester").unwrap();
    assert!(!target.exists());
    assert_eq!(orch.list_quarantine().unwrap().len(), 1);

    // Restore (back to original path), then re-quarantine to delete.
    orch.restore_file(&record.id, None, "tester").unwrap();
    assert!(target.exists());

    let record2 = orch.quarantine_detection(det, "tester").unwrap();
    orch.delete_quarantine_item(&record2.id, "tester").unwrap();
    assert!(!Path::new(&record2.quarantine_path).exists());
}

#[test]
fn windows_scan_through_service() {
    let (orch, _data, _work) = orchestrator();
    use aegis_windows::{PersistenceEntry, PersistenceKind};
    let entries = vec![
        PersistenceEntry::new(
            PersistenceKind::RegistryRunKey,
            "Evil",
            "C:\\Windows\\Temp\\evil.exe",
            "HKCU\\...\\Run",
        ),
        PersistenceEntry::new(
            PersistenceKind::RegistryRunKey,
            "OneDrive",
            "\"C:\\Program Files\\OneDrive\\onedrive.exe\"",
            "HKCU\\...\\Run",
        ),
    ];
    let dets = orch.analyze_windows_entries(&entries).unwrap();
    assert_eq!(dets.len(), 1); // only the temp-dropped exe
    assert!(orch.get_threats().unwrap().iter().any(|t| t.path.contains("evil.exe")));

    // scan_all() must not panic (returns [] off real-Windows surfaces in CI).
    let _ = orch.run_windows_scan().unwrap();
}

#[test]
fn service_health_reports_components() {
    let (orch, _data, _work) = orchestrator();
    let h = orch.get_service_health();
    assert_eq!(h.scanner, ComponentStatus::Ok);
    assert_eq!(h.database, ComponentStatus::Ok);
    // No YARA rules loaded yet ⇒ rules degraded (hash + heuristics still work).
    assert_eq!(h.rules, ComponentStatus::Degraded);
    assert_eq!(h.quarantine, ComponentStatus::Ok);

    // Load a rule ⇒ rules become Ok.
    orch.add_yara_rule("t", r#"rule m { strings: $a = "x" condition: $a }"#).unwrap();
    assert_eq!(orch.get_service_health().rules, ComponentStatus::Ok);
}

#[test]
fn realtime_start_stop_status() {
    let (orch, _data, watch) = orchestrator();
    use aegis_service::ProtectionMode;

    assert!(!orch.get_realtime_status().running);
    orch.start_realtime_with_paths(ProtectionMode::NotifyOnly, vec![watch.path().display().to_string()])
        .unwrap();
    let s = orch.get_realtime_status();
    assert!(s.running);
    assert_eq!(s.mode, ProtectionMode::NotifyOnly);
    assert_eq!(s.watched_paths.len(), 1);

    orch.stop_realtime().unwrap();
    assert!(!orch.get_realtime_status().running);
}

#[test]
fn secure_update_flow_through_service() {
    use base64::Engine;
    use ed25519_dalek::{Signer, SigningKey};

    let (orch, data, feed) = orchestrator();
    let sk = SigningKey::from_bytes(&[5u8; 32]);
    let pubkey_hex: String =
        sk.verifying_key().to_bytes().iter().map(|b| format!("{b:02x}")).collect();

    orch.init_updates(
        &pubkey_hex,
        Box::new(aegis_update::LocalFetcher::new(feed.path())),
        "1.0.0",
    )
    .unwrap();

    // A valid signature-database bundle (one known-bad hash line).
    let payload = format!("sha256:{}\n", "a".repeat(64));
    let file = "signature_database-2024.06.22.05.bin";
    std::fs::write(feed.path().join(file), &payload).unwrap();

    let mut manifest = aegis_update::UpdateManifest {
        version: "2024.06.22.05".into(),
        published_at: chrono::Utc::now(),
        sha256: aegis_update::sha256_hex(payload.as_bytes()),
        signature: String::new(),
        url: format!("https://feed/{file}"),
        size: payload.len() as u64,
        component: aegis_update::UpdateComponent::SignatureDatabase,
        minimum_app_version: "1.0.0".into(),
    };
    manifest.signature = base64::engine::general_purpose::STANDARD
        .encode(sk.sign(manifest.signed_message().as_bytes()).to_bytes());

    assert_eq!(orch.check_updates(std::slice::from_ref(&manifest)).unwrap().len(), 1);
    orch.download_updates(&manifest).unwrap();
    let outcome = orch.install_updates(&manifest).unwrap();
    assert_eq!(outcome.version, "2024.06.22.05");

    let status = orch.get_update_status().unwrap();
    assert!(status.iter().any(|(c, v)| c == "signature_database" && v == "2024.06.22.05"));

    let _ = data; // keep tempdir alive
}

#[test]
fn stop_scan_cancels() {
    let (orch, _data, work) = orchestrator();
    for i in 0..200 {
        fs::write(work.path().join(format!("f{i}.bin")), vec![0u8; 4096]).unwrap();
    }
    let id = orch.start_scan(vec![work.path().to_path_buf()], ScanMode::Full).unwrap();
    let cancelled = orch.stop_scan(&id).unwrap();
    assert!(cancelled);
    let s = wait_terminal(&orch, &id);
    assert!(s.status.is_terminal());
}
