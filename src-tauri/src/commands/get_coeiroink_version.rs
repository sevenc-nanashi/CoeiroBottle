use anyhow::{bail, Result};
use tracing::info;

#[cfg(windows)]
fn get_version(exe_path: std::path::PathBuf) -> Result<Option<String>> {
    unsafe {
        let exe_path = windows::core::HSTRING::from(exe_path.as_os_str());

        let size = windows::Win32::Storage::FileSystem::GetFileVersionInfoSizeW(&exe_path, None);
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
        Ok(Some(version))
    }
}

pub async fn get_coeiroink_version(app_handle: tauri::AppHandle) -> Result<Option<String>> {
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

    let version = get_version(coeiroink_v2_exe)?;

    info!("coeiroink version: {:?}", version);
    Ok(version)
}
