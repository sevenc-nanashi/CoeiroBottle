// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod coeiroink_scraping;
mod commands;

use tracing::info;

#[tauri::command]
async fn fetch_latest_version() -> Result<commands::fetch_latest_version::CheckVersionResult, String>
{
    commands::fetch_latest_version::fetch_latest_version()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_coeiroink_version(app_handle: tauri::AppHandle) -> Result<Option<String>, String> {
    commands::get_coeiroink_version::get_coeiroink_version(app_handle)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn install_coeiroink(app_handle: tauri::AppHandle, edition: String) -> Result<(), String> {
    commands::install_coeiroink::install_coeiroink(app_handle, edition)
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
            fetch_latest_version,
            get_coeiroink_version,
            install_coeiroink
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
