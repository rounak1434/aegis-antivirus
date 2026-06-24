//! Quarantine adapter over `aegis-quarantine`.

use std::path::Path;

use aegis_detect::ThreatDetection;
use aegis_quarantine::{QuarantineRecord, Vault};

use crate::ServiceError;

/// Thin service-owned wrappers over the encrypted quarantine vault.
pub struct QuarantineService;

impl QuarantineService {
    pub fn quarantine(
        vault: &Vault,
        detection: &ThreatDetection,
        actor: &str,
    ) -> Result<QuarantineRecord, ServiceError> {
        Ok(vault.quarantine_detection(detection, actor)?)
    }

    pub fn restore(
        vault: &Vault,
        id: &str,
        dest: Option<&Path>,
        actor: &str,
    ) -> Result<String, ServiceError> {
        let path = vault.restore_file(id, dest, actor)?;
        Ok(path.display().to_string())
    }

    pub fn delete(vault: &Vault, id: &str, actor: &str) -> Result<(), ServiceError> {
        Ok(vault.delete_file(id, actor)?)
    }

    pub fn list(vault: &Vault) -> Result<Vec<QuarantineRecord>, ServiceError> {
        Ok(vault.list_records()?)
    }
}
