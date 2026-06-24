//! AES-256-GCM encryption for the quarantine vault.
//!
//! On-disk format per quarantined file: `[12-byte random nonce][ciphertext+tag]`.
//! The 256-bit vault key is generated once and stored in the vault directory.
//!
//! NOTE: storing the key beside the vault protects against casual access and
//! ensures malware is never at rest in plaintext, but it is not a substitute for
//! OS-backed key protection. A future hardening step should wrap the key with
//! Windows DPAPI / a TPM. Documented in `QUARANTINE_SYSTEM.md`.

use std::fs;
use std::path::{Path, PathBuf};

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use rand::rngs::OsRng;
use rand::RngCore;

const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("io error on key file {path}: {source}")]
    KeyIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("vault key is corrupt (expected {KEY_LEN} bytes, got {0})")]
    BadKey(usize),
    #[error("ciphertext too short to contain a nonce")]
    ShortCiphertext,
    #[error("decryption failed (wrong key or tampered data)")]
    Decrypt,
    #[error("encryption failed")]
    Encrypt,
}

/// The vault's symmetric key.
pub struct VaultKey {
    cipher: Aes256Gcm,
}

impl VaultKey {
    /// Load the key from `path`, or generate and persist a new one if absent.
    pub fn load_or_create(path: impl AsRef<Path>) -> Result<Self, CryptoError> {
        let path = path.as_ref();
        let bytes = if path.exists() {
            let b = fs::read(path).map_err(|source| CryptoError::KeyIo {
                path: path.to_path_buf(),
                source,
            })?;
            if b.len() != KEY_LEN {
                return Err(CryptoError::BadKey(b.len()));
            }
            b
        } else {
            let mut b = vec![0u8; KEY_LEN];
            OsRng.fill_bytes(&mut b);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|source| CryptoError::KeyIo {
                    path: parent.to_path_buf(),
                    source,
                })?;
            }
            fs::write(path, &b).map_err(|source| CryptoError::KeyIo {
                path: path.to_path_buf(),
                source,
            })?;
            restrict_permissions(path);
            b
        };
        let key = Key::<Aes256Gcm>::from_slice(&bytes);
        Ok(Self { cipher: Aes256Gcm::new(key) })
    }

    /// Build a key directly from raw bytes (tests / key import).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() != KEY_LEN {
            return Err(CryptoError::BadKey(bytes.len()));
        }
        let key = Key::<Aes256Gcm>::from_slice(bytes);
        Ok(Self { cipher: Aes256Gcm::new(key) })
    }

    /// Encrypt `plaintext` → `nonce || ciphertext`.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ct = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| CryptoError::Encrypt)?;
        let mut out = Vec::with_capacity(NONCE_LEN + ct.len());
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ct);
        Ok(out)
    }

    /// Decrypt a `nonce || ciphertext` blob.
    pub fn decrypt(&self, blob: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if blob.len() < NONCE_LEN {
            return Err(CryptoError::ShortCiphertext);
        }
        let (nonce_bytes, ct) = blob.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce_bytes);
        self.cipher
            .decrypt(nonce, ct)
            .map_err(|_| CryptoError::Decrypt)
    }
}

#[cfg(windows)]
fn restrict_permissions(_path: &Path) {
    // Windows ACL tightening is deferred to the installer (service account owns
    // the vault dir). No-op here to avoid requiring elevation in dev.
}

#[cfg(not(windows))]
fn restrict_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let key = VaultKey::from_bytes(&[7u8; 32]).unwrap();
        let blob = key.encrypt(b"malware bytes").unwrap();
        assert_ne!(&blob[12..], b"malware bytes"); // never plaintext
        assert_eq!(key.decrypt(&blob).unwrap(), b"malware bytes");
    }

    #[test]
    fn wrong_key_fails() {
        let k1 = VaultKey::from_bytes(&[1u8; 32]).unwrap();
        let k2 = VaultKey::from_bytes(&[2u8; 32]).unwrap();
        let blob = k1.encrypt(b"x").unwrap();
        assert!(matches!(k2.decrypt(&blob), Err(CryptoError::Decrypt)));
    }

    #[test]
    fn tamper_detected() {
        let key = VaultKey::from_bytes(&[9u8; 32]).unwrap();
        let mut blob = key.encrypt(b"payload").unwrap();
        let last = blob.len() - 1;
        blob[last] ^= 0xff;
        assert!(key.decrypt(&blob).is_err());
    }

    #[test]
    fn unique_nonce_per_encrypt() {
        let key = VaultKey::from_bytes(&[3u8; 32]).unwrap();
        let a = key.encrypt(b"same").unwrap();
        let b = key.encrypt(b"same").unwrap();
        assert_ne!(a, b); // random nonce → different ciphertext
    }
}
