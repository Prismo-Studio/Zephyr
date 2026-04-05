use eyre::{Result, bail};
use reqwest::Client;
use serde::Deserialize;
use tracing::info;

use super::registry::ModSource;
use super::types::*;

const REGISTRY_URL: &str =
    "https://raw.githubusercontent.com/Prismo-Studio/zephyr-mods/master/registry.json";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Registry {
    #[allow(dead_code)]
    version: u32,
    #[allow(dead_code)]
    last_updated: String,
    mods: Vec<CommunityMod>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommunityMod {
    name: String,
    slug: String,
    author: String,
    version: String,
    description: String,
    games: Vec<String>,
    #[serde(default)]
    dependencies: Vec<String>,
    #[allow(dead_code)]
    repository: String,
    download: String,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    nsfw: bool,
    #[serde(default)]
    deprecated: bool,
    icon: Option<String>,
    readme: Option<String>,
    changelog: Option<String>,
}

pub struct CommunitySource {
    http: Client,
}

impl CommunitySource {
    pub fn new(http: Client) -> Self {
        info!("Zephyr Community source initialized");
        Self { http }
    }

    async fn fetch_registry(&self) -> Result<Registry> {
        let resp = self
            .http
            .get(REGISTRY_URL)
            .header("Accept", "application/json")
            .send()
            .await?;

        if !resp.status().is_success() {
            bail!("Failed to fetch community registry: {}", resp.status());
        }

        let registry: Registry = resp.json().await?;
        Ok(registry)
    }

    fn mod_to_unified(&self, m: &CommunityMod) -> UnifiedMod {
        UnifiedMod {
            source_id: SourceId::GitHub,
            external_id: m.slug.clone(),
            name: m.name.clone(),
            author: m.author.clone(),
            description: Some(m.description.clone()),
            version: m.version.clone(),
            versions: vec![UnifiedModVersion {
                version: m.version.clone(),
                external_id: m.slug.clone(),
                date_created: None,
                downloads: None,
                file_size: 0,
            }],
            categories: m.categories.clone(),
            downloads: None,
            rating: None,
            icon_url: m.icon.clone(),
            website_url: Some(m.download.clone()),
            date_updated: None,
            date_created: None,
            file_size: 0,
            is_deprecated: m.deprecated,
            is_nsfw: m.nsfw,
            dependencies: m.dependencies.clone(),
        }
    }
}

#[async_trait::async_trait]
impl ModSource for CommunitySource {
    fn id(&self) -> SourceId {
        SourceId::GitHub
    }

    fn display_name(&self) -> &str {
        "Zephyr Community"
    }

    fn info(&self) -> SourceInfo {
        SourceInfo {
            id: self.id(),
            display_name: self.display_name().to_string(),
            is_enabled: true,
            requires_auth: false,
            is_authenticated: true,
            supported_games: None,
        }
    }

    async fn search(&self, filters: &SearchFilters) -> Result<SearchResult> {
        let registry = self.fetch_registry().await?;

        let game_slug = filters.game_slug.as_deref().unwrap_or("");

        let mut mods: Vec<UnifiedMod> = registry
            .mods
            .iter()
            .filter(|m| {
                // Filter by game
                if !game_slug.is_empty() {
                    let matches_game = m.games.iter().any(|g| {
                        g.eq_ignore_ascii_case(game_slug)
                    });
                    if !matches_game {
                        return false;
                    }
                }

                // Filter by search term
                if !filters.search_term.is_empty() {
                    let term = filters.search_term.to_lowercase();
                    let matches = m.name.to_lowercase().contains(&term)
                        || m.description.to_lowercase().contains(&term)
                        || m.author.to_lowercase().contains(&term);
                    if !matches {
                        return false;
                    }
                }

                // Filter by categories
                if !filters.categories.is_empty() {
                    let has_cat = filters.categories.iter().any(|c| {
                        m.categories.iter().any(|mc| mc.eq_ignore_ascii_case(c))
                    });
                    if !has_cat {
                        return false;
                    }
                }

                // NSFW filter
                if !filters.include_nsfw && m.nsfw {
                    return false;
                }

                // Deprecated filter
                if !filters.include_deprecated && m.deprecated {
                    return false;
                }

                true
            })
            .map(|m| self.mod_to_unified(m))
            .collect();

        let total = mods.len() as u64;

        // Apply offset and limit
        if filters.offset > 0 {
            mods = mods.into_iter().skip(filters.offset).collect();
        }
        if filters.max_count > 0 {
            mods.truncate(filters.max_count);
        }

        Ok(SearchResult {
            mods,
            source: SourceId::GitHub,
            total_count: Some(total),
        })
    }

    async fn get_mod(&self, external_id: &str) -> Result<UnifiedMod> {
        let registry = self.fetch_registry().await?;

        let m = registry
            .mods
            .iter()
            .find(|m| m.slug == external_id)
            .ok_or_else(|| eyre::eyre!("Mod not found: {}", external_id))?;

        Ok(self.mod_to_unified(m))
    }

    async fn get_readme(&self, external_id: &str, _version: &str) -> Result<Option<String>> {
        let registry = self.fetch_registry().await?;

        let m = match registry.mods.iter().find(|m| m.slug == external_id) {
            Some(m) => m,
            None => return Ok(None),
        };

        if let Some(readme_url) = &m.readme {
            let resp = self.http.get(readme_url).send().await?;
            if resp.status().is_success() {
                return Ok(Some(resp.text().await?));
            }
        }

        Ok(None)
    }

    async fn get_changelog(&self, external_id: &str, _version: &str) -> Result<Option<String>> {
        let registry = self.fetch_registry().await?;

        let m = match registry.mods.iter().find(|m| m.slug == external_id) {
            Some(m) => m,
            None => return Ok(None),
        };

        if let Some(changelog_url) = &m.changelog {
            let resp = self.http.get(changelog_url).send().await?;
            if resp.status().is_success() {
                return Ok(Some(resp.text().await?));
            }
        }

        Ok(None)
    }

    async fn get_categories(&self) -> Result<Vec<SourceCategory>> {
        Ok(vec![
            SourceCategory { name: "Mods".to_string(), slug: "mods".to_string() },
            SourceCategory { name: "Tools".to_string(), slug: "tools".to_string() },
            SourceCategory { name: "Libraries".to_string(), slug: "libraries".to_string() },
        ])
    }

    async fn get_trending(
        &self,
        _period: TrendingPeriod,
        max_count: usize,
    ) -> Result<Vec<UnifiedMod>> {
        let registry = self.fetch_registry().await?;
        let mods: Vec<UnifiedMod> = registry
            .mods
            .iter()
            .take(max_count)
            .map(|m| self.mod_to_unified(m))
            .collect();
        Ok(mods)
    }

    async fn download(&self, external_id: &str, _version: &str) -> Result<DownloadResult> {
        let registry = self.fetch_registry().await?;

        let m = registry
            .mods
            .iter()
            .find(|m| m.slug == external_id)
            .ok_or_else(|| eyre::eyre!("Mod not found: {}", external_id))?;

        let resp = self.http.get(&m.download).send().await?;
        if !resp.status().is_success() {
            bail!("Failed to download mod: {}", resp.status());
        }

        let bytes = resp.bytes().await?;
        let file_name = m.download.split('/').last().unwrap_or("mod.dll").to_string();

        let temp_dir = std::env::temp_dir().join("zephyr-downloads");
        std::fs::create_dir_all(&temp_dir)?;
        let path = temp_dir.join(&file_name);
        std::fs::write(&path, &bytes)?;

        Ok(DownloadResult { path, file_name })
    }
}
