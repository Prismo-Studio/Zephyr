//! Adapter that wraps the existing Thunderstore module to implement the ModSource trait.
//!
//! This bridges the gap between Gale's original Thunderstore-specific code and
//! Zephyr's multi-source abstraction. The existing query/fetch/cache infrastructure
//! continues to work unchanged — this adapter just provides a unified interface.

use eyre::{eyre, Result};
use tauri::AppHandle;

use crate::state::ManagerExt;
use crate::thunderstore::token;

use super::registry::ModSource;
use super::types::*;

/// Wraps the existing Thunderstore infrastructure into the ModSource trait.
pub struct ThunderstoreSource {
    app: AppHandle,
}

impl ThunderstoreSource {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl ModSource for ThunderstoreSource {
    fn id(&self) -> SourceId {
        SourceId::Thunderstore
    }

    fn display_name(&self) -> &str {
        "Thunderstore"
    }

    fn requires_auth(&self) -> bool {
        false // Thunderstore works without auth, token is optional for uploads
    }

    fn is_authenticated(&self) -> bool {
        // Check if user has set a Thunderstore token
        token::get().unwrap_or(None).is_some()
    }

    fn info(&self) -> SourceInfo {
        SourceInfo {
            id: self.id(),
            display_name: self.display_name().to_string(),
            is_enabled: true,
            requires_auth: self.requires_auth(),
            is_authenticated: self.is_authenticated(),
            supported_games: None, // Thunderstore supports all games that Zephyr knows about
        }
    }

    async fn search(&self, filters: &SearchFilters) -> Result<SearchResult> {
        // Delegate to existing Thunderstore query infrastructure.
        // The existing code uses a continuous query loop with events,
        // so for now we do a direct search against the in-memory package list.
        let thunderstore = self.app.lock_thunderstore();

        let mut results: Vec<UnifiedMod> = thunderstore
            .latest()
            .filter(|borrowed| {
                let pkg = borrowed.package;
                let ver = borrowed.version;

                // NSFW filter
                if !filters.include_nsfw && pkg.has_nsfw_content {
                    return false;
                }

                // Deprecated filter
                if !filters.include_deprecated && pkg.is_deprecated {
                    return false;
                }

                // Category filter
                if !filters.categories.is_empty() {
                    let has_match = filters.categories.iter().any(|cat| {
                        pkg.categories.iter().any(|c| c.as_str() == cat.as_str())
                    });
                    if !has_match {
                        return false;
                    }
                }

                // Search term filter
                if !filters.search_term.is_empty() {
                    let term = filters.search_term.to_lowercase();
                    let name_match = pkg.name().to_lowercase().contains(&term)
                        || pkg.full_name().to_lowercase().contains(&term);
                    let desc_match = ver.description.to_lowercase().contains(&term);
                    if !name_match && !desc_match {
                        return false;
                    }
                }

                true
            })
            .map(|borrowed| {
                let pkg = borrowed.package;
                let ver = borrowed.version;

                UnifiedMod {
                    source_id: SourceId::Thunderstore,
                    external_id: pkg.uuid.to_string(),
                    name: pkg.name().to_string(),
                    author: pkg.owner().to_string(),
                    description: Some(ver.description.to_string()),
                    version: ver.version().to_string(),
                    versions: pkg
                        .versions
                        .iter()
                        .map(|v| UnifiedModVersion {
                            version: v.version().to_string(),
                            external_id: v.uuid.to_string(),
                            date_created: Some(v.date_created),
                            downloads: Some(v.downloads as u64),
                            file_size: v.file_size,
                        })
                        .collect(),
                    categories: pkg.categories.iter().map(|c| c.to_string()).collect(),
                    downloads: Some(pkg.total_downloads() as u64),
                    rating: Some(pkg.rating_score),
                    icon_url: None, // Icons are handled separately by the existing infrastructure
                    website_url: Some(ver.website_url.to_string()).filter(|s| !s.is_empty()),
                    date_updated: Some(pkg.date_updated),
                    date_created: Some(pkg.date_created),
                    file_size: ver.file_size,
                    is_deprecated: pkg.is_deprecated,
                    is_nsfw: pkg.has_nsfw_content,
                    dependencies: ver
                        .dependencies
                        .iter()
                        .map(|d| d.full_name().to_string())
                        .collect(),
                }
            })
            .collect();

        // Sort
        match filters.sort_by {
            SortField::Downloads => results.sort_by(|a, b| {
                b.downloads.unwrap_or(0).cmp(&a.downloads.unwrap_or(0))
            }),
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
            source: SourceId::Thunderstore,
            total_count: Some(total),
        })
    }

    async fn get_mod(&self, external_id: &str) -> Result<UnifiedMod> {
        let uuid: uuid::Uuid = external_id
            .parse()
            .map_err(|_| eyre!("invalid Thunderstore package UUID: {}", external_id))?;

        let thunderstore = self.app.lock_thunderstore();
        let pkg = thunderstore.get_package(uuid)?;
        let ver = pkg.latest();

        Ok(UnifiedMod {
            source_id: SourceId::Thunderstore,
            external_id: pkg.uuid.to_string(),
            name: pkg.name().to_string(),
            author: pkg.owner().to_string(),
            description: Some(ver.description.to_string()),
            version: ver.version().to_string(),
            versions: pkg
                .versions
                .iter()
                .map(|v| UnifiedModVersion {
                    version: v.version().to_string(),
                    external_id: v.uuid.to_string(),
                    date_created: Some(v.date_created),
                    downloads: Some(v.downloads as u64),
                    file_size: v.file_size,
                })
                .collect(),
            categories: pkg.categories.iter().map(|c| c.to_string()).collect(),
            downloads: Some(pkg.total_downloads() as u64),
            rating: Some(pkg.rating_score),
            icon_url: None,
            website_url: Some(ver.website_url.to_string()).filter(|s| !s.is_empty()),
            date_updated: Some(pkg.date_updated),
            date_created: Some(pkg.date_created),
            file_size: ver.file_size,
            is_deprecated: pkg.is_deprecated,
            is_nsfw: pkg.has_nsfw_content,
            dependencies: ver
                .dependencies
                .iter()
                .map(|d| d.full_name().to_string())
                .collect(),
        })
    }

    async fn get_readme(&self, _external_id: &str, _version: &str) -> Result<Option<String>> {
        // Markdown fetching is handled by the existing thunderstore::cache module
        // which the frontend already calls via get_markdown command.
        // This is a passthrough for now.
        Ok(None)
    }

    async fn get_changelog(&self, _external_id: &str, _version: &str) -> Result<Option<String>> {
        Ok(None)
    }

    async fn get_categories(&self) -> Result<Vec<SourceCategory>> {
        let thunderstore = self.app.lock_thunderstore();
        let mut categories = std::collections::HashSet::new();

        for borrowed in thunderstore.latest() {
            for cat in &borrowed.package.categories {
                categories.insert(cat.to_string());
            }
        }

        let mut result: Vec<SourceCategory> = categories
            .into_iter()
            .map(|name| {
                let slug = name.to_lowercase().replace(' ', "-");
                SourceCategory { name, slug }
            })
            .collect();

        result.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(result)
    }

    async fn get_trending(
        &self,
        _period: TrendingPeriod,
        max_count: usize,
    ) -> Result<Vec<UnifiedMod>> {
        // Return top mods by downloads
        let filters = SearchFilters {
            sort_by: SortField::Downloads,
            sort_order: SortDirection::Descending,
            max_count,
            ..Default::default()
        };

        let result = self.search(&filters).await?;
        Ok(result.mods)
    }

    async fn download(&self, _external_id: &str, _version: &str) -> Result<DownloadResult> {
        // Downloads are handled by the existing profile::install infrastructure
        // which the frontend already calls via install_mod command.
        // This will be implemented when we unify the download path.
        Err(eyre!(
            "Direct download via ModSource not yet implemented for Thunderstore. \
             Use the existing install_mod command instead."
        ))
    }
}
