use crate::coeiroink_scraping::Edition;
use anyhow::Result;
use futures_util::StreamExt;
use lazy_regex::regex;
use path_dedot::ParseDot as _;
use tauri::Manager;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio_util::compat::TokioAsyncWriteCompatExt;
use tracing::info;
use windows::core::Interface;

#[derive(Clone, serde::Serialize)]
#[serde(tag = "type")]
enum DownloadProgress {
    Initializing,
    Downloading {
        progress: u64,
        total: u64,
    },
    Extracting {
        progress: u64,
        total: u64,
    },
    Installing {
        progress: u64,
        total: u64,
        current: String,
    },
    Configuring,
    Done,
}

async fn download(app_handle: tauri::AppHandle, url: &str) -> Result<async_tempfile::TempFile> {
    info!("Downloading coeiroink bootstrap: {}", url);

    let mut zip_file = async_tempfile::TempFile::new().await?;
    info!("Downloading to: {:?}", zip_file.file_path());

    let download_response = reqwest::get(url).await?;
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

    zip_file.flush().await?;

    Ok(zip_file)
}

async fn extract_bootstrap(
    app_handle: tauri::AppHandle,
    zip: async_zip::tokio::read::fs::ZipFileReader,
) -> Result<tempfile::TempDir> {
    let total_entries = zip.file().entries().len() as u64;

    let extract_dir = tempfile::tempdir()?;

    app_handle.emit(
        "installing_coeiroink",
        DownloadProgress::Extracting {
            progress: 0,
            total: total_entries,
        },
    )?;

    for i in 0..zip.file().entries().len() {
        let entry = zip.reader_with_entry(i).await?;
        let path = entry.entry().filename().as_str().unwrap().to_owned();
        if path.contains("..") {
            return Err(anyhow::anyhow!("Invalid path: {}", path));
        }
        let path = extract_dir.path().join(path);
        info!("Extracting: {:?}", path);
        app_handle.emit(
            "installing_coeiroink",
            DownloadProgress::Extracting {
                progress: (i + 1) as u64,
                total: total_entries,
            },
        )?;
        if entry.entry().dir()? {
            fs_err::tokio::create_dir_all(&path).await?;
        } else {
            fs_err::tokio::create_dir_all(path.parent().unwrap()).await?;
            let file = fs_err::tokio::File::create(&path).await?;
            futures::io::copy(entry, &mut file.compat_write()).await?;
        }
    }

    info!("Extracted coeiroink bootstrap");

    Ok(extract_dir)
}

async fn find_first_7z(extract_dir: &tempfile::TempDir) -> Result<std::path::PathBuf> {
    let mut files = async_walkdir::WalkDir::new(extract_dir.path());
    let mut first_7z = None;
    while let Some(file) = files.next().await {
        let file = file?;
        if file.file_name().to_string_lossy().ends_with(".001") {
            first_7z = Some(file.path());
            break;
        }
    }
    let Some(first_7z) = first_7z else {
        return Err(anyhow::anyhow!("Could not find 7z file"));
    };
    info!("Found 7z file: {:?}", first_7z);

    Ok(first_7z)
}

async fn list_files(
    first_7z: &std::path::Path,
    exclude_speaker_infos: bool,
) -> Result<Vec<String>> {
    info!("Listing files in 7z");
    let files = tokio::process::Command::new(assets::sevenzip_path())
        .arg("l")
        .arg(&first_7z)
        .output()
        .await?;

    if !files.status.success() {
        return Err(anyhow::anyhow!("Failed to list files in 7z"));
    }

    let stdout = String::from_utf8_lossy(&files.stdout);

    let lines = stdout.lines();

    let files = lines
        .skip_while(|line| !line.starts_with("----------")) // header
        .skip(1) // header separator
        .take_while(|line| !line.starts_with("----------")) // footer
        .map(|line| {
            let spaces = regex!(r"\s+");
            let parts = spaces.splitn(line, 6).collect::<Vec<_>>();
            parts[parts.len() - 1].to_owned()
        })
        .filter(|file| {
            if file.contains("__pycache__") {
                return false;
            }
            if exclude_speaker_infos && file.contains("speaker_info") {
                return false;
            }

            true
        })
        .collect::<Vec<_>>();
    info!("Found {} files in 7z", files.len());

    Ok(files)
}

async fn extract_7z(
    app_handle: tauri::AppHandle,
    first_7z: &std::path::Path,
    temporary_extract_dir: &std::path::Path,
    files: &[String],
    exclude_speaker_infos: bool,
) -> Result<()> {
    info!("Extracting 7z");
    app_handle.emit(
        "installing_coeiroink",
        DownloadProgress::Installing {
            progress: 0,
            total: files.len() as u64,
            current: files[0].clone(),
        },
    )?;

    let mut extract_process = tokio::process::Command::new(assets::sevenzip_path())
        .arg("x")
        .arg(format!("-o{}", temporary_extract_dir.to_string_lossy()))
        .arg("-y")
        .arg("-bb3")
        .arg("-xr!__pycache__")
        .args(if exclude_speaker_infos {
            vec!["-xr!speaker_info"]
        } else {
            vec![]
        })
        .arg(&first_7z)
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    let mut extract_stdout = tokio::io::BufReader::new(extract_process.stdout.as_mut().unwrap());

    let mut line = String::new();
    let mut extracted_files = 0;
    while extract_stdout.read_line(&mut line).await? > 0 {
        if line.starts_with("- ") {
            extracted_files += 1;
            app_handle.emit(
                "installing_coeiroink",
                DownloadProgress::Installing {
                    progress: extracted_files,
                    total: files.len() as u64,
                    current: line[2..].trim().to_owned(),
                },
            )?;
        }
        line.clear();
    }

    if !extract_process.wait().await?.success() {
        return Err(anyhow::anyhow!("Failed to extract 7z"));
    }

    Ok(())
}

async fn move_coeiroink(
    temporary_extract_dir: &std::path::Path,
    install_dir: &std::path::Path,
    exclude_speaker_infos: bool,
) -> Result<()> {
    let actual_extracted_dir = fs_err::tokio::read_dir(&temporary_extract_dir)
        .await?
        .next_entry()
        .await?;
    let Some(actual_extracted_dir) = actual_extracted_dir else {
        return Err(anyhow::anyhow!("Could not find actual extracted dir"));
    };
    let actual_extracted_dir = actual_extracted_dir.path();

    info!(
        "Moving coeiroink: {:?} -> {:?}",
        &actual_extracted_dir, install_dir
    );

    if install_dir.exists() {
        info!("Removing existing install dir, except speaker_info");
        let mut files = fs_err::tokio::read_dir(&install_dir).await?;
        while let Some(entry) = files.next_entry().await? {
            if entry.file_name() == "speaker_info" {
                continue;
            }
            info!("Removing {:?}", entry.path());
            if entry.file_type().await?.is_dir() {
                fs_err::tokio::remove_dir_all(entry.path()).await?;
            } else {
                fs_err::tokio::remove_file(entry.path()).await?;
            }
        }
    }
    fs_err::tokio::create_dir_all(&install_dir).await?;

    let mut file_paths: Vec<std::path::PathBuf> = vec![];
    let mut files = fs_err::tokio::read_dir(&actual_extracted_dir).await?;
    while let Some(entry) = files.next_entry().await? {
        if exclude_speaker_infos
            && entry.file_type().await?.is_dir()
            && entry.file_name() == "speaker_info"
        {
            continue;
        }
        file_paths.push(entry.path());
    }
    info!("Moving {} files", file_paths.len());

    let install_dir = install_dir.to_owned();
    for file_path in file_paths {
        let file_name = file_path.file_name().unwrap();
        let install_path = install_dir.join(file_name);
        if install_path.exists() {
            if install_path.is_dir() {
                fs_err::tokio::remove_dir_all(&install_path).await?;
            } else {
                fs_err::tokio::remove_file(&install_path).await?;
            }
        }

        info!("Moving {:?} -> {:?}", &file_path, &install_path);

        fs_err::tokio::rename(&file_path, &install_path).await?;
    }

    Ok(())
}

struct Com();

impl Com {
    fn new() -> Result<Self> {
        unsafe {
            windows::Win32::System::Com::CoInitializeEx(
                None,
                windows::Win32::System::Com::COINIT_MULTITHREADED,
            )
            .ok()?;
        }
        Ok(Self())
    }
}

impl Drop for Com {
    fn drop(&mut self) {
        unsafe {
            windows::Win32::System::Com::CoUninitialize();
        }
    }
}

async fn create_shortcut(install_dir: &std::path::Path, path: &std::path::Path) -> Result<()> {
    info!("Setting up shortcuts");
    let _com = Com::new()?;

    fs_err::tokio::create_dir_all(&path.parent().unwrap()).await?;

    let coeiroink_exe = install_dir.join("COEIROINKv2.exe");

    unsafe {
        let shell_link: windows::Win32::UI::Shell::IShellLinkW =
            windows::Win32::System::Com::CoCreateInstance(
                &windows::Win32::UI::Shell::ShellLink,
                None,
                windows::Win32::System::Com::CLSCTX_LOCAL_SERVER,
            )?;

        let exe_path = windows::core::HSTRING::from(coeiroink_exe.as_os_str());

        shell_link.SetIconLocation(&exe_path, 0)?;
        shell_link.SetPath(&exe_path)?;
        shell_link.SetDescription(&windows::core::HSTRING::from("Coeiroink v2"))?;

        let exe_parent = coeiroink_exe.parent().unwrap();
        let exe_parent = windows::core::HSTRING::from(exe_parent.as_os_str());

        shell_link.SetWorkingDirectory(&exe_parent)?;

        let lnk_path = windows::core::HSTRING::from(path.as_os_str());

        shell_link
            .cast::<windows::Win32::System::Com::IPersistFile>()?
            .Save(&lnk_path, true)?;
    }

    Ok(())
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallParams {
    pub edition: Edition,
    pub version: String,
    pub path: String,
    pub desktop_shortcut: bool,
    pub start_menu_shortcut: bool,
}

pub async fn install_coeiroink(app_handle: tauri::AppHandle, params: InstallParams) -> Result<()> {
    info!("Installing coeiroink");
    let edition = params.edition;
    let version = params.version;
    let path = params.path;

    let install_dir = std::path::PathBuf::from(path);
    let install_dir = install_dir.parse_dot()?;

    let temporary_extract_dir = if cfg!(windows) {
        let install_dir_drive = install_dir
            .to_string_lossy()
            .chars()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Could not get drive letter"))?;

        tempfile::Builder::new()
            .prefix("coeirobottle_temporary_directory_")
            .tempdir_in(format!("{}:\\", install_dir_drive))?
    } else {
        panic!("Unsupported platform");
    };

    info!("Install dir: {:?}", install_dir);
    info!("Temporary extract dir: {:?}", temporary_extract_dir.path());
    let exclude_speaker_infos = install_dir.join("speaker_info").exists();
    info!("Speaker infos exist?: {}", exclude_speaker_infos);

    info!("Fetching downloads");
    app_handle.emit("installing_coeiroink", DownloadProgress::Initializing)?;
    let downloads = crate::coeiroink_scraping::fetch_downloads().await?;

    let download_item = downloads
        .iter()
        .find(|d| d.edition == edition && d.version == version);

    let download_item = match download_item {
        Some(download) => download,
        None => {
            return Err(anyhow::anyhow!(
                "Could not find download for edition {}",
                edition
            ));
        }
    };

    let zip_file = download(app_handle.clone(), &download_item.link).await?;
    let zip = async_zip::tokio::read::fs::ZipFileReader::new(&zip_file.file_path()).await?;

    let bootstrap_dir = extract_bootstrap(app_handle.clone(), zip).await?;

    let first_7z = find_first_7z(&bootstrap_dir).await?;

    let files = list_files(&first_7z, exclude_speaker_infos).await?;

    extract_7z(
        app_handle.clone(),
        &first_7z,
        temporary_extract_dir.path(),
        &files,
        exclude_speaker_infos,
    )
    .await?;

    app_handle.emit("installing_coeiroink", DownloadProgress::Configuring)?;

    move_coeiroink(
        temporary_extract_dir.path(),
        &install_dir,
        exclude_speaker_infos,
    )
    .await?;
    if params.desktop_shortcut {
        let desktop = std::path::PathBuf::from(std::env::var("USERPROFILE").unwrap())
            .join("Desktop")
            .join("Coeiroink v2.lnk");

        create_shortcut(&install_dir, &desktop).await?;
    }
    if params.start_menu_shortcut {
        let start_menu = std::path::PathBuf::from(std::env::var("APPDATA").unwrap())
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("Coeiroink v2.lnk");

        create_shortcut(&install_dir, &start_menu).await?;
    }

    let mut store = tauri_plugin_store::StoreBuilder::new("store.json").build(app_handle.clone());

    // is latest version
    if downloads[0].version == version {
        store.insert(
            "coeiroink_root".into(),
            install_dir.to_string_lossy().to_string().into(),
        )?;

        store.save()?;
    }

    info!("Installed coeiroink");

    app_handle.emit("installing_coeiroink", DownloadProgress::Done)?;

    Ok(())
}
