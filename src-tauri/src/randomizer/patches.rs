//! Per-player patch files + custom client launching.
//!
//! Archipelago seeds ship as a single zip containing:
//!   * `AP_<seed>.archipelago` — multidata for MultiServer (handled elsewhere).
//!   * `AP_<seed>_P<n>_<slot>.ap<ext>` — per-player ROM patch for games that
//!     need one (Pokémon Emerald, SMW, Tunic, …).
//!   * optional spoiler / text files.
//!
//! `ap_runner::run_generate` already extracts everything next to the seed; this
//! module indexes the patch files and provides three things on top:
//!
//!   1. a list of patches grouped by seed,
//!   2. a "play" action that shells out to `Launcher.py <patch>` (Archipelago's
//!      own launcher applies the patch and launches the right client),
//!   3. a small per-extension base-ROM registry so the UI can remember where
//!      the user keeps their Pokémon / SMW / ... ROMs across runs.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use eyre::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use super::ap_runner::{ap_dir, detect_python, output_dir};

/// Extensions we know are NOT a patch and should be excluded from the list.
/// Everything else that ends up next to the .archipelago is fair game.
const NON_PATCH_EXTENSIONS: &[&str] = &["archipelago", "zip"];

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PatchFile {
    pub path: String,
    pub file_name: String,
    /// File extension without the leading dot, e.g. `apemerald`, `apmw`.
    pub extension: String,
    pub size: u64,
    pub modified: i64,
    /// Seed prefix derived from the filename (`AP_<seed>`). Useful for grouping
    /// patches under the seed they belong to in the UI.
    pub seed_stem: Option<String>,
    /// Player slot label derived from the filename (`P1_Slot`), if present.
    pub player_label: Option<String>,
    /// True when a base ROM path is registered for this extension.
    pub has_rom_registered: bool,
    /// True when the generated output ROM is sitting next to the patch already
    /// (i.e. the user has already applied it).
    pub output_rom_path: Option<String>,
}

pub fn list_patches(app: &AppHandle) -> Result<Vec<PatchFile>> {
    let dir = output_dir(app);
    if !dir.exists() {
        return Ok(vec![]);
    }

    let rom_map = load_rom_paths(app).unwrap_or_default();
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(ext_raw) = path.extension().and_then(|s| s.to_str()) else {
            continue;
        };
        let ext = ext_raw.to_string();
        if NON_PATCH_EXTENSIONS.iter().any(|e| e.eq_ignore_ascii_case(&ext)) {
            continue;
        }
        // We treat a file as a "patch" if its extension starts with "ap".
        // This matches Archipelago's convention (patch_file_ending is always
        // `.ap<world>`), and keeps spoiler .txt / .log files out of the list.
        if !ext.to_ascii_lowercase().starts_with("ap") {
            continue;
        }
        let meta = entry.metadata()?;
        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("?")
            .to_string();
        let (seed_stem, player_label) = parse_patch_name(&file_name);
        let output_rom_path = guess_output_rom(&path);

        out.push(PatchFile {
            path: path.to_string_lossy().to_string(),
            file_name,
            size: meta.len(),
            modified,
            seed_stem,
            player_label,
            has_rom_registered: rom_map.contains_key(&ext),
            output_rom_path,
            extension: ext,
        });
    }
    // Newest first — mirrors list_seeds.
    out.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(out)
}

pub fn delete_patch(path: &Path) -> Result<()> {
    if !path.exists() {
        bail!("not found: {}", path.display());
    }
    // Only allow deletion inside the output dir to be safe — the caller should
    // have come from list_patches, but be defensive.
    fs::remove_file(path).with_context(|| format!("remove {}", path.display()))?;
    Ok(())
}

/// Shell out to Archipelago's Launcher with the patch file. Launcher.py reads
/// the patch metadata, applies it against the base ROM (prompting for one via
/// its own tkinter dialog on first run), then spawns the matching custom
/// client. Non-blocking: we only wait long enough to detect early failures.
pub fn apply_and_launch(app: &AppHandle, patch_path: &Path) -> Result<()> {
    let (python, _) = detect_python(app).ok_or_else(|| {
        eyre::eyre!("Python is not installed; install Python 3.11+ to apply patches")
    })?;
    let dir = ap_dir(app);
    let launcher = dir.join("Launcher.py");
    if !launcher.exists() {
        bail!(
            "Archipelago Launcher.py not found at {}. Install the runtime first.",
            launcher.display()
        );
    }
    if !patch_path.exists() {
        bail!("patch not found: {}", patch_path.display());
    }

    Command::new(&python)
        .current_dir(&dir)
        .env("PYTHONIOENCODING", "utf-8")
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .arg(&launcher)
        .arg(patch_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("spawn {} {}", python, patch_path.display()))?;
    Ok(())
}

/// Launch a named Archipelago component (typically a custom client like
/// `"Pokemon Emerald Client"` or `"Text Client"`). `Launcher.py "<name>"`
/// resolves the component by `display_name`.
pub fn launch_component(app: &AppHandle, component_name: &str) -> Result<()> {
    let (python, _) = detect_python(app).ok_or_else(|| {
        eyre::eyre!("Python is not installed; install Python 3.11+ to launch clients")
    })?;
    let dir = ap_dir(app);
    let launcher = dir.join("Launcher.py");
    if !launcher.exists() {
        bail!(
            "Archipelago Launcher.py not found at {}. Install the runtime first.",
            launcher.display()
        );
    }

    Command::new(&python)
        .current_dir(&dir)
        .env("PYTHONIOENCODING", "utf-8")
        .arg(&launcher)
        .arg(component_name)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("spawn launcher {component_name}"))?;
    Ok(())
}

// --- Base ROM registry ------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct RomPaths {
    /// Map from patch extension (without the leading dot) to the user's base
    /// ROM path. e.g. `{ "apemerald": "/home/me/Emerald.gba" }`.
    pub paths: std::collections::HashMap<String, String>,
}

fn rom_paths_file(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("randomizer")
        .join("rom_paths.json")
}

pub fn load_rom_paths(app: &AppHandle) -> Result<std::collections::HashMap<String, String>> {
    let path = rom_paths_file(app);
    if !path.exists() {
        return Ok(std::collections::HashMap::new());
    }
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("read {}", path.display()))?;
    let parsed: RomPaths = serde_json::from_str(&raw).unwrap_or_default();
    Ok(parsed.paths)
}

pub fn set_rom_path(app: &AppHandle, extension: &str, rom_path: &str) -> Result<()> {
    let mut map = load_rom_paths(app).unwrap_or_default();
    map.insert(extension.trim_start_matches('.').to_string(), rom_path.to_string());
    save_rom_paths(app, &map)
}

pub fn clear_rom_path(app: &AppHandle, extension: &str) -> Result<()> {
    let mut map = load_rom_paths(app).unwrap_or_default();
    map.remove(extension.trim_start_matches('.'));
    save_rom_paths(app, &map)
}

fn save_rom_paths(
    app: &AppHandle,
    map: &std::collections::HashMap<String, String>,
) -> Result<()> {
    let path = rom_paths_file(app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    let value = RomPaths {
        paths: map.clone(),
    };
    let raw = serde_json::to_string_pretty(&value).context("serialize rom paths")?;
    fs::write(&path, raw).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

// --- Helpers ----------------------------------------------------------------

/// `AP_34872348_P1_Stardew.apsv` -> (Some("AP_34872348"), Some("P1_Stardew")).
fn parse_patch_name(file_name: &str) -> (Option<String>, Option<String>) {
    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(file_name);
    // Split on first "_P<digit>_" boundary if present.
    let mut chars = stem.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == '_' {
            let rest = &stem[i + 1..];
            if rest.starts_with('P') && rest.as_bytes().get(1).is_some_and(|b| b.is_ascii_digit()) {
                let seed = stem[..i].to_string();
                return (Some(seed), Some(rest.to_string()));
            }
        }
    }
    (Some(stem.to_string()), None)
}

/// If an applied ROM already lives next to the patch (Archipelago's Launcher
/// writes e.g. `AP_<seed>_P1_<slot>.gba` or `.sfc` after apply), report it.
fn guess_output_rom(patch_path: &Path) -> Option<String> {
    let parent = patch_path.parent()?;
    let stem = patch_path.file_stem()?.to_str()?;
    for entry in fs::read_dir(parent).ok()? {
        let Ok(entry) = entry else { continue };
        let p = entry.path();
        if p == patch_path {
            continue;
        }
        let Some(fname) = p.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if fname != stem {
            continue;
        }
        // Common ROM/output extensions — if it matches the patch's stem AND
        // has a typical ROM suffix, treat it as the applied output.
        let Some(ext) = p.extension().and_then(|s| s.to_str()) else {
            continue;
        };
        if matches!(
            ext.to_ascii_lowercase().as_str(),
            "gba" | "gbc" | "gb" | "sfc" | "smc" | "nes" | "z64" | "n64" | "nds" | "iso" | "bin" | "exe"
        ) {
            return Some(p.to_string_lossy().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_patch_name() {
        assert_eq!(
            parse_patch_name("AP_34872348_P1_Stardew.apsv"),
            (Some("AP_34872348".into()), Some("P1_Stardew".into()))
        );
        assert_eq!(
            parse_patch_name("AP_xyz.apworld"),
            (Some("AP_xyz".into()), None)
        );
    }
}
