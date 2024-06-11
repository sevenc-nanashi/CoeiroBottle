use anyhow::Result;
use lazy_regex::regex;
use tracing::warn;

#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    strum::EnumString,
    strum::Display,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
#[strum(ascii_case_insensitive)]
pub enum Edition {
    Cpu,
    Gpu,
}

#[derive(Debug, PartialEq, Eq, Clone, strum::EnumString, strum::Display, serde::Serialize)]
#[serde(rename_all = "lowercase")]
#[strum(ascii_case_insensitive)]
pub enum Os {
    Windows,
    Mac,
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize)]
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

    let old_version_selector = scraper::Selector::parse(
        r#"div.flex.justify-center.items-center.flex-wrap.lg\:px-5.relative.z-10.md\:pt-40.pt-28"#,
    )
    .unwrap();

    let old_version_header = parsed
        .select(&old_version_selector)
        .find(|e| {
            e.children()
                .map(|e| {
                    e.first_children()
                        .next()
                        .map(|e| e.value().as_text().map(|t| t.trim()))
                        .flatten()
                        .unwrap_or("")
                })
                .collect::<Vec<_>>()
                .join("")
                == "旧バージョン"
        })
        .ok_or_else(|| anyhow::anyhow!("Could not find old version header"))?;

    let old_version_table = old_version_header
        .next_sibling()
        .ok_or_else(|| anyhow::anyhow!("Could not find old version table"))?;

    let old_version_links = scraper::ElementRef::wrap(old_version_table)
        .ok_or_else(|| anyhow::anyhow!("Could not wrap old version table"))?
        .select(&scraper::Selector::parse("a").unwrap())
        .collect::<Vec<_>>();

    let old_version_name_pattern =
        regex!(r#"v\.(?P<version>[0-9.]+)-(?P<os>windows|mac)-(?P<edition>cpu|gpu)"#);

    for link in old_version_links {
        let text = link
            .text()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Could not find text"))?;

        let Some(captures) = old_version_name_pattern.captures(text) else {
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

    downloads.sort_by_key(|d| semver::Version::parse(&d.version).unwrap());
    downloads.reverse();

    let downloads = downloads
        .into_iter()
        .filter(|d| d.os == Os::Windows && d.version.starts_with("2."))
        .collect();

    Ok(downloads)
}
