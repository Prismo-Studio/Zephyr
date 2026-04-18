//! Download / install / remove the Archipelago runtime.
//!
//! Releases of Zephyr ship without the Archipelago runtime (~hundreds of MB
//! of worlds + assets). This module pulls a zip tarball from a GitHub repo
//! and extracts it into the per-user install location returned by
//! [`super::ap_runner::ap_install_dir`] — for Linux that's
//! `~/.local/share/<bundle-id>/randomizer/archipelago-runtime/`.
//!
//! The dev-tree copy (checked in under `src-tauri/archipelago-runtime`) still
//! takes precedence when present, so `cargo tauri dev` doesn't need a network
//! round-trip.

use std::{
    fs,
    io::{Cursor, Write},
    path::{Path, PathBuf},
};

use eyre::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use zip::ZipArchive;

use super::ap_runner::ap_install_dir;

/// Default source: the source tarball of the runtime repo's main branch.
/// Overridable by the caller (UI can pass a pinned release URL later).
pub const DEFAULT_RUNTIME_URL: &str =
    "https://github.com/Prismo-Studio/randomizer-server/archive/refs/heads/main.zip";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RuntimeStatus {
    pub installed: bool,
    pub path: String,
    pub bytes_on_disk: u64,
    pub world_count: u32,
    pub default_download_url: String,
}

pub fn status(app: &AppHandle) -> RuntimeStatus {
    let install = ap_install_dir(app);
    let installed = install.join("Generate.py").exists();

    // Prefer whichever path `ap_dir` resolves to — dev checkout in dev builds,
    // the install dir in release. That way the UI reflects reality.
    let effective = super::ap_runner::ap_dir(app);
    let effective_installed = effective.join("Generate.py").exists();

    let world_count = count_worlds(&effective).unwrap_or(0);
    let bytes = dir_size(&effective).unwrap_or(0);

    RuntimeStatus {
        installed: installed || effective_installed,
        path: effective.to_string_lossy().to_string(),
        bytes_on_disk: bytes,
        world_count,
        default_download_url: DEFAULT_RUNTIME_URL.to_string(),
    }
}

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "stage", rename_all = "snake_case")]
pub enum ProgressEvent {
    Downloading { received: u64, total: Option<u64> },
    Extracting { entry: String, done: u32, total: u32 },
    Installed { path: String },
    Failed { error: String },
}

/// Download a zip from `url` and extract it into the user install dir.
/// Progress is emitted on the `randomizer://runtime-progress` Tauri event.
pub async fn install(app: &AppHandle, url: Option<String>) -> Result<RuntimeStatus> {
    let url = url.unwrap_or_else(|| DEFAULT_RUNTIME_URL.to_string());
    let install_dir = ap_install_dir(app);

    let emit = |event: ProgressEvent| {
        let _ = app.emit("randomizer://runtime-progress", event);
    };

    // --- Download ---------------------------------------------------------
    let client = reqwest::Client::builder()
        .user_agent("Zephyr/1 (archipelago-runtime-installer)")
        .build()
        .context("build http client")?;

    let resp = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("GET {url}"))?;
    if !resp.status().is_success() {
        bail!("download failed: HTTP {}", resp.status());
    }
    let total = resp.content_length();
    emit(ProgressEvent::Downloading {
        received: 0,
        total,
    });

    // Stream into a temp file so we don't need to hold the whole zip in RAM.
    let tmp_parent = install_dir
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::temp_dir());
    fs::create_dir_all(&tmp_parent).ok();
    let mut tmp = tempfile::Builder::new()
        .prefix("zephyr-ap-runtime-")
        .suffix(".zip")
        .tempfile_in(&tmp_parent)
        .context("create tempfile")?;

    let mut received: u64 = 0;
    let mut stream = resp.bytes_stream();
    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("download chunk")?;
        tmp.as_file_mut().write_all(&chunk).context("write chunk")?;
        received += chunk.len() as u64;
        emit(ProgressEvent::Downloading {
            received,
            total,
        });
    }
    tmp.as_file_mut().sync_all().ok();

    // --- Extract ----------------------------------------------------------
    let bytes = fs::read(tmp.path()).context("read downloaded zip")?;
    let reader = Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader).context("open zip archive")?;

    // Clean a previous install to avoid half-old / half-new runtimes.
    if install_dir.exists() {
        fs::remove_dir_all(&install_dir)
            .with_context(|| format!("wipe old install at {}", install_dir.display()))?;
    }
    fs::create_dir_all(&install_dir)
        .with_context(|| format!("create {}", install_dir.display()))?;

    // GitHub's archive zips wrap everything inside a single top-level folder
    // like `archipelago-runtime-main/`. Detect it so we can strip that prefix
    // and land `Generate.py` at the install-dir root where `ap_dir` expects it.
    let strip_prefix = detect_root_prefix(&mut archive);
    let total_entries = archive.len() as u32;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .with_context(|| format!("read zip entry {i}"))?;
        let raw_name = file.name().to_string();

        // Normalise separators.
        let rel = raw_name.replace('\\', "/");
        let stripped = match &strip_prefix {
            Some(p) => rel.strip_prefix(p).unwrap_or(&rel).to_string(),
            None => rel,
        };
        if stripped.is_empty() {
            continue;
        }

        let out_path = install_dir.join(&stripped);

        if !is_enclosed(&out_path, &install_dir) {
            tracing::warn!(
                "skipping zip entry that escapes install dir: {}",
                stripped
            );
            continue;
        }

        if file.is_dir() || stripped.ends_with('/') {
            fs::create_dir_all(&out_path).ok();
            continue;
        }
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let mut out = fs::File::create(&out_path)
            .with_context(|| format!("create {}", out_path.display()))?;
        std::io::copy(&mut file, &mut out)
            .with_context(|| format!("write {}", out_path.display()))?;

        #[cfg(unix)]
        if let Some(mode) = file.unix_mode() {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&out_path, fs::Permissions::from_mode(mode));
        }

        if i % 25 == 0 || (i as u32 + 1) == total_entries {
            emit(ProgressEvent::Extracting {
                entry: stripped.clone(),
                done: i as u32 + 1,
                total: total_entries,
            });
        }
    }

    let st = status(app);
    if !st.installed {
        let msg = format!(
            "install finished but {} was not written — is the downloaded archive the correct runtime?",
            install_dir.join("Generate.py").display()
        );
        emit(ProgressEvent::Failed { error: msg.clone() });
        bail!(msg);
    }
    emit(ProgressEvent::Installed {
        path: st.path.clone(),
    });
    Ok(st)
}

pub fn remove(app: &AppHandle) -> Result<()> {
    let dir = ap_install_dir(app);
    if dir.exists() {
        fs::remove_dir_all(&dir).with_context(|| format!("remove {}", dir.display()))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// If every entry in the zip starts with the same top-level segment (e.g.
/// `archipelago-runtime-main/`), return that prefix so we can strip it
/// during extraction. Returns `None` if the zip is already flat.
fn detect_root_prefix<R: std::io::Read + std::io::Seek>(archive: &mut ZipArchive<R>) -> Option<String> {
    let mut prefix: Option<String> = None;
    for i in 0..archive.len() {
        let Ok(entry) = archive.by_index(i) else {
            continue;
        };
        let name = entry.name().replace('\\', "/");
        if name.is_empty() {
            continue;
        }
        let top = name.split('/').next().unwrap_or("").to_string();
        if top.is_empty() {
            return None;
        }
        match &prefix {
            None => prefix = Some(top),
            Some(existing) => {
                if existing != &top {
                    return None;
                }
            }
        }
    }
    prefix.map(|p| format!("{p}/"))
}

fn is_enclosed(path: &Path, root: &Path) -> bool {
    let normalised: PathBuf = path
        .components()
        .filter(|c| !matches!(c, std::path::Component::ParentDir))
        .collect();
    normalised.starts_with(root)
}

fn count_worlds(dir: &Path) -> Option<u32> {
    let worlds = dir.join("worlds");
    if !worlds.exists() {
        return None;
    }
    let mut count = 0u32;
    for entry in fs::read_dir(&worlds).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('_') || name.starts_with('.') {
            continue;
        }
        if path.is_dir() && path.join("__init__.py").exists() {
            count += 1;
        } else if name.ends_with(".apworld") {
            count += 1;
        }
    }
    Some(count)
}

fn dir_size(dir: &Path) -> Option<u64> {
    if !dir.exists() {
        return None;
    }
    let mut total = 0u64;
    walk(dir, &mut |p| {
        if let Ok(m) = fs::metadata(p) {
            if m.is_file() {
                total += m.len();
            }
        }
    });
    Some(total)
}

fn walk(dir: &Path, f: &mut dyn FnMut(&Path)) {
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_dir() {
            walk(&p, f);
        } else {
            f(&p);
        }
    }
}
