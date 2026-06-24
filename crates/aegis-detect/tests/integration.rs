//! End-to-end detection tests: scan a real temp tree, run the engine over the
//! scanner output, and verify detections + persistence.

use std::fs;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use aegis_common::ScanMode;
use aegis_detect::{
    detection_count, persist_detection, record_scan_event, DetectionEngine, HashAlgo, RuleManager,
    SignatureDatabase, ThreatEvidence, ThreatLevel,
};
use aegis_scan::{scan, ScanOptions};

fn write(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).unwrap();
}

fn sha256_of(path: &Path) -> String {
    use sha2::{Digest, Sha256};
    let data = fs::read(path).unwrap();
    let digest = Sha256::digest(&data);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// Build a corpus and return (tempdir, sha256 of the known-bad file).
fn corpus() -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    write(&dir.path().join("clean.txt"), b"benign document");
    let evil = dir.path().join("evil.bin");
    write(&evil, b"this is a known bad sample");
    write(&dir.path().join("invoice.pdf.exe"), b"MZ decoy payload");
    write(
        &dir.path().join("update.ps1"),
        b"powershell -EncodedCommand AAA; IEX (New-Object Net.WebClient).DownloadString('x')",
    );
    write(&dir.path().join("marker.dat"), b"xx MALMARKER xx");
    let sha = sha256_of(&evil);
    (dir, sha)
}

fn engine_parts(known_sha: &str) -> (SignatureDatabase, RuleManager) {
    let mut sigs = SignatureDatabase::new();
    sigs.insert(HashAlgo::Sha256, known_sha);
    let mut yara = RuleManager::new();
    yara.add_source(
        "test",
        r#"rule marker { strings: $a = "MALMARKER" condition: $a }"#,
    );
    yara.compile_rules().unwrap();
    (sigs, yara)
}

#[test]
fn detects_all_layers_end_to_end() {
    let (dir, sha) = corpus();
    let (sigs, yara) = engine_parts(&sha);
    let opts = ScanOptions::for_mode(ScanMode::Full, vec![dir.path().to_path_buf()]);
    let report = scan(&opts, Arc::new(AtomicBool::new(false)), |_p| {}).unwrap();

    let engine = DetectionEngine::new();
    let dets = engine.analyze_report(&report, &sigs, Some(&yara));

    let by_name = |needle: &str| dets.iter().find(|d| d.path.contains(needle));

    // Hash match → Critical.
    let evil = by_name("evil.bin").expect("evil.bin detected");
    assert_eq!(evil.threat_level, ThreatLevel::Critical);
    assert!(evil.evidence.iter().any(|e| matches!(e, ThreatEvidence::HashMatch { .. })));

    // Double extension.
    let dbl = by_name("invoice.pdf.exe").expect("double-ext detected");
    assert!(dbl.evidence.iter().any(|e| matches!(e, ThreatEvidence::DoubleExtension { .. })));

    // PowerShell + suspicious extension on the .ps1.
    let ps = by_name("update.ps1").expect("ps1 detected");
    assert!(ps.evidence.iter().any(|e| matches!(e, ThreatEvidence::PowerShellIndicator { .. })));
    assert!(ps.evidence.iter().any(|e| matches!(e, ThreatEvidence::SuspiciousExtension { .. })));

    // YARA match.
    let marker = by_name("marker.dat").expect("yara detected");
    assert!(marker.evidence.iter().any(|e| matches!(e, ThreatEvidence::YaraMatch { .. })));

    // Clean file is not reported.
    assert!(by_name("clean.txt").is_none());
}

#[test]
fn detections_persist_to_database() {
    let (dir, sha) = corpus();
    let (sigs, yara) = engine_parts(&sha);
    let opts = ScanOptions::for_mode(ScanMode::Full, vec![dir.path().to_path_buf()]);
    let report = scan(&opts, Arc::new(AtomicBool::new(false)), |_p| {}).unwrap();
    let dets = DetectionEngine::new().analyze_report(&report, &sigs, Some(&yara));
    assert!(!dets.is_empty());

    let mut conn = aegis_db::open_in_memory_database().unwrap();
    aegis_db::apply_migrations(&mut conn).unwrap();

    record_scan_event(
        &conn,
        Some("scan-1"),
        "scan_started",
        None,
        &serde_json::json!({ "roots": [dir.path().to_string_lossy()] }),
    )
    .unwrap();

    for d in &dets {
        persist_detection(&conn, d).unwrap();
    }

    assert_eq!(detection_count(&conn).unwrap(), dets.len() as i64);

    // Stored level round-trips and high scores are queryable.
    let crit: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM detection_results WHERE threat_level = 'critical'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(crit >= 1);
}

#[test]
fn score_accumulates_and_clamps() {
    // A file that trips double-extension (20) + suspicious-ext (15) +
    // powershell (25) + script (10) should sum without exceeding 100.
    let engine = DetectionEngine::new();
    let ev = engine.content_evidence(
        Path::new("invoice.pdf.ps1"),
        b"powershell -EncodedCommand X; IEX",
    );
    // content_evidence covers script/ps/entropy; combine with name evidence:
    let mut all = ev;
    all.push(ThreatEvidence::DoubleExtension {
        file_name: "invoice.pdf.ps1".into(),
        decoy_ext: "pdf".into(),
        real_ext: "ps1".into(),
    });
    all.push(ThreatEvidence::SuspiciousExtension { ext: "ps1".into() });
    let det = aegis_detect::ThreatDetection::from_evidence("invoice.pdf.ps1", all, chrono::Utc::now())
        .unwrap();
    assert!(det.score <= 100);
    assert!(det.score >= 60, "expected High+, got {}", det.score);
}
