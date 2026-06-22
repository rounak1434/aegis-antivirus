use crate::service_client::ServiceClient;
use aegis_common::ProtectionStatus;
use aegis_ipc::{StartScanAccepted, StartScanCommand};

#[tauri::command]
pub async fn get_service_status() -> Result<ProtectionStatus, String> {
    ServiceClient::new().get_status().await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn start_scan(command: StartScanCommand) -> Result<StartScanAccepted, String> {
    ServiceClient::new().start_scan(command).await.map_err(|error| error.to_string())
}
