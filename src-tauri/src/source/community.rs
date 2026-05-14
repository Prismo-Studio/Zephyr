//! Zephyr Mods source.
//!
//! Reads the community registry generated at
//! https://github.com/Prismo-Studio/zephyr-mods. Each mod ships a list of
//! immutable GitHub Release URLs with a SHA-256 hash; the hash is verified
//! after download to guarantee the bytes match what the maintainer signed off
//! on at PR review time.
use chrono::{DateTime, Utc};
use eyre::{bail, Result};
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::info;

use super::registry::ModSource;
use super::types::*;

const REGISTRY_URL: &str =
    "https://raw.githubusercontent.com/Prismo-Studio/zephyr-mods/master/registry.json";
const CACHE_TTL: Duration = Duration::from_secs(300);

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Registry {
    #[allow(dead_code)]
    version: u32,
    #[allow(dead_code)]
    generated: Option<String>,
    mods: Vec<CommunityMod>,
}

#[derive(Debug, Clone, Deserialize)]
struct CommunityMod {
    name: String,
    slug: String,
    author: String,
    description: String,
    game: String,
    latest: String,
    repository: String,
    #[serde(default)]
    website: Option<String>,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    dependencies: Vec<String>,
    #[serde(default)]
    nsfw: bool,
    #[serde(default)]
    deprecated: bool,
    icon: String,
    #[serde(default)]
    readme: Option<String>,
    #[serde(default)]
    changelog: Option<String>,
    versions: Vec<CommunityVersion>,
}

#[derive(Debug, Clone, Deserialize)]
struct CommunityVersion {
    version: String,
    released: Option<String>,
    url: String,
    sha256: String,
    size: u64,
    #[serde(default)]
    #[allow(dead_code)]
    changelog: Option<String>,
}

struct CachedRegistry {
    registry: Registry,
    fetched_at: Instant,
}

pub struct CommunitySource {
    http: Client,
    cache: Mutex<Option<CachedRegistry>>,
}

impl CommunitySource {
    pub fn new(http: Client) -> Self {
        info!("Zephyr Mods source initialized");
        Self {
            http,
            cache: Mutex::new(None),
        }
    }

    async fn fetch_registry(&self) -> Result<Registry> {
        if let Ok(guard) = self.cache.lock() {
            if let Some(c) = guard.as_ref() {
                if c.fetched_at.elapsed() < CACHE_TTL {
                    return Ok(c.registry.clone());
                }
            }
        }

        let bust = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let url = format!("{}?_t={}", REGISTRY_URL, bust);
        let resp = self
            .http
            .get(&url)
            .header("Accept", "application/json")
            .header("Cache-Control", "no-cache")
            .send()
            .await?;
        if !resp.status().is_success() {
            bail!("Failed to fetch Zephyr Mods registry: {}", resp.status());
        }
        let registry: Registry = resp.json().await?;

        if let Ok(mut guard) = self.cache.lock() {
            *guard = Some(CachedRegistry {
                registry: registry.clone(),
                fetched_at: Instant::now(),
            });
        }
        Ok(registry)
    }

    fn pick_version<'a>(m: &'a CommunityMod, requested: &str) -> Option<&'a CommunityVersion> {
        if requested.is_empty() {
            m.versions.iter().find(|v| v.version == m.latest)
        } else {
            m.versions.iter().find(|v| v.version == requested)
        }
    }

    fn mod_to_unified(&self, m: &CommunityMod) -> UnifiedMod {
        let latest = Self::pick_version(m, "");
        let date_updated = latest
            .and_then(|v| v.released.as_deref())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.with_timezone(&Utc));
        let file_size = latest.map(|v| v.size).unwrap_or(0);

        let versions = m
            .versions
            .iter()
            .map(|v| UnifiedModVersion {
                version: v.version.clone(),
                external_id: v.version.clone(),
                date_created: v
                    .released
                    .as_deref()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|d| d.with_timezone(&Utc)),
                downloads: None,
                file_size: v.size,
            })
            .collect();

        let external_id = format!("{}/{}", m.author, m.slug);

        UnifiedMod {
            source_id: SourceId::ZephyrMods,
            external_id,
            name: m.name.clone(),
            author: m.author.clone(),
            description: Some(m.description.clone()),
            version: m.latest.clone(),
            versions,
            categories: m.categories.clone(),
            downloads: None,
            rating: None,
            icon_url: Some(m.icon.clone()),
            website_url: Some(m.website.clone().unwrap_or_else(|| m.repository.clone())),
            date_updated,
            date_created: None,
            file_size,
            is_deprecated: m.deprecated,
            is_nsfw: m.nsfw,
            dependencies: m.dependencies.clone(),
        }
    }

    fn parse_external_id(external_id: &str) -> (Option<&str>, &str) {
        match external_id.split_once('/') {
            Some((author, slug)) => (Some(author), slug),
            None => (None, external_id),
        }
    }

    fn find_mod<'a>(reg: &'a Registry, external_id: &str) -> Option<&'a CommunityMod> {
        let (author, slug) = Self::parse_external_id(external_id);
        reg.mods.iter().find(|m| {
            m.slug == slug && author.map_or(true, |a| a.eq_ignore_ascii_case(&m.author))
        })
    }
}

#[async_trait::async_trait]
impl ModSource for CommunitySource {
    fn id(&self) -> SourceId {
        SourceId::ZephyrMods
    }

    fn display_name(&self) -> &str {
        "Zephyr Mods"
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
                if !game_slug.is_empty() && !m.game.eq_ignore_ascii_case(game_slug) {
                    return false;
                }
                if !filters.search_term.is_empty() {
                    let term = filters.search_term.to_lowercase();
                    let hit = m.name.to_lowercase().contains(&term)
                        || m.description.to_lowercase().contains(&term)
                        || m.author.to_lowercase().contains(&term);
                    if !hit {
                        return false;
                    }
                }
                if !filters.categories.is_empty() {
                    let hit = filters
                        .categories
                        .iter()
                        .any(|c| m.categories.iter().any(|mc| mc.eq_ignore_ascii_case(c)));
                    if !hit {
                        return false;
                    }
                }
                if !filters.include_nsfw && m.nsfw {
                    return false;
                }
                if !filters.include_deprecated && m.deprecated {
                    return false;
                }
                true
            })
            .map(|m| self.mod_to_unified(m))
            .collect();

        let total = mods.len() as u64;
        if filters.offset > 0 {
            mods = mods.into_iter().skip(filters.offset).collect();
        }
        if filters.max_count > 0 {
            mods.truncate(filters.max_count);
        }

        Ok(SearchResult {
            mods,
            source: SourceId::ZephyrMods,
            total_count: Some(total),
        })
    }

    async fn get_mod(&self, external_id: &str) -> Result<UnifiedMod> {
        let registry = self.fetch_registry().await?;
        let m = Self::find_mod(&registry, external_id)
            .ok_or_else(|| eyre::eyre!("Mod not found: {}", external_id))?;
        Ok(self.mod_to_unified(m))
    }

    async fn get_readme(&self, external_id: &str, _version: &str) -> Result<Option<String>> {
        let registry = self.fetch_registry().await?;
        let m = match Self::find_mod(&registry, external_id) {
            Some(m) => m,
            None => return Ok(None),
        };
        if let Some(url) = &m.readme {
            let resp = self.http.get(url).send().await?;
            if resp.status().is_success() {
                return Ok(Some(resp.text().await?));
            }
        }
        Ok(None)
    }

    async fn get_changelog(&self, external_id: &str, _version: &str) -> Result<Option<String>> {
        let registry = self.fetch_registry().await?;
        let m = match Self::find_mod(&registry, external_id) {
            Some(m) => m,
            None => return Ok(None),
        };
        if let Some(url) = &m.changelog {
            let resp = self.http.get(url).send().await?;
            if resp.status().is_success() {
                return Ok(Some(resp.text().await?));
            }
        }
        Ok(None)
    }

    async fn get_categories(&self) -> Result<Vec<SourceCategory>> {
        let cats = [
            "gameplay",
            "cosmetic",
            "quality-of-life",
            "library",
            "audio",
            "visual",
            "server",
            "client",
            "tool",
            "misc",
        ];
        Ok(cats
            .into_iter()
            .map(|c| SourceCategory {
                name: c.to_string(),
                slug: c.to_string(),
            })
            .collect())
    }

    async fn get_trending(
        &self,
        _period: TrendingPeriod,
        max_count: usize,
    ) -> Result<Vec<UnifiedMod>> {
        let registry = self.fetch_registry().await?;
        Ok(registry
            .mods
            .iter()
            .take(max_count)
            .map(|m| self.mod_to_unified(m))
            .collect())
    }

    async fn download(&self, external_id: &str, version: &str) -> Result<DownloadResult> {
        let registry = self.fetch_registry().await?;
        let m = Self::find_mod(&registry, external_id)
            .ok_or_else(|| eyre::eyre!("Mod not found: {}", external_id))?;
        let v = Self::pick_version(m, version)
            .ok_or_else(|| eyre::eyre!("Version not found: {}", version))?;

        let resp = self.http.get(&v.url).send().await?;
        if !resp.status().is_success() {
            bail!("Failed to download mod: {}", resp.status());
        }
        let bytes = resp.bytes().await?;

        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let actual = format!("{:x}", hasher.finalize());
        if !actual.eq_ignore_ascii_case(&v.sha256) {
            bail!(
                "SHA-256 mismatch for {} v{}: registry says {}, downloaded bytes hash to {}. Refusing to install.",
                m.slug, v.version, v.sha256, actual
            );
        }

        let file_name = v
            .url
            .rsplit('/')
            .next()
            .unwrap_or(&format!("{}.dll", m.slug))
            .to_string();
        let temp_dir = std::env::temp_dir().join("zephyr-mods-downloads");
        std::fs::create_dir_all(&temp_dir)?;
        let path = temp_dir.join(&file_name);
        std::fs::write(&path, &bytes)?;

        Ok(DownloadResult { path, file_name })
    }
}
