use anyhow::Result;

static ALLOWED_FILES: &[&str] = &["COEIROINKv2.exe", "engine", "speaker_info"];

pub async fn is_safe_to_install(path: String) -> Result<bool> {
    let path = std::path::Path::new(&path);
    if !path.exists() {
        return Ok(true);
    }
    let mut files = fs_err::tokio::read_dir(path).await?;
    while let Some(entry) = files.next_entry().await? {
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if !ALLOWED_FILES.contains(&file_name.as_ref()) {
            return Ok(false);
        }
    }
    Ok(true)
}

static REQUIRED_FILES: &[&str] = &["COEIROINKv2.exe", "engine", "speaker_info"];

pub async fn is_coeiroink_dir(path: String) -> Result<bool> {
    let path = std::path::Path::new(&path);
    if !path.exists() {
        return Ok(false);
    }
    let mut files = fs_err::tokio::read_dir(path).await?;
    let mut found_files = vec![];
    while let Some(entry) = files.next_entry().await? {
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        found_files.push(file_name.to_string());
    }

    Ok(REQUIRED_FILES
        .iter()
        .all(|f| found_files.contains(&f.to_string())))
}
