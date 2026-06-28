//! Windows persistence-scan adapter over `aegis-windows`.

use aegis_detect::ThreatDetection;
use aegis_windows::WindowsScanner;

/// Runs Windows persistence sweeps through the verified scanner.
#[derive(Debug, Default)]
pub struct WindowsSecurityService {
    scanner: WindowsScanner,
}

impl WindowsSecurityService {
    pub fn new() -> Self {
        Self {
            scanner: WindowsScanner::new(),
        }
    }

    /// Collect every persistence surface and return suspicious detections.
    /// On non-Windows hosts the collectors are empty, so this returns `[]`.
    pub fn run(&self) -> Vec<ThreatDetection> {
        self.scanner.scan_all()
    }

    /// Analyze a caller-supplied batch (used by tests with mock fixtures).
    pub fn analyze(&self, entries: &[aegis_windows::PersistenceEntry]) -> Vec<ThreatDetection> {
        self.scanner.analyze_entries(entries)
    }
}
