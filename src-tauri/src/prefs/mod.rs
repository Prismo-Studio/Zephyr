use std::{
    collections::{HashMap, HashSet},
    fs,
    ops::Deref,
    path::{Path, PathBuf},
};

use eyre::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tracing::{debug, info, warn};

use crate::{
    db::{self, Db},
    game::{self, platform::Platform},
    logger, plugins,
    profile::launch::LaunchMode,
    randomizer,
    state::ManagerExt,
    util::{
        self,
        error::IoResultExt,
        fs::{Overwrite, UseLinks},
        window::WindowExt,
    },
};

pub mod commands;
pub mod pointer;

#[cfg(test)]
mod tests;

#[derive(Serialize, Deserialize, Clone, Debug, Eq)]
#[serde(transparent)]
pub struct DirPref {
    value: PathBuf,
    #[serde(skip)]
    keep_files: Vec<&'static str>,
}

impl DirPref {
    fn new(value: PathBuf) -> Self {
        Self {
            value,
            keep_files: Vec::new(),
        }
    }

    pub fn get(&self) -> &Path {
        &self.value
    }

    pub fn set(&mut self, new_value: PathBuf) -> Result<bool> {
        if self.value == new_value {
            return Ok(false);
        }

        ensure!(new_value.is_dir(), "new value is not a directory");
        ensure!(
            !new_value.starts_with(&self.value),
            "value cannot be a subdirectory of the current directory"
        );
        ensure!(
            new_value.read_dir()?.next().is_none(),
            "new directory is not empty"
        );

        move_contents(&self.value, &new_value, &self.keep_files)?;

        // remove only if empty, the kept files stay behind
        fs::remove_dir(&self.value).ok();

        self.value = new_value;

        Ok(true)
    }
}

/// Moves every entry of `from` into `to`, except for the entries named in `keep`.
///
/// The directory itself is never renamed. The data directory holds the SQLite
/// database and the log file, both of which the running app keeps open; renaming
/// their parent takes the open files along on unix, so every later write - the one
/// that persists the new directory included - would land in a database that is
/// never opened again.
///
/// A missing `from` is not an error: the previously configured directory may have
/// been deleted while the app wasn't running.
///
/// Whatever was already moved is moved back when an entry fails, so a rejected
/// change leaves both directories the way they were and can be retried.
fn move_contents(from: &Path, to: &Path, keep: &[&'static str]) -> Result<()> {
    fs::create_dir_all(to).fs_context("creating new directory", to)?;

    if !from.exists() {
        warn!(
            "previous directory {} does not exist, nothing to move",
            from.display()
        );

        return Ok(());
    }

    info!("moving {} -> {}", from.display(), to.display());

    // Read the whole listing before touching anything. Moving entries out of a
    // directory while its iterator is still open lets the platform skip or repeat
    // entries, which would silently leave files behind. Sorting on top of that
    // keeps an interrupted move reproducible.
    let mut entries = from
        .read_dir()
        .fs_context("reading old directory", from)?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<std::io::Result<Vec<_>>>()
        .context("failed to read file in old directory")?;
    entries.sort();

    let mut moved = Vec::new();

    for file_name in entries {
        if keep.iter().any(|file| file_name == *file) {
            debug!("keeping {}", file_name.to_string_lossy());
            continue;
        }

        if let Err(err) = move_entry(&from.join(&file_name), &to.join(&file_name)) {
            warn!(
                "move failed, putting back what was already moved: {:#}",
                err
            );

            for file_name in moved {
                move_entry(&to.join(&file_name), &from.join(&file_name))
                    .unwrap_or_else(|err| warn!("failed to move {:?} back: {:#}", file_name, err));
            }

            return Err(err);
        }

        moved.push(file_name);
    }

    Ok(())
}

fn move_entry(old_path: &Path, new_path: &Path) -> Result<()> {
    // a rename is instant within a volume, but fails across volumes
    if fs::rename(old_path, new_path).is_ok() {
        return Ok(());
    }

    if old_path.is_dir() {
        debug!("copying dir {:?} -> {:?}", old_path, new_path);

        util::fs::copy_dir(old_path, new_path, Overwrite::Yes, UseLinks::No)
            .context("failed to copy subdirectory")?;
        fs::remove_dir_all(old_path).fs_context("removing old subdirectory", old_path)?;
    } else {
        debug!("copying file {:?} -> {:?}", old_path, new_path);

        fs::copy(old_path, new_path).fs_context("copying file", new_path)?;
        fs::remove_file(old_path).fs_context("removing old file", old_path)?;
    }

    Ok(())
}

impl AsRef<Path> for DirPref {
    fn as_ref(&self) -> &Path {
        self.get()
    }
}

impl Deref for DirPref {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl PartialEq for DirPref {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl From<PathBuf> for DirPref {
    fn from(value: PathBuf) -> Self {
        Self::new(value)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct Prefs {
    pub data_dir: DirPref,
    pub cache_dir: DirPref,

    pub fetch_mods_automatically: bool,
    pub zoom_factor: f32,
    pub dpi_scale: f32,
    pub pull_before_launch: bool,
    pub language: String,
    pub gamepad_enabled: bool,

    pub game_prefs: HashMap<String, GamePrefs>,

    #[serde(default)]
    pub disabled_plugins: HashSet<String>,

    #[serde(default)]
    pub installed_plugins: HashSet<String>,

    /// Directories that were unusable on startup and got reset to their default.
    /// Sent to the frontend so it can tell the user; never read back in.
    #[serde(skip_deserializing, skip_serializing_if = "Vec::is_empty")]
    pub dir_fallbacks: Vec<DirFallback>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DirFallback {
    /// Name of the pref as the frontend knows it, `dataDir` or `cacheDir`.
    pub field: &'static str,
    pub configured: PathBuf,
    pub fallback: PathBuf,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct GamePrefs {
    pub dir_override: Option<PathBuf>,
    pub custom_args: Option<Vec<String>>,
    pub custom_args_enabled: bool,
    pub launch_mode: LaunchMode,
    pub platform: Option<Platform>,
}

/// Files and directories that are always addressed inside the *default* data
/// directory, whatever `data_dir` is set to. They have to stay behind when it
/// moves, otherwise the app would look for them in a place they no longer are.
fn pinned_files() -> Vec<&'static str> {
    let mut files = vec![
        logger::FILE_NAME,
        db::FILE_NAME,
        db::SHM_FILE_NAME,
        db::WAL_FILE_NAME,
        game::CACHE_FILE_NAME,
        randomizer::DATA_DIR_NAME,
    ];

    files.extend_from_slice(plugins::DATA_FILE_NAMES);
    files
}

fn default_cache_dir() -> PathBuf {
    util::path::default_app_data_dir().join("cache")
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            data_dir: DirPref {
                value: util::path::default_app_data_dir(),
                keep_files: pinned_files(),
            },

            cache_dir: DirPref::new(default_cache_dir()),

            fetch_mods_automatically: true,
            pull_before_launch: true,

            zoom_factor: 1.0,
            dpi_scale: 1.1,
            language: "en".to_string(),
            gamepad_enabled: false,

            game_prefs: HashMap::new(),

            disabled_plugins: HashSet::new(),
            installed_plugins: HashSet::new(),

            dir_fallbacks: Vec::new(),
        }
    }
}

impl Prefs {
    pub fn init(&mut self, db: &Db, app: &AppHandle) -> Result<()> {
        self.data_dir.keep_files = pinned_files();

        // The database this was just read from lives in the default data dir, so
        // it can go stale if the data dir was ever moved. The pointer file cannot,
        // which makes it the authority on where the directories are.
        let pointer = pointer::read();

        if let Some(data_dir) = pointer.data_dir {
            self.data_dir.value = data_dir;
        }

        if let Some(cache_dir) = pointer.cache_dir {
            self.cache_dir.value = cache_dir;
        }

        self.resolve_dirs();
        reclaim_pinned_entries(
            &self.data_dir.value,
            &util::path::default_app_data_dir(),
            &self.data_dir.keep_files,
        );

        let window = app.get_webview_window("main").unwrap();
        let effective_zoom = self.zoom_factor as f64 * self.dpi_scale as f64;
        window.zoom(effective_zoom).ok();

        self.save(db)?;

        Ok(())
    }

    /// Makes sure both directories are usable, falling back to the defaults for
    /// the ones that aren't - a configured folder may sit on a drive that isn't
    /// plugged in anymore. Every fallback is recorded for the frontend to show.
    fn resolve_dirs(&mut self) {
        self.dir_fallbacks.clear();
        self.dir_fallbacks.extend(ensure_dir(
            &mut self.data_dir.value,
            "dataDir",
            util::path::default_app_data_dir(),
        ));
        self.dir_fallbacks.extend(ensure_dir(
            &mut self.cache_dir.value,
            "cacheDir",
            default_cache_dir(),
        ));
    }

    fn save(&self, db: &Db) -> Result<()> {
        db.save_prefs(self)?;

        // Best effort: the database is still the primary store, the pointer only
        // has to survive a data dir move.
        pointer::write(&pointer::DirPointer {
            data_dir: Some(self.data_dir.value.clone()),
            cache_dir: Some(self.cache_dir.value.clone()),
        })
        .unwrap_or_else(|err| warn!("failed to save directory pointer: {:#}", err));

        Ok(())
    }

    pub fn save_to_db(&self, db: &Db) -> Result<()> {
        self.save(db)
    }

    fn set(&mut self, value: Self, app: &AppHandle) -> Result<()> {
        let result = self.apply(value, app);

        // Moving a directory cannot be undone, so persist whatever was applied
        // even when a later step failed - otherwise the files would sit in one
        // place while the stored paths still point at the other one.
        let saved = self.save(app.db()).context("failed to save prefs");

        result.and(saved)
    }

    fn apply(&mut self, value: Self, app: &AppHandle) -> Result<()> {
        let mut game_prefs = value.game_prefs;
        validate_game_prefs(&mut game_prefs)?;
        self.game_prefs = game_prefs;

        let old_data_dir = self.data_dir.value.clone();
        let old_cache_dir = self.cache_dir.value.clone();

        let data_dir_changed = self.data_dir.set(value.data_dir.value)?;

        let mut new_cache_dir = value.cache_dir.value;
        if data_dir_changed && new_cache_dir == old_cache_dir {
            // The cache lives inside the data directory by default, in which case
            // it was just moved along with it and only needs to be re-pointed.
            if let Ok(relative) = old_cache_dir.strip_prefix(&old_data_dir) {
                new_cache_dir = self.data_dir.join(relative);
                self.cache_dir.value = new_cache_dir.clone();

                info!(
                    "cache directory follows the data directory to {}",
                    new_cache_dir.display()
                );
            }
        }

        self.cache_dir.set(new_cache_dir)?;

        if data_dir_changed {
            // the profiles moved with the data dir, point them at their new home
            let mut manager = app.lock_manager();

            let mut path = self.data_dir.to_path_buf();
            for (key, game) in &mut manager.games {
                path.push(&*key.slug);

                game.path = path.clone();

                path.push("profiles");

                for profile in &mut game.profiles {
                    profile.path = path.join(&profile.name);
                }

                path.pop();
                path.pop();
            }

            manager.save_all(app)?;
        }

        // a directory that was reset on startup is no longer wrong once it's set
        let cache_dir_changed = self.cache_dir.value != old_cache_dir;
        self.dir_fallbacks.retain(|fallback| match fallback.field {
            "dataDir" => !data_dir_changed,
            _ => !cache_dir_changed,
        });

        let new_dpi = value.dpi_scale.clamp(0.5, 2.0);
        if self.zoom_factor != value.zoom_factor || self.dpi_scale != new_dpi {
            let effective_zoom = value.zoom_factor as f64 * new_dpi as f64;
            let window = app.get_webview_window("main").unwrap();
            window
                .zoom(effective_zoom)
                .context("failed to set zoom level")?;
        }
        self.zoom_factor = value.zoom_factor;
        self.dpi_scale = new_dpi;
        self.language = value.language;

        self.fetch_mods_automatically = value.fetch_mods_automatically;
        self.pull_before_launch = value.pull_before_launch;
        self.gamepad_enabled = value.gamepad_enabled;
        self.disabled_plugins = value.disabled_plugins;
        self.installed_plugins = value.installed_plugins;

        Ok(())
    }

    pub fn cache_dir(&self) -> PathBuf {
        self.cache_dir.to_path_buf()
    }
}

/// Brings back pinned entries that an earlier version left behind in a relocated
/// data directory.
///
/// Before pinning existed, moving the data directory took the plugin and
/// randomizer data along with it while the code that reads them kept looking in
/// the default directory, so plugins dropped off the list and the Archipelago
/// runtime looked uninstalled. Anything found stranded is moved back; an entry
/// that already exists at the default is left alone rather than overwritten,
/// since that copy is the one actually in use.
///
/// Reclaiming a runtime from another volume is a copy and can take a while, but
/// it only ever runs once - afterwards there is nothing left to reclaim.
fn reclaim_pinned_entries(data_dir: &Path, default: &Path, pinned: &[&'static str]) {
    if data_dir == default {
        return;
    }

    for name in pinned {
        let stray = data_dir.join(name);
        let home = default.join(name);

        if !stray.exists() || home.exists() {
            continue;
        }

        info!(
            "{} was left in {} by an older version, moving it back to {}",
            name,
            data_dir.display(),
            default.display()
        );

        fs::create_dir_all(default).ok();
        move_entry(&stray, &home)
            .unwrap_or_else(|err| warn!("failed to move {} back: {:#}", name, err));
    }
}

/// Creates `dir`, or replaces it with `default` when it cannot be created - the
/// drive it lives on may be gone. Returns the fallback that was applied, if any.
fn ensure_dir(dir: &mut PathBuf, field: &'static str, default: PathBuf) -> Option<DirFallback> {
    let Err(err) = fs::create_dir_all(&dir) else {
        return None;
    };

    if *dir == default {
        warn!("failed to create {}: {}", dir.display(), err);
        return None;
    }

    warn!(
        "{} is set to {}, which is unavailable ({}); falling back to {}",
        field,
        dir.display(),
        err,
        default.display()
    );

    let configured = std::mem::replace(dir, default.clone());

    fs::create_dir_all(&dir)
        .unwrap_or_else(|err| warn!("failed to create {}: {}", dir.display(), err));

    Some(DirFallback {
        field,
        configured,
        fallback: default,
    })
}

fn validate_game_prefs(game_prefs: &mut HashMap<String, GamePrefs>) -> Result<()> {
    for (slug, value) in game_prefs {
        let Some(game) = game::from_slug(slug) else {
            warn!("game prefs key {} is invalid", slug);
            continue;
        };

        if let Some(platform) = game.platforms.iter().next() {
            value.platform.get_or_insert(platform);
        } else {
            value.platform = None;
            if let LaunchMode::Launcher = value.launch_mode {
                value.launch_mode = LaunchMode::Direct {
                    instances: 1,
                    interval_secs: 10.0,
                };
            }
        }

        // make sure people don't select the steam library
        if value.dir_override.as_ref().is_some_and(|path| {
            path.file_name().is_some_and(|name| {
                let name = name.to_string_lossy().to_lowercase();
                name.contains("steam") || name.contains("common") || name.contains("steamapps")
            })
        }) {
            value.dir_override = None;
            bail!(
                    "Location override for {} is invalid. Please ensure you selected the game's directory.",
                    slug
                );
        }
    }

    Ok(())
}
