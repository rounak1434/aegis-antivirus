//! End-to-end update tests: signed flow, tampering, rollback, anti-rollback.

use base64::Engine;
use chrono::Utc;
use ed25519_dalek::{Signer, SigningKey};

use aegis_update::{
    sha256_hex, LocalFetcher, UpdateComponent, UpdateEngine, UpdateError, UpdateManifest,
    UpdateVerifier, VerifyError,
};

const APP_VERSION: &str = "1.0.0";

fn signing_key() -> SigningKey {
    SigningKey::from_bytes(&[7u8; 32])
}

fn verifier() -> UpdateVerifier {
    UpdateVerifier::from_bytes(&signing_key().verifying_key().to_bytes()).unwrap()
}

/// Write `payload` to the feed dir and return a signed manifest for it.
fn publish(
    feed: &std::path::Path,
    sk: &SigningKey,
    component: UpdateComponent,
    version: &str,
    payload: &[u8],
    min_app: &str,
) -> UpdateManifest {
    let file = format!("{}-{}.bin", component.as_str(), version);
    std::fs::write(feed.join(&file), payload).unwrap();
    let mut m = UpdateManifest {
        version: version.to_string(),
        published_at: Utc::now(),
        sha256: sha256_hex(payload),
        signature: String::new(),
        url: format!("https://feed/{file}"),
        size: payload.len() as u64,
        component,
        minimum_app_version: min_app.to_string(),
    };
    let sig = sk.sign(m.signed_message().as_bytes());
    m.signature = base64::engine::general_purpose::STANDARD.encode(sig.to_bytes());
    m
}

struct Env {
    engine: UpdateEngine,
    feed: tempfile::TempDir,
    _data: tempfile::TempDir,
}

fn env() -> Env {
    let data = tempfile::tempdir().unwrap();
    let feed = tempfile::tempdir().unwrap();
    let db_path = data.path().join("aegis.db");
    let mut conn = aegis_db::open_database(&db_path).unwrap();
    aegis_db::apply_migrations(&mut conn).unwrap();
    let engine = UpdateEngine::new(
        data.path().join("update"),
        verifier(),
        Box::new(LocalFetcher::new(feed.path())),
        db_path,
        APP_VERSION,
    )
    .unwrap();
    Env { engine, feed, _data: data }
}

#[test]
fn signed_update_downloads_installs_and_records() {
    let env = env();
    let sk = signing_key();
    let m = publish(env.feed.path(), &sk, UpdateComponent::SignatureDatabase, "2024.06.22.02", b"sig-bundle-v2", "1.0.0");

    let applicable = env.engine.check(std::slice::from_ref(&m)).unwrap();
    assert_eq!(applicable.len(), 1);

    env.engine.download(&m).unwrap();
    let outcome = env.engine.install(&m).unwrap();
    assert_eq!(outcome.version, "2024.06.22.02");
    assert_eq!(std::fs::read(&outcome.installed_path).unwrap(), b"sig-bundle-v2");

    let status = env.engine.status().unwrap();
    assert!(status.iter().any(|(c, v)| c == "signature_database" && v == "2024.06.22.02"));
}

#[test]
fn tampered_payload_is_rejected() {
    let env = env();
    let sk = signing_key();
    let mut m = publish(env.feed.path(), &sk, UpdateComponent::YaraRules, "1.0.0", b"real-rules", "1.0.0");
    // Corrupt the feed file *after* signing the manifest (hash now mismatches).
    std::fs::write(env.feed.path().join("yara_rules-1.0.0.bin"), b"EVIL-tampered").unwrap();
    // re-point url filename stays same
    m.url = "https://feed/yara_rules-1.0.0.bin".into();

    let err = env.engine.download(&m).unwrap_err();
    assert!(matches!(err, UpdateError::Verify(VerifyError::HashMismatch)));
}

#[test]
fn invalid_signature_is_rejected() {
    let env = env();
    let sk = signing_key();
    let mut m = publish(env.feed.path(), &sk, UpdateComponent::ThreatMetadata, "2.0.0", b"meta", "1.0.0");
    m.version = "9.9.9".into(); // mutate signed field ⇒ signature no longer valid

    assert!(env.engine.check(std::slice::from_ref(&m)).unwrap().is_empty());
    let err = env.engine.download(&m).unwrap_err();
    assert!(matches!(err, UpdateError::Verify(VerifyError::BadSignature)));
}

#[test]
fn rollback_restores_previous_version() {
    let env = env();
    let sk = signing_key();
    let v1 = publish(env.feed.path(), &sk, UpdateComponent::SignatureDatabase, "1.0.0", b"bundle-v1", "1.0.0");
    let v2 = publish(env.feed.path(), &sk, UpdateComponent::SignatureDatabase, "1.1.0", b"bundle-v2", "1.0.0");

    env.engine.download(&v1).unwrap();
    env.engine.install(&v1).unwrap();
    env.engine.download(&v2).unwrap();
    let out2 = env.engine.install(&v2).unwrap();
    assert_eq!(std::fs::read(&out2.installed_path).unwrap(), b"bundle-v2");

    let rolled = env.engine.rollback(UpdateComponent::SignatureDatabase).unwrap();
    assert_eq!(rolled.version, "1.0.0");
    assert_eq!(std::fs::read(&rolled.installed_path).unwrap(), b"bundle-v1");
}

#[test]
fn anti_rollback_rejects_older_version() {
    let env = env();
    let sk = signing_key();
    let v2 = publish(env.feed.path(), &sk, UpdateComponent::SignatureDatabase, "2.0.0", b"v2", "1.0.0");
    let v1 = publish(env.feed.path(), &sk, UpdateComponent::SignatureDatabase, "1.0.0", b"v1", "1.0.0");

    env.engine.download(&v2).unwrap();
    env.engine.install(&v2).unwrap();

    // v1 is older ⇒ excluded by check() and rejected by install().
    assert!(env.engine.check(std::slice::from_ref(&v1)).unwrap().is_empty());
    env.engine.download(&v1).unwrap();
    let err = env.engine.install(&v1).unwrap_err();
    assert!(matches!(err, UpdateError::Verify(VerifyError::Rollback { .. })));
}

#[test]
fn min_app_version_blocks_install() {
    let env = env();
    let sk = signing_key();
    let m = publish(env.feed.path(), &sk, UpdateComponent::EngineConfig, "3.0.0", b"cfg", "9.0.0");
    assert!(env.engine.check(std::slice::from_ref(&m)).unwrap().is_empty());
    env.engine.download(&m).unwrap();
    let err = env.engine.install(&m).unwrap_err();
    assert!(matches!(err, UpdateError::Verify(VerifyError::AppTooOld { .. })));
}
