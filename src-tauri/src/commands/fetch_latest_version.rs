use anyhow::Result;
use lazy_regex::regex;
use serde::Serialize;
use tracing::{error, info};

#[derive(Clone, Serialize)]
pub struct CheckVersionResult {
    coeiroink: Option<String>,
    coeirobottle: Option<String>,
}

pub async fn fetch_latest_version() -> Result<CheckVersionResult> {
    let coeiroink = fetch_latest_coeiroink_version()
        .await
        .inspect_err(|e| {
            error!("Failed to fetch latest coeiroink version: {:?}", e);
        })
        .map_or(None, Some);
    let coeirobottle = fetch_latest_coeirobottle_version()
        .await
        .inspect_err(|e| {
            error!("Failed to fetch latest coeirobottle version: {:?}", e);
        })
        .map_or(None, Some);

    Ok(CheckVersionResult {
        coeiroink,
        coeirobottle,
    })
}

pub async fn fetch_latest_coeiroink_version() -> Result<String> {
    let downloads = crate::coeiroink_scraping::fetch_downloads().await?;

    info!("Found coeiroink version: {}", downloads[0].version);

    Ok(downloads[0].version.clone())
}

pub async fn fetch_latest_coeirobottle_version() -> Result<String> {
    // TODO: implement
    Ok("0.0.0".to_string())
}
