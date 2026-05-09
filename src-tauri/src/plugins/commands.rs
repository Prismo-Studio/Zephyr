use eyre::{eyre, Result};
use serde::Serialize;
use tauri::{command, AppHandle, Emitter, Manager};

use super::{is_built_in_feature, registry::PluginRegistryState, PluginEntry, PluginType, RegistryEntry};
use crate::{state::ManagerExt, util::cmd::Result as CmdResult};

fn collect(app: &AppHandle) -> Vec<PluginEntry> {
    let state = app.state::<PluginRegistryState>();
    let entries = state.entries.lock().unwrap();
    let prefs = app.lock_prefs();

    entries
        .iter()
        .map(|entry| {
            let built_in = is_built_in_feature(entry);
            let installed = prefs.installed_plugins.contains(&entry.id);
            // Built-in features default to enabled unless explicitly disabled.
            // Registry plugins (themes etc.) report enabled when the user has
            // installed them via the install pipeline.
            let enabled =
                (built_in || installed) && !prefs.disabled_plugins.contains(&entry.id);
            PluginEntry {
                id: entry.id.clone(),
                name: entry.name.clone(),
                description: entry.description.clone(),
                author: entry.author.name.clone(),
                version: entry.version.clone(),
                icon: super::registry::icon_url(entry),
                kind: entry.kind,
                built_in,
                removable: entry.removable,
                enabled,
            }
        })
        .collect()
}

pub fn emit_changed(app: &AppHandle) -> Result<()> {
    app.emit("plugins_changed", collect(app))?;
    Ok(())
}

fn find_entry(app: &AppHandle, id: &str) -> Result<RegistryEntry> {
    let state = app.state::<PluginRegistryState>();
    let entries = state.entries.lock().unwrap();
    entries
        .iter()
        .find(|e| e.id == id)
        .cloned()
        .ok_or_else(|| eyre!("unknown plugin id: {id}"))
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstalledTheme {
    pub id: String,
    pub name: String,
    pub css: String,
}

#[command]
pub fn get_plugins(app: AppHandle) -> Vec<PluginEntry> {
    collect(&app)
}

#[command]
pub fn set_plugin_enabled(id: String, enabled: bool, app: AppHandle) -> CmdResult<()> {
    {
        let mut prefs = app.lock_prefs();
        if enabled {
            prefs.disabled_plugins.remove(&id);
        } else {
            prefs.disabled_plugins.insert(id);
        }
        prefs.save_to_db(app.db())?;
    }
    emit_changed(&app)?;
    Ok(())
}

#[command]
pub async fn refresh_plugins(app: AppHandle) -> CmdResult<()> {
    super::registry::fetch_and_update(app)
        .await
        .map_err(Into::into)
}

#[command]
pub async fn install_plugin(id: String, app: AppHandle) -> CmdResult<InstalledTheme> {
    let entry = find_entry(&app, &id)?;

    let css = match entry.kind {
        PluginType::Theme => super::install::install_theme(&app, &entry).await?,
        PluginType::Feature => {
            return Err(eyre!("feature plugins are bundled, not installable").into())
        }
        PluginType::Game | PluginType::Mod => {
            return Err(eyre!(
                "install pipeline for {:?} plugins is not implemented yet",
                entry.kind
            )
            .into())
        }
    };

    {
        let mut prefs = app.lock_prefs();
        prefs.installed_plugins.insert(entry.id.clone());
        prefs.save_to_db(app.db())?;
    }
    emit_changed(&app)?;

    Ok(InstalledTheme {
        id: entry.id,
        name: entry.name,
        css,
    })
}

#[command]
pub fn uninstall_plugin(id: String, app: AppHandle) -> CmdResult<()> {
    super::install::uninstall(&id)?;
    {
        let mut prefs = app.lock_prefs();
        prefs.installed_plugins.remove(&id);
        prefs.save_to_db(app.db())?;
    }
    emit_changed(&app)?;
    Ok(())
}

/// Read every installed theme from disk so the frontend can inject the CSS
/// on boot. Silently skips entries whose CSS file is missing — the UI
/// degrades to "not installed" instead of crashing.
#[command]
pub fn get_installed_themes(app: AppHandle) -> Vec<InstalledTheme> {
    let state = app.state::<PluginRegistryState>();
    let entries = state.entries.lock().unwrap();
    let prefs = app.lock_prefs();

    entries
        .iter()
        .filter(|e| {
            matches!(e.kind, PluginType::Theme) && prefs.installed_plugins.contains(&e.id)
        })
        .filter_map(|e| {
            super::install::read_theme_css(e)
                .ok()
                .map(|css| InstalledTheme {
                    id: e.id.clone(),
                    name: e.name.clone(),
                    css,
                })
        })
        .collect()
}
