use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateManifest {
    pub channel: String,
    pub bundle_version: String,
    pub manifest_hash: String,
    pub signature: String,
    pub published_at_utc: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("update manifest is missing {0}")]
    MissingField(&'static str),
}

impl UpdateManifest {
    pub fn validate_metadata(&self) -> Result<(), UpdateError> {
        if self.channel.trim().is_empty() {
            return Err(UpdateError::MissingField("channel"));
        }
        if self.bundle_version.trim().is_empty() {
            return Err(UpdateError::MissingField("bundle_version"));
        }
        if self.manifest_hash.trim().is_empty() {
            return Err(UpdateError::MissingField("manifest_hash"));
        }
        if self.signature.trim().is_empty() {
            return Err(UpdateError::MissingField("signature"));
        }
        Ok(())
    }
}
