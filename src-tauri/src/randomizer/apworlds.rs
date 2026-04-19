//! Custom .apworld management.
//!
//! Archipelago's loader (`worlds/__init__.py`) auto-discovers `.apworld` zips
//! and folder worlds dropped into `custom_worlds/` next to the bundled runtime.
//! This module lets the UI:
//!   * install a `.apworld` (from a path or from raw bytes / base64 when a
//!     browser drag-drop gave us a File instead of a filesystem path),
//!   * list what's currently installed,
//!   * remove one,
//!   * re-run the Python schema extractor for the new worlds so the catalog
//!     picks them up.

use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use base64::Engine;
use eyre::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use zip::ZipArchive;

use super::ap_runner::{ap_dir, detect_python, sanitize_python_env};
use super::schema::user_schemas_dir;

/// Archipelago schema-extractor helper, embedded into the binary at build time
/// so release builds don't need `scripts/` on disk. Materialised next to the
/// runtime on every `refresh_schemas` call (mirrors the apply_patch.py flow
/// in patches.rs).
const EXTRACT_AP_SCHEMAS_PY: &str = include_str!("../../../scripts/extract_ap_schemas.py");

/// Where Archipelago looks for additional worlds.
pub fn custom_worlds_dir(app: &AppHandle) -> PathBuf {
    ap_dir(app).join("custom_worlds")
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CustomApworld {
    /// Filename as stored on disk (e.g. `myworld.apworld`).
    pub file_name: String,
    pub path: String,
    pub size: u64,
    /// Python package name read from the zip's top-level directory
    /// (e.g. `worlds/myworld/` -> `myworld`). This is the "world id".
    pub world_id: Option<String>,
    /// Human-readable game name from `archipelago.json` manifest if present.
    pub display_name: Option<String>,
    /// World version from the manifest, if present.
    pub world_version: Option<String>,
    /// Did the last schema refresh produce a schema JSON for this world?
    pub has_schema: bool,
}

pub fn list_custom_apworlds(app: &AppHandle) -> Result<Vec<CustomApworld>> {
    let dir = custom_worlds_dir(app);
    if !dir.exists() {
        return Ok(vec![]);
    }

    let schemas_dir = user_schemas_dir(app);
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if !path.is_file() || !name.ends_with(".apworld") {
            continue;
        }
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        let (world_id, display_name, world_version) =
            inspect_apworld(&path).unwrap_or((None, None, None));
        let has_schema = world_id
            .as_ref()
            .map(|id| schemas_dir.join(format!("{id}.json")).exists())
            .unwrap_or(false);
        out.push(CustomApworld {
            file_name: name,
            path: path.to_string_lossy().to_string(),
            size,
            world_id,
            display_name,
            world_version,
            has_schema,
        });
    }
    out.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    Ok(out)
}

/// Copy a `.apworld` file into the custom_worlds dir. Returns the installed record.
pub fn install_from_path(app: &AppHandle, src_path: &Path) -> Result<CustomApworld> {
    if !src_path.exists() {
        bail!("file does not exist: {}", src_path.display());
    }
    let name = src_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| eyre::eyre!("path has no filename: {}", src_path.display()))?;
    if !name.ends_with(".apworld") {
        bail!("expected a .apworld file, got: {name}");
    }

    // Validate before copying so we fail fast.
    let (world_id, _, _) =
        inspect_apworld(src_path).with_context(|| format!("validate {}", src_path.display()))?;

    let dir = custom_worlds_dir(app);
    fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    let dest = dir.join(name);
    fs::copy(src_path, &dest).with_context(|| format!("copy to {}", dest.display()))?;

    Ok(CustomApworld {
        file_name: name.to_string(),
        path: dest.to_string_lossy().to_string(),
        size: fs::metadata(&dest).map(|m| m.len()).unwrap_or(0),
        world_id: world_id.clone(),
        display_name: None,
        world_version: None,
        has_schema: world_id
            .map(|id| user_schemas_dir(app).join(format!("{id}.json")).exists())
            .unwrap_or(false),
    })
}

/// Write a .apworld uploaded as raw bytes (base64). Used when the drag source
/// is an HTML `File` object rather than an OS path.
pub fn install_from_bytes(
    app: &AppHandle,
    file_name: &str,
    bytes_b64: &str,
) -> Result<CustomApworld> {
    if !file_name.ends_with(".apworld") {
        bail!("expected a .apworld filename, got: {file_name}");
    }
    let sanitized = sanitize_filename(file_name);
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(bytes_b64.trim())
        .context("decode base64")?;

    // Write to a temp file first so we can validate without leaving a bad
    // zip in custom_worlds/.
    let tmp = tempfile::Builder::new()
        .prefix("zephyr-apworld-")
        .suffix(".apworld")
        .tempfile()
        .context("create temp file")?;
    tmp.as_file().write_all(&bytes).context("write temp")?;
    tmp.as_file().sync_all().ok();
    let tmp_path = tmp.path().to_path_buf();
    let (world_id, _, _) = inspect_apworld(&tmp_path).context("validate apworld")?;

    let dir = custom_worlds_dir(app);
    fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    let dest = dir.join(&sanitized);
    // Atomic-ish: persist moves the tempfile onto dest when possible.
    tmp.persist(&dest)
        .map_err(|e| eyre::eyre!("persist to {}: {e}", dest.display()))?;

    Ok(CustomApworld {
        file_name: sanitized,
        path: dest.to_string_lossy().to_string(),
        size: fs::metadata(&dest).map(|m| m.len()).unwrap_or(0),
        world_id: world_id.clone(),
        display_name: None,
        world_version: None,
        has_schema: world_id
            .map(|id| user_schemas_dir(app).join(format!("{id}.json")).exists())
            .unwrap_or(false),
    })
}

pub fn remove_apworld(app: &AppHandle, file_name: &str) -> Result<()> {
    let dir = custom_worlds_dir(app);
    let target = dir.join(sanitize_filename(file_name));
    if !target.exists() {
        bail!("not found: {}", target.display());
    }
    // Only allow deletion of files inside custom_worlds/.
    let canonical = target.canonicalize().unwrap_or(target.clone());
    let dir_canonical = dir.canonicalize().unwrap_or(dir.clone());
    if !canonical.starts_with(&dir_canonical) {
        bail!("refusing to delete outside custom_worlds");
    }

    // Figure out world_id so we can prune the generated schema.
    let world_id = inspect_apworld(&target)
        .ok()
        .and_then(|(id, _, _)| id);

    fs::remove_file(&target).with_context(|| format!("remove {}", target.display()))?;

    if let Some(id) = world_id {
        let schema_path = user_schemas_dir(app).join(format!("{id}.json"));
        if schema_path.exists() {
            let _ = fs::remove_file(&schema_path);
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RefreshResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub out_dir: String,
}

/// Materialise the embedded extractor next to the runtime and run
/// `python <helper> --runtime <ap_dir> --out-dir <user_schemas_dir>` so
/// schemas for freshly installed custom apworlds land in the overlay dir.
pub fn refresh_schemas(app: &AppHandle) -> Result<RefreshResult> {
    let (python, _) = detect_python(app).ok_or_else(|| {
        eyre::eyre!("Python is not installed; install Python 3.11+ to extract schemas")
    })?;

    let out_dir = user_schemas_dir(app);
    fs::create_dir_all(&out_dir).with_context(|| format!("create {}", out_dir.display()))?;

    let runtime = ap_dir(app);
    if !runtime.join("Generate.py").exists() {
        bail!(
            "Archipelago runtime not installed at {}. Install it from the randomizer page first.",
            runtime.display()
        );
    }

    let helper = runtime.join(".zephyr_extract_schemas.py");
    fs::write(&helper, EXTRACT_AP_SCHEMAS_PY)
        .with_context(|| format!("write {}", helper.display()))?;

    let mut cmd = Command::new(&python);
    sanitize_python_env(&mut cmd);
    let out = cmd
        .current_dir(&runtime)
        .env("PYTHONIOENCODING", "utf-8")
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .arg(&helper)
        .arg("--runtime")
        .arg(&runtime)
        .arg("--out-dir")
        .arg(&out_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("run extract_ap_schemas.py")?;

    Ok(RefreshResult {
        success: out.status.success(),
        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
        out_dir: out_dir.to_string_lossy().to_string(),
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn sanitize_filename(name: &str) -> String {
    let trimmed = name.trim();
    // Strip any directory components.
    let base = Path::new(trimmed)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(trimmed);
    crate::util::fs::sanitize_filename_chars(base, &['.', ' ', '(', ')'])
}

/// Peek inside a `.apworld` zip and pull out (world_id, display_name, world_version).
/// Fails only when the zip is unreadable; missing fields become `None`.
fn inspect_apworld(
    path: &Path,
) -> Result<(Option<String>, Option<String>, Option<String>)> {
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut zip = ZipArchive::new(file).context("not a valid zip")?;

    // Find the top-level directory name (worlds/<id>/ layout, or <id>/).
    let mut world_id: Option<String> = None;
    let mut manifest: Option<String> = None;
    let mut has_init = false;

    for i in 0..zip.len() {
        let entry = zip.by_index(i)?;
        let name = entry.name().to_string();
        let parts: Vec<&str> = name.split('/').collect();
        // Candidate top dir is either parts[0] or parts[1] for old zip layouts
        // that wrap inside `worlds/`.
        let top = if parts.len() >= 2 && parts[0] == "worlds" {
            parts[1]
        } else {
            parts[0]
        };
        if world_id.is_none() && !top.is_empty() {
            world_id = Some(top.to_string());
        }
        if name.ends_with("__init__.py") || name.ends_with("__init__.pyc") {
            has_init = true;
        }
        if name.ends_with("archipelago.json") && manifest.is_none() {
            drop(entry);
            let mut e = zip.by_index(i)?;
            let mut s = String::new();
            e.read_to_string(&mut s).ok();
            manifest = Some(s);
        }
    }

    if !has_init {
        bail!("missing __init__.py — not a valid apworld");
    }

    let (display_name, world_version) = if let Some(json) = manifest.as_ref() {
        match serde_json::from_str::<serde_json::Value>(json) {
            Ok(v) => (
                v.get("game").and_then(|g| g.as_str()).map(|s| s.to_string()),
                v.get("world_version")
                    .and_then(|g| g.as_str())
                    .map(|s| s.to_string()),
            ),
            Err(_) => (None, None),
        }
    } else {
        (None, None)
    };

    Ok((world_id, display_name, world_version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_strips_paths() {
        assert_eq!(sanitize_filename("../evil.apworld"), "evil.apworld");
        assert_eq!(sanitize_filename("/abs/evil.apworld"), "evil.apworld");
        assert_eq!(sanitize_filename("good.apworld"), "good.apworld");
        assert_eq!(sanitize_filename("we;ird/name.apworld"), "name.apworld");
    }
}
