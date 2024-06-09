use anyhow::Result;
use futures_util::StreamExt;
use tauri::Manager;
use tokio::io::AsyncWriteExt;
use tokio_util::compat::TokioAsyncWriteCompatExt;
use tracing::info;

#[derive(Clone, serde::Serialize)]
#[serde(tag = "type")]
enum DownloadProgress {
    Initializing,
    Downloading { progress: u64, total: u64 },
    Extracting { progress: u64, total: u64 },
    Installing,
    Configuring,
    Done,
}

pub async fn install_coeiroink(app_handle: tauri::AppHandle, edition: String) -> Result<()> {
    let edition: crate::coeiroink_scraping::Edition = edition
        .parse()
        .map_err(|e| anyhow::anyhow!("Failed to parse edition: {:?}", e))?;

    info!("Fetching downloads");
    app_handle.emit("installing_coeiroink", DownloadProgress::Initializing)?;
    let downloads = crate::coeiroink_scraping::fetch_downloads().await?;

    let download = downloads.iter().find(|d| d.edition == edition);

    let download = match download {
        Some(download) => download,
        None => {
            return Err(anyhow::anyhow!(
                "Could not find download for edition {}",
                edition
            ));
        }
    };

    info!("Downloading coeiroink bootstrap: {}", download.link);

    let mut zip_file = async_tempfile::TempFile::new().await?;
    info!("Downloading to: {:?}", zip_file.file_path());

    let download_response = reqwest::get(&download.link).await?;
    let download_size = download_response.content_length().unwrap_or(0);
    let mut download_progress = 0;
    let mut last_progress = 0;
    let mut stream = download_response.bytes_stream();

    while let Some(item) = stream.next().await {
        let item = item?;
        zip_file.write_all(&item).await?;
        download_progress += item.len() as u64;
        if download_progress - last_progress > 8 * 1024 * 1024 {
            info!(
                "Downloading coeiroink: {} / {}",
                download_progress, download_size
            );
            last_progress = download_progress;
            app_handle.emit(
                "installing_coeiroink",
                DownloadProgress::Downloading {
                    progress: download_progress,
                    total: download_size,
                },
            )?;
        }
    }
    info!("Downloaded coeiroink");

    app_handle.emit("installing_coeiroink", DownloadProgress::Extracting)?;
    let extract_dir = tempfile::tempdir()?;

    let zip = async_zip::tokio::read::fs::ZipFileReader::new(&zip_file.file_path()).await?;
    let total_entries = zip.file().entries().len() as u64;

    for i in 0..zip.file().entries().len() {
        let entry = zip.reader_with_entry(i).await?;
        let path = entry.entry().filename().as_str().unwrap().to_owned();
        if path.contains("..") {
            return Err(anyhow::anyhow!("Invalid path: {}", path));
        }
        let path = extract_dir.path().join(path);
        info!("Extracting: {:?}", path);
        if entry.entry().dir()? {
            tokio::fs::create_dir_all(&path).await?;
        } else {
            tokio::fs::create_dir_all(path.parent().unwrap()).await?;
            let file = tokio::fs::File::create(&path).await?;
            futures::io::copy(entry, &mut file.compat_write()).await?;
        }
        app_handle.emit(
            "installing_coeiroink",
            DownloadProgress::Extracting {
                progress: i as u64,
                total: total_entries,
            },
        )?;
    }

    info!("Extracted coeiroink bootstrap");
    app_handle.emit("installing_coeiroink", DownloadProgress::Installing)?;
    let install_dir = std::path::PathBuf::from(std::env::var("LOCALAPPDATA").unwrap())
        .join("Programs")
        .join("coeiroink");

    info!("Installing to: {:?}", install_dir);

    Ok(())
}
