mod commands;

use aegis_service::{AegisOrchestrator, ServiceConfig};
use commands::AppState;

/// Resolve the per-user data directory for the orchestrator's DB + vault.
fn data_dir() -> std::path::PathBuf {
    let base = std::env::var("LOCALAPPDATA")
        .or_else(|_| std::env::var("HOME"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    base.join("Aegis")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let orch = AegisOrchestrator::new(&ServiceConfig::new(data_dir()))
        .expect("initialize AegisService orchestrator");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState { orch })
        .invoke_handler(tauri::generate_handler![
            commands::get_service_health,
            commands::start_scan,
            commands::stop_scan,
            commands::get_scan_status,
            commands::list_jobs,
            commands::list_threats,
            commands::list_quarantine,
            commands::quarantine_detection,
            commands::restore_file,
            commands::delete_quarantine_item,
            commands::run_windows_scan,
            commands::start_realtime,
            commands::stop_realtime,
            commands::get_realtime_status,
            commands::check_updates,
            commands::download_updates,
            commands::install_updates,
            commands::rollback_updates,
            commands::get_update_status,
            commands::load_settings,
            commands::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Aegis desktop UI");
}

fn main() {
    run();
}
