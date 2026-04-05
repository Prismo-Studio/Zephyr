use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Identifies which source a mod comes from.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceId {
    Thunderstore,
    NexusMods,
    CurseForge,
    GitHub,
    Local,
}

impl SourceId {
    pub fn display_name(&self) -> &str {
        match self {
            SourceId::Thunderstore => "Thunderstore",
            SourceId::NexusMods => "NexusMods",
            SourceId::CurseForge => "CurseForge",
            SourceId::GitHub => "GitHub Releases",
            SourceId::Local => "Local",
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            SourceId::Thunderstore => "thunderstore",
            SourceId::NexusMods => "nexusmods",
            SourceId::CurseForge => "curseforge",
            SourceId::GitHub => "github",
            SourceId::Local => "local",
        }
    }
}

/// A mod listing from any source — the universal mod representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedMod {
    /// Unique identifier within its source
    pub source_id: SourceId,
    /// Source-specific unique ID (e.g., Thunderstore UUID, Nexus mod ID)
    pub external_id: String,
    /// Display name
    pub name: String,
    /// Mod author/owner
    pub author: String,
    /// Short description
    pub description: Option<String>,
    /// Currently selected/latest version
    pub version: String,
    /// All available versions
    pub versions: Vec<UnifiedModVersion>,
    /// Categories/tags
    pub categories: Vec<String>,
    /// Number of downloads (total)
    pub downloads: Option<u64>,
    /// Rating score
    pub rating: Option<u32>,
    /// Icon/thumbnail URL
    pub icon_url: Option<String>,
    /// Website URL
    pub website_url: Option<String>,
    /// When the mod was last updated
    pub date_updated: Option<DateTime<Utc>>,
    /// When the mod was first created
    pub date_created: Option<DateTime<Utc>>,
    /// File size in bytes
    pub file_size: u64,
    /// Whether the mod is deprecated
    pub is_deprecated: bool,
    /// Whether the mod contains NSFW content
    pub is_nsfw: bool,
    /// Dependencies as source-specific identifiers
    pub dependencies: Vec<String>,
}

/// A single version of a mod from any source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedModVersion {
    /// Version string (semver or source-specific)
    pub version: String,
    /// Source-specific version ID
    pub external_id: String,
    /// When this version was published
    pub date_created: Option<DateTime<Utc>>,
    /// Download count for this version
    pub downloads: Option<u64>,
    /// File size
    pub file_size: u64,
}

/// Search filters that work across all sources.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchFilters {
    pub search_term: String,
    pub categories: Vec<String>,
    pub sort_by: SortField,
    pub sort_order: SortDirection,
    pub include_nsfw: bool,
    pub include_deprecated: bool,
    pub max_count: usize,
    pub sources: Vec<SourceId>,
    #[serde(default)]
    pub game_slug: Option<String>,
    #[serde(default)]
    pub offset: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortField {
    #[default]
    Downloads,
    Rating,
    Newest,
    Updated,
    Name,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    #[default]
    Descending,
    Ascending,
}

/// Result from a multi-source search.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub mods: Vec<UnifiedMod>,
    pub source: SourceId,
    pub total_count: Option<u64>,
}

/// Represents a source's category/tag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCategory {
    pub name: String,
    pub slug: String,
}

/// Trending period for popular mods.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrendingPeriod {
    Day,
    Week,
    Month,
}

/// Information about a source's status and capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInfo {
    pub id: SourceId,
    pub display_name: String,
    pub is_enabled: bool,
    pub requires_auth: bool,
    pub is_authenticated: bool,
    pub supported_games: Option<Vec<String>>,
}

/// Download result pointing to a local file.
pub struct DownloadResult {
    pub path: PathBuf,
    pub file_name: String,
}
