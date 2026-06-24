//! Aegis real-time protection.
//!
//! Monitors filesystem + process activity and feeds events into the verified
//! engines (scan → detect → quarantine). The engines are **reused, not
//! rewritten**: the pipeline calls `aegis-scan`, `aegis-detect`, and
//! `aegis-quarantine` directly. Policy decides per-detection action
//! (monitor / notify / auto-quarantine).

mod db;
mod monitor;
mod pipeline;
pub mod policy;

mod model;

pub use model::{
    FileEvent, FileEventKind, ProcessEvent, ProtectionMode, RealtimeAction, RealtimeAlert,
    RealtimeStatus, ThreatLevel,
};
pub use monitor::{default_watched_paths, Debouncer, RealtimeMonitor};
pub use pipeline::RealtimeEngine;
