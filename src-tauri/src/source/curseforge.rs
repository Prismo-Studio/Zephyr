use chrono::DateTime;
use eyre::{eyre, Context, Result};
use serde::Deserialize;
use tracing::debug;

use super::registry::ModSource;
use super::types::*;

const BASE_URL: &str = "https://api.curseforge.com";

// classId 6 = Mods, 12 = Resource Packs, 17 = Modpacks
const CLASS_ID_MODS: u32 = 6;

fn game_id_from_slug(slug: &str) -> Option<u32> {
    match slug {
        "minecraft" => Some(432),
        "minecraft-bedrock" => Some(78022),
        "minecraft-dungeons" => Some(69271),
        "worldofwarcraft" | "wow" => Some(1),
        "stardewvalley" | "stardew-valley" => Some(669),
        "terraria" => Some(431),
        "palworld" => Some(85196),
        "helldivers-2" | "helldivers2" => Some(85440),
        "among-us" | "amongus" => Some(69761),
        "the-elder-scrolls-online" | "teso" => Some(455),
        "starcraft-ii" | "sc2" => Some(65),
        "civilization-vi" | "civ6" => Some(727),
        "kerbal-space-program" | "kerbal" => Some(4401),
        "darkest-dungeon" | "darkestdungeon" => Some(608),
        "surviving-mars" => Some(61489),
        "sims4" | "the-sims-4" => Some(78062),
        _ => None,
    }
}

pub struct CurseForgeSource {
    http: reqwest::Client,
    api_key: String,
    enabled: bool,
}

impl CurseForgeSource {
    pub fn new(api_key: String, http: reqwest::Client) -> Self {
        Self {
            http,
            api_key,
            enabled: true,
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", BASE_URL, path);
        debug!("CurseForge GET: {}", url);

        let resp = self
            .http
            .get(&url)
            .header("x-api-key", &self.api_key)
            .header("Accept", "application/json")
            .send()
            .await
            .context("CurseForge request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(eyre!("CurseForge API {}. {}", status, body));
        }

        resp.json::<T>()
            .await
            .context("failed to parse CurseForge JSON")
    }

    // POST method ready for batch endpoints (fingerprints, batch fetch)
    #[allow(dead_code)]
    async fn post<T: for<'de> Deserialize<'de>, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", BASE_URL, path);
        debug!("CurseForge POST: {}", url);

        let resp = self
            .http
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await
            .context("CurseForge POST failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(eyre!("CurseForge API {}. {}", status, body));
        }

        resp.json::<T>()
            .await
            .context("failed to parse CurseForge JSON")
    }
}

// ── API response types ──

#[derive(Deserialize)]
struct CfResponse<T> {
    data: T,
}

#[derive(Deserialize)]
struct CfSearchResponse {
    data: Vec<CfMod>,
    pagination: CfPagination,
}

#[derive(Deserialize)]
struct CfPagination {
    #[serde(rename = "totalCount")]
    total_count: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfMod {
    id: u32,
    name: String,
    summary: String,
    download_count: u64,
    thumbs_up_count: Option<u32>,
    date_created: String,
    date_modified: String,
    logo: Option<CfLogo>,
    authors: Vec<CfAuthor>,
    categories: Vec<CfCategory>,
    latest_files: Vec<CfFile>,
    links: CfLinks,
    #[serde(default)]
    #[allow(dead_code)]
    screenshots: Vec<CfScreenshot>,
}

#[derive(Deserialize)]
struct CfLogo {
    url: String,
}

#[derive(Deserialize)]
struct CfAuthor {
    name: String,
}

#[derive(Deserialize)]
struct CfCategory {
    name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfFile {
    id: u32,
    display_name: String,
    file_name: String,
    file_length: u64,
    file_date: String,
    download_count: u64,
    download_url: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    hashes: Vec<CfHash>,
    #[serde(default)]
    #[allow(dead_code)]
    game_versions: Vec<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct CfHash {
    value: String,
    algo: u32,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct CfScreenshot {
    url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfLinks {
    website_url: Option<String>,
}

impl CfMod {
    fn to_unified(&self) -> UnifiedMod {
        let author = self
            .authors
            .first()
            .map(|a| a.name.clone())
            .unwrap_or_default();

        let versions: Vec<UnifiedModVersion> = self
            .latest_files
            .iter()
            .map(|f| UnifiedModVersion {
                version: f.display_name.clone(),
                external_id: f.id.to_string(),
                date_created: DateTime::parse_from_rfc3339(&f.file_date)
                    .ok()
                    .map(|d| d.with_timezone(&chrono::Utc)),
                downloads: Some(f.download_count),
                file_size: f.file_length,
            })
            .collect();

        let version = versions
            .first()
            .map(|v| v.version.clone())
            .unwrap_or_default();
        let file_size = versions.first().map(|v| v.file_size).unwrap_or(0);

        UnifiedMod {
            source_id: SourceId::CurseForge,
            external_id: self.id.to_string(),
            name: self.name.clone(),
            author,
            description: Some(self.summary.clone()),
            version,
            versions,
            categories: self.categories.iter().map(|c| c.name.clone()).collect(),
            downloads: Some(self.download_count),
            rating: self.thumbs_up_count,
            icon_url: self.logo.as_ref().map(|l| l.url.clone()),
            website_url: self.links.website_url.clone(),
            date_updated: DateTime::parse_from_rfc3339(&self.date_modified)
                .ok()
                .map(|d| d.with_timezone(&chrono::Utc)),
            date_created: DateTime::parse_from_rfc3339(&self.date_created)
                .ok()
                .map(|d| d.with_timezone(&chrono::Utc)),
            file_size,
            is_deprecated: false,
            is_nsfw: false,
            dependencies: Vec::new(),
        }
    }
}

// ── ModSource implementation ──

#[async_trait::async_trait]
impl ModSource for CurseForgeSource {
    fn id(&self) -> SourceId {
        SourceId::CurseForge
    }

    fn display_name(&self) -> &str {
        "CurseForge"
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
            is_enabled: self.enabled,
            requires_auth: true,
            is_authenticated: self.is_authenticated(),
            supported_games: None,
        }
    }

    async fn search(&self, filters: &SearchFilters) -> Result<SearchResult> {
        if !self.enabled || self.api_key.is_empty() {
            return Ok(SearchResult {
                mods: Vec::new(),
                source: SourceId::CurseForge,
                total_count: Some(0),
            });
        }

        let cf_game_id = match filters
            .game_slug
            .as_deref()
            .and_then(game_id_from_slug)
        {
            Some(id) => id,
            None => {
                return Ok(SearchResult {
                    mods: Vec::new(),
                    source: SourceId::CurseForge,
                    total_count: Some(0),
                });
            }
        };

        let sort_field = match filters.sort_by {
            SortField::Downloads => 6,
            SortField::Rating => 3,
            SortField::Newest => 11,
            SortField::Updated => 2,
            SortField::Name => 1,
        };

        let sort_order = match filters.sort_order {
            SortDirection::Descending => "desc",
            SortDirection::Ascending => "asc",
        };

        let page_size = if filters.max_count > 0 && filters.max_count <= 50 {
            filters.max_count
        } else {
            25
        };

        // classId 6 = Mods only works for Minecraft; skip for other games
        let class_param = if cf_game_id == 432 {
            format!("&classId={}", CLASS_ID_MODS)
        } else {
            String::new()
        };

        let mut url = format!(
            "/v1/mods/search?gameId={}{}&sortField={}&sortOrder={}&pageSize={}&index={}",
            cf_game_id, class_param, sort_field, sort_order, page_size, filters.offset
        );

        if !filters.search_term.is_empty() {
            url.push_str(&format!(
                "&searchFilter={}",
                urlencoding::encode(&filters.search_term)
            ));
        }

        let response: CfSearchResponse = self.get(&url).await?;

        let mods = response.data.iter().map(|m| m.to_unified()).collect();

        Ok(SearchResult {
            mods,
            source: SourceId::CurseForge,
            total_count: Some(response.pagination.total_count),
        })
    }

    async fn get_mod(&self, external_id: &str) -> Result<UnifiedMod> {
        let response: CfResponse<CfMod> =
            self.get(&format!("/v1/mods/{}", external_id)).await?;
        Ok(response.data.to_unified())
    }

    async fn get_readme(&self, external_id: &str, _version: &str) -> Result<Option<String>> {
        let response: CfResponse<String> = self
            .get(&format!("/v1/mods/{}/description", external_id))
            .await?;
        Ok(Some(response.data))
    }

    async fn get_changelog(&self, external_id: &str, version: &str) -> Result<Option<String>> {
        let response: CfResponse<String> = self
            .get(&format!(
                "/v1/mods/{}/files/{}/changelog",
                external_id, version
            ))
            .await?;
        Ok(Some(response.data))
    }

    async fn get_categories(&self) -> Result<Vec<SourceCategory>> {
        Ok(Vec::new())
    }

    async fn get_trending(&self, _period: TrendingPeriod, max_count: usize) -> Result<Vec<UnifiedMod>> {
        let filters = SearchFilters {
            sort_by: SortField::Downloads,
            sort_order: SortDirection::Descending,
            max_count,
            ..Default::default()
        };
        let result = self.search(&filters).await?;
        Ok(result.mods)
    }

    async fn download(&self, external_id: &str, version: &str) -> Result<DownloadResult> {
        // Get the file info to find the download URL
        let file: CfResponse<CfFile> = self
            .get(&format!("/v1/mods/{}/files/{}", external_id, version))
            .await?;

        match file.data.download_url {
            Some(url) => {
                let tmp_dir = std::env::temp_dir().join("zephyr-cf-downloads");
                std::fs::create_dir_all(&tmp_dir)?;
                let file_path = tmp_dir.join(&file.data.file_name);

                let resp = self
                    .http
                    .get(&url)
                    .send()
                    .await
                    .context("failed to download CF mod")?;

                let bytes = resp.bytes().await?;
                std::fs::write(&file_path, &bytes)?;

                Ok(DownloadResult {
                    path: file_path,
                    file_name: file.data.file_name,
                })
            }
            None => Err(eyre!(
                "This mod has blocked direct downloads. Visit CurseForge to download manually."
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_slugs_map_correctly() {
        assert_eq!(game_id_from_slug("minecraft"), Some(432));
        assert_eq!(game_id_from_slug("terraria"), Some(431));
        assert_eq!(game_id_from_slug("palworld"), Some(85196));
        assert_eq!(game_id_from_slug("kerbal-space-program"), Some(4401));
    }

    #[test]
    fn alternate_slugs_resolve() {
        assert_eq!(game_id_from_slug("wow"), Some(1));
        assert_eq!(game_id_from_slug("worldofwarcraft"), Some(1));
        assert_eq!(game_id_from_slug("stardew-valley"), Some(669));
        assert_eq!(game_id_from_slug("stardewvalley"), Some(669));
        assert_eq!(game_id_from_slug("helldivers-2"), Some(85440));
        assert_eq!(game_id_from_slug("helldivers2"), Some(85440));
    }

    #[test]
    fn unknown_slug_returns_none() {
        assert_eq!(game_id_from_slug("nonexistent-game"), None);
    }

    #[test]
    fn empty_slug_returns_none() {
        assert_eq!(game_id_from_slug(""), None);
    }
}
