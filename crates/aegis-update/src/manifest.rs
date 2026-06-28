//! Update manifest + component model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The four updatable components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateComponent {
    SignatureDatabase,
    YaraRules,
    ThreatMetadata,
    EngineConfig,
}

impl UpdateComponent {
    pub fn as_str(&self) -> &'static str {
        match self {
            UpdateComponent::SignatureDatabase => "signature_database",
            UpdateComponent::YaraRules => "yara_rules",
            UpdateComponent::ThreatMetadata => "threat_metadata",
            UpdateComponent::EngineConfig => "engine_config",
        }
    }
}

/// A signed description of one downloadable update.
///
/// `signature` is base64 Ed25519 over [`UpdateManifest::signed_message`], which
/// binds the component, version, file digest, and size together — so none can be
/// tampered without invalidating the signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateManifest {
    pub version: String,
    pub published_at: DateTime<Utc>,
    pub sha256: String,
    pub signature: String,
    pub url: String,
    pub size: u64,
    pub component: UpdateComponent,
    pub minimum_app_version: String,
}

impl UpdateManifest {
    /// Canonical message the signature is computed over.
    pub fn signed_message(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.component.as_str(),
            self.version,
            self.sha256.to_ascii_lowercase(),
            self.size
        )
    }
}
