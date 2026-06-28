//! Update benchmark: download (local), verification (sha256 + Ed25519), install.
//!
//! Run: cargo bench -p aegis-update --bench update_bench [payload_mib]

use std::time::Instant;

use base64::Engine;
use chrono::Utc;
use ed25519_dalek::{Signer, SigningKey};

use aegis_update::{
    sha256_hex, LocalFetcher, UpdateComponent, UpdateEngine, UpdateManifest, UpdateVerifier,
};

fn main() {
    let mib: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(8);
    let payload = vec![0xA5u8; mib * 1024 * 1024];

    let data = tempfile::tempdir().unwrap();
    let feed = tempfile::tempdir().unwrap();
    let db_path = data.path().join("aegis.db");
    let mut conn = aegis_db::open_database(&db_path).unwrap();
    aegis_db::apply_migrations(&mut conn).unwrap();

    let sk = SigningKey::from_bytes(&[9u8; 32]);
    let verifier = UpdateVerifier::from_bytes(&sk.verifying_key().to_bytes()).unwrap();

    let file = "signature_database-2024.06.22.10.bin";
    std::fs::write(feed.path().join(file), &payload).unwrap();
    let mut m = UpdateManifest {
        version: "2024.06.22.10".into(),
        published_at: Utc::now(),
        sha256: sha256_hex(&payload),
        signature: String::new(),
        url: format!("https://feed/{file}"),
        size: payload.len() as u64,
        component: UpdateComponent::SignatureDatabase,
        minimum_app_version: "1.0.0".into(),
    };
    m.signature = base64::engine::general_purpose::STANDARD
        .encode(sk.sign(m.signed_message().as_bytes()).to_bytes());

    let engine = UpdateEngine::new(
        data.path().join("update"),
        verifier,
        Box::new(LocalFetcher::new(feed.path())),
        db_path,
        "1.0.0",
    )
    .unwrap();

    let mb = payload.len() as f64 / (1024.0 * 1024.0);

    let t = Instant::now();
    engine.download(&m).unwrap(); // fetch + sha256 + signature verify
    let dl_verify = t.elapsed();

    let t = Instant::now();
    let _ = engine.install(&m).unwrap(); // re-verify + backup + swap + persist
    let install = t.elapsed();

    // Isolate raw hashing throughput.
    let t = Instant::now();
    let _ = sha256_hex(&payload);
    let hash = t.elapsed();

    println!("aegis-update benchmark");
    println!("  payload            : {mib} MiB");
    println!(
        "  download+verify    : {:.1} ms  ({:.0} MiB/s)",
        dl_verify.as_secs_f64() * 1000.0,
        mb / dl_verify.as_secs_f64()
    );
    println!(
        "  install            : {:.1} ms",
        install.as_secs_f64() * 1000.0
    );
    println!(
        "  sha256 throughput  : {:.0} MiB/s",
        mb / hash.as_secs_f64().max(1e-6)
    );
}
