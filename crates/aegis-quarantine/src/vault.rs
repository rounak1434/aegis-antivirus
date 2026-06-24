//! The quarantine vault: encrypts and isolates malicious files, and safely
//! restores or permanently deletes them. All actions are audited.

use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use chrono::Utc;
use rusqlite::Connection;
use sha2::{Digest, Sha256};

use crate::crypto::VaultKey;
use crate::db;
use crate::model::{
    AuditAction, QuarantineError, QuarantineRecord, QuarantineStatus, ThreatLevel,
};

const KEY_FILE: &str = "vault.key";
const VAULT_EXT: &str = "qbin";

/// An encrypted, audited quarantine vault backed by a directory + SQLite.
pub struct Vault {
    dir: PathBuf,
    key: VaultKey,
    conn: Connection,
}

impl Vault {
    /// Open (creating if needed) a vault at `dir`, using `conn` for records and
    /// audit. The connection must already have migrations applied.
    pub fn open(dir: impl AsRef<Path>, conn: Connection) -> Result<Self, QuarantineError> {
        let dir = dir.as_ref().to_path_buf();
        fs::create_dir_all(&dir).map_err(|source| QuarantineError::Io {
            path: dir.display().to_string(),
            source,
        })?;
        let key = VaultKey::load_or_create(dir.join(KEY_FILE))?;
        Ok(Self { dir, key, conn })
    }

    /// Borrow the underlying connection (e.g. for joined reporting queries).
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Encrypt and isolate a file, then remove the plaintext original.
    pub fn quarantine_file(
        &self,
        original_path: impl AsRef<Path>,
        threat_level: ThreatLevel,
        reason: impl Into<String>,
        actor: &str,
    ) -> Result<QuarantineRecord, QuarantineError> {
        let original_path = original_path.as_ref();
        let plaintext = fs::read(original_path).map_err(|source| QuarantineError::Io {
            path: original_path.display().to_string(),
            source,
        })?;
        let sha256 = sha256_hex(&plaintext);
        let id = uuid::Uuid::new_v4().to_string();
        let vault_path = self.dir.join(format!("{id}.{VAULT_EXT}"));

        let blob = self.key.encrypt(&plaintext)?;
        fs::write(&vault_path, &blob).map_err(|source| QuarantineError::Io {
            path: vault_path.display().to_string(),
            source,
        })?;

        // Neutralise the original (best-effort shred, then remove).
        shred(original_path)?;

        let record = QuarantineRecord {
            id: id.clone(),
            original_path: original_path.display().to_string(),
            quarantine_path: vault_path.display().to_string(),
            sha256,
            threat_level,
            reason: reason.into(),
            timestamp: Utc::now(),
            size: plaintext.len() as u64,
            encrypted: true,
            status: QuarantineStatus::Quarantined,
        };
        db::insert_record(&self.conn, &record)?;
        db::write_audit(&self.conn, AuditAction::Quarantine, actor, &record.id, "ok")?;
        Ok(record)
    }

    /// Convenience: quarantine the file named by a detection.
    pub fn quarantine_detection(
        &self,
        det: &aegis_detect::ThreatDetection,
        actor: &str,
    ) -> Result<QuarantineRecord, QuarantineError> {
        let reason = det
            .evidence
            .iter()
            .map(|e| e.label())
            .collect::<Vec<_>>()
            .join(", ");
        self.quarantine_file(&det.path, det.threat_level, reason, actor)
    }

    /// Restore a quarantined file. Verifies the record, decrypts, checks SHA-256
    /// integrity, and writes to a validated target (defaults to the original
    /// path). Refuses path traversal, missing parents, and overwrites.
    pub fn restore_file(
        &self,
        id: &str,
        dest: Option<&Path>,
        actor: &str,
    ) -> Result<PathBuf, QuarantineError> {
        let record = self.require_quarantined(id)?;

        let blob = fs::read(&record.quarantine_path).map_err(|source| QuarantineError::Io {
            path: record.quarantine_path.clone(),
            source,
        })?;
        let plaintext = self.key.decrypt(&blob)?;

        // Integrity: decrypted content must match the recorded digest.
        let actual = sha256_hex(&plaintext);
        if actual != record.sha256 {
            db::write_audit(&self.conn, AuditAction::Restore, actor, id, "integrity_mismatch")?;
            return Err(QuarantineError::IntegrityMismatch {
                id: id.to_string(),
                expected: record.sha256,
                actual,
            });
        }

        let target = match dest {
            Some(p) => p.to_path_buf(),
            None => PathBuf::from(&record.original_path),
        };
        validate_restore_path(&target)?;

        fs::write(&target, &plaintext).map_err(|source| QuarantineError::Io {
            path: target.display().to_string(),
            source,
        })?;

        // Remove the vault copy now that the file is restored.
        let _ = shred(Path::new(&record.quarantine_path));
        db::set_status(&self.conn, id, QuarantineStatus::Restored)?;
        db::write_audit(&self.conn, AuditAction::Restore, actor, id, "ok")?;
        Ok(target)
    }

    /// Permanently shred and delete a quarantined file.
    pub fn delete_file(&self, id: &str, actor: &str) -> Result<(), QuarantineError> {
        let record = self.require_quarantined(id)?;
        let _ = shred(Path::new(&record.quarantine_path));
        db::set_status(&self.conn, id, QuarantineStatus::Deleted)?;
        db::write_audit(&self.conn, AuditAction::Delete, actor, id, "ok")?;
        Ok(())
    }

    pub fn get_record(&self, id: &str) -> Result<Option<QuarantineRecord>, QuarantineError> {
        db::get_record(&self.conn, id)
    }

    pub fn list_records(&self) -> Result<Vec<QuarantineRecord>, QuarantineError> {
        db::list_records(&self.conn)
    }

    fn require_quarantined(&self, id: &str) -> Result<QuarantineRecord, QuarantineError> {
        let record = db::get_record(&self.conn, id)?
            .ok_or_else(|| QuarantineError::NotFound(id.to_string()))?;
        if record.status != QuarantineStatus::Quarantined {
            return Err(QuarantineError::NotInVault {
                id: id.to_string(),
                status: match record.status {
                    QuarantineStatus::Restored => "restored",
                    QuarantineStatus::Deleted => "deleted",
                    QuarantineStatus::Quarantined => "quarantined",
                },
            });
        }
        Ok(record)
    }
}

fn sha256_hex(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    let mut s = String::with_capacity(64);
    for b in digest {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Reject path traversal, relative paths, missing parents, and overwrites.
fn validate_restore_path(target: &Path) -> Result<(), QuarantineError> {
    if !target.is_absolute() {
        return Err(QuarantineError::UnsafePath(format!(
            "{} is not absolute",
            target.display()
        )));
    }
    if target.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(QuarantineError::UnsafePath(format!(
            "{} contains '..'",
            target.display()
        )));
    }
    match target.parent() {
        Some(parent) if parent.is_dir() => {}
        _ => {
            return Err(QuarantineError::UnsafePath(format!(
                "parent directory of {} does not exist",
                target.display()
            )))
        }
    }
    if target.exists() {
        return Err(QuarantineError::TargetExists(target.display().to_string()));
    }
    Ok(())
}

/// Overwrite a file's bytes with zeros, then remove it (best-effort shred).
fn shred(path: &Path) -> Result<(), QuarantineError> {
    if let Ok(meta) = fs::metadata(path) {
        if meta.is_file() {
            if let Ok(mut f) = fs::OpenOptions::new().write(true).open(path) {
                let zeros = vec![0u8; meta.len().min(1024 * 1024) as usize];
                let mut remaining = meta.len();
                while remaining > 0 {
                    let chunk = remaining.min(zeros.len() as u64) as usize;
                    if f.write_all(&zeros[..chunk]).is_err() {
                        break;
                    }
                    remaining -= chunk as u64;
                }
                let _ = f.flush();
            }
        }
    }
    fs::remove_file(path).map_err(|source| QuarantineError::Io {
        path: path.display().to_string(),
        source,
    })
}
