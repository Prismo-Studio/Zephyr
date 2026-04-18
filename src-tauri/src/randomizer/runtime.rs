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
    io::{BufRead, BufReader, Cursor, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use eyre::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use zip::ZipArchive;

use super::ap_runner::{ap_dir, ap_install_dir, venv_dir};

/// Default source: the source tarball of the runtime repo's main branch.
/// Overridable by the caller (UI can pass a pinned release URL later).
pub const DEFAULT_RUNTIME_URL: &str =
    "https://github.com/Prismo-Studio/randomizer-server/archive/refs/heads/main.zip";

/// Upstream location of SNIClient.py. Prismo-Studio/randomizer-server strips
/// this file; without it, Launcher.py can't bridge SNES games (ALttP, SMZ3,
/// Super Metroid, SMW, etc.) between the MultiServer and the emulator via SNI.
const SNICLIENT_URL: &str =
    "https://raw.githubusercontent.com/ArchipelagoMW/Archipelago/main/SNIClient.py";

async fn ensure_sni_client(install_dir: &Path, client: &reqwest::Client) -> Result<()> {
    let target = install_dir.join("SNIClient.py");
    if target.exists() {
        return Ok(());
    }
    let resp = client
        .get(SNICLIENT_URL)
        .send()
        .await
        .context("fetch SNIClient.py")?;
    if !resp.status().is_success() {
        bail!("SNIClient.py download failed: HTTP {}", resp.status());
    }
    let body = resp.bytes().await.context("read SNIClient.py body")?;
    fs::write(&target, &body).with_context(|| format!("write {}", target.display()))?;
    tracing::info!("installed SNIClient.py into {}", install_dir.display());
    Ok(())
}

/// Flip `snes_rom_start: true` to `false` in host.yaml so SNIClient does not
/// try to auto-open the patched ROM with the OS default handler (which often
/// resolves to a browser on Linux / misconfigured Windows installs).
fn patch_host_yaml(install_dir: &Path) -> Result<()> {
    let path = install_dir.join("host.yaml");
    if !path.exists() {
        return Ok(());
    }
    let contents = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let patched = contents.replace("snes_rom_start: true", "snes_rom_start: false");
    if patched != contents {
        fs::write(&path, patched).with_context(|| format!("write {}", path.display()))?;
        tracing::info!("patched host.yaml snes_rom_start -> false");
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RuntimeStatus {
    pub installed: bool,
    /// The Python venv at `<runtime>/venv/` exists and has a python binary.
    /// False means Generate.py will fall back to system Python and may error
    /// on world imports because dependencies aren't installed.
    pub venv_ready: bool,
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
    let effective = ap_dir(app);
    let effective_installed = effective.join("Generate.py").exists();

    let world_count = count_worlds(&effective).unwrap_or(0);
    let bytes = dir_size(&effective).unwrap_or(0);

    RuntimeStatus {
        installed: installed || effective_installed,
        // venv_ready only when BOTH the venv python exists AND our marker
        // file is present. The marker is written at the end of a successful
        // provisioning run, so a half-installed venv still shows as "not
        // ready" and gives the user an obvious "Install dependencies" action.
        venv_ready: venv_python_path(&effective).exists()
            && provision_marker(&effective).exists(),
        path: effective.to_string_lossy().to_string(),
        bytes_on_disk: bytes,
        world_count,
        default_download_url: DEFAULT_RUNTIME_URL.to_string(),
    }
}

fn provision_marker(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("venv").join(".zephyr-deps-installed")
}

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "stage", rename_all = "snake_case")]
pub enum ProgressEvent {
    Downloading { received: u64, total: Option<u64> },
    Extracting { entry: String, done: u32, total: u32 },
    ProvisioningVenv { message: String },
    InstallingDeps { message: String },
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
            "install finished but {} was not written, is the downloaded archive the correct runtime?",
            install_dir.join("Generate.py").display()
        );
        emit(ProgressEvent::Failed { error: msg.clone() });
        bail!(msg);
    }

    if let Err(err) = ensure_sni_client(&install_dir, &client).await {
        tracing::warn!("SNIClient.py bootstrap skipped: {err:#}");
    }
    if let Err(err) = patch_host_yaml(&install_dir) {
        tracing::warn!("host.yaml patch skipped: {err:#}");
    }

    // --- Provision venv + install deps -----------------------------------
    // Non-fatal: if this fails, the runtime still exists on disk and the user
    // can retry via the "Install Python dependencies" button in the UI.
    if let Err(err) = provision_venv_at(app, &ap_dir(app), &emit) {
        let msg = format!("runtime extracted but venv provisioning failed: {err:#}");
        tracing::warn!("{msg}");
        emit(ProgressEvent::Failed { error: msg });
        // Return the status anyway — partially installed is still usable once
        // the user runs provision_venv manually.
        return Ok(status(app));
    }

    let st = status(app);
    emit(ProgressEvent::Installed {
        path: st.path.clone(),
    });
    Ok(st)
}

/// Create `<runtime>/venv/`, upgrade pip/setuptools, install Archipelago's
/// main requirements.txt, then best-effort install each `worlds/*/requirements.txt`.
/// Safe to call repeatedly.
pub async fn provision_venv(app: &AppHandle) -> Result<RuntimeStatus> {
    let emit = |event: ProgressEvent| {
        let _ = app.emit("randomizer://runtime-progress", event);
    };
    let dir = ap_dir(app);
    if !dir.join("Generate.py").exists() {
        bail!(
            "Archipelago runtime not installed at {} — download it first.",
            dir.display()
        );
    }
    let client = reqwest::Client::builder()
        .user_agent("Zephyr/1 (archipelago-runtime-installer)")
        .build()
        .context("build http client")?;
    if let Err(err) = ensure_sni_client(&dir, &client).await {
        tracing::warn!("SNIClient.py bootstrap skipped: {err:#}");
    }
    if let Err(err) = patch_host_yaml(&dir) {
        tracing::warn!("host.yaml patch skipped: {err:#}");
    }
    provision_venv_at(app, &dir, &emit)?;
    Ok(status(app))
}

pub fn remove(app: &AppHandle) -> Result<()> {
    let dir = ap_install_dir(app);
    if dir.exists() {
        fs::remove_dir_all(&dir).with_context(|| format!("remove {}", dir.display()))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Venv provisioning
// ---------------------------------------------------------------------------

fn venv_python_path(runtime_dir: &Path) -> PathBuf {
    let venv = runtime_dir.join("venv");
    if cfg!(target_os = "windows") {
        venv.join("Scripts").join("python.exe")
    } else {
        venv.join("bin").join("python")
    }
}

fn provision_venv_at(
    app: &AppHandle,
    runtime_dir: &Path,
    emit: &dyn Fn(ProgressEvent),
) -> Result<()> {
    emit(ProgressEvent::ProvisioningVenv {
        message: "Detecting system Python…".into(),
    });
    // We need a system Python to bootstrap the venv. detect_python prefers
    // the venv first, so call it AFTER ruling the venv in/out.
    let venv = venv_dir(app);
    let system_python = if venv_python_path(runtime_dir).exists() {
        // venv already exists — keep using it.
        venv_python_path(runtime_dir).to_string_lossy().to_string()
    } else {
        emit(ProgressEvent::ProvisioningVenv {
            message: format!("Creating venv at {}", venv.display()),
        });
        let (bin, prefix_args) = find_bootstrap_python().ok_or_else(|| {
            eyre::eyre!(
                "Supported Python not found (need 3.11, 3.12 or 3.13). Install one from python.org then retry."
            )
        })?;
        let mut cmd = Command::new(&bin);
        for arg in &prefix_args {
            cmd.arg(arg);
        }
        cmd.arg("-m").arg("venv").arg(&venv);
        run_to_log(&mut cmd, emit).context("create venv")?;
        venv_python_path(runtime_dir).to_string_lossy().to_string()
    };

    // Upgrade pip + wheel + setuptools. We pin setuptools to <81 because
    // `pkg_resources` was deprecated and ultimately gated behind an import
    // warning in setuptools 81+; several Archipelago worlds still
    // `import pkg_resources` directly (Pokémon Emerald's data.py is the
    // canonical offender). <81 keeps the import working on Python 3.13.
    emit(ProgressEvent::InstallingDeps {
        message: "Upgrading pip, setuptools, wheel…".into(),
    });
    run_to_log(
        Command::new(&system_python)
            .arg("-m")
            .arg("pip")
            .arg("install")
            .arg("--upgrade")
            .arg("pip")
            .arg("setuptools<81")
            .arg("wheel"),
        emit,
    )
    .context("upgrade pip/setuptools — base bootstrap failed; cannot continue")?;

    // Sanity check: confirm pkg_resources actually imports inside the venv.
    // If this fails we want a loud error, not a silent fallthrough that
    // leaves Pokémon Emerald broken.
    let pkg_check = Command::new(&system_python)
        .arg("-c")
        .arg("import pkg_resources")
        .output()
        .context("probe pkg_resources")?;
    if !pkg_check.status.success() {
        let stderr = String::from_utf8_lossy(&pkg_check.stderr).to_string();
        bail!(
            "pkg_resources not importable after installing setuptools; most likely the pinned version didn't land. stderr:\n{stderr}"
        );
    }

    // Main requirements.
    let main_req = runtime_dir.join("requirements.txt");
    if main_req.exists() {
        emit(ProgressEvent::InstallingDeps {
            message: "Installing Archipelago core dependencies…".into(),
        });
        run_to_log(
            Command::new(&system_python)
                .arg("-m")
                .arg("pip")
                .arg("install")
                .arg("-r")
                .arg(&main_req),
            emit,
        )
        .context("install core requirements")?;
    } else {
        tracing::warn!(
            "no requirements.txt at {} — skipping core deps",
            main_req.display()
        );
    }

    // Mark core deps as installed BEFORE the per-world loop so that a failure
    // in one optional world's deps doesn't reset the "venv is ready for the
    // standard worlds" signal. We overwrite this at the very end too.
    let marker = provision_marker(runtime_dir);
    if let Some(parent) = marker.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(&marker, "core\n").ok();

    // Per-world requirements. Best-effort: one broken world's install failure
    // shouldn't kill the whole provisioning step.
    let worlds_dir = runtime_dir.join("worlds");
    if worlds_dir.exists() {
        if let Ok(entries) = fs::read_dir(&worlds_dir) {
            let mut reqs: Vec<PathBuf> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .map(|p| p.join("requirements.txt"))
                .filter(|p| p.exists())
                .collect();
            reqs.sort();
            for (idx, req) in reqs.iter().enumerate() {
                let world = req
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|s| s.to_str())
                    .unwrap_or("?");
                emit(ProgressEvent::InstallingDeps {
                    message: format!("Installing {world} deps ({}/{})…", idx + 1, reqs.len()),
                });
                if let Err(err) = run_to_log(
                    Command::new(&system_python)
                        .arg("-m")
                        .arg("pip")
                        .arg("install")
                        .arg("-r")
                        .arg(req),
                    emit,
                ) {
                    tracing::warn!("skipping {world}: {err:#}");
                    emit(ProgressEvent::InstallingDeps {
                        message: format!("⚠ {world}: {err}"),
                    });
                }
            }
        }
    }

    // Final marker — everything succeeded. `venv_ready` in status() keys off
    // this file so that a torn-down install doesn't masquerade as ready.
    fs::write(&marker, "full\n").ok();
    Ok(())
}

/// Probe a Python candidate: return (major, minor) on success.
fn probe_python_version(candidate: &str, extra_args: &[&str]) -> Option<(u32, u32)> {
    let mut cmd = Command::new(candidate);
    for arg in extra_args {
        cmd.arg(arg);
    }
    let out = cmd.arg("--version").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let text = if stdout.trim().is_empty() { stderr } else { stdout };
    let digits = text
        .trim()
        .trim_start_matches("Python ")
        .split('.')
        .take(2)
        .filter_map(|s| s.parse::<u32>().ok())
        .collect::<Vec<_>>();
    if digits.len() != 2 {
        return None;
    }
    Some((digits[0], digits[1]))
}

fn is_supported(version: (u32, u32)) -> bool {
    version.0 == 3 && (11..=13).contains(&version.1)
}

/// Supported Python range: 3.11–3.13. Kivy (required by a few AP worlds)
/// doesn't ship 3.14 wheels yet; 3.10 and below are below AP's minimum.
///
/// Returns `(binary, extra_args)` so the caller can spawn `py -3.11 ...`
/// correctly without us trying to exec a single string.
pub fn find_bootstrap_python() -> Option<(String, Vec<String>)> {
    for candidate in ["python3.13", "python3.12", "python3.11"] {
        if let Some(v) = probe_python_version(candidate, &[]) {
            if is_supported(v) {
                return Some((candidate.to_string(), vec![]));
            }
        }
    }

    for candidate in ["python3", "python"] {
        if let Some(v) = probe_python_version(candidate, &[]) {
            if is_supported(v) {
                return Some((candidate.to_string(), vec![]));
            }
        }
    }

    for flag in ["-3.13", "-3.12", "-3.11"] {
        if let Some(v) = probe_python_version("py", &[flag]) {
            if is_supported(v) {
                return Some(("py".to_string(), vec![flag.to_string()]));
            }
        }
    }

    if let Some(v) = probe_python_version("py", &[]) {
        if is_supported(v) {
            return Some(("py".to_string(), vec![]));
        }
    }

    None
}

/// Spawn a command, stream stdout to the progress channel line-by-line, and
/// capture stderr for error reporting. Returns the tail of stderr when the
/// process exits non-zero.
///
/// We read stdout on the calling thread (safe with `emit`) and buffer stderr
/// in a spawned thread that doesn't touch `emit`.
fn run_to_log(cmd: &mut Command, emit: &dyn Fn(ProgressEvent)) -> Result<()> {
    cmd.env("PYTHONIOENCODING", "utf-8")
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .env("PIP_DISABLE_PIP_VERSION_CHECK", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().context("spawn pip")?;

    // Drain stderr in a background thread into a shared buffer so we don't
    // deadlock on full pipe buffers. No `emit` cross-thread dance needed.
    let stderr_buf: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    if let Some(stderr) = child.stderr.take() {
        let buf = stderr_buf.clone();
        std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().flatten() {
                if let Ok(mut g) = buf.lock() {
                    g.push(line);
                    // Cap to avoid unbounded growth on a chatty failure.
                    if g.len() > 500 {
                        let drop = g.len() - 500;
                        g.drain(0..drop);
                    }
                }
            }
        });
    }

    // Stream stdout on the current thread — safe to call `emit` here.
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines().flatten() {
            emit(ProgressEvent::InstallingDeps { message: line });
        }
    }

    let status = child.wait().context("wait pip")?;
    if !status.success() {
        let tail: String = stderr_buf
            .lock()
            .map(|g| g.iter().rev().take(10).cloned().collect::<Vec<_>>())
            .unwrap_or_default()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        bail!("pip exited with status {status}\n{tail}");
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
