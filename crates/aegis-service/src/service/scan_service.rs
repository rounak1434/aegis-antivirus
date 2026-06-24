//! Scan adapter over `aegis-scan`.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use aegis_scan::{scan, ScanOptions, ScanProgress, ScanReport};

/// Runs file scans through the verified scanner engine.
#[derive(Debug, Default)]
pub struct ScanService;

impl ScanService {
    pub fn new() -> Self {
        Self
    }

    /// Execute a scan, forwarding progress to `on_progress` and honoring `cancel`.
    pub fn run(
        &self,
        opts: &ScanOptions,
        cancel: Arc<AtomicBool>,
        on_progress: impl Fn(ScanProgress) + Sync + Send,
    ) -> Result<ScanReport, aegis_scan::ScanError> {
        scan(opts, cancel, on_progress)
    }
}
