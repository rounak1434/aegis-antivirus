mod commands;
mod service_client;

use commands::{get_service_status, start_scan};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_service_status, start_scan])
        .run(tauri::generate_context!())
        .expect("failed to run Aegis desktop UI");
}

fn main() {
    run();
}
