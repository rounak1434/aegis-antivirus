//! Threat model: evidence, scoring, levels, and the detection record.
//!
//! Scores are never black-box — every point comes from a typed [`ThreatEvidence`]
//! item that carries its own data and an explainable [`ThreatEvidence::reason`].

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub use aegis_common::ThreatLevel;

/// One piece of explainable evidence contributing to a detection. Each variant
/// preserves the concrete data that triggered it (rule name, matched hash,
/// entropy score, extension, matched string).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ThreatEvidence {
    /// File digest matched a known-bad signature.
    HashMatch {
        algo: String,
        hex: String,
        threat_name: Option<String>,
    },
    /// A compiled YARA rule matched.
    YaraMatch {
        rule: String,
        namespace: String,
        patterns: Vec<String>,
    },
    /// Shannon entropy exceeded the high-entropy threshold.
    EntropyDetection { entropy: f64 },
    /// High-entropy executable — likely packed/encrypted payload.
    PackedExecutable { entropy: f64 },
    /// Decoy double extension, e.g. `invoice.pdf.exe`.
    DoubleExtension {
        file_name: String,
        decoy_ext: String,
        real_ext: String,
    },
    /// File carries a dangerous extension.
    SuspiciousExtension { ext: String },
    /// Script-host indicator string found in content.
    ScriptIndicator { indicator: String },
    /// PowerShell abuse indicator found in content.
    PowerShellIndicator { indicator: String },
    /// A Windows persistence mechanism (startup, registry Run key, scheduled
    /// task, service, driver, browser extension, hosts file). Added in Phase 5
    /// so the Windows scanner emits uniform `ThreatDetection`s.
    PersistenceMechanism {
        mechanism: String,
        name: String,
        detail: String,
    },
    /// A file/command located somewhere suspicious (temp dir, startup folder,
    /// unsigned binary, suspicious command line).
    SuspiciousLocation { path: String, reason: String },
}

impl ThreatEvidence {
    /// Score contribution (points) for this evidence.
    pub fn weight(&self) -> u32 {
        match self {
            ThreatEvidence::HashMatch { .. } => 100,
            ThreatEvidence::YaraMatch { .. } => 80,
            ThreatEvidence::PowerShellIndicator { .. } => 25,
            ThreatEvidence::DoubleExtension { .. } => 20,
            ThreatEvidence::PackedExecutable { .. } => 20,
            ThreatEvidence::SuspiciousExtension { .. } => 15,
            ThreatEvidence::EntropyDetection { .. } => 15,
            ThreatEvidence::ScriptIndicator { .. } => 10,
            ThreatEvidence::SuspiciousLocation { .. } => 20,
            ThreatEvidence::PersistenceMechanism { .. } => 15,
        }
    }

    /// Human-readable explanation of why this evidence fired.
    pub fn reason(&self) -> String {
        match self {
            ThreatEvidence::HashMatch {
                algo,
                hex,
                threat_name,
            } => match threat_name {
                Some(n) => format!("{algo} digest {hex} matches known threat {n}"),
                None => format!("{algo} digest {hex} matches a known-bad signature"),
            },
            ThreatEvidence::YaraMatch {
                rule,
                namespace,
                patterns,
            } => format!(
                "YARA rule {namespace}:{rule} matched (patterns: {})",
                patterns.join(", ")
            ),
            ThreatEvidence::EntropyDetection { entropy } => {
                format!("high file entropy {entropy:.2}/8.0 (possible obfuscation)")
            }
            ThreatEvidence::PackedExecutable { entropy } => {
                format!("executable with high entropy {entropy:.2}/8.0 (likely packed)")
            }
            ThreatEvidence::DoubleExtension {
                file_name,
                decoy_ext,
                real_ext,
            } => format!("double extension: {file_name} poses as .{decoy_ext} but is .{real_ext}"),
            ThreatEvidence::SuspiciousExtension { ext } => {
                format!("dangerous file extension .{ext}")
            }
            ThreatEvidence::ScriptIndicator { indicator } => {
                format!("script-host indicator: {indicator}")
            }
            ThreatEvidence::PowerShellIndicator { indicator } => {
                format!("PowerShell abuse indicator: {indicator}")
            }
            ThreatEvidence::PersistenceMechanism {
                mechanism,
                name,
                detail,
            } => {
                format!("{mechanism} persistence '{name}': {detail}")
            }
            ThreatEvidence::SuspiciousLocation { path, reason } => {
                format!("suspicious location: {path} ({reason})")
            }
        }
    }

    /// Short stable tag for the evidence type.
    pub fn label(&self) -> &'static str {
        match self {
            ThreatEvidence::HashMatch { .. } => "hash_match",
            ThreatEvidence::YaraMatch { .. } => "yara_match",
            ThreatEvidence::EntropyDetection { .. } => "entropy",
            ThreatEvidence::PackedExecutable { .. } => "packed_executable",
            ThreatEvidence::DoubleExtension { .. } => "double_extension",
            ThreatEvidence::SuspiciousExtension { .. } => "suspicious_extension",
            ThreatEvidence::ScriptIndicator { .. } => "script_indicator",
            ThreatEvidence::PowerShellIndicator { .. } => "powershell_indicator",
            ThreatEvidence::PersistenceMechanism { .. } => "persistence_mechanism",
            ThreatEvidence::SuspiciousLocation { .. } => "suspicious_location",
        }
    }
}

/// Clamp a raw score sum into the 0–100 range.
pub fn clamp_score(raw: u32) -> u8 {
    raw.min(100) as u8
}

/// Map a 0–100 score to a [`ThreatLevel`] per the Phase 3 thresholds.
pub fn level_for_score(score: u8) -> ThreatLevel {
    match score {
        0..=9 => ThreatLevel::Safe,
        10..=29 => ThreatLevel::Low,
        30..=59 => ThreatLevel::Medium,
        60..=84 => ThreatLevel::High,
        _ => ThreatLevel::Critical,
    }
}

/// A complete, explainable detection for one file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreatDetection {
    pub id: String,
    pub path: String,
    pub threat_level: ThreatLevel,
    pub score: u8,
    pub evidence: Vec<ThreatEvidence>,
    pub timestamp: DateTime<Utc>,
}

impl ThreatDetection {
    /// Build a detection from collected evidence, computing score + level.
    /// Returns `None` when there is no evidence (nothing to report).
    pub fn from_evidence(
        path: impl Into<String>,
        evidence: Vec<ThreatEvidence>,
        timestamp: DateTime<Utc>,
    ) -> Option<Self> {
        if evidence.is_empty() {
            return None;
        }
        let raw: u32 = evidence.iter().map(ThreatEvidence::weight).sum();
        let score = clamp_score(raw);
        Some(Self {
            id: uuid::Uuid::new_v4().to_string(),
            path: path.into(),
            threat_level: level_for_score(score),
            score,
            evidence,
            timestamp,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weights_and_clamp() {
        assert_eq!(
            ThreatEvidence::HashMatch {
                algo: "sha256".into(),
                hex: "x".into(),
                threat_name: None
            }
            .weight(),
            100
        );
        assert_eq!(clamp_score(130), 100);
        assert_eq!(clamp_score(45), 45);
    }

    #[test]
    fn level_thresholds() {
        assert_eq!(level_for_score(0), ThreatLevel::Safe);
        assert_eq!(level_for_score(9), ThreatLevel::Safe);
        assert_eq!(level_for_score(10), ThreatLevel::Low);
        assert_eq!(level_for_score(29), ThreatLevel::Low);
        assert_eq!(level_for_score(30), ThreatLevel::Medium);
        assert_eq!(level_for_score(59), ThreatLevel::Medium);
        assert_eq!(level_for_score(60), ThreatLevel::High);
        assert_eq!(level_for_score(84), ThreatLevel::High);
        assert_eq!(level_for_score(85), ThreatLevel::Critical);
        assert_eq!(level_for_score(100), ThreatLevel::Critical);
    }

    #[test]
    fn empty_evidence_no_detection() {
        assert!(ThreatDetection::from_evidence("/x", vec![], Utc::now()).is_none());
    }

    #[test]
    fn hash_match_is_critical() {
        let ev = vec![ThreatEvidence::HashMatch {
            algo: "sha256".into(),
            hex: "ab".into(),
            threat_name: Some("Wacatac".into()),
        }];
        let d = ThreatDetection::from_evidence("/x", ev, Utc::now()).unwrap();
        assert_eq!(d.score, 100);
        assert_eq!(d.threat_level, ThreatLevel::Critical);
        assert!(d.evidence[0].reason().contains("Wacatac"));
    }
}
