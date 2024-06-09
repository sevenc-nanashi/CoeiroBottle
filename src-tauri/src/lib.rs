// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use tracing::info;

#[tauri::command]
#[cached::proc_macro::once]
async fn check_latest_version() -> Result<commands::check_latest_version::CheckVersionResult, String>
{
    commands::check_latest_version::check_latest_version()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_coeiroink_version(app_handle: tauri::AppHandle) -> Result<Option<String>, String> {
    commands::get_coeiroink_version::get_coeiroink_version(app_handle)
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(cfg!(debug_assertions))
        .init();

    info!("Starting tauri application");
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            check_latest_version,
            get_coeiroink_version
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
