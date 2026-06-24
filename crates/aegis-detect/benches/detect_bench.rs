//! Detection-engine benchmark. Generates a synthetic corpus (default 10k
//! files, a realistic mix of clean + malicious-looking files), scans it with
//! aegis-scan, then runs the detection engine and reports:
//!   files/sec · per-file detection latency · detections found · peak memory.
//!
//! Run: cargo run --release --bench detect_bench -p aegis-detect [num_files]

use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Instant;

use aegis_common::ScanMode;
use aegis_detect::{DetectionEngine, HashAlgo, RuleManager, SignatureDatabase};
use aegis_scan::{scan, ScanOptions};

fn main() {
    let num_files: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(10_000);

    let dir = tempfile::tempdir().expect("tempdir");
    let known_sha = generate(dir.path(), num_files);

    // Signature DB with one known-bad hash that one generated file will match.
    let mut sigs = SignatureDatabase::new();
    sigs.insert(HashAlgo::Sha256, &known_sha);

    // YARA manager with a simple rule.
    let mut yara = RuleManager::new();
    yara.add_source(
        "bench",
        r#"rule bench_marker { strings: $a = "MALMARKER" condition: $a }"#,
    );
    yara.compile_rules().expect("compile rules");

    // Scan phase.
    let scan_opts = ScanOptions::for_mode(ScanMode::Full, vec![dir.path().to_path_buf()]);
    let scan_start = Instant::now();
    let report = scan(&scan_opts, Arc::new(AtomicBool::new(false)), |_p| {}).expect("scan");
    let scan_ms = scan_start.elapsed().as_secs_f64() * 1000.0;

    // Detection phase.
    let engine = DetectionEngine::new();
    let det_start = Instant::now();
    let detections = engine.analyze_report(&report, &sigs, Some(&yara));
    let det_elapsed = det_start.elapsed();
    let det_secs = det_elapsed.as_secs_f64().max(1e-6);

    let files = report.files_scanned as f64;
    println!("aegis-detect benchmark");
    println!("  files            : {}", report.files_scanned);
    println!("  bytes            : {}", report.bytes_scanned);
    println!("  scan phase       : {scan_ms:.0} ms");
    println!("  detect phase     : {:.0} ms", det_secs * 1000.0);
    println!("  detect files/sec : {:.0}", files / det_secs);
    println!(
        "  detect latency   : {:.1} us/file",
        det_elapsed.as_micros() as f64 / files.max(1.0)
    );
    println!("  detections       : {}", detections.len());
    println!("  peak memory      : {}", format_mem(peak_working_set()));
}

fn generate(root: &Path, n: usize) -> String {
    // Distribute across 16 subdirs; sprinkle malicious-looking files.
    let high_entropy: Vec<u8> = {
        let mut v = b"MZ".to_vec();
        v.extend((0..=255u8).cycle().take(16 * 1024));
        v
    };
    let mut known_sha = String::new();
    for i in 0..n {
        let sub = root.join(format!("d{}", i % 16));
        let _ = fs::create_dir_all(&sub);
        match i % 50 {
            0 => write(&sub.join(format!("packed{i}.exe")), &high_entropy),
            1 => write(&sub.join(format!("invoice{i}.pdf.exe")), b"MZ decoy"),
            2 => write(
                &sub.join(format!("update{i}.ps1")),
                b"powershell -EncodedCommand AAA; IEX (DownloadString)",
            ),
            3 => {
                let p = sub.join(format!("known{i}.bin"));
                write(&p, b"MALMARKER known-bad sample");
                if known_sha.is_empty() {
                    // hash this file so the signature DB has a real match
                    known_sha = sha256_of(&p);
                }
            }
            _ => write(&sub.join(format!("clean{i}.txt")), b"benign document text"),
        }
    }
    if known_sha.is_empty() {
        known_sha = "0".repeat(64);
    }
    known_sha
}

fn write(path: &Path, bytes: &[u8]) {
    let mut f = fs::File::create(path).unwrap();
    f.write_all(bytes).unwrap();
}

fn sha256_of(path: &Path) -> String {
    use sha2::{Digest, Sha256};
    let data = fs::read(path).unwrap();
    let mut h = Sha256::new();
    h.update(&data);
    let digest = h.finalize();
    let mut s = String::with_capacity(64);
    for b in digest {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn format_mem(bytes: u64) -> String {
    if bytes == 0 {
        "n/a".to_string()
    } else {
        format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
    }
}

#[cfg(windows)]
fn peak_working_set() -> u64 {
    #[repr(C)]
    struct ProcessMemoryCounters {
        cb: u32,
        page_fault_count: u32,
        peak_working_set_size: usize,
        working_set_size: usize,
        quota_peak_paged_pool_usage: usize,
        quota_paged_pool_usage: usize,
        quota_peak_non_paged_pool_usage: usize,
        quota_non_paged_pool_usage: usize,
        pagefile_usage: usize,
        peak_pagefile_usage: usize,
    }
    extern "system" {
        fn GetCurrentProcess() -> isize;
        fn K32GetProcessMemoryInfo(
            process: isize,
            counters: *mut ProcessMemoryCounters,
            cb: u32,
        ) -> i32;
    }
    unsafe {
        let mut counters: ProcessMemoryCounters = std::mem::zeroed();
        counters.cb = std::mem::size_of::<ProcessMemoryCounters>() as u32;
        if K32GetProcessMemoryInfo(GetCurrentProcess(), &mut counters, counters.cb) != 0 {
            counters.peak_working_set_size as u64
        } else {
            0
        }
    }
}

#[cfg(not(windows))]
fn peak_working_set() -> u64 {
    0
}
