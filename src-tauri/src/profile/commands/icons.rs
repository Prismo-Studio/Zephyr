//! Profile icon commands: set from local file, set from URL, upload to Cloudinary, remove.

use std::path::PathBuf;

use eyre::{eyre, Context};
use tauri::{command, AppHandle};
use tracing::warn;

use crate::{state::ManagerExt, util::cmd::Result};

/// Safely removes a local profile icon, validating that the path is under the profile directory.
fn remove_local_icon(icon: &str, profile_path: &std::path::Path) {
    if icon.starts_with("http") {
        return;
    }
    let path = PathBuf::from(icon);
    if let Ok(canonical) = path.canonicalize() {
        if let Ok(profile_canonical) = profile_path.canonicalize() {
            if canonical.starts_with(&profile_canonical) && canonical.exists() {
                std::fs::remove_file(&canonical).ok();
            } else {
                warn!(
                    "refused to delete icon outside profile dir: {}",
                    canonical.display()
                );
            }
        }
    }
}

#[command]
pub fn set_profile_icon(profile_id: i64, image_path: String, app: AppHandle) -> Result<String> {
    let mut manager = app.lock_manager();
    let (_, profile) = manager.profile_by_id_mut(profile_id)?;

    let src = PathBuf::from(&image_path);
    if !src.exists() {
        return Err(eyre!("image file does not exist").into());
    }

    // Load and resize to 128x128
    let img = image::open(&src).context("failed to open image")?;
    let resized = img.resize_to_fill(128, 128, image::imageops::FilterType::Lanczos3);

    // Save as PNG in profile directory
    let icon_path = profile.path.join(".profile-icon.png");
    resized.save(&icon_path).context("failed to save icon")?;

    let icon_str = icon_path.to_string_lossy().to_string();
    profile.icon = Some(icon_str.clone());
    profile.save(&app, true)?;

    Ok(icon_str)
}

#[command]
pub fn set_profile_icon_url(profile_id: i64, url: String, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();
    let (_, profile) = manager.profile_by_id_mut(profile_id)?;

    if let Some(ref icon) = profile.icon {
        remove_local_icon(icon, &profile.path);
    }

    profile.icon = Some(url);
    profile.save(&app, true)?;

    Ok(())
}

#[command]
pub async fn upload_profile_icon(
    profile_id: i64,
    image_path: String,
    app: AppHandle,
) -> Result<String> {
    use base64::prelude::*;

    let src = PathBuf::from(&image_path);
    if !src.exists() {
        return Err(eyre!("image file does not exist").into());
    }

    // Load and resize to 128x128
    let img = image::open(&src).context("failed to open image")?;
    let resized = img.resize_to_fill(128, 128, image::imageops::FilterType::Lanczos3);

    // Encode to PNG bytes
    let mut png_bytes = Vec::new();
    resized
        .write_to(
            &mut std::io::Cursor::new(&mut png_bytes),
            image::ImageFormat::Png,
        )
        .context("failed to encode image")?;

    let base64_data = BASE64_STANDARD.encode(&png_bytes);
    let data_uri = format!("data:image/png;base64,{}", base64_data);

    // Upload to Cloudinary
    let form = reqwest::multipart::Form::new()
        .text("file", data_uri)
        .text("upload_preset", "zephyr_avatars");

    let response = app
        .http()
        .post("https://api.cloudinary.com/v1_1/djmsz47e5/image/upload")
        .multipart(form)
        .send()
        .await?
        .error_for_status()
        .context("cloudinary upload failed")?;

    let json: serde_json::Value = response.json().await?;
    let base_url = json["secure_url"]
        .as_str()
        .ok_or_else(|| eyre!("no secure_url in response"))?
        .to_string();
    // Add cache-bust param so frontend shows the new image
    let url = format!("{}?t={}", base_url, chrono::Utc::now().timestamp());

    // Store URL in profile
    {
        let mut manager = app.lock_manager();
        let (_, profile) = manager.profile_by_id_mut(profile_id)?;

        if let Some(ref icon) = profile.icon {
            remove_local_icon(icon, &profile.path);
        }

        profile.icon = Some(url.clone());
        profile.save(&app, true)?;
    }

    Ok(url)
}

#[command]
pub fn remove_profile_icon(profile_id: i64, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();
    let (_, profile) = manager.profile_by_id_mut(profile_id)?;

    if let Some(ref icon) = profile.icon {
        remove_local_icon(icon, &profile.path);
    }

    profile.icon = None;
    profile.save(&app, true)?;

    Ok(())
}
