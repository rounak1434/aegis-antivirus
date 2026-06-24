//! Integration tests for the aegis-scan engine. Each test builds a real
//! temporary directory tree and drives the public `scan` API end-to-end.

use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use aegis_common::ScanMode;
use aegis_scan::{scan, ScanError, ScanOptions};

fn write(path: &Path, bytes: &[u8]) {
    let mut f = fs::File::create(path).unwrap();
    f.write_all(bytes).unwrap();
}

fn no_cancel() -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(false))
}

#[test]
fn nested_directories_are_scanned() {
    let dir = tempfile::tempdir().unwrap();
    write(&dir.path().join("root.txt"), b"r");
    let deep = dir.path().join("a").join("b").join("c");
    fs::create_dir_all(&deep).unwrap();
    write(&deep.join("buried.txt"), b"deep");
    write(&dir.path().join("a").join("mid.txt"), b"mid");

    let opts = ScanOptions::for_mode(ScanMode::Full, vec![dir.path().to_path_buf()]);
    let report = scan(&opts, no_cancel(), |_p| {}).unwrap();

    assert_eq!(report.files_scanned, 3);
    assert!(report.files.iter().any(|f| f.metadata.path.ends_with("buried.txt")));
}

#[cfg(windows)]
fn make_hidden(path: &Path) {
    // Use attrib so the Windows hidden attribute is actually set on disk.
    let status = std::process::Command::new("attrib")
        .arg("+h")
        .arg(path)
        .status()
        .unwrap();
    assert!(status.success());
}

#[cfg(not(windows))]
fn make_hidden(_path: &Path) {
    // On non-Windows a dotfile name is treated as hidden by the scanner.
}

#[test]
fn hidden_files_excluded_then_included() {
    let dir = tempfile::tempdir().unwrap();
    write(&dir.path().join("visible.txt"), b"v");
    let hidden_name = if cfg!(windows) { "secret.txt" } else { ".secret.txt" };
    let hidden = dir.path().join(hidden_name);
    write(&hidden, b"h");
    make_hidden(&hidden);

    // Quick: hidden excluded.
    let quick = ScanOptions::for_mode(ScanMode::Quick, vec![dir.path().to_path_buf()]);
    let r1 = scan(&quick, no_cancel(), |_p| {}).unwrap();
    assert_eq!(r1.files_scanned, 1, "hidden file should be skipped");

    // Full: hidden included.
    let full = ScanOptions::for_mode(ScanMode::Full, vec![dir.path().to_path_buf()]);
    let r2 = scan(&full, no_cancel(), |_p| {}).unwrap();
    assert_eq!(r2.files_scanned, 2, "hidden file should be included");
    assert!(r2.files.iter().any(|f| f.metadata.is_hidden));
}

#[test]
fn symlinks_are_handled() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target.txt");
    write(&target, b"target");
    let link = dir.path().join("link.txt");

    let created = symlink_file(&target, &link);
    if !created {
        eprintln!("skipping symlink test: insufficient privilege to create symlinks");
        return;
    }

    // follow_symlinks = false (Full): link is recorded but not hashed.
    let full = ScanOptions::for_mode(ScanMode::Full, vec![dir.path().to_path_buf()]);
    let r = scan(&full, no_cancel(), |_p| {}).unwrap();
    let link_entry = r.files.iter().find(|f| f.metadata.is_symlink);
    assert!(link_entry.is_some(), "symlink entry must be present");
    assert!(link_entry.unwrap().hashes.is_none(), "symlinks are not hashed");
}

#[cfg(windows)]
fn symlink_file(target: &Path, link: &Path) -> bool {
    std::os::windows::fs::symlink_file(target, link).is_ok()
}

#[cfg(not(windows))]
fn symlink_file(target: &Path, link: &Path) -> bool {
    std::os::unix::fs::symlink(target, link).is_ok()
}

#[test]
fn permission_denied_is_counted_not_fatal() {
    let dir = tempfile::tempdir().unwrap();
    write(&dir.path().join("ok.txt"), b"ok");
    let locked = dir.path().join("locked.txt");
    write(&locked, b"locked");

    let _guard = deny_read(&locked);

    let opts = ScanOptions::for_mode(ScanMode::Full, vec![dir.path().to_path_buf()]);
    let report = scan(&opts, no_cancel(), |_p| {}).unwrap();

    // Both files traversed; the unreadable one is an error, not a panic.
    assert_eq!(report.files_scanned, 2);
    assert!(report.errors >= 1, "locked file should be counted as an error");
    assert!(report.files.iter().any(|f| f.error.is_some()));
}

/// Hold an exclusive (no-share) handle so the scanner's open fails with a
/// sharing violation — a portable stand-in for "permission denied".
#[cfg(windows)]
fn deny_read(path: &Path) -> fs::File {
    use std::os::windows::fs::OpenOptionsExt;
    fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(path)
        .unwrap()
}

#[cfg(not(windows))]
fn deny_read(path: &Path) -> fs::File {
    use std::os::unix::fs::PermissionsExt;
    let f = fs::File::open(path).unwrap();
    let mut perms = f.metadata().unwrap().permissions();
    perms.set_mode(0o000);
    fs::set_permissions(path, perms).unwrap();
    f
}

#[test]
fn large_file_hashing_and_throughput() {
    let dir = tempfile::tempdir().unwrap();
    let big = dir.path().join("big.bin");
    let chunk = vec![0u8; 1024 * 1024]; // 1 MiB
    {
        let mut f = fs::File::create(&big).unwrap();
        for _ in 0..8 {
            f.write_all(&chunk).unwrap(); // 8 MiB total
        }
    }

    let opts = ScanOptions::for_mode(ScanMode::Full, vec![dir.path().to_path_buf()]);
    let report = scan(&opts, no_cancel(), |_p| {}).unwrap();

    assert_eq!(report.files_scanned, 1);
    assert_eq!(report.bytes_scanned, 8 * 1024 * 1024);
    let f = &report.files[0];
    let h = f.hashes.as_ref().expect("large file hashed");
    assert_eq!(h.sha256.len(), 64);
    assert_eq!(h.md5.len(), 32);
    assert!(report.bytes_per_sec > 0.0);
    assert!(report.files_per_sec > 0.0);
}

#[test]
fn scan_cancellation_stops_work() {
    let dir = tempfile::tempdir().unwrap();
    for i in 0..50 {
        write(&dir.path().join(format!("f{i}.txt")), b"data");
    }
    let opts = ScanOptions::for_mode(ScanMode::Full, vec![dir.path().to_path_buf()]);
    let err = scan(&opts, Arc::new(AtomicBool::new(true)), |_p| {}).unwrap_err();
    assert!(matches!(err, ScanError::Cancelled));
}

#[test]
fn progress_callback_reports_metrics() {
    let dir = tempfile::tempdir().unwrap();
    for i in 0..5 {
        write(&dir.path().join(format!("f{i}.txt")), b"abcdef");
    }
    let opts = ScanOptions::for_mode(ScanMode::Full, vec![dir.path().to_path_buf()]);

    let saw_total = std::sync::Mutex::new(0u64);
    let saw_pct = std::sync::Mutex::new(0.0f64);
    scan(&opts, no_cancel(), |p| {
        *saw_total.lock().unwrap() = p.total_files;
        let mut m = saw_pct.lock().unwrap();
        if p.percent > *m {
            *m = p.percent;
        }
    })
    .unwrap();

    assert_eq!(*saw_total.lock().unwrap(), 5);
    assert!((*saw_pct.lock().unwrap() - 100.0).abs() < 1e-6);
}
