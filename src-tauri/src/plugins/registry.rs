use std::{path::PathBuf, sync::Mutex};

use eyre::{Context, Result};
use tauri::{AppHandle, Manager};
use tracing::{info, warn};

use super::{PluginType, RegistryEntry};
use crate::{
    constants::{PLUGIN_REGISTRY_RAW_BASE, PLUGIN_REGISTRY_URL},
    state::ManagerExt,
    util,
};

fn fallback_iconify(kind: PluginType) -> &'static str {
    match kind {
        PluginType::Feature => "mdi:puzzle",
        PluginType::Theme => "mdi:palette",
        PluginType::Game => "mdi:gamepad-variant",
        PluginType::Mod => "mdi:package-variant",
    }
}

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
    let Some(icon) = entry.icon.as_deref() else {
        return fallback_iconify(entry.kind).to_string();
    };
    // Absolute URL or app-root path (local static asset) → use as-is.
    if icon.starts_with("http://") || icon.starts_with("https://") || icon.starts_with('/') {
        return icon.to_string();
    }
    format!("{}{}/{}", PLUGIN_REGISTRY_RAW_BASE, entry.path, icon)
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
    // Append a timestamp query param so raw.githubusercontent.com's CDN
    // doesn't serve a stale registry.json for up to 5 minutes after the
    // bot rebuilds. Combined with the Cache-Control: no-cache header for
    // belt-and-braces.
    let bust = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let url = format!("{}?_t={}", PLUGIN_REGISTRY_URL, bust);
    info!("fetching plugin registry from {}", url);

    let response: RegistryFile = app
        .http()
        .get(&url)
        .header("Cache-Control", "no-cache")
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
