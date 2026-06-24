//! Per-engine service adapters. Each wraps one verified engine crate behind a
//! thin, service-owned API. The orchestrator composes these; the UI never calls
//! the engine crates directly.

pub mod detection_service;
pub mod quarantine_service;
pub mod scan_service;
pub mod status_service;
pub mod windows_service;
