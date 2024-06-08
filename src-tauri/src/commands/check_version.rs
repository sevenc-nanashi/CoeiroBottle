use anyhow::Result;
use lazy_regex::regex;
use serde::Serialize;
use tracing::{error, info};

#[derive(Serialize)]
pub struct CheckVersionResult {
    coeiroink: Option<String>,
    coeirobottle: Option<String>,
}

pub async fn check_version() -> Result<CheckVersionResult> {
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
    let coeiroink_download = reqwest::get("https://coeiroink.com/download")
        .await?
        .text()
        .await?;

    let parsed = scraper::Html::parse_document(&coeiroink_download);
    let selector = scraper::Selector::parse("div.text-center.mt-20.text-xl.font-bold").unwrap();
    let download_header = parsed
        .select(&selector)
        .find(|e| {
            e.first_child()
                .map(|e| e.value().as_text().map(|t| t.trim()) == Some("DropBoxからダウンロード"))
                .unwrap_or(false)
        })
        .ok_or_else(|| anyhow::anyhow!("Could not find download header"))?;

    let table = download_header
        .next_sibling()
        .ok_or_else(|| anyhow::anyhow!("Could not find table"))?;

    let links = scraper::ElementRef::wrap(table)
        .ok_or_else(|| anyhow::anyhow!("Could not wrap table"))?
        .select(&scraper::Selector::parse("a").unwrap())
        .collect::<Vec<_>>();

    let name_pattern =
        regex!(r#"COEIROINK-(?P<edition>CPU|GPU)-v\.(?P<version>[0-9.+]+)\(Windows\)"#);

    let first_link = links
        .first()
        .ok_or_else(|| anyhow::anyhow!("Could not find link"))?
        .text()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Could not find text"))?;

    let captures = name_pattern
        .captures(first_link)
        .ok_or_else(|| anyhow::anyhow!("Could not find captures"))?;

    let version: &str = captures.name("version").unwrap().as_str();

    info!("Found coeiroink version: {}", version);

    Ok(version.to_string())
}

pub async fn fetch_latest_coeirobottle_version() -> Result<String> {
    // TODO: implement
    Ok("0.0.0".to_string())
}
