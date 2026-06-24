//! Detection engine — sits above `aegis-scan` output and produces explainable
//! [`ThreatDetection`]s by combining hash signatures, YARA rules, and heuristics.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use aegis_scan::ScannedFile;
use aegis_signatures::SignatureDatabase;
use aegis_yara::RuleManager;
use chrono::Utc;

use crate::heuristics;
use crate::model::{ThreatDetection, ThreatEvidence};

/// Tunable thresholds for the entropy heuristic.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Entropy at/above which an *executable* is flagged as packed.
    pub packed_entropy_min: f64,
    /// Entropy at/above which any file is flagged high-entropy.
    pub generic_entropy_min: f64,
    /// Files smaller than this (bytes) are skipped by the entropy heuristic
    /// (tiny files trivially reach high entropy and are noisy).
    pub min_entropy_size: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self { packed_entropy_min: 7.0, generic_entropy_min: 7.5, min_entropy_size: 256 }
    }
}

/// Stateless detection engine. Borrowed signature DB and optional YARA manager
/// are passed per call so callers control their lifecycle.
#[derive(Debug, Default)]
pub struct DetectionEngine {
    config: EngineConfig,
}

impl DetectionEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: EngineConfig) -> Self {
        Self { config }
    }

    /// Analyze one scanned file. Returns `None` if no evidence is found.
    pub fn analyze(
        &self,
        scanned: &ScannedFile,
        signatures: &SignatureDatabase,
        yara: Option<&RuleManager>,
    ) -> Option<ThreatDetection> {
        let path = &scanned.metadata.path;
        let mut evidence = Vec::new();

        // 1. Hash signatures (from scanner-provided digests).
        if let Some(h) = &scanned.hashes {
            if signatures.contains_sha256(&h.sha256) {
                evidence.push(ThreatEvidence::HashMatch {
                    algo: "sha256".into(),
                    hex: h.sha256.clone(),
                    threat_name: None,
                });
            }
            if signatures.contains_md5(&h.md5) {
                evidence.push(ThreatEvidence::HashMatch {
                    algo: "md5".into(),
                    hex: h.md5.clone(),
                    threat_name: None,
                });
            }
        }

        // 2. Name-based heuristics (no content needed).
        if let Some((file_name, decoy_ext, real_ext)) = heuristics::double_extension(path) {
            evidence.push(ThreatEvidence::DoubleExtension { file_name, decoy_ext, real_ext });
        }
        if let Some(ext) = heuristics::suspicious_extension(path) {
            evidence.push(ThreatEvidence::SuspiciousExtension { ext });
        }

        // 3. Content heuristics + YARA (best-effort; unreadable files skip these).
        if !scanned.metadata.is_symlink {
            if let Ok(content) = read_head(path, heuristics::CONTENT_SCAN_LIMIT) {
                evidence.extend(self.content_evidence(path, &content));
                if let Some(mgr) = yara {
                    if mgr.is_compiled() {
                        if let Ok(hits) = mgr.scan_bytes(&content) {
                            for m in hits {
                                evidence.push(ThreatEvidence::YaraMatch {
                                    rule: m.rule,
                                    namespace: m.namespace,
                                    patterns: m.patterns,
                                });
                            }
                        }
                    }
                }
            }
        }

        ThreatDetection::from_evidence(path.display().to_string(), evidence, Utc::now())
    }

    /// Heuristic evidence derived purely from file content + name (no I/O).
    /// Public so tests and callers can analyze in-memory buffers.
    pub fn content_evidence(&self, path: &Path, content: &[u8]) -> Vec<ThreatEvidence> {
        let mut evidence = Vec::new();

        if content.len() >= self.config.min_entropy_size {
            let entropy = heuristics::shannon_entropy(content);
            if heuristics::is_executable(path, content) && entropy >= self.config.packed_entropy_min {
                evidence.push(ThreatEvidence::PackedExecutable { entropy });
            } else if entropy >= self.config.generic_entropy_min {
                evidence.push(ThreatEvidence::EntropyDetection { entropy });
            }
        }

        for indicator in heuristics::script_indicators(content) {
            evidence.push(ThreatEvidence::ScriptIndicator { indicator });
        }
        for indicator in heuristics::powershell_indicators(content) {
            evidence.push(ThreatEvidence::PowerShellIndicator { indicator });
        }
        evidence
    }

    /// Analyze every successfully-scanned file in a report.
    pub fn analyze_report(
        &self,
        report: &aegis_scan::ScanReport,
        signatures: &SignatureDatabase,
        yara: Option<&RuleManager>,
    ) -> Vec<ThreatDetection> {
        report
            .files
            .iter()
            .filter_map(|f| self.analyze(f, signatures, yara))
            .collect()
    }
}

/// Read up to `limit` bytes from the head of a file.
fn read_head(path: &Path, limit: usize) -> std::io::Result<Vec<u8>> {
    let file = File::open(path)?;
    let mut buf = Vec::with_capacity(limit.min(64 * 1024));
    file.take(limit as u64).read_to_end(&mut buf)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aegis_scan::{FileHashes, FileMetadata, ScannedFile};
    use std::path::PathBuf;

    fn scanned(path: &str, sha: &str, md5: &str) -> ScannedFile {
        ScannedFile {
            metadata: FileMetadata {
                path: PathBuf::from(path),
                size_bytes: 10,
                modified_utc: None,
                is_hidden: false,
                is_symlink: false,
            },
            hashes: Some(FileHashes { sha256: sha.into(), md5: md5.into() }),
            error: None,
        }
    }

    #[test]
    fn hash_match_produces_detection() {
        let mut db = SignatureDatabase::new();
        let sha = "a".repeat(64);
        db.insert(aegis_signatures::HashAlgo::Sha256, &sha);
        let engine = DetectionEngine::new();
        let f = scanned("C:/tmp/evil.bin", &sha, &"0".repeat(32));
        let d = engine.analyze(&f, &db, None).unwrap();
        assert_eq!(d.score, 100);
        assert!(matches!(d.evidence[0], ThreatEvidence::HashMatch { .. }));
    }

    #[test]
    fn clean_file_no_detection() {
        let db = SignatureDatabase::new();
        let engine = DetectionEngine::new();
        let f = scanned("C:/tmp/readme.txt", &"f".repeat(64), &"e".repeat(32));
        assert!(engine.analyze(&f, &db, None).is_none());
    }

    #[test]
    fn powershell_content_scores() {
        let engine = DetectionEngine::new();
        let ev = engine.content_evidence(
            &PathBuf::from("update.ps1"),
            b"powershell -EncodedCommand X; IEX (DownloadString)",
        );
        assert!(ev.iter().any(|e| matches!(e, ThreatEvidence::PowerShellIndicator { .. })));
        assert!(ev.iter().any(|e| matches!(e, ThreatEvidence::ScriptIndicator { .. })));
    }

    #[test]
    fn packed_executable_flagged() {
        let engine = DetectionEngine::new();
        let mut content = b"MZ".to_vec();
        content.extend((0..=255u8).cycle().take(8192));
        let ev = engine.content_evidence(&PathBuf::from("a.exe"), &content);
        assert!(ev.iter().any(|e| matches!(e, ThreatEvidence::PackedExecutable { .. })));
    }
}
