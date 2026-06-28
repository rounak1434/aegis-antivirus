//! Detection adapter over `aegis-detect`.

use aegis_detect::{persist_detection, DetectionEngine, SignatureDatabase, ThreatDetection};
use aegis_scan::ScanReport;
use aegis_yara::RuleManager;
use rusqlite::Connection;

use crate::ServiceError;

/// Runs the detection engine over scanner output and persists results.
#[derive(Debug, Default)]
pub struct DetectionService {
    engine: DetectionEngine,
}

impl DetectionService {
    pub fn new() -> Self {
        Self {
            engine: DetectionEngine::new(),
        }
    }

    /// Analyze every file in a scan report into explainable detections.
    pub fn analyze(
        &self,
        report: &ScanReport,
        signatures: &SignatureDatabase,
        yara: Option<&RuleManager>,
    ) -> Vec<ThreatDetection> {
        self.engine.analyze_report(report, signatures, yara)
    }

    /// Persist detections to `detection_results`.
    pub fn persist(
        &self,
        conn: &Connection,
        detections: &[ThreatDetection],
    ) -> Result<(), ServiceError> {
        for det in detections {
            persist_detection(conn, det).map_err(|e| ServiceError::Other(e.to_string()))?;
        }
        Ok(())
    }
}
