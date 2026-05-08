use eyre::Result;
use tauri::{command, AppHandle, Emitter, Manager};

use super::{is_built_in_feature, registry::PluginRegistryState, PluginEntry};
use crate::{state::ManagerExt, util::cmd::Result as CmdResult};

fn collect(app: &AppHandle) -> Vec<PluginEntry> {
    let state = app.state::<PluginRegistryState>();
    let entries = state.entries.lock().unwrap();
    let prefs = app.lock_prefs();

    entries
        .iter()
        .map(|entry| {
            let built_in = is_built_in_feature(entry);
            // Built-in features default to enabled unless explicitly disabled.
            // Other registry plugins need an external install pipeline (TBD),
            // so they show as not enabled.
            let enabled = built_in && !prefs.disabled_plugins.contains(&entry.id);
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
