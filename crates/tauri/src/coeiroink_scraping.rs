use anyhow::Result;
use lazy_regex::regex;
use tracing::warn;

#[derive(Debug, PartialEq, Eq, Clone, strum::EnumString, strum::Display)]
#[strum(ascii_case_insensitive)]
pub enum Edition {
    Cpu,
    Gpu,
}

#[derive(Debug, PartialEq, Eq, Clone, strum::EnumString, strum::Display)]
#[strum(ascii_case_insensitive)]
pub enum Os {
    Windows,
    Mac,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DownloadInfo {
    pub edition: Edition,
    pub os: Os,
    pub version: String,
    pub link: String,
}

#[cached::proc_macro::once(result)]
pub async fn fetch_downloads() -> Result<Vec<DownloadInfo>> {
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

    let latest_name_pattern =
        regex!(r#"COEIROINK-(?P<edition>CPU|GPU)-v\.(?P<version>[0-9.+]+)\((?P<os>Windows|Mac)\)"#);

    let mut downloads = Vec::new();

    for link in links {
        let text = link
            .text()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Could not find text"))?;

        let Some(captures) = latest_name_pattern.captures(text) else {
            warn!("Could not find captures for {}", text);
            continue;
        };

        let edition = captures["edition"].parse()?;

        let version = captures["version"].to_string();

        let os = captures["os"].parse()?;

        let link = link
            .value()
            .attr("href")
            .ok_or_else(|| anyhow::anyhow!("Could not find link"))?;

        downloads.push(DownloadInfo {
            edition,
            os,
            version,
            link: link.to_string(),
        });
    }

    Ok(downloads)
}
