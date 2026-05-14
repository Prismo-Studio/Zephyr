use std::{fs, path::PathBuf};

use eyre::{bail, Context, Result};
use tauri::AppHandle;
use tracing::info;

use super::RegistryEntry;
use crate::{constants::PLUGIN_REGISTRY_RAW_BASE, state::ManagerExt, util};

fn plugins_root() -> PathBuf {
    util::path::default_app_data_dir().join("plugins")
}

pub fn plugin_dir(id: &str) -> PathBuf {
    plugins_root().join(id)
}

fn theme_filename(entry: &RegistryEntry) -> &str {
    entry.entry.as_deref().unwrap_or("theme.css")
}

pub fn theme_path(entry: &RegistryEntry) -> PathBuf {
    plugin_dir(&entry.id).join(theme_filename(entry))
}

pub async fn install_theme(app: &AppHandle, entry: &RegistryEntry) -> Result<String> {
    let asset = theme_filename(entry);
    let url = format!("{}{}/{}", PLUGIN_REGISTRY_RAW_BASE, entry.path, asset);
    info!("downloading theme {} from {}", entry.id, url);

    let css = app
        .http()
        .get(&url)
        .send()
        .await
        .context("download theme")?
        .error_for_status()
        .context("theme download returned non-2xx")?
        .text()
        .await
        .context("read theme body")?;

    let dir = plugin_dir(&entry.id);
    fs::create_dir_all(&dir).context("create plugin dir")?;
    fs::write(dir.join(asset), &css).context("write theme css")?;

    Ok(css)
}

pub fn read_theme_css(entry: &RegistryEntry) -> Result<String> {
    let path = theme_path(entry);
    if !path.exists() {
        bail!("theme {} is not installed on disk", entry.id);
    }
    fs::read_to_string(path).context("read theme css")
}

pub fn read_dev_theme(dev: &super::dev::DevPlugin) -> Result<String> {
    let entry = &dev.manifest;
    let asset = entry.entry.as_deref().unwrap_or("theme.css");
    let path = dev.path.join(asset);
    if !path.exists() {
        bail!("dev theme asset missing: {}", path.display());
    }
    fs::read_to_string(path).context("read dev theme css")
}

pub fn uninstall(id: &str) -> Result<()> {
    let dir = plugin_dir(id);
    if dir.exists() {
        fs::remove_dir_all(&dir).context("delete plugin dir")?;
    }
    Ok(())
}
