//! Aegis quarantine system.
//!
//! Receives malicious files (by path or [`aegis_detect::ThreatDetection`]) and
//! safely isolates them in an AES-256-GCM-encrypted vault. Plaintext malware is
//! never stored at rest. Every quarantine / restore / delete action is audited,
//! and restores are integrity-checked (SHA-256) and path-validated.

mod crypto;
mod db;
mod model;
mod vault;

pub use crypto::{CryptoError, VaultKey};
pub use model::{AuditAction, QuarantineError, QuarantineRecord, QuarantineStatus, ThreatLevel};
pub use vault::Vault;

// Low-level persistence is exposed for reporting/service integration.
pub use db::{get_record, list_records, write_audit};
