//! In-process "Dev Mode" plugin store.
//!
//! Lets a plugin author point Zephyr at a folder on disk that contains a
//! `manifest.json`. The folder is treated as an extra registry entry alongside
//! the remote registry, with the SHA / network fetch bypassed so the author
//! can iterate on theme.css or ui/index.html without re-publishing.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use eyre::{bail, Context, Result};
use notify_debouncer_mini::notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tracing::warn;

use super::RegistryEntry;

pub const PERSIST_FILE: &str = "dev-plugins.json";

#[derive(Debug, Clone)]
pub struct DevPlugin {
    pub path: PathBuf,
    pub manifest: RegistryEntry,
}

#[derive(Default)]
pub struct DevPluginState {
    pub entries: Mutex<Vec<DevPlugin>>,
    pub watchers: Mutex<HashMap<String, Debouncer<notify_debouncer_mini::notify::RecommendedWatcher>>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawManifest {
    id: String,
    name: String,
    version: String,
    #[serde(rename = "type")]
    kind: super::PluginType,
    author: super::RegistryAuthor,
    description: String,
    #[serde(default)]
    icon: Option<String>,
    #[serde(default)]
    entry: Option<String>,
    #[serde(default)]
    sidebar_label: Option<String>,
    #[serde(default)]
    sidebar_icon: Option<String>,
}

/// Spawn a debounced filesystem watcher on a dev plugin folder. Emits
/// `plugin_changed` to the frontend (with the plugin id) whenever anything
/// inside changes, with a 200ms debounce so saving a file doesn't fire 3
/// events per save.
pub fn spawn_watcher(
    app: AppHandle,
    plugin_id: String,
    path: PathBuf,
) -> Result<Debouncer<notify_debouncer_mini::notify::RecommendedWatcher>> {
    use tauri::Emitter;

    let id_for_handler = plugin_id.clone();
    let mut debouncer = new_debouncer(
        Duration::from_millis(200),
        move |res: DebounceEventResult| {
            if res.is_err() {
                return;
            }
            // Re-read manifest + emit so the frontend re-fetches the CSS / UI.
            let _ = app.emit(
                "dev_plugin_changed",
                serde_json::json!({ "id": id_for_handler }),
            );
        },
    )
    .context("create debouncer")?;

    debouncer
        .watcher()
        .watch(&path, RecursiveMode::Recursive)
        .context("watch dev plugin folder")?;

    let _ = plugin_id;
    Ok(debouncer)
}

fn persist_path() -> PathBuf {
    crate::util::path::default_app_data_dir().join(PERSIST_FILE)
}

#[derive(Serialize, Deserialize, Default)]
struct PersistFile {
    paths: Vec<String>,
}

pub fn save_paths(paths: &[PathBuf]) {
    let raw = PersistFile {
        paths: paths.iter().map(|p| p.to_string_lossy().to_string()).collect(),
    };
    let json = match serde_json::to_string(&raw) {
        Ok(s) => s,
        Err(err) => {
            warn!("serialise dev plugin paths: {err:#}");
            return;
        }
    };
    if let Err(err) = std::fs::write(persist_path(), json) {
        warn!("write dev plugin paths: {err:#}");
    }
}

pub fn load_paths() -> Vec<PathBuf> {
    let bytes = match std::fs::read(persist_path()) {
        Ok(b) => b,
        Err(_) => return vec![],
    };
    let raw: PersistFile = match serde_json::from_slice(&bytes) {
        Ok(r) => r,
        Err(err) => {
            warn!("parse dev plugin paths: {err:#}");
            return vec![];
        }
    };
    raw.paths.into_iter().map(PathBuf::from).collect()
}

pub fn read_manifest(path: &PathBuf) -> Result<RegistryEntry> {
    let manifest_path = path.join("manifest.json");
    if !manifest_path.exists() {
        bail!("manifest.json not found in {}", path.display());
    }
    let bytes = std::fs::read(&manifest_path).context("read manifest.json")?;
    let raw: RawManifest = serde_json::from_slice(&bytes).context("parse manifest.json")?;
    Ok(RegistryEntry {
        id: raw.id,
        name: raw.name,
        version: raw.version,
        kind: raw.kind,
        author: raw.author,
        description: raw.description,
        icon: raw.icon,
        default_installed: false,
        removable: true,
        path: path.to_string_lossy().to_string(),
        entry: raw.entry,
        sidebar_label: raw.sidebar_label,
        sidebar_icon: raw.sidebar_icon,
    })
}
