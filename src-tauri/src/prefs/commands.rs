use eyre::anyhow;
use font_kit::source::SystemSource;
use serde::Deserialize;
use tauri::{command, AppHandle, Manager, Window};

use super::Prefs;
use crate::{
    state::ManagerExt,
    util::{cmd::Result, fs::open_path, window::WindowExt},
};

#[command]
pub fn get_prefs(app: AppHandle) -> Prefs {
    app.lock_prefs().clone()
}

#[command]
pub fn set_prefs(value: Prefs, app: AppHandle) -> Result<()> {
    let mut prefs = app.lock_prefs();
    prefs.set(value, &app)?;
    Ok(())
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Zoom {
    Set { factor: f32 },
    Modify { delta: f32 },
}

#[command]
pub fn zoom_window(value: Zoom, window: Window, app: AppHandle) -> Result<()> {
    let mut prefs = app.lock_prefs();
    prefs.zoom_factor = match value {
        Zoom::Set { factor } => factor,
        Zoom::Modify { delta } => prefs.zoom_factor + delta,
    }
    .clamp(0.5, 1.5);

    window
        .get_webview_window("main")
        .unwrap()
        .zoom(prefs.zoom_factor as f64)
        .map_err(|err| anyhow!(err))?;

    prefs.save(app.db())?;

    Ok(())
}

#[command]
pub fn set_dpi_scale(value: f32, app: AppHandle) -> Result<f32> {
    let mut prefs = app.lock_prefs();
    let new_dpi = value.clamp(0.5, 2.0);
    prefs.dpi_scale = new_dpi;
    let effective_zoom = prefs.zoom_factor as f64 * new_dpi as f64;
    if let Some(window) = app.get_webview_window("main") {
        window
            .zoom(effective_zoom)
            .map_err(|err| anyhow!(err))?;
    }
    prefs.save(app.db())?;
    Ok(new_dpi)
}

#[command]
pub fn get_system_fonts() -> Result<Vec<String>> {
    let fonts = SystemSource::new().all_families().unwrap();

    Ok(fonts)
}

#[command]
pub fn open_dir(path: std::path::PathBuf) -> Result<()> {
	open_path(path)?;
	Ok(())
}
