use std::{path::PathBuf, sync::Mutex};

use eyre::{Context, Result};
use tauri::{AppHandle, Manager};
use tracing::{info, warn};

use super::RegistryEntry;
use crate::{
    constants::{PLUGIN_REGISTRY_RAW_BASE, PLUGIN_REGISTRY_URL},
    state::ManagerExt,
    util,
};

const CACHE_FILE_NAME: &str = "plugin-registry.json";

#[derive(Default)]
pub struct PluginRegistryState {
    pub entries: Mutex<Vec<RegistryEntry>>,
}

#[derive(serde::Deserialize)]
struct RegistryFile {
    plugins: Vec<RegistryEntry>,
}

pub fn icon_url(entry: &RegistryEntry) -> String {
    // Absolute URL or app-root path (local static asset) → use as-is.
    if entry.icon.starts_with("http://")
        || entry.icon.starts_with("https://")
        || entry.icon.starts_with('/')
    {
        return entry.icon.clone();
    }
    format!("{}{}/{}", PLUGIN_REGISTRY_RAW_BASE, entry.path, entry.icon)
}

fn cache_path() -> PathBuf {
    util::path::default_app_data_dir().join(CACHE_FILE_NAME)
}

pub fn load_cache() -> Option<Vec<RegistryEntry>> {
    let bytes = std::fs::read(cache_path()).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn save_cache(plugins: &[RegistryEntry]) -> Result<()> {
    let json = serde_json::to_string(plugins).context("serialize registry cache")?;
    std::fs::write(cache_path(), json).context("write registry cache")?;
    Ok(())
}

pub async fn fetch_and_update(app: AppHandle) -> Result<()> {
    info!("fetching plugin registry from {}", PLUGIN_REGISTRY_URL);

    let response: RegistryFile = app
        .http()
        .get(PLUGIN_REGISTRY_URL)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    info!("loaded {} plugins from registry", response.plugins.len());

    if let Err(err) = save_cache(&response.plugins) {
        warn!("failed to save plugin registry cache: {err:#}");
    }

    {
        let state = app.state::<PluginRegistryState>();
        *state.entries.lock().unwrap() = response.plugins;
    }

    super::commands::emit_changed(&app)?;
    Ok(())
}
