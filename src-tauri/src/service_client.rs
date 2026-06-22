use aegis_common::{AegisError, ProtectionStatus};
use aegis_ipc::{StartScanAccepted, StartScanCommand};

#[derive(Debug, Clone, Default)]
pub struct ServiceClient;

impl ServiceClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_status(&self) -> Result<ProtectionStatus, AegisError> {
        Ok(ProtectionStatus::service_unavailable())
    }

    pub async fn start_scan(&self, _command: StartScanCommand) -> Result<StartScanAccepted, AegisError> {
        Err(AegisError::ServiceUnavailable)
    }
}
