//! Aegis detection engine.
//!
//! Consumes `aegis-scan` output and produces explainable threat detections by
//! combining three layers:
//! - **hash signatures** ([`aegis_signatures::SignatureDatabase`]),
//! - **YARA-X rules** ([`aegis_yara::RuleManager`]),
//! - **heuristics** (double/suspicious extension, entropy/packing, script and
//!   PowerShell-abuse indicators).
//!
//! Each detection carries typed [`ThreatEvidence`], an additive 0–100 score,
//! and a [`ThreatLevel`] — no black-box scoring.

mod db;
mod engine;
pub mod heuristics;
mod model;

pub use db::{detection_count, persist_detection, record_scan_event, DetectDbError};
pub use engine::{DetectionEngine, EngineConfig};
pub use model::{clamp_score, level_for_score, ThreatDetection, ThreatEvidence, ThreatLevel};

// Re-export the layer types callers commonly need.
pub use aegis_signatures::{HashAlgo, SignatureDatabase};
pub use aegis_yara::{RuleManager, YaraMatch};
