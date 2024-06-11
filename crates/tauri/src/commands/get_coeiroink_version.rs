use crate::coeiroink_scraping::Edition;
use anyhow::{bail, Result};
use tracing::info;

static GET_VERSION_LOCK: once_cell::sync::Lazy<tokio::sync::Mutex<()>> =
    once_cell::sync::Lazy::new(|| tokio::sync::Mutex::new(()));

#[cfg(windows)]
async fn get_version(exe_path: &std::path::Path) -> Result<String> {
    unsafe {
        let _lock = GET_VERSION_LOCK.lock().await;

        let exe_path = windows::core::HSTRING::from(exe_path.as_os_str());

        let size = windows::Win32::Storage::FileSystem::GetFileVersionInfoSizeW(&exe_path, None);

        if size == 0 {
            return Err(windows_result::Error::from_win32().into());
        }

        let mut buffer = vec![0u8; size as usize];
        windows::Win32::Storage::FileSystem::GetFileVersionInfoW(
            &exe_path,
            0,
            size,
            buffer.as_mut_ptr() as *mut _,
        )?;

        let mut version_ptr = std::ptr::null_mut();
        let mut len = 0u32;
        if !windows::Win32::Storage::FileSystem::VerQueryValueW(
            buffer.as_ptr() as *const _,
            windows::core::w!("\\StringFileInfo\\040904E4\\FileVersion"),
            &mut version_ptr,
            &mut len,
        )
        .as_bool()
        {
            bail!("VerQueryValueW failed");
        }

        let version = std::slice::from_raw_parts(version_ptr as *const u16, len as usize);
        let version = String::from_utf16_lossy(version)
            .trim_end_matches('\0')
            .to_string();
        Ok(version)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct VersionInfo {
    pub version: String,
    pub edition: Edition,
}

pub async fn get_coeiroink_version(app_handle: tauri::AppHandle) -> Result<Option<VersionInfo>> {
    let mut store = tauri_plugin_store::StoreBuilder::new("store.json").build(app_handle.clone());
    let _ = store.load();
    info!("Getting coeiroink version");

    let coeiroink_root = store.get("coeiroink_root");

    let Some(coeiroink_root) = coeiroink_root.map(|v| v.as_str()).flatten() else {
        info!("No coeiroink_root found in store, returning None");
        return Ok(None);
    };

    let coeiroink_root = std::path::Path::new(coeiroink_root);

    let coeiroink_v2_exe = coeiroink_root.join("COEIROINKv2.exe");

    info!("Getting version of {:?}", coeiroink_v2_exe);

    let version = get_version(&coeiroink_v2_exe).await?;

    info!("coeiroink version: {:?}", &version);

    let edition = if tokio::fs::metadata(
        coeiroink_root
            .join("engine")
            .join("torch")
            .join("lib")
            .join("cudnn64_8.dll"),
    )
    .await
    .is_ok()
    {
        Edition::Gpu
    } else {
        Edition::Cpu
    };

    info!("coeiroink edition: {:?}", &edition);

    Ok(Some(VersionInfo { version, edition }))
}
