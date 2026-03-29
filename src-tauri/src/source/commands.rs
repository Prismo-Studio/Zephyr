use tauri::{command, AppHandle};

use crate::state::ManagerExt;

use super::types::*;

/// Get list of all registered sources and their status.
///
/// Returns live status from the SourceRegistry for registered sources,
/// plus placeholder entries for sources not yet implemented.
#[command]
pub fn get_sources(app: AppHandle) -> Vec<SourceInfo> {
    let registry = app.source_registry();
    let mut sources = registry.list_sources();

    // Add placeholder entries for sources that are not yet registered
    // but will be available in the future.
    let registered_ids: Vec<_> = sources.iter().map(|s| s.id.clone()).collect();

    let upcoming = [
        SourceInfo {
            id: SourceId::NexusMods,
            display_name: "NexusMods".to_string(),
            is_enabled: false,
            requires_auth: true,
            is_authenticated: false,
            supported_games: None,
        },
        SourceInfo {
            id: SourceId::CurseForge,
            display_name: "CurseForge".to_string(),
            is_enabled: false,
            requires_auth: true,
            is_authenticated: false,
            supported_games: None,
        },
        SourceInfo {
            id: SourceId::GitHub,
            display_name: "GitHub Releases".to_string(),
            is_enabled: false,
            requires_auth: false,
            is_authenticated: true,
            supported_games: None,
        },
    ];

    for info in upcoming {
        if !registered_ids.contains(&info.id) {
            sources.push(info);
        }
    }

    sources
}
