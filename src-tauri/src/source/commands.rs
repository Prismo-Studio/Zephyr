use serde::Serialize;
use tauri::{command, AppHandle};

use crate::{
    profile::{
        import::{import_local_mod_with_meta, LocalModMeta},
        install::InstallOptions,
    },
    state::ManagerExt,
    util::cmd::Result,
};

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
    registry.list_sources()
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
pub async fn install_source_mod(
    source: SourceId,
    external_id: String,
    version: String,
    app: AppHandle,
) -> Result<()> {
    let (download, unified, readme, changelog) = {
        let registry = app.source_registry();
        let src = registry
            .get(&source)
            .ok_or_else(|| eyre::eyre!("Source not registered: {:?}", source))?;
        let download = src.download(&external_id, &version).await?;
        let unified = src.get_mod(&external_id).await.ok();
        let readme = src.get_readme(&external_id, &version).await.ok().flatten();
        let changelog = src
            .get_changelog(&external_id, &version)
            .await
            .ok()
            .flatten();
        (download, unified, readme, changelog)
    };

    let source_str = match source {
        SourceId::ZephyrMods => "zephyrmods".to_string(),
        SourceId::CurseForge => "curseforge".to_string(),
        SourceId::Thunderstore => "thunderstore".to_string(),
        SourceId::NexusMods => "nexusmods".to_string(),
        SourceId::Local => "local".to_string(),
    };
    let ext_id_clone = external_id.clone();
    let installed_version = if version.is_empty() {
        unified.as_ref().map(|u| u.version.clone())
    } else {
        Some(version.clone())
    };
    let parsed_version = installed_version
        .as_deref()
        .and_then(|v| v.parse::<semver::Version>().ok());
    let meta = Some(match unified {
        Some(u) => {
            let deps: Vec<crate::thunderstore::VersionIdent> = u
                .dependencies
                .iter()
                .filter_map(|d| d.parse().ok())
                .collect();
            LocalModMeta {
                name: u.name,
                author: Some(u.author),
                version: parsed_version,
                description: u.description,
                icon: u.icon_url.map(std::path::PathBuf::from),
                readme,
                changelog,
                dependencies: if deps.is_empty() { None } else { Some(deps) },
                source: Some(source_str),
                external_id: Some(ext_id_clone),
            }
        }
        None => LocalModMeta {
            name: ext_id_clone.rsplit('/').next().unwrap_or(&ext_id_clone).to_string(),
            author: None,
            version: parsed_version,
            description: None,
            icon: None,
            readme,
            changelog,
            dependencies: None,
            source: Some(source_str),
            external_id: Some(ext_id_clone),
        },
    });

    import_local_mod_with_meta(download.path, None, meta, &app, InstallOptions::default()).await?;
    Ok(())
}

#[command]
pub async fn get_source_mod_info(
    source: SourceId,
    external_id: String,
    app: AppHandle,
) -> Result<Option<UnifiedMod>> {
    let registry = app.source_registry();
    if let Some(src) = registry.get(&source) {
        let m = src.get_mod(&external_id).await.ok();
        Ok(m)
    } else {
        Ok(None)
    }
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
            crate::util::keys::nexusmods_key()
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
        .header("x-api-key", &crate::util::keys::curseforge_key())
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
