//! Throughput benchmark for the aegis-scan engine.
//!
//! Generates a synthetic file tree in a temp dir, runs a Full scan, and prints
//! file-count / bytes / duration / throughput metrics. Run with:
//!   cargo run --release --example bench -p aegis-scan
//! Optional args: <num_files> <file_kib>  (defaults: 4000 files, 16 KiB each)

use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use aegis_common::ScanMode;
use aegis_scan::{scan, ScanOptions};

fn main() {
    let mut args = std::env::args().skip(1);
    let num_files: usize = args.next().and_then(|a| a.parse().ok()).unwrap_or(4000);
    let file_kib: usize = args.next().and_then(|a| a.parse().ok()).unwrap_or(16);

    let dir = tempfile::tempdir().expect("tempdir");
    generate(dir.path(), num_files, file_kib);

    let opts = ScanOptions::for_mode(ScanMode::Full, vec![dir.path().to_path_buf()]);
    let report = scan(&opts, Arc::new(AtomicBool::new(false)), |_p| {}).expect("scan");

    let mb = report.bytes_scanned as f64 / (1024.0 * 1024.0);
    println!("aegis-scan benchmark");
    println!("  files scanned : {}", report.files_scanned);
    println!("  bytes scanned : {} ({:.1} MiB)", report.bytes_scanned, mb);
    println!("  errors        : {}", report.errors);
    println!("  duration      : {} ms", report.duration_ms);
    println!("  throughput    : {:.0} files/sec", report.files_per_sec);
    println!("  throughput    : {:.1} MiB/sec", report.bytes_per_sec / (1024.0 * 1024.0));
    println!("  threads       : {}", rayon_threads());
}

fn generate(root: &Path, num_files: usize, file_kib: usize) {
    // Spread files across 16 subdirectories to exercise traversal.
    let buf = vec![0xABu8; file_kib * 1024];
    for d in 0..16 {
        let sub = root.join(format!("d{d}"));
        fs::create_dir_all(&sub).unwrap();
        let count = num_files / 16 + usize::from(d < num_files % 16);
        for i in 0..count {
            let mut f = fs::File::create(sub.join(format!("f{i}.bin"))).unwrap();
            f.write_all(&buf).unwrap();
        }
    }
}

fn rayon_threads() -> usize {
    rayon::current_num_threads()
}
