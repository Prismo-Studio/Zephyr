//! Per-player patch files + custom client launching.
//!
//! Archipelago seeds ship as a single zip containing:
//!   * `AP_<seed>.archipelago`. Multidata for MultiServer (handled elsewhere).
//!   * `AP_<seed>_P<n>_<slot>.ap<ext>`. Per-player ROM patch for games that
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
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use eyre::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::util::process::CommandExt as _;
use super::ap_runner::{
    ap_dir, detect_python, output_dir, ping_local_port, sanitize_python_env, ServerState,
};

/// Archipelago's default MultiServer port. Used as a fallback when
/// Zephyr's `ServerState` doesn't track a running child (external server,
/// or one started outside the Server tab).
const DEFAULT_AP_PORT: u16 = 38281;

/// Resolve the AP server that a freshly-launched SNIClient should connect
/// to. Priority:
///   1. Zephyr-managed MultiServer (ServerState, started via Server tab).
///   2. Anything listening on 127.0.0.1:38281 (external server, or one
///      started outside Zephyr).
///
/// Returns `(host_port, password)`. SNIClient's `--connect` accepts
/// `host:port` form; no `ws://` prefix needed.
fn resolve_ap_target(app: &AppHandle) -> Option<(String, Option<String>)> {
    use tauri::Manager;
    if let Some(state) = app.try_state::<ServerState>() {
        let st = state.status();
        if let (true, Some(port)) = (st.running, st.port) {
            return Some((
                format!("127.0.0.1:{port}"),
                st.password.filter(|p| !p.is_empty()),
            ));
        }
    }
    if ping_local_port(DEFAULT_AP_PORT) {
        return Some((format!("127.0.0.1:{DEFAULT_AP_PORT}"), None));
    }
    None
}

/// Archipelago patch-applying helper, embedded into the binary at build time
/// so we can drop it next to any runtime (dev tree, user install dir, or a
/// remote one) without relying on `scripts/` being co-located with the app.
const APPLY_PATCH_PY: &str = include_str!("../../../scripts/apply_patch.py");

/// Extensions that sit next to a generated seed but are NOT per-player patch
/// files. They share the `.ap…` prefix so the regex-style filter below would
/// catch them otherwise.
///
/// * `archipelago`. Multidata, handled by the seeds panel.
/// * `apsave`. MultiServer's periodic save snapshot (server-side state).
/// * `apbp` / `apmc`. Internal patch-format intermediates that occasionally
///   leak out in certain worlds; they aren't playable on their own.
/// * `zip`. The raw generate output, stripped after extraction.
const NON_PATCH_EXTENSIONS: &[&str] = &["archipelago", "zip", "apsave", "apbp", "apmc"];

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
    // Newest first. Mirrors list_seeds.
    out.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(out)
}

pub fn delete_patch(path: &Path) -> Result<()> {
    if !path.exists() {
        bail!("not found: {}", path.display());
    }
    // Only allow deletion inside the output dir to be safe. The caller should
    // have come from list_patches, but be defensive.
    fs::remove_file(path).with_context(|| format!("remove {}", path.display()))?;
    Ok(())
}

/// Extensions handled by Archipelago's external SNI Client (SNES/SFC games).
/// This runtime doesn't bundle `SNIClient.py`, so Launcher.py can't spawn a
/// client for these. We apply the patch directly and skip step 2 with a
/// friendly message instead of letting the user see a confusing
/// `can't open 'SNIClient.py'` error in the Console.
const SNI_EXTENSIONS: &[&str] = &[
    ".aplttp",
    ".apz3",
    ".apdkc3",
    ".apeb",
    ".apkdl3",
    ".apl2ac",
    ".aplufia2ac",
    ".apmlss",
    ".apsm",
    ".apm3",
    ".apsmw",
    ".apsmz3",
    ".apyi",
    ".apsoe",
    ".apcv64",
];

fn is_sni_patch(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    SNI_EXTENSIONS.iter().any(|ext| name.ends_with(ext))
}

/// Apply a patch + fire up its custom client.
///
/// Two-step flow so games whose client isn't bundled still get patched:
///
/// 1. **Direct patch** via `scripts/apply_patch.py`. This walks Archipelago's
///    `AutoPatchRegister`, finds the class for the patch's extension, and
///    writes the output ROM next to the patch. Works for every game whose
///    world is installed. Including SNI-based ones (ALttP, SMW, SM, etc.)
///    that don't ship a launchable `SNIClient.py` in this runtime.
/// 2. **Launcher.py** with `--nogui` to spawn the per-game client for its
///    emulator bridge (BizHawk connector, LADX bridge, …). If the Launcher
///    can't resolve a client (SNI games with no bundled SNIClient), the
///    patch is already on disk so the user can load it manually.
///
/// Step 1 is blocking + surfaces its error. Step 2 is fire-and-forget.
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

    // --- Step 1: produce the output ROM. ----------------------------------
    // We surface the helper's output through the same `randomizer://bridge-log`
    // channel the Console listens on, so the user sees "applying ... -> X.sfc"
    // in the feed instead of silent success.
    let emit_log = |text: String| {
        let _ = app.emit(
            "randomizer://bridge-log",
            BridgeLog {
                stream: "stdout",
                text,
            },
        );
    };
    emit_log(format!(
        "[zephyr] Play -> {}",
        patch_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("patch")
    ));
    emit_log(format!("[zephyr]   python: {python}"));
    emit_log(format!("[zephyr]   runtime: {}", dir.display()));

    // Always materialise the embedded helper next to the runtime so it works
    // regardless of whether the runtime is the in-tree dev copy or a freshly
    // downloaded one under `app_data_dir`. Overwrites on every Play so a
    // Zephyr update with a newer helper always wins.
    let helper = dir.join(".zephyr_apply_patch.py");
    if let Err(err) = fs::write(&helper, APPLY_PATCH_PY) {
        emit_log(format!(
            "[zephyr] failed to write helper at {}: {err}",
            helper.display()
        ));
    }

    if helper.exists() {
        emit_log(format!("[zephyr]   applying via {}", helper.display()));
        let mut cmd = Command::new(&python);
        sanitize_python_env(&mut cmd);
        let out = cmd
            .current_dir(&dir)
            .env("PYTHONIOENCODING", "utf-8")
            .env("PYTHONDONTWRITEBYTECODE", "1")
            .arg(&helper)
            .arg(patch_path)
            .arg("--runtime")
            .arg(&dir)
            .stdin(Stdio::null())
            .no_window()
            .output()
            .with_context(|| format!("run apply_patch.py for {}", patch_path.display()))?;

        for line in String::from_utf8_lossy(&out.stdout).lines() {
            if !line.trim().is_empty() {
                let _ = app.emit(
                    "randomizer://bridge-log",
                    BridgeLog {
                        stream: "stdout",
                        text: line.to_string(),
                    },
                );
            }
        }
        for line in String::from_utf8_lossy(&out.stderr).lines() {
            if !line.trim().is_empty() {
                let _ = app.emit(
                    "randomizer://bridge-log",
                    BridgeLog {
                        stream: "stderr",
                        text: line.to_string(),
                    },
                );
            }
        }

        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let tail = stderr
                .lines()
                .rev()
                .take(8)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n");
            bail!("patch failed: {tail}");
        }
    }
    // (If the helper was missing we already emitted a bridge-log message above.)

    if is_sni_patch(patch_path) && !dir.join("SNIClient.py").exists() {
        let ext = patch_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("ap*");
        for line in [
            "[zephyr] SNES game detected. The patched ROM is ready next to the patch.".to_string(),
            format!("[zephyr] SNIClient.py is not bundled; .{ext} needs an external SNI stack."),
            "[zephyr] To play:".to_string(),
            "[zephyr]   1. Download SNI daemon: https://github.com/alttpo/sni/releases".to_string(),
            "[zephyr]   2. Run SNI alongside an SNI-compatible emulator.".to_string(),
            "[zephyr]   3. Launch SNIClient.py from an upstream Archipelago install.".to_string(),
            "[zephyr]   4. Use this Zephyr Console for chat / hints.".to_string(),
        ] {
            let _ = app.emit(
                "randomizer://bridge-log",
                BridgeLog {
                    stream: "stdout",
                    text: line,
                },
            );
        }
        return Ok(());
    }

    // --- Step 2: fire up the per-game client (bridge-only). ---------------
    // Non-fatal: the ROM already exists on disk after step 1, so even if the
    // Launcher can't find a client (e.g. SNI games w/o SNIClient.py) the user
    // can still run the ROM manually.
    //
    // We pipe stdout/stderr so bridge events (BizHawk connect/disconnect,
    // item receive, deathlink) can be forwarded to the Zephyr Console as a
    // log feed. See the `randomizer://bridge-log` event below.
    // Build the Launcher.py invocation. After `--`, everything is forwarded
    // verbatim to the resolved component (SNIClient / BizHawkClient / ...).
    // We always pass `--nogui`, and when we can identify a running AP server
    // we also pass `--connect host:port` (+ `--password` if set) so the
    // client doesn't sit at "no active multiworld server connection" ,
    // stdin is piped to null so the user can't type /connect manually.
    let mut cmd = Command::new(&python);
    sanitize_python_env(&mut cmd);
    cmd.current_dir(&dir)
        .env("PYTHONIOENCODING", "utf-8")
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .arg(&launcher)
        .arg(patch_path)
        .arg("--")
        .arg("--nogui");
    if let Some((target, password)) = resolve_ap_target(app) {
        cmd.arg("--connect").arg(&target);
        if let Some(pw) = password {
            cmd.arg("--password").arg(pw);
        }
        let _ = app.emit(
            "randomizer://bridge-log",
            BridgeLog {
                stream: "stdout",
                text: format!("[zephyr] Auto-connecting client to {target}"),
            },
        );
    } else {
        let _ = app.emit(
            "randomizer://bridge-log",
            BridgeLog {
                stream: "stdout",
                text: format!(
                    "[zephyr] No AP server detected on 127.0.0.1:{DEFAULT_AP_PORT}. Start your server first, or use /connect inside the client."
                ),
            },
        );
    }
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .no_window();

    match cmd.spawn() {
        Ok(mut child) => {
            let patch_name = patch_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("patch")
                .to_string();
            let _ = app.emit(
                "randomizer://bridge-started",
                BridgeStarted {
                    pid: child.id(),
                    patch: patch_name.clone(),
                },
            );

            // Drain stdout → bridge-log(stream="stdout").
            if let Some(stdout) = child.stdout.take() {
                let app_h = app.clone();
                std::thread::spawn(move || {
                    for line in BufReader::new(stdout).lines().flatten() {
                        let _ = app_h.emit(
                            "randomizer://bridge-log",
                            BridgeLog {
                                stream: "stdout",
                                text: line,
                            },
                        );
                    }
                });
            }
            // Drain stderr → bridge-log(stream="stderr") so Python tracebacks
            // from the client (failed SNI connect, missing base ROM, etc.)
            // surface in the Console instead of disappearing into /dev/null.
            if let Some(stderr) = child.stderr.take() {
                let app_h = app.clone();
                std::thread::spawn(move || {
                    for line in BufReader::new(stderr).lines().flatten() {
                        let _ = app_h.emit(
                            "randomizer://bridge-log",
                            BridgeLog {
                                stream: "stderr",
                                text: line,
                            },
                        );
                    }
                });
            }
            // Reap the child + announce exit on a separate thread so we don't
            // leave a zombie process and the UI knows when to re-enable Play.
            let app_exit = app.clone();
            std::thread::spawn(move || {
                let code = child.wait().ok().and_then(|s| s.code());
                let _ = app_exit.emit(
                    "randomizer://bridge-exited",
                    BridgeExited {
                        code,
                        patch: patch_name,
                    },
                );
            });
        }
        Err(err) => {
            tracing::warn!(
                "failed to spawn Launcher for {}: {err:#}",
                patch_path.display()
            );
        }
    }
    Ok(())
}

#[derive(Serialize, Clone, Debug)]
struct BridgeStarted {
    pid: u32,
    patch: String,
}

#[derive(Serialize, Clone, Debug)]
struct BridgeLog {
    stream: &'static str,
    text: String,
}

#[derive(Serialize, Clone, Debug)]
struct BridgeExited {
    code: Option<i32>,
    patch: String,
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

    let mut cmd = Command::new(&python);
    sanitize_python_env(&mut cmd);
    cmd.current_dir(&dir)
        .env("PYTHONIOENCODING", "utf-8")
        .arg(&launcher)
        .arg(component_name)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .no_window()
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
        // Common ROM/output extensions. If it matches the patch's stem AND
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
