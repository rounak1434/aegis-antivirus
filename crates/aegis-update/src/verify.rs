//! Cryptographic verification: SHA-256 integrity + Ed25519 signature, plus
//! rollback and minimum-app-version guards.

use base64::Engine;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::manifest::UpdateManifest;
use crate::version;

#[derive(Debug, Error)]
pub enum VerifyError {
    #[error("invalid public key")]
    BadKey,
    #[error("malformed signature encoding")]
    BadSignatureEncoding,
    #[error("signature verification failed")]
    BadSignature,
    #[error("hash mismatch: file does not match manifest sha256")]
    HashMismatch,
    #[error("rollback rejected: candidate {candidate} is not newer than installed {installed}")]
    Rollback {
        candidate: String,
        installed: String,
    },
    #[error("app too old: update needs app >= {required}, have {have}")]
    AppTooOld { required: String, have: String },
}

/// Lowercase hex SHA-256 of `data`.
pub fn sha256_hex(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    let mut s = String::with_capacity(64);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for b in digest {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

/// Verifies update manifests + payloads against a pinned Ed25519 public key.
pub struct UpdateVerifier {
    key: VerifyingKey,
}

impl UpdateVerifier {
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, VerifyError> {
        let key = VerifyingKey::from_bytes(bytes).map_err(|_| VerifyError::BadKey)?;
        Ok(Self { key })
    }

    /// Build from a hex-encoded 32-byte public key (as pinned in config).
    pub fn from_hex(hex: &str) -> Result<Self, VerifyError> {
        let bytes = decode_hex(hex).ok_or(VerifyError::BadKey)?;
        let arr: [u8; 32] = bytes.try_into().map_err(|_| VerifyError::BadKey)?;
        Self::from_bytes(&arr)
    }

    /// Verify the manifest's Ed25519 signature over its canonical message.
    pub fn verify_manifest(&self, manifest: &UpdateManifest) -> Result<(), VerifyError> {
        let sig_bytes = base64::engine::general_purpose::STANDARD
            .decode(manifest.signature.as_bytes())
            .map_err(|_| VerifyError::BadSignatureEncoding)?;
        let sig_arr: [u8; 64] = sig_bytes
            .try_into()
            .map_err(|_| VerifyError::BadSignatureEncoding)?;
        let signature = Signature::from_bytes(&sig_arr);
        self.key
            .verify(manifest.signed_message().as_bytes(), &signature)
            .map_err(|_| VerifyError::BadSignature)
    }

    /// Verify a downloaded payload's SHA-256 matches the manifest.
    pub fn verify_payload(
        &self,
        manifest: &UpdateManifest,
        data: &[u8],
    ) -> Result<(), VerifyError> {
        if sha256_hex(data) == manifest.sha256.to_ascii_lowercase() {
            Ok(())
        } else {
            Err(VerifyError::HashMismatch)
        }
    }

    /// Reject rollback: candidate must be strictly newer than what's installed.
    pub fn check_rollback(
        &self,
        manifest: &UpdateManifest,
        installed_version: Option<&str>,
    ) -> Result<(), VerifyError> {
        if let Some(installed) = installed_version {
            if !version::is_newer(&manifest.version, installed) {
                return Err(VerifyError::Rollback {
                    candidate: manifest.version.clone(),
                    installed: installed.to_string(),
                });
            }
        }
        Ok(())
    }

    pub fn check_min_app(
        &self,
        manifest: &UpdateManifest,
        app_version: &str,
    ) -> Result<(), VerifyError> {
        if version::at_least(app_version, &manifest.minimum_app_version) {
            Ok(())
        } else {
            Err(VerifyError::AppTooOld {
                required: manifest.minimum_app_version.clone(),
                have: app_version.to_string(),
            })
        }
    }

    /// Full gate: signature + payload hash + rollback + min-app.
    pub fn verify_full(
        &self,
        manifest: &UpdateManifest,
        data: &[u8],
        installed_version: Option<&str>,
        app_version: &str,
    ) -> Result<(), VerifyError> {
        self.verify_manifest(manifest)?;
        self.verify_payload(manifest, data)?;
        self.check_rollback(manifest, installed_version)?;
        self.check_min_app(manifest, app_version)?;
        Ok(())
    }
}

fn decode_hex(s: &str) -> Option<Vec<u8>> {
    let s = s.trim();
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_known() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn bad_key_encoding_rejected() {
        assert!(UpdateVerifier::from_hex("zz").is_err()); // not hex
        assert!(UpdateVerifier::from_hex("abcd").is_err()); // wrong length
    }
}
