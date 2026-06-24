//! Quarantine benchmark: quarantine throughput, restore throughput, and raw
//! AES-256-GCM encryption overhead.
//!
//! Run: cargo bench -p aegis-quarantine --bench quarantine_bench [num_files] [kib]

use std::fs;
use std::path::Path;
use std::time::Instant;

use aegis_quarantine::{ThreatLevel, Vault, VaultKey};

fn main() {
    let mut args = std::env::args().skip(1);
    let n: usize = args.next().and_then(|a| a.parse().ok()).unwrap_or(1000);
    let kib: usize = args.next().and_then(|a| a.parse().ok()).unwrap_or(64);
    let size = kib * 1024;

    let vault_dir = tempfile::tempdir().unwrap();
    let work = tempfile::tempdir().unwrap();
    let mut conn = aegis_db::open_in_memory_database().unwrap();
    aegis_db::apply_migrations(&mut conn).unwrap();
    let vault = Vault::open(vault_dir.path(), conn).unwrap();

    let payload = vec![0xABu8; size];
    let mut paths = Vec::with_capacity(n);
    for i in 0..n {
        let p = work.path().join(format!("f{i}.bin"));
        fs::write(&p, &payload).unwrap();
        paths.push(p);
    }
    let total_mib = (n * size) as f64 / (1024.0 * 1024.0);

    // Quarantine throughput.
    let t = Instant::now();
    let mut ids = Vec::with_capacity(n);
    for p in &paths {
        let rec = vault.quarantine_file(p, ThreatLevel::High, "bench", "bench").unwrap();
        ids.push(rec.id);
    }
    let q = t.elapsed().as_secs_f64().max(1e-6);

    // Restore throughput (back to original paths).
    let t = Instant::now();
    for id in &ids {
        vault.restore_file(id, None, "bench").unwrap();
    }
    let r = t.elapsed().as_secs_f64().max(1e-6);

    // Raw encryption overhead.
    let key = VaultKey::from_bytes(&[42u8; 32]).unwrap();
    let t = Instant::now();
    let mut sink = 0usize;
    for _ in 0..n {
        sink += key.encrypt(&payload).unwrap().len();
    }
    let e = t.elapsed().as_secs_f64().max(1e-6);
    std::hint::black_box(sink);

    println!("aegis-quarantine benchmark");
    println!("  files            : {n}  ({kib} KiB each, {total_mib:.1} MiB total)");
    println!("  quarantine       : {:.0} ms  | {:.0} files/s | {:.1} MiB/s", q * 1000.0, n as f64 / q, total_mib / q);
    println!("  restore          : {:.0} ms  | {:.0} files/s | {:.1} MiB/s", r * 1000.0, n as f64 / r, total_mib / r);
    println!("  aes-256-gcm only : {:.0} ms  | {:.1} MiB/s (encryption overhead)", e * 1000.0, total_mib / e);

    let _ = Path::new("x");
}
