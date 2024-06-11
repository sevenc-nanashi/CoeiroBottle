// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod coeiroink_scraping;
mod commands;

use coeiroink_scraping::DownloadInfo;
use tracing::info;

#[tauri::command]
async fn fetch_latest_version() -> Result<commands::fetch_latest_version::CheckVersionResult, String>
{
    commands::fetch_latest_version::fetch_latest_version()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn fetch_coeiroink_versions() -> Result<Vec<DownloadInfo>, String> {
    crate::coeiroink_scraping::fetch_downloads()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_coeiroink_version(
    app_handle: tauri::AppHandle,
) -> Result<Option<commands::get_coeiroink_version::VersionInfo>, String> {
    commands::get_coeiroink_version::get_coeiroink_version(app_handle)
        .await
        .map_err(|e| e.to_string())
}

static ABORT_INSTALL: once_cell::sync::Lazy<tokio::sync::Mutex<Option<tokio::task::AbortHandle>>> =
    once_cell::sync::Lazy::new(|| tokio::sync::Mutex::new(None));

#[tauri::command]
async fn install_coeiroink(
    app_handle: tauri::AppHandle,
    params: commands::install_coeiroink::InstallParams,
) -> Result<(), String> {
    {
        if let Some(abort_handle) = ABORT_INSTALL.lock().await.as_ref() {
            if !abort_handle.is_finished() {
                return Err("Installation already in progress".to_string());
            }
        }
    }
    let task = tokio::task::spawn(commands::install_coeiroink::install_coeiroink(
        app_handle, params,
    ));
    {
        let mut guard = ABORT_INSTALL.lock().await;
        *guard = Some(task.abort_handle());
    }

    let result = task
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string());

    {
        let mut guard = ABORT_INSTALL.lock().await;
        *guard = None;
    }

    result
}

#[tauri::command]
async fn cancel_install_coeiroink() -> Result<(), String> {
    let mut guard = ABORT_INSTALL.lock().await;
    if let Some(abort_handle) = guard.take() {
        abort_handle.abort();
    }

    Ok(())
}

#[tauri::command]
async fn default_install_path_root() -> String {
    let install_dir =
        std::path::PathBuf::from(std::env::var("LOCALAPPDATA").unwrap()).join("Programs");

    return install_dir.to_str().unwrap().to_string();
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
            install_coeiroink,
            cancel_install_coeiroink,
            default_install_path_root,
            fetch_coeiroink_versions
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
