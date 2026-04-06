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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_id_display_names() {
        assert_eq!(SourceId::Thunderstore.display_name(), "Thunderstore");
        assert_eq!(SourceId::NexusMods.display_name(), "NexusMods");
        assert_eq!(SourceId::CurseForge.display_name(), "CurseForge");
        assert_eq!(SourceId::GitHub.display_name(), "GitHub Releases");
        assert_eq!(SourceId::Local.display_name(), "Local");
    }

    #[test]
    fn source_id_as_str() {
        assert_eq!(SourceId::Thunderstore.as_str(), "thunderstore");
        assert_eq!(SourceId::NexusMods.as_str(), "nexusmods");
        assert_eq!(SourceId::CurseForge.as_str(), "curseforge");
        assert_eq!(SourceId::GitHub.as_str(), "github");
        assert_eq!(SourceId::Local.as_str(), "local");
    }

    #[test]
    fn sort_field_default_is_downloads() {
        assert!(matches!(SortField::default(), SortField::Downloads));
    }

    #[test]
    fn sort_direction_default_is_descending() {
        assert!(matches!(SortDirection::default(), SortDirection::Descending));
    }

    #[test]
    fn search_filters_default() {
        let f = SearchFilters::default();
        assert!(f.search_term.is_empty());
        assert!(f.categories.is_empty());
        assert!(f.sources.is_empty());
        assert!(!f.include_nsfw);
        assert!(!f.include_deprecated);
        assert_eq!(f.offset, 0);
    }

    #[test]
    fn source_id_serde_roundtrip() {
        let ids = vec![
            SourceId::Thunderstore,
            SourceId::NexusMods,
            SourceId::CurseForge,
            SourceId::GitHub,
            SourceId::Local,
        ];
        for id in ids {
            let json = serde_json::to_string(&id).unwrap();
            let back: SourceId = serde_json::from_str(&json).unwrap();
            assert_eq!(back, id);
        }
    }

    #[test]
    fn source_id_serde_lowercase() {
        let json = serde_json::to_string(&SourceId::NexusMods).unwrap();
        assert_eq!(json, "\"nexusmods\"");
    }
}
