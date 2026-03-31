use std::collections::HashMap;
use std::sync::Arc;

use eyre::Result;
use tokio::sync::RwLock;
use tracing::info;

use super::types::*;

/// Trait that every mod source must implement.
///
/// This is the core abstraction that makes Zephyr multi-source.
/// Each source (Thunderstore, NexusMods, CurseForge, GitHub) implements
/// this trait to provide a unified interface for searching, downloading,
/// and managing mods.
#[async_trait::async_trait]
pub trait ModSource: Send + Sync {
    /// Returns the unique identifier for this source.
    fn id(&self) -> SourceId;

    /// Returns human-readable name.
    fn display_name(&self) -> &str;

    /// Whether this source requires authentication.
    fn requires_auth(&self) -> bool {
        false
    }

    /// Whether the user is currently authenticated.
    fn is_authenticated(&self) -> bool {
        !self.requires_auth()
    }

    /// Get source info for the frontend.
    fn info(&self) -> SourceInfo {
        SourceInfo {
            id: self.id(),
            display_name: self.display_name().to_string(),
            is_enabled: true,
            requires_auth: self.requires_auth(),
            is_authenticated: self.is_authenticated(),
            supported_games: None,
        }
    }

    /// Search for mods matching the given filters.
    async fn search(&self, filters: &SearchFilters) -> Result<SearchResult>;

    /// Get detailed info about a specific mod.
    async fn get_mod(&self, external_id: &str) -> Result<UnifiedMod>;

    /// Get the README/description for a mod.
    async fn get_readme(&self, external_id: &str, version: &str) -> Result<Option<String>>;

    /// Get the changelog for a mod version.
    async fn get_changelog(&self, external_id: &str, version: &str) -> Result<Option<String>>;

    /// Get available categories for the current game.
    async fn get_categories(&self) -> Result<Vec<SourceCategory>>;

    /// Get trending/popular mods.
    async fn get_trending(&self, period: TrendingPeriod, max_count: usize) -> Result<Vec<UnifiedMod>>;

    /// Download a mod to a temporary location.
    async fn download(&self, external_id: &str, version: &str) -> Result<DownloadResult>;
}

/// Registry that manages all mod sources.
///
/// This is the central hub that the frontend talks to. It routes
/// search queries to the appropriate sources and merges results.
pub struct SourceRegistry {
    sources: HashMap<SourceId, Arc<dyn ModSource>>,
    enabled: RwLock<Vec<SourceId>>,
}

impl Default for SourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceRegistry {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
            enabled: RwLock::new(Vec::new()),
        }
    }

    /// Register a new mod source.
    pub fn register(&mut self, source: Arc<dyn ModSource>) {
        let id = source.id();
        info!("registered mod source: {}", source.display_name());
        self.sources.insert(id.clone(), source);

        // Enable by default
        let enabled = self.enabled.get_mut();
        enabled.push(id);
    }

    /// Get a specific source by ID.
    pub fn get(&self, id: &SourceId) -> Option<&Arc<dyn ModSource>> {
        self.sources.get(id)
    }

    /// Get info about all registered sources.
    pub fn list_sources(&self) -> Vec<SourceInfo> {
        self.sources
            .values()
            .map(|s| s.info())
            .collect()
    }

    /// Search across multiple sources, merging results.
    pub async fn search(&self, filters: &SearchFilters) -> Result<Vec<SearchResult>> {
        let sources_to_query: Vec<_> = if filters.sources.is_empty() {
            // Query all enabled sources
            let enabled = self.enabled.read().await;
            enabled
                .iter()
                .filter_map(|id| self.sources.get(id))
                .cloned()
                .collect()
        } else {
            filters
                .sources
                .iter()
                .filter_map(|id| self.sources.get(id))
                .cloned()
                .collect()
        };

        let mut results = Vec::new();

        // Query sources concurrently
        let handles: Vec<_> = sources_to_query
            .into_iter()
            .map(|source| {
                let filters = filters.clone();
                tokio::spawn(async move { source.search(&filters).await })
            })
            .collect();

        for handle in handles {
            match handle.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(err)) => {
                    tracing::warn!("source search failed: {:#}", err);
                }
                Err(err) => {
                    tracing::warn!("source search task panicked: {:#}", err);
                }
            }
        }

        Ok(results)
    }

    /// Get trending mods from all sources.
    pub async fn get_trending(
        &self,
        period: TrendingPeriod,
        max_count: usize,
    ) -> Result<Vec<UnifiedMod>> {
        let enabled = self.enabled.read().await;
        let mut all_trending = Vec::new();

        let handles: Vec<_> = enabled
            .iter()
            .filter_map(|id| self.sources.get(id))
            .map(|source| {
                let source = source.clone();
                let period = period.clone();
                tokio::spawn(async move { source.get_trending(period, max_count).await })
            })
            .collect();

        for handle in handles {
            if let Ok(Ok(mods)) = handle.await {
                all_trending.extend(mods);
            }
        }

        // Sort merged results by downloads
        all_trending.sort_by(|a, b| b.downloads.unwrap_or(0).cmp(&a.downloads.unwrap_or(0)));
        all_trending.truncate(max_count);

        Ok(all_trending)
    }
}
