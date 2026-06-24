//! Aegis Windows security scanner.
//!
//! Discovers persistence mechanisms across Windows autostart surfaces — startup
//! folders, registry Run/RunOnce keys, scheduled tasks, services, drivers,
//! browser extensions, and the hosts file — applies heuristics, and converts
//! suspicious findings into uniform [`aegis_detect::ThreatDetection`]s.
//!
//! Collectors are best-effort and Windows-only (they return empty on other
//! platforms); the parsers and heuristics are pure and unit-tested everywhere.

pub mod browser_extensions;
pub mod drivers;
pub mod heuristics;
pub mod hosts_file;
pub mod registry;
pub mod scheduled_tasks;
pub mod services;
pub mod startup;

mod engine;
mod model;
mod util;

pub use engine::{collect_all, WindowsScanner};
pub use model::{PersistenceEntry, PersistenceKind};

// Re-export the detection types callers receive.
pub use aegis_detect::{ThreatDetection, ThreatEvidence, ThreatLevel};
