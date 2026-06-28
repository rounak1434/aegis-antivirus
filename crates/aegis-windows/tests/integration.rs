//! Integration tests: mock persistence fixtures → detections.

use std::collections::HashSet;

use aegis_windows::{
    browser_extensions, drivers, hosts_file, registry, scheduled_tasks, startup, PersistenceEntry,
    PersistenceKind, ThreatEvidence, WindowsScanner,
};

fn mechanisms(dets: &[aegis_windows::ThreatDetection]) -> HashSet<String> {
    let mut set = HashSet::new();
    for d in dets {
        for e in &d.evidence {
            if let ThreatEvidence::PersistenceMechanism { mechanism, .. } = e {
                set.insert(mechanism.clone());
            }
        }
    }
    set
}

#[test]
fn mock_fixtures_across_all_surfaces() {
    let mut entries: Vec<PersistenceEntry> = Vec::new();

    // Startup folder: one malicious script + one benign shortcut.
    let startup_dir = tempfile::tempdir().unwrap();
    std::fs::write(startup_dir.path().join("evil.vbs"), b"x").unwrap();
    std::fs::write(startup_dir.path().join("Steam.lnk"), b"x").unwrap();
    entries.extend(startup::scan_dir(startup_dir.path()));

    // Registry Run: temp-dropped exe + legit app.
    entries.extend(registry::entries_from_pairs(
        PersistenceKind::RegistryRunKey,
        "HKCU\\...\\Run",
        vec![
            ("Updater", "C:\\Users\\a\\AppData\\Local\\Temp\\u.exe"),
            ("Steam", "\"C:\\Program Files\\Steam\\steam.exe\" -silent"),
        ],
    ));

    // Scheduled tasks: encoded PowerShell + benign (N/A filtered at parse).
    entries.extend(scheduled_tasks::parse_schtasks_csv(
        "\"HostName\",\"TaskName\",\"Task To Run\",\"Author\"\n\
         \"PC\",\"\\Evil\",\"powershell -w hidden -enc SQBFAFgA\",\"x\"\n\
         \"PC\",\"\\GoogleUpdate\",\"\\\"C:\\Program Files\\Google\\update.exe\\\"\",\"Google\"\n",
    ));

    // Drivers: driver in temp.
    entries.extend(drivers::parse_driverquery_csv(
        "\"Module Name\",\"Display Name\",\"Path\",\"Start Mode\"\n\
         \"evilsys\",\"Evil\",\"C:\\Windows\\Temp\\evil.sys\",\"Auto\"\n\
         \"acpi\",\"ACPI\",\"C:\\Windows\\System32\\drivers\\acpi.sys\",\"Boot\"\n",
    ));

    // Browser: unpacked Chrome extension.
    let ext_dir = tempfile::tempdir().unwrap();
    let ext = ext_dir.path().join("aaaabbbbccccdddd").join("1.0");
    std::fs::create_dir_all(&ext).unwrap();
    std::fs::write(ext.join("manifest.json"), r#"{"name":"Sneaky"}"#).unwrap();
    entries.extend(browser_extensions::scan_chromium_extensions(
        ext_dir.path(),
        "Chrome",
    ));

    // Hosts: sensitive redirect + benign loopback.
    entries.extend(hosts_file::parse_hosts(
        "127.0.0.1 localhost\n10.0.0.5 www.update.microsoft.com\n",
    ));

    let scanner = WindowsScanner::new();
    let dets = scanner.analyze_entries(&entries);

    // Exactly the six malicious fixtures should be detected.
    assert_eq!(
        dets.len(),
        6,
        "got: {:#?}",
        dets.iter().map(|d| &d.path).collect::<Vec<_>>()
    );

    let mechs = mechanisms(&dets);
    for m in [
        "startup_entry",
        "registry_run_key",
        "scheduled_task",
        "driver_persistence",
        "browser_extension",
        "hosts_file_modification",
    ] {
        assert!(mechs.contains(m), "missing mechanism {m}");
    }

    // Every detection is explainable.
    for d in &dets {
        assert!(!d.evidence.is_empty());
        assert!(d.evidence.iter().all(|e| !e.reason().is_empty()));
        assert!(d.score > 0);
    }
}

#[test]
fn benign_only_environment_is_clean() {
    let entries = registry::entries_from_pairs(
        PersistenceKind::RegistryRunKey,
        "HKLM\\...\\Run",
        vec![
            (
                "SecurityHealth",
                "C:\\Windows\\System32\\SecurityHealthSystray.exe",
            ),
            ("RtkAudio", "\"C:\\Program Files\\Realtek\\Audio\\rtk.exe\""),
        ],
    );
    let dets = WindowsScanner::new().analyze_entries(&entries);
    assert!(dets.is_empty());
}

#[test]
fn empty_scan_on_non_collected() {
    // scan_all() must never panic and returns [] when collectors find nothing
    // (always true off-Windows; best-effort on Windows).
    let _ = WindowsScanner::new().scan_all();
}
