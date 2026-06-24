//! Real-time pipeline benchmark: event latency, scan latency, events/sec.
//!
//! Run: cargo bench -p aegis-realtime --bench realtime_bench [num_events]

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use aegis_quarantine::Vault;
use aegis_realtime::{FileEvent, FileEventKind, ProtectionMode, RealtimeEngine};
use aegis_signatures::SignatureDatabase;
use aegis_yara::RuleManager;

fn main() {
    let n: usize = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(2000);

    let data = tempfile::tempdir().unwrap();
    let work = tempfile::tempdir().unwrap();
    let db_path = data.path().join("aegis.db");
    let mut conn = aegis_db::open_database(&db_path).unwrap();
    aegis_db::apply_migrations(&mut conn).unwrap();
    let vault_conn = aegis_db::open_database(&db_path).unwrap();
    let vault = Vault::open(data.path().join("quarantine"), vault_conn).unwrap();

    let engine = RealtimeEngine::new(
        Arc::new(Mutex::new(SignatureDatabase::new())),
        Arc::new(Mutex::new(RuleManager::new())),
        Arc::new(Mutex::new(vault)),
        db_path,
        ProtectionMode::NotifyOnly,
    );

    // Generate files: ~10% malicious-looking (.ps1 with abuse), rest clean.
    let mut events = Vec::with_capacity(n);
    for i in 0..n {
        let (name, content): (String, &[u8]) = if i % 10 == 0 {
            (format!("drop{i}.ps1"), b"powershell -enc AAAA; IEX (DownloadString)")
        } else {
            (format!("file{i}.txt"), b"benign content")
        };
        let p = work.path().join(name);
        fs::write(&p, content).unwrap();
        events.push(FileEvent { kind: FileEventKind::Create, path: p.display().to_string() });
    }

    // Single-file scan latency (warm).
    let t = Instant::now();
    let _ = engine.handle_file_event(&events[0]);
    let scan_latency = t.elapsed();

    // Throughput over all events.
    let t = Instant::now();
    for ev in &events {
        let _ = engine.handle_file_event(ev);
    }
    let elapsed = t.elapsed().as_secs_f64().max(1e-6);

    println!("aegis-realtime benchmark");
    println!("  events            : {n}");
    println!("  alerts raised     : {}", engine.alerts_raised());
    println!("  total time        : {:.0} ms", elapsed * 1000.0);
    println!("  events/sec        : {:.0}", n as f64 / elapsed);
    println!("  event latency avg : {:.1} us", elapsed * 1_000_000.0 / n as f64);
    println!("  single-file scan  : {:.1} us", scan_latency.as_micros() as f64);
    let _ = PathBuf::from("x");
}
