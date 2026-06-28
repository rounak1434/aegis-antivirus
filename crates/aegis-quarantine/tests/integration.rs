//! End-to-end quarantine vault tests.

use std::fs;
use std::path::{Path, PathBuf};

use aegis_quarantine::{QuarantineError, QuarantineStatus, ThreatLevel, Vault};

struct Fixture {
    vault: Vault,
    _vault_dir: tempfile::TempDir,
    work: tempfile::TempDir,
}

fn fixture() -> Fixture {
    let vault_dir = tempfile::tempdir().unwrap();
    let work = tempfile::tempdir().unwrap();
    let mut conn = aegis_db::open_in_memory_database().unwrap();
    aegis_db::apply_migrations(&mut conn).unwrap();
    let vault = Vault::open(vault_dir.path(), conn).unwrap();
    Fixture {
        vault,
        _vault_dir: vault_dir,
        work,
    }
}

fn plant(dir: &Path, name: &str, content: &[u8]) -> PathBuf {
    let p = dir.join(name);
    fs::write(&p, content).unwrap();
    p
}

#[test]
fn quarantine_isolates_and_encrypts() {
    let fx = fixture();
    let file = plant(fx.work.path(), "malware.exe", b"MZ evil payload");

    let rec = fx
        .vault
        .quarantine_file(&file, ThreatLevel::Critical, "hash_match", "tester")
        .unwrap();

    assert_eq!(rec.status, QuarantineStatus::Quarantined);
    assert!(rec.encrypted);
    assert_eq!(rec.size, 15);
    assert!(!file.exists(), "original must be removed");

    // Vault file exists and is NOT plaintext.
    let blob = fs::read(&rec.quarantine_path).unwrap();
    assert!(!blob.windows(2).any(|w| w == b"MZ"));
    assert_eq!(fx.vault.list_records().unwrap().len(), 1);
}

#[test]
fn restore_round_trips_content() {
    let fx = fixture();
    let file = plant(fx.work.path(), "doc.bin", b"original-content-1234");
    let rec = fx
        .vault
        .quarantine_file(&file, ThreatLevel::High, "heuristic", "tester")
        .unwrap();

    // Restore to original path (parent exists, file was removed).
    let restored = fx.vault.restore_file(&rec.id, None, "tester").unwrap();
    assert_eq!(restored, file);
    assert_eq!(fs::read(&file).unwrap(), b"original-content-1234");
    assert_eq!(
        fx.vault.get_record(&rec.id).unwrap().unwrap().status,
        QuarantineStatus::Restored
    );
}

#[test]
fn delete_shreds_vault_file() {
    let fx = fixture();
    let file = plant(fx.work.path(), "x.scr", b"payload");
    let rec = fx
        .vault
        .quarantine_file(&file, ThreatLevel::Medium, "ext", "tester")
        .unwrap();
    assert!(Path::new(&rec.quarantine_path).exists());

    fx.vault.delete_file(&rec.id, "tester").unwrap();
    assert!(!Path::new(&rec.quarantine_path).exists());
    assert_eq!(
        fx.vault.get_record(&rec.id).unwrap().unwrap().status,
        QuarantineStatus::Deleted
    );
}

#[test]
fn integrity_mismatch_blocks_restore() {
    let fx = fixture();
    let file = plant(fx.work.path(), "a.bin", b"content");
    let rec = fx
        .vault
        .quarantine_file(&file, ThreatLevel::High, "r", "tester")
        .unwrap();

    // Corrupt the recorded digest so decrypted content won't match.
    fx.vault
        .connection()
        .execute(
            "UPDATE quarantine_records SET sha256 = 'deadbeef' WHERE id = ?1",
            rusqlite::params![rec.id],
        )
        .unwrap();

    let err = fx.vault.restore_file(&rec.id, None, "tester").unwrap_err();
    assert!(matches!(err, QuarantineError::IntegrityMismatch { .. }));
}

#[test]
fn tampered_ciphertext_fails_to_decrypt() {
    let fx = fixture();
    let file = plant(fx.work.path(), "b.bin", b"content");
    let rec = fx
        .vault
        .quarantine_file(&file, ThreatLevel::High, "r", "tester")
        .unwrap();

    // Flip a byte in the encrypted blob → GCM auth tag rejects it.
    let mut blob = fs::read(&rec.quarantine_path).unwrap();
    let last = blob.len() - 1;
    blob[last] ^= 0xff;
    fs::write(&rec.quarantine_path, &blob).unwrap();

    assert!(fx.vault.restore_file(&rec.id, None, "tester").is_err());
}

#[test]
fn quarantine_missing_file_errors() {
    let fx = fixture();
    let missing = fx.work.path().join("nope.exe");
    let err = fx
        .vault
        .quarantine_file(&missing, ThreatLevel::Low, "r", "tester")
        .unwrap_err();
    assert!(matches!(err, QuarantineError::Io { .. }));
}

#[test]
fn restore_unknown_record_errors() {
    let fx = fixture();
    let err = fx
        .vault
        .restore_file("no-such-id", None, "tester")
        .unwrap_err();
    assert!(matches!(err, QuarantineError::NotFound(_)));
}

#[test]
fn path_traversal_and_overwrite_rejected() {
    let fx = fixture();
    let file = plant(fx.work.path(), "c.bin", b"data");
    let rec = fx
        .vault
        .quarantine_file(&file, ThreatLevel::High, "r", "tester")
        .unwrap();

    // Relative path rejected.
    let rel = fx
        .vault
        .restore_file(&rec.id, Some(Path::new("rel.bin")), "tester")
        .unwrap_err();
    assert!(matches!(rel, QuarantineError::UnsafePath(_)));

    // Parent-dir traversal rejected.
    let trav = fx.work.path().join("..").join("escape.bin");
    let t = fx
        .vault
        .restore_file(&rec.id, Some(&trav), "tester")
        .unwrap_err();
    assert!(matches!(t, QuarantineError::UnsafePath(_)));

    // Overwrite of an existing file rejected.
    let existing = plant(fx.work.path(), "existing.bin", b"keep me");
    let ov = fx
        .vault
        .restore_file(&rec.id, Some(&existing), "tester")
        .unwrap_err();
    assert!(matches!(ov, QuarantineError::TargetExists(_)));
    assert_eq!(fs::read(&existing).unwrap(), b"keep me"); // untouched
}

#[test]
fn double_restore_blocked() {
    let fx = fixture();
    let file = plant(fx.work.path(), "d.bin", b"data");
    let rec = fx
        .vault
        .quarantine_file(&file, ThreatLevel::High, "r", "tester")
        .unwrap();
    fx.vault.restore_file(&rec.id, None, "tester").unwrap();
    let err = fx.vault.restore_file(&rec.id, None, "tester").unwrap_err();
    assert!(matches!(err, QuarantineError::NotInVault { .. }));
}

#[test]
fn audit_log_records_actions() {
    let fx = fixture();
    let file = plant(fx.work.path(), "e.bin", b"data");
    let rec = fx
        .vault
        .quarantine_file(&file, ThreatLevel::High, "r", "tester")
        .unwrap();
    fx.vault.delete_file(&rec.id, "tester").unwrap();

    let count: i64 = fx
        .vault
        .connection()
        .query_row("SELECT COUNT(*) FROM audit_log", [], |r| r.get(0))
        .unwrap();
    assert!(count >= 2, "quarantine + delete should both be audited");
}
