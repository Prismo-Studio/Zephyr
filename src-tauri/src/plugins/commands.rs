use std::path::PathBuf;

use eyre::{eyre, Result};
use serde::Serialize;
use tauri::{command, AppHandle, Emitter, Manager};

use super::{
    dev::{self, DevPluginState},
    is_built_in_feature,
    registry::PluginRegistryState,
    PluginEntry, PluginType, RegistryEntry,
};
use crate::{state::ManagerExt, util::cmd::Result as CmdResult};

fn collect(app: &AppHandle) -> Vec<PluginEntry> {
    let state = app.state::<PluginRegistryState>();
    let entries = state.entries.lock().unwrap();
    let dev_state = app.state::<DevPluginState>();
    let dev_entries = dev_state.entries.lock().unwrap();
    let prefs = app.lock_prefs();

    let dev_ids: std::collections::HashSet<String> =
        dev_entries.iter().map(|d| d.manifest.id.clone()).collect();

    let mut out: Vec<PluginEntry> = Vec::with_capacity(entries.len() + dev_entries.len());

    for entry in entries.iter() {
        if dev_ids.contains(&entry.id) {
            continue;
        }
        let built_in = is_built_in_feature(entry);
        let installed = prefs.installed_plugins.contains(&entry.id);
        let enabled = (built_in || installed) && !prefs.disabled_plugins.contains(&entry.id);
        out.push(PluginEntry {
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
            dev: false,
            dev_path: String::new(),
            sidebar_label: entry.sidebar_label.clone(),
        });
    }

    for d in dev_entries.iter() {
        let entry = &d.manifest;
        let enabled = !prefs.disabled_plugins.contains(&entry.id);
        out.push(PluginEntry {
            id: entry.id.clone(),
            name: entry.name.clone(),
            description: entry.description.clone(),
            author: entry.author.name.clone(),
            version: entry.version.clone(),
            icon: dev_icon_url(d, entry),
            kind: entry.kind,
            built_in: false,
            removable: true,
            enabled,
            dev: true,
            dev_path: d.path.to_string_lossy().to_string(),
            sidebar_label: entry.sidebar_label.clone(),
        });
    }

    out
}

fn dev_icon_url(d: &super::dev::DevPlugin, entry: &RegistryEntry) -> String {
    let icon = match &entry.icon {
        Some(i) => i,
        None => return super::registry::icon_url(entry),
    };
    if icon.starts_with("http://") || icon.starts_with("https://") || icon.starts_with('/') {
        return icon.clone();
    }
    let path = d.path.join(icon);
    let normalised = path.to_string_lossy().replace('\\', "/");
    format!("file://{}", normalised)
}

pub fn emit_changed(app: &AppHandle) -> Result<()> {
    app.emit("plugins_changed", collect(app))?;
    Ok(())
}

fn find_entry(app: &AppHandle, id: &str) -> Result<RegistryEntry> {
    let dev_state = app.state::<DevPluginState>();
    if let Some(d) = dev_state.entries.lock().unwrap().iter().find(|d| d.manifest.id == id) {
        return Ok(d.manifest.clone());
    }
    let state = app.state::<PluginRegistryState>();
    let entries = state.entries.lock().unwrap();
    entries
        .iter()
        .find(|e| e.id == id)
        .cloned()
        .ok_or_else(|| eyre!("unknown plugin id: {id}"))
}

fn find_dev(app: &AppHandle, id: &str) -> Option<super::dev::DevPlugin> {
    let dev_state = app.state::<DevPluginState>();
    let list = dev_state.entries.lock().unwrap();
    let found = list.iter().find(|d| d.manifest.id == id).cloned();
    found
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
    let dev = find_dev(&app, &id);

    let css = match entry.kind {
        PluginType::Theme => {
            if let Some(d) = &dev {
                super::install::read_dev_theme(d)?
            } else {
                super::install::install_theme(&app, &entry).await?
            }
        }
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
    let dev_state = app.state::<DevPluginState>();
    let entries = state.entries.lock().unwrap();
    let dev_entries = dev_state.entries.lock().unwrap();
    let prefs = app.lock_prefs();

    let dev_ids: std::collections::HashSet<String> =
        dev_entries.iter().map(|d| d.manifest.id.clone()).collect();

    let mut out: Vec<InstalledTheme> = Vec::new();

    for e in entries.iter() {
        if !matches!(e.kind, PluginType::Theme) {
            continue;
        }
        if dev_ids.contains(&e.id) {
            continue;
        }
        if !prefs.installed_plugins.contains(&e.id) {
            continue;
        }
        if let Ok(css) = super::install::read_theme_css(e) {
            out.push(InstalledTheme {
                id: e.id.clone(),
                name: e.name.clone(),
                css,
            });
        }
    }

    for d in dev_entries.iter() {
        if !matches!(d.manifest.kind, PluginType::Theme) {
            continue;
        }
        if !prefs.installed_plugins.contains(&d.manifest.id) {
            continue;
        }
        if let Ok(css) = super::install::read_dev_theme(d) {
            out.push(InstalledTheme {
                id: d.manifest.id.clone(),
                name: d.manifest.name.clone(),
                css,
            });
        }
    }

    out
}

#[command]
pub fn register_local_plugin(path: String, app: AppHandle) -> CmdResult<PluginEntry> {
    let pb = PathBuf::from(&path);
    if !pb.is_dir() {
        return Err(eyre!("not a directory: {}", path).into());
    }

    let manifest = dev::read_manifest(&pb)?;
    let dev_plugin = super::dev::DevPlugin {
        path: pb,
        manifest: manifest.clone(),
    };

    {
        let dev_state = app.state::<DevPluginState>();
        let mut list = dev_state.entries.lock().unwrap();
        list.retain(|d| d.manifest.id != manifest.id);
        list.push(dev_plugin.clone());
        let paths: Vec<_> = list.iter().map(|d| d.path.clone()).collect();
        super::dev::save_paths(&paths);
    }

    if matches!(manifest.kind, PluginType::Theme) {
        let mut prefs = app.lock_prefs();
        prefs.installed_plugins.insert(manifest.id.clone());
        prefs.save_to_db(app.db())?;
    }

    {
        let dev_state = app.state::<DevPluginState>();
        let mut watchers = dev_state.watchers.lock().unwrap();
        watchers.remove(&manifest.id);
        match dev::spawn_watcher(app.clone(), manifest.id.clone(), dev_plugin.path.clone()) {
            Ok(w) => {
                watchers.insert(manifest.id.clone(), w);
            }
            Err(err) => {
                tracing::warn!("dev plugin watcher failed for {}: {:#}", manifest.id, err);
            }
        }
    }

    emit_changed(&app)?;
    let snapshot = collect(&app)
        .into_iter()
        .find(|e| e.id == manifest.id)
        .ok_or_else(|| eyre!("registered plugin disappeared"))?;
    Ok(snapshot)
}

#[command]
pub fn unregister_local_plugin(id: String, app: AppHandle) -> CmdResult<()> {
    {
        let dev_state = app.state::<DevPluginState>();
        let mut list = dev_state.entries.lock().unwrap();
        list.retain(|d| d.manifest.id != id);
        let mut watchers = dev_state.watchers.lock().unwrap();
        watchers.remove(&id);
        let paths: Vec<_> = list.iter().map(|d| d.path.clone()).collect();
        super::dev::save_paths(&paths);
    }
    {
        let mut prefs = app.lock_prefs();
        prefs.installed_plugins.remove(&id);
        prefs.save_to_db(app.db())?;
    }
    emit_changed(&app)?;
    Ok(())
}

#[command]
pub fn get_plugin_ui_url(id: String, app: AppHandle) -> CmdResult<String> {
    let dev = find_dev(&app, &id);
    if let Some(d) = dev {
        for candidate in ["dist/index.html", "ui/index.html"] {
            let path = d.path.join(candidate);
            if path.exists() {
                let normalised = path.to_string_lossy().replace('\\', "/");
                return Ok(format!("file://{}", normalised));
            }
        }
        return Ok(String::new());
    }

    let state = app.state::<PluginRegistryState>();
    let entries = state.entries.lock().unwrap();
    let entry = entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| eyre!("unknown plugin id: {id}"))?;
    Ok(format!(
        "{}{}/dist/index.html",
        crate::constants::PLUGIN_REGISTRY_RAW_BASE,
        entry.path
    ))
}

fn plugin_storage_path(id: &str) -> std::path::PathBuf {
    crate::util::path::default_app_data_dir()
        .join("plugin-storage")
        .join(format!("{id}.json"))
}

#[command]
pub fn plugin_storage_get(id: String) -> CmdResult<serde_json::Value> {
    let path = plugin_storage_path(&id);
    if !path.exists() {
        return Ok(serde_json::Value::Null);
    }
    let bytes = std::fs::read(&path).map_err(|e| eyre!(e))?;
    let value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    Ok(value)
}

#[command]
pub fn plugin_storage_set(id: String, value: serde_json::Value) -> CmdResult<()> {
    let path = plugin_storage_path(&id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| eyre!(e))?;
    }
    let json = serde_json::to_string(&value).map_err(|e| eyre!(e))?;
    std::fs::write(&path, json).map_err(|e| eyre!(e))?;
    Ok(())
}

#[command]
pub fn plugin_open_external(url: String, app: AppHandle) -> CmdResult<()> {
    use tauri_plugin_shell::ShellExt;
    app.shell()
        .open(&url, None)
        .map_err(|e| eyre!(e))?;
    Ok(())
}

fn plugin_files_dir(id: &str) -> std::path::PathBuf {
    crate::util::path::default_app_data_dir()
        .join("plugin-storage")
        .join(id)
        .join("files")
}

fn safe_filename(name: &str) -> Result<String> {
    if name.is_empty() || name.contains('/') || name.contains('\\') || name == "." || name == ".." {
        eyre::bail!("invalid filename");
    }
    Ok(name.to_string())
}

#[command]
pub fn plugin_fs_write_blob(id: String, filename: String, bytes: Vec<u8>) -> CmdResult<String> {
    let safe = safe_filename(&filename)?;
    let dir = plugin_files_dir(&id);
    std::fs::create_dir_all(&dir).map_err(|e| eyre!(e))?;
    let path = dir.join(&safe);
    std::fs::write(&path, &bytes).map_err(|e| eyre!(e))?;
    Ok(path.to_string_lossy().replace('\\', "/"))
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginFileEntry {
    pub name: String,
    pub size: u64,
    pub created_at: u64,
}

#[command]
pub fn plugin_fs_list(id: String, extension: Option<String>) -> CmdResult<Vec<PluginFileEntry>> {
    let dir = plugin_files_dir(&id);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let want_ext = extension.as_deref().map(|e| e.trim_start_matches('.').to_lowercase());

    let mut out: Vec<PluginFileEntry> = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| eyre!(e))? {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if let Some(ref want) = want_ext {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());
            if ext.as_deref() != Some(want.as_str()) {
                continue;
            }
        }
        let Ok(meta) = entry.metadata() else { continue };
        let created_at = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        out.push(PluginFileEntry {
            name,
            size: meta.len(),
            created_at,
        });
    }
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(out)
}

#[command]
pub fn plugin_fs_delete(id: String, filename: String) -> CmdResult<()> {
    let safe = safe_filename(&filename)?;
    let path = plugin_files_dir(&id).join(safe);
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| eyre!(e))?;
    }
    Ok(())
}

#[command]
pub fn plugin_fs_get_url(id: String, filename: String) -> CmdResult<String> {
    let safe = safe_filename(&filename)?;
    let path = plugin_files_dir(&id).join(safe);
    if !path.exists() {
        return Err(eyre!("file not found").into());
    }
    let normalised = path.to_string_lossy().replace('\\', "/");
    Ok(format!("file://{}", normalised))
}

#[command]
pub fn plugin_fs_open_folder(id: String, app: AppHandle) -> CmdResult<()> {
    let dir = plugin_files_dir(&id);
    std::fs::create_dir_all(&dir).map_err(|e| eyre!(e))?;
    let _ = app;
    crate::util::fs::open_path(&dir).map_err(|e| eyre!(e))?;
    Ok(())
}

#[command]
pub fn reload_local_plugin(id: String, app: AppHandle) -> CmdResult<PluginEntry> {
    let path = {
        let dev_state = app.state::<DevPluginState>();
        let list = dev_state.entries.lock().unwrap();
        list.iter()
            .find(|d| d.manifest.id == id)
            .map(|d| d.path.clone())
            .ok_or_else(|| eyre!("plugin not registered: {id}"))?
    };

    let manifest = dev::read_manifest(&path)?;
    {
        let dev_state = app.state::<DevPluginState>();
        let mut list = dev_state.entries.lock().unwrap();
        if let Some(d) = list.iter_mut().find(|d| d.manifest.id == manifest.id) {
            d.manifest = manifest.clone();
        }
    }
    emit_changed(&app)?;
    let snapshot = collect(&app)
        .into_iter()
        .find(|e| e.id == manifest.id)
        .ok_or_else(|| eyre!("plugin not found after reload"))?;
    Ok(snapshot)
}
