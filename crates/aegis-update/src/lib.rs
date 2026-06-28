//! Aegis secure update system.
//!
//! Downloads, **cryptographically verifies**, installs, and rolls back updates
//! for signatures, YARA rules, threat metadata, and engine configuration. Every
//! payload is gated by SHA-256 integrity + an Ed25519 signature over a canonical
//! manifest message, with anti-rollback and minimum-app-version checks.
//!
//! This crate only produces verified files on disk and records what's installed;
//! reloading the running engines from those files is the service's job (no
//! engine crate is modified here).

mod db;
mod download;
mod engine;
mod manifest;
mod scheduler;
mod storage;
mod verify;
mod version;

pub use download::{DownloadError, Fetcher, LocalFetcher, ReqwestFetcher};
pub use engine::{InstallOutcome, UpdateEngine, UpdateError};
pub use manifest::{UpdateComponent, UpdateManifest};
pub use scheduler::UpdateSchedule;
pub use storage::{StorageError, UpdateStorage};
pub use verify::{sha256_hex, UpdateVerifier, VerifyError};
pub use version::{at_least, compare, is_newer};
