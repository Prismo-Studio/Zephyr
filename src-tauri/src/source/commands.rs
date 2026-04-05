use serde::Serialize;
use tauri::{command, AppHandle};

use crate::{state::ManagerExt, util::cmd::Result};

use super::types::*;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceGame {
    pub name: String,
    pub slug: String,
    pub icon_url: Option<String>,
    pub mod_count: u64,
}

#[command]
pub fn get_sources(app: AppHandle) -> Vec<SourceInfo> {
    let registry = app.source_registry();
    let mut sources = registry.list_sources();

    let registered_ids: Vec<_> = sources.iter().map(|s| s.id.clone()).collect();

    if !registered_ids.contains(&SourceId::GitHub) {
        sources.push(SourceInfo {
            id: SourceId::GitHub,
            display_name: "GitHub Releases".to_string(),
            is_enabled: false,
            requires_auth: false,
            is_authenticated: true,
            supported_games: None,
        });
    }

    sources
}

#[command]
pub async fn search_sources(
    mut filters: SearchFilters,
    app: AppHandle,
) -> Result<Vec<SearchResult>> {
    if filters.game_slug.is_none() {
        let manager = app.lock_manager();
        filters.game_slug = Some(manager.active_game.slug.to_string());
    }

    let registry = app.source_registry();
    let results = registry.search(&filters).await?;
    Ok(results)
}

#[command]
pub async fn get_source_mod_description(
    source: SourceId,
    external_id: String,
    app: AppHandle,
) -> Result<Option<String>> {
    let registry = app.source_registry();
    if let Some(src) = registry.get(&source) {
        let desc = src.get_readme(&external_id, "").await?;
        Ok(desc)
    } else {
        Ok(None)
    }
}

#[command]
pub async fn get_source_mod_changelog(
    source: SourceId,
    external_id: String,
    file_id: String,
    app: AppHandle,
) -> Result<Option<String>> {
    let registry = app.source_registry();
    if let Some(src) = registry.get(&source) {
        let cl = src.get_changelog(&external_id, &file_id).await?;
        Ok(cl)
    } else {
        Ok(None)
    }
}

#[command]
pub async fn get_nexusmods_games(app: AppHandle) -> Result<Vec<SourceGame>> {
    let http = app.http().clone();
    let registry = app.source_registry();

    let api_key = match registry.get(&SourceId::NexusMods) {
        Some(src) => {
            if !src.is_authenticated() {
                return Ok(Vec::new());
            }
            // Get the API key from the source info
            "NpZIiZhZZ2++vggbQP7B/YV0wxHEtuaK3AI54ToNPixXCWo=--ooji3W0wNmBn2dew--1FPMTUaqOZStJWkqJPvaJg==".to_string()
        }
        None => return Ok(Vec::new()),
    };

    let response = http
        .get("https://api.nexusmods.com/v1/games.json?include_unapproved=false")
        .header("apikey", &api_key)
        .header("Accept", "application/json")
        .send()
        .await?;

    if !response.status().is_success() {
        return Ok(Vec::new());
    }

    #[derive(serde::Deserialize)]
    struct NxGameEntry {
        name: String,
        domain_name: String,
        #[serde(default)]
        mods: u64,
        #[serde(default)]
        #[allow(dead_code)]
        downloads: u64,
    }

    let games: Vec<NxGameEntry> = response.json().await?;

    let mut result: Vec<SourceGame> = games
        .into_iter()
        .filter(|g| g.mods > 0)
        .map(|g| SourceGame {
            name: g.name,
            slug: g.domain_name,
            icon_url: None,
            mod_count: g.mods,
        })
        .collect();

    result.sort_by(|a, b| b.mod_count.cmp(&a.mod_count));

    Ok(result)
}

#[command]
pub async fn get_curseforge_games(app: AppHandle) -> Result<Vec<SourceGame>> {
    let http = app.http().clone();

    let response = http
        .get("https://api.curseforge.com/v1/games?pageSize=500")
        .header(
            "x-api-key",
            "$2a$10$OY0apZlG0KEHe3CTgumu6u2uodPke309xuW4W/SmhhXe2KsVI4KKu",
        )
        .header("Accept", "application/json")
        .send()
        .await?;

    if !response.status().is_success() {
        return Ok(Vec::new());
    }

    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct CfGameEntry {
        id: u32,
        name: String,
        slug: String,
    }

    #[derive(serde::Deserialize)]
    struct CfGamesResponse {
        data: Vec<CfGameEntry>,
    }

    let resp: CfGamesResponse = response.json().await?;

    let result: Vec<SourceGame> = resp
        .data
        .into_iter()
        .map(|g| SourceGame {
            name: g.name,
            slug: g.slug,
            icon_url: None,
            mod_count: 0,
        })
        .collect();

    Ok(result)
}
