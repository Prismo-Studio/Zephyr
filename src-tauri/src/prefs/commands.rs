use eyre::{anyhow, eyre, Context};
use font_kit::source::SystemSource;
use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Manager, Window};

use super::Prefs;
use crate::{
    constants::CLOUDINARY_AUTO_UPLOAD,
    state::ManagerExt,
    util::{cmd::Result, fs::open_path, window::WindowExt},
};

#[derive(Serialize)]
pub struct CustomBgUpload {
    pub url: String,
    pub kind: String,
}

pub const MAX_IMAGE_BYTES: u64 = 10 * 1024 * 1024;
pub const MAX_VIDEO_BYTES: u64 = 50 * 1024 * 1024;

#[derive(Serialize)]
pub struct CustomBgProbe {
    pub size: u64,
    pub kind: String,
    pub max_image_bytes: u64,
    pub max_video_bytes: u64,
}

#[command]
pub fn probe_custom_background(file_path: String) -> Result<CustomBgProbe> {
    let src = std::path::PathBuf::from(&file_path);
    if !src.exists() {
        return Err(eyre!("file does not exist: {}", src.display()).into());
    }
    let size = std::fs::metadata(&src).context("stat file")?.len();
    let mime = mime_guess::from_path(&src)
        .first_or_octet_stream()
        .essence_str()
        .to_string();
    let kind = if mime.starts_with("video/") {
        "video".to_string()
    } else {
        "image".to_string()
    };
    Ok(CustomBgProbe {
        size,
        kind,
        max_image_bytes: MAX_IMAGE_BYTES,
        max_video_bytes: MAX_VIDEO_BYTES,
    })
}

#[command]
pub async fn upload_custom_background(file_path: String, app: AppHandle) -> Result<CustomBgUpload> {
    let src = std::path::PathBuf::from(&file_path);
    if !src.exists() {
        return Err(eyre!("file does not exist: {}", src.display()).into());
    }

    let file_name = src
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("upload")
        .to_string();

    let mime = mime_guess::from_path(&src)
        .first_or_octet_stream()
        .essence_str()
        .to_string();

    let size = std::fs::metadata(&src).context("stat file")?.len();
    let is_video = mime.starts_with("video/");
    let limit = if is_video { MAX_VIDEO_BYTES } else { MAX_IMAGE_BYTES };
    if size > limit {
        return Err(eyre!("file_too_large").into());
    }

    let bytes = std::fs::read(&src).context("read file")?;
    let part = reqwest::multipart::Part::bytes(bytes)
        .file_name(file_name)
        .mime_str(&mime)
        .map_err(|err: reqwest::Error| anyhow!(err))?;
    let form = reqwest::multipart::Form::new()
        .part("file", part)
        .text("upload_preset", "zephyr_avatars");

    let response = app
        .http()
        .post(CLOUDINARY_AUTO_UPLOAD)
        .multipart(form)
        .send()
        .await?
        .error_for_status()
        .context("cloudinary upload failed")?;

    let json: serde_json::Value = response.json().await?;
    let url = json["secure_url"]
        .as_str()
        .ok_or_else(|| eyre!("no secure_url in response"))?
        .to_string();
    let resource_type = json["resource_type"].as_str().unwrap_or("image");
    let kind = if resource_type == "video" {
        "video".to_string()
    } else {
        "image".to_string()
    };

    Ok(CustomBgUpload { url, kind })
}

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
