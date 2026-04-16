//! Subprocess wrappers for Archipelago's `Generate.py` and `MultiServer.py`.
//!
//! All paths are resolved relative to `<repo>/src-tauri/archipelago-runtime`
//! (the Archipelago code bundled inside Zephyr). A Python venv is expected at
//! `archipelago-runtime/venv/`. Long-running server processes are tracked in a
//! [`ServerHandle`] kept inside the Tauri state.

use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    net::{IpAddr, TcpListener, UdpSocket},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};

use zip::ZipArchive;

use eyre::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tracing::{info, warn};

/// Resolve the directory of the bundled Archipelago runtime.
/// In dev: `src-tauri/archipelago-runtime` (relative to CWD or one level up).
/// In prod: could be the Tauri resource dir in the future.
pub fn ap_dir(_app: &AppHandle) -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let candidates = [
        cwd.join("archipelago-runtime"),
        cwd.join("../src-tauri/archipelago-runtime"),
        cwd.join("src-tauri/archipelago-runtime"),
    ];
    for c in &candidates {
        if c.exists() {
            return c.clone();
        }
    }
    // Fallback — first candidate
    candidates[0].clone()
}

/// Path to the Python venv inside the Archipelago runtime directory.
pub fn venv_dir(_app: &AppHandle) -> PathBuf {
    ap_dir(_app).join("venv")
}

/// Return the venv Python executable path (platform-specific).
fn venv_python(app: &AppHandle) -> PathBuf {
    let venv = venv_dir(app);
    if cfg!(target_os = "windows") {
        venv.join("Scripts").join("python.exe")
    } else {
        venv.join("bin").join("python")
    }
}

/// Workspace where we drop player YAMLs and harvest the generated `.zip`.
pub fn workspace_dir(app: &AppHandle) -> PathBuf {
    let base = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir());
    base.join("randomizer")
}

pub fn players_dir(app: &AppHandle) -> PathBuf {
    workspace_dir(app).join("Players")
}

pub fn output_dir(app: &AppHandle) -> PathBuf {
    workspace_dir(app).join("output")
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PythonStatus {
    pub available: bool,
    pub executable: Option<String>,
    pub version: Option<String>,
    pub ap_dir: String,
    pub ap_present: bool,
}

/// Find a usable Python executable.
/// Priority: venv Python > system python > python3 > py.
pub fn detect_python(app: &AppHandle) -> Option<(String, String)> {
    // 1. Try our own venv first
    let venv_py = venv_python(app);
    let mut candidates: Vec<String> = Vec::new();
    if venv_py.exists() {
        candidates.push(venv_py.to_string_lossy().to_string());
    }
    // 2. System fallbacks
    candidates.extend(["python".to_string(), "python3".to_string(), "py".to_string()]);

    for candidate in &candidates {
        if let Ok(out) = Command::new(candidate).arg("--version").output() {
            if out.status.success() {
                let version = String::from_utf8_lossy(&out.stdout)
                    .trim()
                    .to_string()
                    .replace("Python ", "");
                let v2 = if version.is_empty() {
                    String::from_utf8_lossy(&out.stderr)
                        .trim()
                        .to_string()
                        .replace("Python ", "")
                } else {
                    version
                };
                return Some((candidate.clone(), v2));
            }
        }
    }
    None
}

pub fn check_python(app: &AppHandle) -> PythonStatus {
    let dir = ap_dir(app);
    let present = dir.join("Generate.py").exists();
    match detect_python(app) {
        Some((exe, ver)) => PythonStatus {
            available: true,
            executable: Some(exe),
            version: Some(ver),
            ap_dir: dir.to_string_lossy().to_string(),
            ap_present: present,
        },
        None => PythonStatus {
            available: false,
            executable: None,
            version: None,
            ap_dir: dir.to_string_lossy().to_string(),
            ap_present: present,
        },
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenerateOutcome {
    pub success: bool,
    pub zip_path: Option<String>,
    pub stdout: String,
    pub stderr: String,
}

/// Run `python Generate.py --player_files_path <players> --outputpath <output>`
/// in the bundled Archipelago directory and return the produced `.zip` path.
pub fn run_generate(app: &AppHandle) -> Result<GenerateOutcome> {
    let (python, _) = detect_python(app).ok_or_else(|| {
        eyre::eyre!("python is not installed on this machine; install Python 3.11+ to generate seeds")
    })?;

    let dir = ap_dir(app);
    if !dir.join("Generate.py").exists() {
        bail!("Archipelago Generate.py not found at {}", dir.display());
    }

    let players = players_dir(app);
    let output = output_dir(app);
    std::fs::create_dir_all(&players)?;
    std::fs::create_dir_all(&output)?;

    // Snapshot existing zips so we can identify the new one.
    let before: std::collections::HashSet<PathBuf> = std::fs::read_dir(&output)
        .map(|it| {
            it.flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("zip"))
                .collect()
        })
        .unwrap_or_default();

    let out = Command::new(&python)
        .current_dir(&dir)
        .env("SKIP_REQUIREMENTS_UPDATE", "1")
        .env("PYTHONIOENCODING", "utf-8")
        .arg("Generate.py")
        .arg("--player_files_path")
        .arg(&players)
        .arg("--outputpath")
        .arg(&output)
        .arg("--spoiler")
        .arg("0")
        // close stdin so the atexit "Press enter to close" can't hang us
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("failed to spawn Generate.py")?;

    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();

    if !out.status.success() {
        return Ok(GenerateOutcome {
            success: false,
            zip_path: None,
            stdout,
            stderr,
        });
    }

    // Find the new zip
    let after: Vec<PathBuf> = std::fs::read_dir(&output)
        .map(|it| {
            it.flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("zip"))
                .collect()
        })
        .unwrap_or_default();

    let new_zip = after
        .into_iter()
        .filter(|p| !before.contains(p))
        .max_by_key(|p| {
            std::fs::metadata(p)
                .and_then(|m| m.modified())
                .ok()
        });

    // Auto-extract the .archipelago multidata next to the zip; that's what we
    // hand to MultiServer.py (its zip support is unreliable).
    // After successful extraction, delete the zip to avoid ghost duplicates
    // when list_seeds() tries to re-extract from renamed zips.
    let extracted = if let Some(zip) = new_zip.as_ref() {
        match extract_archipelago(zip) {
            Ok(p) => {
                // Remove the zip — the .archipelago is all we need
                let _ = std::fs::remove_file(zip);
                Some(p.to_string_lossy().to_string())
            }
            Err(err) => {
                tracing::warn!("failed to extract .archipelago from {}: {err:#}", zip.display());
                None
            }
        }
    } else {
        None
    };

    Ok(GenerateOutcome {
        success: true,
        zip_path: extracted.or_else(|| new_zip.map(|p| p.to_string_lossy().to_string())),
        stdout,
        stderr,
    })
}

/// Persist a player's YAML to the workspace players dir.
pub fn save_player_yaml(app: &AppHandle, slot_name: &str, yaml: &str) -> Result<PathBuf> {
    let dir = players_dir(app);
    std::fs::create_dir_all(&dir)?;
    // sanitize slot name for filename
    let safe: String = slot_name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect();
    let safe = if safe.is_empty() { "player".to_string() } else { safe };
    let path = dir.join(format!("{safe}.yaml"));
    std::fs::write(&path, yaml).with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerFile {
    pub name: String,
    pub path: String,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SeedFile {
    pub name: String,
    pub path: String,
    pub size: u64,
    /// Unix epoch seconds — for sorting newest-first
    pub modified: i64,
}

pub fn list_seeds(app: &AppHandle) -> Result<Vec<SeedFile>> {
    let dir = output_dir(app);
    if !dir.exists() {
        return Ok(vec![]);
    }

    // First pass: ensure every AP_*.zip has a matching extracted .archipelago.
    // This makes existing zips (from older builds) usable on the next refresh.
    let entries: Vec<PathBuf> = std::fs::read_dir(&dir)?
        .flatten()
        .map(|e| e.path())
        .collect();

    for path in &entries {
        if path.extension().and_then(|s| s.to_str()) == Some("zip") {
            if let Err(err) = extract_archipelago(path) {
                tracing::warn!(
                    "failed to extract .archipelago from {}: {err:#}",
                    path.display()
                );
            }
        }
    }

    // Second pass: list .archipelago files (they're what we host).
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("archipelago") {
            continue;
        }
        let meta = entry.metadata()?;
        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        out.push(SeedFile {
            name: path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("?")
                .to_string(),
            path: path.to_string_lossy().to_string(),
            size: meta.len(),
            modified,
        });
    }
    // newest first
    out.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(out)
}

pub fn rename_seed(path: &Path, new_name: &str) -> Result<PathBuf> {
    if !path.exists() {
        bail!("seed file not found: {}", path.display());
    }
    let parent = path.parent().ok_or_else(|| eyre::eyre!("no parent"))?;
    let old_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let safe: String = new_name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' { c } else { '_' })
        .collect();
    let safe = if safe.is_empty() { "seed".to_string() } else { safe };

    // Rename the .archipelago file
    let new_path = parent.join(format!("{safe}.archipelago"));
    if new_path != path {
        std::fs::rename(path, &new_path)
            .with_context(|| format!("rename {} -> {}", path.display(), new_path.display()))?;
    }

    // Rename companion files (.zip, .apsave, _Spoiler.txt)
    for ext in &[".zip", ".apsave"] {
        let old = parent.join(format!("{old_stem}{ext}"));
        if old.exists() {
            let _ = std::fs::rename(&old, parent.join(format!("{safe}{ext}")));
        }
    }
    let old_spoiler = parent.join(format!("{old_stem}_Spoiler.txt"));
    if old_spoiler.exists() {
        let _ = std::fs::rename(&old_spoiler, parent.join(format!("{safe}_Spoiler.txt")));
    }

    Ok(new_path)
}

pub fn delete_seed(path: &Path) -> Result<()> {
    if path.exists() {
        std::fs::remove_file(path).with_context(|| format!("delete {}", path.display()))?;
    }
    // Also clean siblings sharing the same stem: .zip, .apsave, _Spoiler.txt
    if let (Some(parent), Some(stem)) = (path.parent(), path.file_stem()) {
        let stem = stem.to_string_lossy();
        for ext in &[".zip", ".apsave"] {
            let p = parent.join(format!("{stem}{ext}"));
            let _ = std::fs::remove_file(p);
        }
        let _ = std::fs::remove_file(parent.join(format!("{stem}_Spoiler.txt")));
    }
    Ok(())
}

pub fn clear_seeds(app: &AppHandle) -> Result<usize> {
    let dir = output_dir(app);
    if !dir.exists() {
        return Ok(0);
    }
    let mut count = 0;
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str());
        if matches!(ext, Some("zip") | Some("apsave") | Some("archipelago"))
            || path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.ends_with("_Spoiler.txt"))
                .unwrap_or(false)
        {
            if std::fs::remove_file(&path).is_ok() {
                count += 1;
            }
        }
    }
    Ok(count)
}

pub fn list_player_yamls(app: &AppHandle) -> Result<Vec<PlayerFile>> {
    let dir = players_dir(app);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
            continue;
        }
        let meta = entry.metadata()?;
        out.push(PlayerFile {
            name: path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("?")
                .to_string(),
            path: path.to_string_lossy().to_string(),
            size: meta.len(),
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

pub fn delete_player_yaml(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    std::fs::remove_file(path).with_context(|| format!("delete {}", path.display()))?;
    Ok(())
}

pub fn rename_player_yaml(path: &Path, new_name: &str) -> Result<PathBuf> {
    if !path.exists() {
        bail!("player file not found: {}", path.display());
    }
    let safe: String = new_name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect();
    let safe = if safe.is_empty() { "player".to_string() } else { safe };
    let new_path = path.with_file_name(format!("{safe}.yaml"));
    if new_path != path {
        std::fs::rename(path, &new_path)
            .with_context(|| format!("rename {} -> {}", path.display(), new_path.display()))?;
    }
    Ok(new_path)
}

// ---- Server lifecycle ----

#[derive(Default)]
pub struct ServerState {
    inner: Mutex<Option<RunningServer>>,
    log_buffer: Arc<Mutex<Vec<String>>>,
    cached_public_ip: Mutex<Option<String>>,
}

struct RunningServer {
    child: Child,
    port: u16,
    password: Option<String>,
    multidata: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerStatus {
    pub running: bool,
    pub port: Option<u16>,
    pub password: Option<String>,
    pub multidata: Option<String>,
    pub pid: Option<u32>,
    pub recent_log: Vec<String>,
    pub local_ip: Option<String>,
    pub public_ip: Option<String>,
    /// True iff `127.0.0.1:port` accepts a TCP connection right now.
    pub port_reachable: bool,
}

/// Detect the local LAN IP by opening a UDP socket "to" a public address.
/// No traffic is actually sent — the OS picks the routable interface for us.
pub fn detect_local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let addr = socket.local_addr().ok()?;
    match addr.ip() {
        IpAddr::V4(v4) if !v4.is_loopback() => Some(v4.to_string()),
        IpAddr::V6(v6) if !v6.is_loopback() => Some(v6.to_string()),
        _ => None,
    }
}

/// Update the `server_options.port` line in `host.yaml` in place, preserving all
/// comments and other keys. Uses a line-based regex replacement because we don't
/// want `serde_yaml` to strip the heavily-commented file.
pub fn update_host_yaml_port(ap_dir: &Path, port: u16) -> Result<()> {
    let path = ap_dir.join("host.yaml");
    if !path.exists() {
        // nothing to do — AP will fall back to its compiled defaults
        return Ok(());
    }

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("read {}", path.display()))?;

    let mut out = String::with_capacity(content.len());
    let mut in_server_options = false;
    let mut replaced = false;

    for line in content.lines() {
        let trimmed = line.trim_start();

        // Track the section we're in (top-level keys have 0 indent).
        if !line.starts_with(' ') && !line.starts_with('\t') && trimmed.ends_with(':') {
            in_server_options = trimmed == "server_options:";
        }

        if in_server_options && !replaced {
            // Match "  port: <anything>" possibly followed by a comment
            if let Some(rest) = trimmed.strip_prefix("port:") {
                let comment = rest.find('#').map(|i| &rest[i..]).unwrap_or("");
                let indent_len = line.len() - trimmed.len();
                let indent = &line[..indent_len];
                out.push_str(indent);
                out.push_str("port: ");
                out.push_str(&port.to_string());
                if !comment.is_empty() {
                    out.push(' ');
                    out.push_str(comment);
                }
                out.push('\n');
                replaced = true;
                continue;
            }
        }

        out.push_str(line);
        out.push('\n');
    }

    if replaced {
        std::fs::write(&path, out).with_context(|| format!("write {}", path.display()))?;
    }
    Ok(())
}

/// Extract the `.archipelago` entry from a generated `AP_*.zip` next to the zip itself.
/// MultiServer.py's zip support is flaky depending on the version, so we always feed it
/// the raw `.archipelago` file.
///
/// If the target file already exists, returns its path without re-extracting.
pub fn extract_archipelago(zip_path: &Path) -> Result<PathBuf> {
    let parent = zip_path
        .parent()
        .ok_or_else(|| eyre::eyre!("zip has no parent dir: {}", zip_path.display()))?;

    let file = File::open(zip_path)
        .with_context(|| format!("open zip {}", zip_path.display()))?;
    let mut archive = ZipArchive::new(file)
        .with_context(|| format!("read zip {}", zip_path.display()))?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        if !name.ends_with(".archipelago") {
            continue;
        }
        let target = parent.join(
            Path::new(&name)
                .file_name()
                .map(|s| s.to_os_string())
                .unwrap_or_else(|| name.clone().into()),
        );
        if !target.exists() {
            let mut out = File::create(&target)
                .with_context(|| format!("create {}", target.display()))?;
            io::copy(&mut entry, &mut out)
                .with_context(|| format!("extract to {}", target.display()))?;
        }
        return Ok(target);
    }

    bail!("no .archipelago entry found in {}", zip_path.display())
}

/// Check whether something is listening on `port` on the wildcard interface.
///
/// MultiServer.py binds to `0.0.0.0:port`, so we must try to bind to the same
/// address to detect the conflict. Binding to `127.0.0.1:port` would succeed
/// on Windows even when `0.0.0.0:port` is taken (different address scope).
///
/// We try `0.0.0.0` first, then `127.0.0.1` as a fallback.
///
/// This is intentionally NOT a TCP connect: the websockets server logs an
/// `EOFError` for every TCP open that doesn't speak a valid HTTP upgrade,
/// which would spam the log on every 2.5s status poll.
pub fn ping_local_port(port: u16) -> bool {
    for addr in &[("0.0.0.0", port), ("127.0.0.1", port)] {
        match TcpListener::bind(*addr) {
            Ok(l) => {
                drop(l);
                // We grabbed THIS interface → not in use here, keep checking
            }
            Err(_) => return true, // bind failed → something owns the port
        }
    }
    false
}


/// Best-effort public IP fetch via a few free no-auth endpoints.
/// Synchronous, short timeout. Returns None on any failure.
pub fn detect_public_ip() -> Option<String> {
    use std::time::Duration;

    let endpoints = [
        "https://api.ipify.org",
        "https://ifconfig.me/ip",
        "https://icanhazip.com",
    ];

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .ok()?;

    for url in endpoints {
        if let Ok(resp) = client.get(url).send() {
            if resp.status().is_success() {
                if let Ok(text) = resp.text() {
                    let ip = text.trim().to_string();
                    if !ip.is_empty() && ip.parse::<IpAddr>().is_ok() {
                        return Some(ip);
                    }
                }
            }
        }
    }
    None
}

impl ServerState {
    fn get_public_ip(&self) -> Option<String> {
        if let Ok(guard) = self.cached_public_ip.lock() {
            if let Some(ip) = guard.as_ref() {
                return Some(ip.clone());
            }
        }
        let fetched = detect_public_ip();
        if let (Some(ip), Ok(mut guard)) = (fetched.as_ref(), self.cached_public_ip.lock()) {
            *guard = Some(ip.clone());
        }
        fetched
    }

    pub fn status(&self) -> ServerStatus {
        let local_ip = detect_local_ip();
        let public_ip = self.get_public_ip();

        let mut guard = self.inner.lock().unwrap();
        if let Some(server) = guard.as_mut() {
            // probe child liveness
            match server.child.try_wait() {
                Ok(Some(_)) => {
                    // process exited; clear it
                    *guard = None;
                }
                Ok(None) => {
                    let port = server.port;
                    return ServerStatus {
                        running: true,
                        port: Some(port),
                        password: server.password.clone(),
                        multidata: Some(server.multidata.clone()),
                        pid: Some(server.child.id()),
                        recent_log: self.recent_log_lines(),
                        local_ip,
                        public_ip,
                        port_reachable: ping_local_port(port),
                    };
                }
                Err(_) => {
                    *guard = None;
                }
            }
        }
        ServerStatus {
            running: false,
            port: None,
            password: None,
            multidata: None,
            pid: None,
            recent_log: self.recent_log_lines(),
            local_ip,
            public_ip,
            port_reachable: false,
        }
    }

    fn recent_log_lines(&self) -> Vec<String> {
        self.log_buffer
            .lock()
            .map(|b| b.iter().rev().take(80).cloned().collect::<Vec<_>>())
            .unwrap_or_default()
            .into_iter()
            .rev()
            .collect()
    }

    pub fn start(
        &self,
        app: &AppHandle,
        multidata: &Path,
        port: u16,
        password: Option<String>,
    ) -> Result<ServerStatus> {
        // stop existing
        let _ = self.stop();

        let (python, _) = detect_python(app).ok_or_else(|| {
            eyre::eyre!("python is not installed on this machine; install Python 3.11+ to host a server")
        })?;

        let dir = ap_dir(app);
        if !dir.join("MultiServer.py").exists() {
            bail!("Archipelago MultiServer.py not found at {}", dir.display());
        }
        if !multidata.exists() {
            bail!("multidata file not found: {}", multidata.display());
        }

        // Keep host.yaml in sync with the port we're about to use.
        // CLI --port still wins at runtime, but this ensures any external tool
        // reading host.yaml sees the same port.
        if let Err(err) = update_host_yaml_port(&dir, port) {
            tracing::warn!("failed to update host.yaml port: {err:#}");
        }

        let mut cmd = Command::new(&python);
        cmd.current_dir(&dir)
            .env("SKIP_REQUIREMENTS_UPDATE", "1")
            .env("PYTHONIOENCODING", "utf-8")
            .arg("MultiServer.py")
            .arg(multidata)
            .arg("--port")
            .arg(port.to_string())
            .arg("--loglevel")
            .arg("info")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(pw) = password.as_ref() {
            cmd.arg("--password").arg(pw);
        }

        // Detach from parent so the server survives a Tauri rebuild / Zephyr restart.
        // On Windows: own process group + no console window.
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW);
        }
        // On Unix: own process group via process_group(0) (stable since Rust 1.64).
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }

        let mut child = cmd.spawn().context("failed to spawn MultiServer.py")?;

        // Drain stdout/stderr into the rolling log buffer
        let buf = Arc::clone(&self.log_buffer);
        if let Some(stdout) = child.stdout.take() {
            let buf = Arc::clone(&buf);
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(|l| l.ok()) {
                    info!("AP server: {line}");
                    if let Ok(mut b) = buf.lock() {
                        b.push(line);
                        let len = b.len();
                        if len > 500 {
                            b.drain(0..len - 500);
                        }
                    }
                }
            });
        }
        if let Some(stderr) = child.stderr.take() {
            let buf = Arc::clone(&buf);
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(|l| l.ok()) {
                    warn!("AP server (err): {line}");
                    if let Ok(mut b) = buf.lock() {
                        b.push(format!("[err] {line}"));
                        let len = b.len();
                        if len > 500 {
                            b.drain(0..len - 500);
                        }
                    }
                }
            });
        }

        let pid = child.id();
        let server = RunningServer {
            child,
            port,
            password: password.clone(),
            multidata: multidata.to_string_lossy().to_string(),
        };

        {
            let mut guard = self.inner.lock().unwrap();
            *guard = Some(server);
        }

        Ok(ServerStatus {
            running: true,
            port: Some(port),
            password,
            multidata: Some(multidata.to_string_lossy().to_string()),
            pid: Some(pid),
            recent_log: self.recent_log_lines(),
            local_ip: detect_local_ip(),
            public_ip: self.get_public_ip(),
            port_reachable: ping_local_port(port),
        })
    }

    pub fn stop(&self) -> Result<()> {
        let mut guard = self.inner.lock().unwrap();
        if let Some(mut server) = guard.take() {
            let _ = server.child.kill();
            let _ = server.child.wait();
        }
        Ok(())
    }
}
