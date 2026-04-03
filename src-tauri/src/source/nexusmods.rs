use chrono::DateTime;
use eyre::{eyre, Context, Result};
use serde::Deserialize;
use tracing::{debug, warn};

use super::registry::ModSource;
use super::types::*;

const BASE_URL: &str = "https://api.nexusmods.com/v1";

fn game_domain_from_slug(slug: &str) -> Option<&'static str> {
    match slug {
        "riskofrain2" => Some("riskofrain2"),
        "valheim" => Some("valheim"),
        "lethalcompany" => Some("lethalcompany"),
        "h3vr" => Some("hotdogs"),
        "content-warning" => Some("contentwarning"),
        "repo" => Some("repo"),
        "titanfall2" => Some("titanfall2"),
        "gtfo" => Some("gtfo"),
        _ => None,
    }
}

pub struct NexusModsSource {
    http: reqwest::Client,
    api_key: String,
    game_domain: Option<String>,
}

impl NexusModsSource {
    pub fn new(api_key: String, http: reqwest::Client) -> Self {
        Self {
            http,
            api_key,
            game_domain: None,
        }
    }

    async fn request<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", BASE_URL, path);
        debug!("NexusMods request: {}", url);

        let response = self
            .http
            .get(&url)
            .header("apikey", &self.api_key)
            .header("Accept", "application/json")
            .send()
            .await
            .context("NexusMods request failed")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(eyre!("NexusMods API error {}: {}", status, body));
        }

        response
            .json::<T>()
            .await
            .context("failed to parse NexusMods response")
    }

    fn current_domain(&self) -> Result<&str> {
        self.game_domain
            .as_deref()
            .ok_or_else(|| eyre!("no game domain set for NexusMods"))
    }
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct NxMod {
    mod_id: u32,
    name: Option<String>,
    summary: Option<String>,
    description: Option<String>,
    version: Option<String>,
    author: Option<String>,
    uploaded_by: Option<String>,
    #[serde(rename = "category_id")]
    category_id: Option<u32>,
    endorsement_count: Option<u32>,
    #[serde(rename = "mod_downloads")]
    downloads: Option<u64>,
    #[serde(rename = "mod_unique_downloads")]
    unique_downloads: Option<u64>,
    picture_url: Option<String>,
    created_timestamp: Option<i64>,
    updated_timestamp: Option<i64>,
    available: Option<bool>,
}

#[derive(Deserialize)]
struct NxModFile {
    #[serde(rename = "file_id")]
    id: u64,
    name: String,
    version: Option<String>,
    size_in_bytes: Option<u64>,
    uploaded_timestamp: Option<i64>,
}

#[derive(Deserialize)]
struct NxModFiles {
    files: Vec<NxModFile>,
}

impl NxMod {
    fn to_unified(&self, domain: &str) -> UnifiedMod {
        let author = self
            .author
            .clone()
            .or_else(|| self.uploaded_by.clone())
            .unwrap_or_default();

        let version = self.version.clone().unwrap_or_else(|| "1.0.0".to_string());

        UnifiedMod {
            source_id: SourceId::NexusMods,
            external_id: format!("{}:{}", domain, self.mod_id),
            name: self.name.clone().unwrap_or_default(),
            author,
            description: self.summary.clone().or_else(|| self.description.clone()),
            version,
            versions: Vec::new(),
            categories: Vec::new(),
            downloads: self.downloads,
            rating: self.endorsement_count,
            icon_url: self.picture_url.clone(),
            website_url: Some(format!(
                "https://www.nexusmods.com/{}/mods/{}",
                domain, self.mod_id
            )),
            date_updated: self
                .updated_timestamp
                .and_then(|t| DateTime::from_timestamp(t, 0)),
            date_created: self
                .created_timestamp
                .and_then(|t| DateTime::from_timestamp(t, 0)),
            file_size: 0,
            is_deprecated: self.available == Some(false),
            is_nsfw: false,
            dependencies: Vec::new(),
        }
    }
}

#[async_trait::async_trait]
impl ModSource for NexusModsSource {
    fn id(&self) -> SourceId {
        SourceId::NexusMods
    }

    fn display_name(&self) -> &str {
        "NexusMods"
    }

    fn requires_auth(&self) -> bool {
        true
    }

    fn is_authenticated(&self) -> bool {
        !self.api_key.is_empty()
    }

    fn info(&self) -> SourceInfo {
        SourceInfo {
            id: self.id(),
            display_name: self.display_name().to_string(),
            is_enabled: true,
            requires_auth: true,
            is_authenticated: self.is_authenticated(),
            supported_games: None,
        }
    }

    async fn search(&self, filters: &SearchFilters) -> Result<SearchResult> {
        if self.api_key.is_empty() {
            return Ok(SearchResult {
                mods: Vec::new(),
                source: SourceId::NexusMods,
                total_count: Some(0),
            });
        }

        // Try mapped domain first, then fall back to using the slug directly
        let domain = filters
            .game_slug
            .as_deref()
            .and_then(|slug| game_domain_from_slug(slug).map(|d| d.to_string()).or_else(|| Some(slug.to_string())))
            .or_else(|| self.game_domain.clone());
        let domain = domain.as_deref();

        let domain = match domain {
            Some(d) => d,
            None => {
                return Ok(SearchResult {
                    mods: Vec::new(),
                    source: SourceId::NexusMods,
                    total_count: Some(0),
                })
            }
        };

        let path = match filters.sort_by {
            SortField::Updated => format!("/games/{}/mods/latest_updated.json", domain),
            _ => format!("/games/{}/mods/latest_added.json", domain),
        };

        let all_mods: Vec<NxMod> = match self.request(&path).await {
            Ok(mods) => mods,
            Err(err) => {
                warn!("NexusMods search failed: {:#}", err);
                return Ok(SearchResult {
                    mods: Vec::new(),
                    source: SourceId::NexusMods,
                    total_count: Some(0),
                });
            }
        };

        let mut results: Vec<UnifiedMod> = all_mods
            .into_iter()
            .filter(|m| {
                if m.name.is_none() || m.available == Some(false) {
                    return false;
                }
                if !filters.search_term.is_empty() {
                    let term = filters.search_term.to_lowercase();
                    let name_match = m.name.as_ref().map(|n| n.to_lowercase().contains(&term)).unwrap_or(false);
                    let desc_match = m
                        .summary
                        .as_ref()
                        .map(|s| s.to_lowercase().contains(&term))
                        .unwrap_or(false);
                    if !name_match && !desc_match {
                        return false;
                    }
                }
                true
            })
            .map(|m| m.to_unified(domain))
            .collect();

        match filters.sort_by {
            SortField::Downloads => {
                results.sort_by(|a, b| b.downloads.unwrap_or(0).cmp(&a.downloads.unwrap_or(0)))
            }
            SortField::Rating => {
                results.sort_by(|a, b| b.rating.unwrap_or(0).cmp(&a.rating.unwrap_or(0)))
            }
            SortField::Newest => results.sort_by(|a, b| b.date_created.cmp(&a.date_created)),
            SortField::Updated => results.sort_by(|a, b| b.date_updated.cmp(&a.date_updated)),
            SortField::Name => results.sort_by(|a, b| a.name.cmp(&b.name)),
        }

        if matches!(filters.sort_order, SortDirection::Ascending) {
            results.reverse();
        }

        let total = results.len() as u64;

        if filters.max_count > 0 {
            results.truncate(filters.max_count);
        }

        Ok(SearchResult {
            mods: results,
            source: SourceId::NexusMods,
            total_count: Some(total),
        })
    }

    async fn get_mod(&self, external_id: &str) -> Result<UnifiedMod> {
        let (domain, mod_id) = external_id
            .split_once(':')
            .ok_or_else(|| eyre!("invalid NexusMods external_id: {}", external_id))?;

        let nx_mod: NxMod = self
            .request(&format!("/games/{}/mods/{}.json", domain, mod_id))
            .await?;

        let mut unified = nx_mod.to_unified(domain);

        if let Ok(files) = self
            .request::<NxModFiles>(&format!("/games/{}/mods/{}/files.json", domain, mod_id))
            .await
        {
            unified.versions = files
                .files
                .iter()
                .map(|f| UnifiedModVersion {
                    version: f.version.clone().unwrap_or_else(|| f.name.clone()),
                    external_id: f.id.to_string(),
                    date_created: f
                        .uploaded_timestamp
                        .and_then(|t| DateTime::from_timestamp(t, 0)),
                    downloads: None,
                    file_size: f.size_in_bytes.unwrap_or(0),
                })
                .collect();
        }

        Ok(unified)
    }

    async fn get_readme(&self, _external_id: &str, _version: &str) -> Result<Option<String>> {
        Ok(None)
    }

    async fn get_changelog(&self, external_id: &str, _version: &str) -> Result<Option<String>> {
        let (domain, mod_id) = external_id
            .split_once(':')
            .ok_or_else(|| eyre!("invalid external_id"))?;

        #[derive(Deserialize)]
        struct Changelogs(std::collections::HashMap<String, Vec<String>>);

        let logs: Changelogs = self
            .request(&format!(
                "/games/{}/mods/{}/changelogs.json",
                domain, mod_id
            ))
            .await?;

        let mut entries: Vec<(String, Vec<String>)> = logs.0.into_iter().collect();
        entries.sort_by(|a, b| b.0.cmp(&a.0));

        let markdown = entries
            .iter()
            .map(|(ver, lines)| format!("## {}\n{}", ver, lines.join("\n")))
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(Some(markdown))
    }

    async fn get_categories(&self) -> Result<Vec<SourceCategory>> {
        Ok(Vec::new())
    }

    async fn get_trending(&self, _period: TrendingPeriod, max_count: usize) -> Result<Vec<UnifiedMod>> {
        if self.api_key.is_empty() {
            return Ok(Vec::new());
        }

        let domain = match self.current_domain() {
            Ok(d) => d,
            Err(_) => return Ok(Vec::new()),
        };

        let mods: Vec<NxMod> = self
            .request(&format!("/games/{}/mods/trending.json", domain))
            .await
            .unwrap_or_default();

        let mut results: Vec<UnifiedMod> = mods.iter().map(|m| m.to_unified(domain)).collect();
        results.truncate(max_count);

        Ok(results)
    }

    async fn download(&self, _external_id: &str, _version: &str) -> Result<DownloadResult> {
        Err(eyre!("NexusMods download requires premium account"))
    }
}
