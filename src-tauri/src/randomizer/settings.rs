//! User-configurable locations for the Archipelago runtime and the player slot
//! YAMLs.
//!
//! These live in their own `randomizer/settings.json` next to `rom_paths.json`
//! rather than in [`crate::prefs`], for the same reasons the ROM registry does:
//! they are randomizer-only, and they are read from sync path helpers that have
//! no access to the prefs state.
//!
//! The file sits under Tauri's `app_data_dir`, which is fixed at the OS level.
//! That is *not* the same as being independent of the *data folder* pref: the
//! pref defaults to that very directory, and changing it moves the directory's
//! contents to the new location. [`crate::randomizer::DATA_DIR_NAME`] is
//! therefore pinned in [`crate::prefs`] so this file - and the runtime and
//! player slots it points at - stay where the path helpers look for them.
//!
//! ## What happens to existing files when a directory changes
//!
//! The caller decides via `move_existing`:
//!
//! * `true`  — everything under the current directory is moved to the new one
//!   (a rename when both sit on the same volume, a copy-then-delete otherwise).
//!   The new directory has to be empty so nothing can be overwritten.
//! * `false` — the current directory is left untouched and the new one is
//!   adopted as-is. This is the path for "point Zephyr at the Archipelago I
//!   already have installed here". The old location is reported back to the UI
//!   so the files are never silently stranded.
//!
//! If a configured directory later disappears or stops being writable, the
//! resolver logs a warning and falls back to the default location; the UI
//! surfaces the same reason through [`dirs`].

use std::{
    fs,
    path::{Path, PathBuf},
    sync::RwLock,
    time::{Duration, Instant},
};

use eyre::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tracing::warn;

use super::ap_runner;
use crate::util::fs::{copy_dir, Overwrite, UseLinks};

const FILE_NAME: &str = "settings.json";

/// Written into a candidate directory to prove we can actually create files
/// there. Some paths (network shares, `Program Files`) pass `is_dir` and then
/// fail on the first write.
const WRITE_PROBE: &str = ".zephyr-write-test";

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct RandomizerSettings {
    /// Where the Archipelago runtime is installed. `None` = default location.
    pub runtime_dir: Option<PathBuf>,
    /// Where player slot YAMLs are stored. `None` = default location.
    pub players_dir: Option<PathBuf>,
}

/// The settings file is read from path helpers that run on every seed list
/// refresh, so keep the parsed value in memory.
static CACHE: RwLock<Option<RandomizerSettings>> = RwLock::new(None);

fn settings_file(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join(super::DATA_DIR_NAME)
        .join(FILE_NAME)
}

pub fn load(app: &AppHandle) -> RandomizerSettings {
    if let Some(cached) = CACHE.read().ok().and_then(|c| c.clone()) {
        return cached;
    }

    let path = settings_file(app);
    let loaded = match fs::read_to_string(&path) {
        Ok(raw) => serde_json::from_str::<RandomizerSettings>(&raw).unwrap_or_else(|err| {
            warn!(
                "randomizer settings at {} are invalid, using defaults: {err}",
                path.display()
            );
            RandomizerSettings::default()
        }),
        Err(_) => RandomizerSettings::default(),
    };

    if let Ok(mut cache) = CACHE.write() {
        *cache = Some(loaded.clone());
    }
    loaded
}

fn save(app: &AppHandle, settings: &RandomizerSettings) -> Result<()> {
    let path = settings_file(app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    let raw = serde_json::to_string_pretty(settings).context("serialize randomizer settings")?;
    fs::write(&path, raw).with_context(|| format!("write {}", path.display()))?;

    if let Ok(mut cache) = CACHE.write() {
        *cache = Some(settings.clone());
    }
    invalidate_probe_cache();
    Ok(())
}

/// Return the configured directory if it is currently usable, else `None` plus
/// a warning. Callers substitute their default.
///
/// The path helpers that call this run inside listing loops, so the write probe
/// is rate-limited per directory rather than run on every lookup. A drive that
/// goes away mid-session is therefore noticed within [`PROBE_TTL`].
fn usable_override(configured: Option<&PathBuf>, label: &str) -> Option<PathBuf> {
    let dir = configured?;
    if cached_probe(dir) {
        return Some(dir.clone());
    }
    warn!(
        "configured randomizer {label} directory {} is unusable, falling back to the default",
        dir.display()
    );
    None
}

/// Recent write-probe results, so repeated path lookups don't each touch the
/// disk. Holds at most one entry per configured directory.
static PROBE_CACHE: RwLock<Vec<(PathBuf, bool, Instant)>> = RwLock::new(Vec::new());

const PROBE_TTL: Duration = Duration::from_secs(10);

fn cached_probe(dir: &Path) -> bool {
    if let Ok(cache) = PROBE_CACHE.read() {
        if let Some((_, ok, at)) = cache.iter().find(|(path, _, _)| path == dir) {
            if at.elapsed() < PROBE_TTL {
                return *ok;
            }
        }
    }

    let ok = match probe_writable(dir) {
        Ok(()) => true,
        Err(err) => {
            warn!("{} is not usable: {err:#}", dir.display());
            false
        }
    };
    if let Ok(mut cache) = PROBE_CACHE.write() {
        cache.retain(|(path, _, at)| path != dir && at.elapsed() < PROBE_TTL);
        cache.push((dir.to_path_buf(), ok, Instant::now()));
    }
    ok
}

fn invalidate_probe_cache() {
    if let Ok(mut cache) = PROBE_CACHE.write() {
        cache.clear();
    }
}

/// Effective Archipelago runtime install directory.
pub fn runtime_dir_override(app: &AppHandle) -> Option<PathBuf> {
    usable_override(load(app).runtime_dir.as_ref(), "runtime")
}

/// Effective player-slot YAML directory.
pub fn players_dir_override(app: &AppHandle) -> Option<PathBuf> {
    usable_override(load(app).players_dir.as_ref(), "player slots")
}

/// Checks that `dir` exists and that files can be created in it.
///
/// Deliberately does not create the directory. A configured folder that has been
/// deleted or unmounted has to fail here so the caller falls back to the default,
/// instead of being silently recreated empty and reported as usable.
fn probe_writable(dir: &Path) -> Result<()> {
    if !dir.is_dir() {
        bail!("{} does not exist", dir.display());
    }

    let probe = dir.join(WRITE_PROBE);
    fs::write(&probe, b"").with_context(|| format!("write test file in {}", dir.display()))?;
    let _ = fs::remove_file(&probe);
    Ok(())
}

/// Same check for a directory we are about to write to ourselves, creating it
/// when it isn't there yet.
fn prepare_writable(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
    probe_writable(dir)
}

// --- Reporting --------------------------------------------------------------

#[derive(Serialize, Clone, Debug)]
pub struct DirSetting {
    /// What the user picked, or `None` when the default is in use.
    pub configured: Option<String>,
    /// The directory actually in use right now.
    pub effective: String,
    pub default: String,
    pub writable: bool,
    /// Set when a configured directory had to be ignored.
    pub fallback_reason: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct RandomizerDirs {
    pub runtime: DirSetting,
    pub players: DirSetting,
}

fn describe(configured: Option<PathBuf>, default: PathBuf) -> DirSetting {
    let mut fallback_reason = None;
    let effective = match configured.as_ref() {
        Some(dir) => match probe_writable(dir) {
            Ok(()) => dir.clone(),
            Err(err) => {
                fallback_reason = Some(format!("{err:#}"));
                default.clone()
            }
        },
        None => default.clone(),
    };

    DirSetting {
        configured: configured.map(|p| p.to_string_lossy().to_string()),
        // The effective directory is one we write to ourselves, so creating it
        // here is the point rather than a side effect.
        writable: prepare_writable(&effective).is_ok(),
        effective: effective.to_string_lossy().to_string(),
        default: default.to_string_lossy().to_string(),
        fallback_reason,
    }
}

pub fn dirs(app: &AppHandle) -> RandomizerDirs {
    // The UI asks for the truth, not a 10-second-old snapshot.
    invalidate_probe_cache();
    let settings = load(app);
    RandomizerDirs {
        runtime: describe(
            settings.runtime_dir.clone(),
            ap_runner::default_ap_install_dir(app),
        ),
        players: describe(settings.players_dir, ap_runner::default_players_dir(app)),
    }
}

// --- Mutation ---------------------------------------------------------------

/// Outcome of a directory change, so the UI can tell the user where their old
/// files ended up.
#[derive(Serialize, Clone, Debug)]
pub struct DirChange {
    pub dirs: RandomizerDirs,
    /// Previous effective directory, when its contents were left behind.
    pub left_behind: Option<String>,
    pub moved: bool,
}

pub fn set_runtime_dir(
    app: &AppHandle,
    requested: Option<PathBuf>,
    move_existing: bool,
) -> Result<DirChange> {
    let current = ap_runner::ap_install_dir(app);
    let default = ap_runner::default_ap_install_dir(app);
    let outcome = change_dir(current, default, requested, move_existing)?;

    let mut settings = load(app);
    settings.runtime_dir = outcome.store.clone();
    save(app, &settings)?;

    Ok(DirChange {
        dirs: dirs(app),
        left_behind: outcome.left_behind,
        moved: outcome.moved,
    })
}

pub fn set_players_dir(
    app: &AppHandle,
    requested: Option<PathBuf>,
    move_existing: bool,
) -> Result<DirChange> {
    let current = ap_runner::players_dir(app);
    let default = ap_runner::default_players_dir(app);
    let outcome = change_dir(current, default, requested, move_existing)?;

    let mut settings = load(app);
    settings.players_dir = outcome.store.clone();
    save(app, &settings)?;

    Ok(DirChange {
        dirs: dirs(app),
        left_behind: outcome.left_behind,
        moved: outcome.moved,
    })
}

struct ChangeOutcome {
    /// Value to persist: `None` when the target is the default location.
    store: Option<PathBuf>,
    left_behind: Option<String>,
    moved: bool,
}

fn change_dir(
    current: PathBuf,
    default: PathBuf,
    requested: Option<PathBuf>,
    move_existing: bool,
) -> Result<ChangeOutcome> {
    let target = requested.unwrap_or_else(|| default.clone());
    let store = if target == default {
        None
    } else {
        Some(target.clone())
    };

    if target == current {
        return Ok(ChangeOutcome {
            store,
            left_behind: None,
            moved: false,
        });
    }

    if !target.is_absolute() {
        bail!("{} is not an absolute path", target.display());
    }
    if target.starts_with(&current) {
        bail!(
            "{} is inside the current directory; pick a location outside {}",
            target.display(),
            current.display()
        );
    }

    prepare_writable(&target)
        .with_context(|| format!("{} cannot be used as a directory", target.display()))?;

    let has_contents = current
        .read_dir()
        .map(|mut it| it.next().is_some())
        .unwrap_or(false);

    if move_existing && has_contents {
        let target_occupied = target
            .read_dir()
            .map(|mut it| it.next().is_some())
            .unwrap_or(false);
        if target_occupied {
            bail!(
                "{} already contains files; empty it first or choose to keep the existing files where they are",
                target.display()
            );
        }
        move_tree(&current, &target).with_context(|| {
            format!(
                "failed to move {} to {}",
                current.display(),
                target.display()
            )
        })?;
        return Ok(ChangeOutcome {
            store,
            left_behind: None,
            moved: true,
        });
    }

    Ok(ChangeOutcome {
        store,
        left_behind: has_contents.then(|| current.to_string_lossy().to_string()),
        moved: false,
    })
}

/// Move `from` onto `to`, preferring a rename and falling back to a recursive
/// copy when the two are on different volumes.
fn move_tree(from: &Path, to: &Path) -> Result<()> {
    if !from.is_dir() {
        return Ok(());
    }

    // fs::rename needs the destination to not exist on Windows, and `to` was
    // just created (empty) by the writability probe.
    let _ = fs::remove_dir(to);
    if fs::rename(from, to).is_ok() {
        return Ok(());
    }

    fs::create_dir_all(to).with_context(|| format!("create {}", to.display()))?;
    copy_dir(from, to, Overwrite::Yes, UseLinks::No)?;
    fs::remove_dir_all(from).with_context(|| format!("remove {}", from.display()))?;
    Ok(())
}
