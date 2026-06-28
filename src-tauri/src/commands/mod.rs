//! Tauri command bridge. Every command delegates to the shared
//! `AegisOrchestrator` — the service layer — and never touches an engine crate
//! directly. Results are mapped to `Result<T, String>` for the JS side.

use aegis_common::ScanMode;
use aegis_detect::ThreatDetection;
use aegis_quarantine::QuarantineRecord;
use aegis_service::{AegisOrchestrator, JobState, ProtectionMode, RealtimeStatus, ServiceHealth};
use aegis_update::{InstallOutcome, UpdateComponent, UpdateManifest};
use serde::de::DeserializeOwned;

/// Shared application state: the one orchestrator the whole UI talks to.
pub struct AppState {
    pub orch: AegisOrchestrator,
}

type R<T> = Result<T, String>;

fn parse_enum<T: DeserializeOwned>(s: &str) -> R<T> {
    serde_json::from_value(serde_json::Value::String(s.to_string())).map_err(|e| e.to_string())
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

// ---- health -------------------------------------------------------------

#[tauri::command]
pub fn get_service_health(state: tauri::State<'_, AppState>) -> ServiceHealth {
    state.orch.get_service_health()
}

// ---- scanning -----------------------------------------------------------

#[tauri::command]
pub fn start_scan(state: tauri::State<'_, AppState>, mode: String, roots: Vec<String>) -> R<String> {
    let mode: ScanMode = parse_enum(&mode)?;
    let roots = roots.into_iter().map(std::path::PathBuf::from).collect();
    state.orch.start_scan(roots, mode).map_err(err)
}

#[tauri::command]
pub fn stop_scan(state: tauri::State<'_, AppState>, job_id: String) -> R<bool> {
    state.orch.stop_scan(&job_id).map_err(err)
}

#[tauri::command]
pub fn get_scan_status(state: tauri::State<'_, AppState>, job_id: String) -> Option<JobState> {
    state.orch.get_scan_status(&job_id)
}

#[tauri::command]
pub fn list_jobs(state: tauri::State<'_, AppState>) -> Vec<JobState> {
    state.orch.list_jobs()
}

// ---- threats ------------------------------------------------------------

#[tauri::command]
pub fn list_threats(state: tauri::State<'_, AppState>) -> R<Vec<ThreatDetection>> {
    state.orch.get_threats().map_err(err)
}

// ---- quarantine ---------------------------------------------------------

#[tauri::command]
pub fn list_quarantine(state: tauri::State<'_, AppState>) -> R<Vec<QuarantineRecord>> {
    state.orch.list_quarantine().map_err(err)
}

#[tauri::command]
pub fn quarantine_detection(state: tauri::State<'_, AppState>, detection: ThreatDetection) -> R<QuarantineRecord> {
    state.orch.quarantine_detection(&detection, "ui").map_err(err)
}

#[tauri::command]
pub fn restore_file(state: tauri::State<'_, AppState>, id: String, dest: Option<String>) -> R<String> {
    let dest = dest.map(std::path::PathBuf::from);
    state.orch.restore_file(&id, dest.as_deref(), "ui").map_err(err)
}

#[tauri::command]
pub fn delete_quarantine_item(state: tauri::State<'_, AppState>, id: String) -> R<()> {
    state.orch.delete_quarantine_item(&id, "ui").map_err(err)
}

// ---- windows persistence scan ------------------------------------------

#[tauri::command]
pub fn run_windows_scan(state: tauri::State<'_, AppState>) -> R<Vec<ThreatDetection>> {
    state.orch.run_windows_scan().map_err(err)
}

// ---- real-time protection ----------------------------------------------

#[tauri::command]
pub fn start_realtime(state: tauri::State<'_, AppState>, mode: String) -> R<()> {
    let mode: ProtectionMode = parse_enum(&mode)?;
    state.orch.start_realtime(mode).map_err(err)
}

#[tauri::command]
pub fn stop_realtime(state: tauri::State<'_, AppState>) -> R<()> {
    state.orch.stop_realtime().map_err(err)
}

#[tauri::command]
pub fn get_realtime_status(state: tauri::State<'_, AppState>) -> RealtimeStatus {
    state.orch.get_realtime_status()
}

// ---- secure updates -----------------------------------------------------

#[tauri::command]
pub fn check_updates(state: tauri::State<'_, AppState>, available: Vec<UpdateManifest>) -> R<Vec<UpdateManifest>> {
    state.orch.check_updates(&available).map_err(err)
}

#[tauri::command]
pub fn download_updates(state: tauri::State<'_, AppState>, manifest: UpdateManifest) -> R<()> {
    state.orch.download_updates(&manifest).map_err(err)
}

#[tauri::command]
pub fn install_updates(state: tauri::State<'_, AppState>, manifest: UpdateManifest) -> R<InstallOutcome> {
    state.orch.install_updates(&manifest).map_err(err)
}

#[tauri::command]
pub fn rollback_updates(state: tauri::State<'_, AppState>, component: String) -> R<InstallOutcome> {
    let component: UpdateComponent = parse_enum(&component)?;
    state.orch.rollback_updates(component).map_err(err)
}

#[tauri::command]
pub fn get_update_status(state: tauri::State<'_, AppState>) -> R<Vec<(String, String)>> {
    state.orch.get_update_status().map_err(err)
}

// ---- settings -----------------------------------------------------------

#[tauri::command]
pub fn load_settings(state: tauri::State<'_, AppState>) -> R<String> {
    state.orch.get_settings().map_err(err)
}

#[tauri::command]
pub fn save_settings(state: tauri::State<'_, AppState>, settings: String) -> R<()> {
    state.orch.save_settings(&settings).map_err(err)
}
