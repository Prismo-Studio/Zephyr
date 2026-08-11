use std::{
    collections::HashMap,
    io::Write,
    path::PathBuf,
    process::{Child, Stdio},
    sync::Mutex,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use eyre::{eyre, Context, Result};
use ffmpeg_sidecar::command::FfmpegCommand;
use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Manager};
use tracing::{info, warn};

use crate::util::cmd::Result as CmdResult;

const QUALITY_HEIGHTS: &[(&str, u32)] = &[
    ("720p", 720),
    ("1080p", 1080),
    ("1440p", 1440),
    ("2160p", 2160),
];

#[derive(Default)]
pub struct RecordingState {
    sessions: Mutex<HashMap<String, Session>>,
}

struct Session {
    filename: String,
    output_path: PathBuf,
    child: Child,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartArgs {
    pub plugin_id: String,
    pub filename: String,
    #[serde(default = "default_fps")]
    pub fps: u32,
    #[serde(default = "default_quality")]
    pub quality: String,
    #[serde(default)]
    pub window_title: Option<String>,
    #[serde(default)]
    pub with_audio: bool,
}

fn default_fps() -> u32 {
    60
}
fn default_quality() -> String {
    "1080p".to_string()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartResult {
    pub session_id: String,
    pub filename: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StopResult {
    pub filename: String,
    pub size: u64,
}

fn ensure_ffmpeg() -> Result<()> {
    if ffmpeg_sidecar::command::ffmpeg_is_installed() {
        return Ok(());
    }
    info!("ffmpeg not found on PATH, downloading bundled build");
    ffmpeg_sidecar::download::auto_download().map_err(|e| eyre!("download ffmpeg: {e}"))?;
    Ok(())
}

fn safe_filename(name: &str) -> Result<String> {
    if name.is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name == "."
        || name == ".."
    {
        eyre::bail!("invalid filename");
    }
    Ok(name.to_string())
}

fn plugin_files_dir(id: &str) -> PathBuf {
    crate::util::path::default_app_data_dir()
        .join(super::commands::STORAGE_DIR_NAME)
        .join(id)
        .join("files")
}

#[cfg(target_os = "windows")]
fn resolve_window_title(query: &str) -> Option<String> {
    use std::sync::Mutex;
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible,
    };

    struct Ctx {
        query_lc: String,
        best: Option<String>,
    }
    let ctx = Mutex::new(Ctx {
        query_lc: query.to_lowercase(),
        best: None,
    });

    unsafe extern "system" fn cb(hwnd: HWND, lp: LPARAM) -> BOOL {
        let ctx = unsafe { &*(lp.0 as *const Mutex<Ctx>) };
        if unsafe { !IsWindowVisible(hwnd).as_bool() } {
            return BOOL(1);
        }
        let len = unsafe { GetWindowTextLengthW(hwnd) };
        if len <= 0 {
            return BOOL(1);
        }
        let mut buf = vec![0u16; (len + 1) as usize];
        let n = unsafe { GetWindowTextW(hwnd, &mut buf) };
        if n <= 0 {
            return BOOL(1);
        }
        let title = String::from_utf16_lossy(&buf[..n as usize]);
        let mut guard = ctx.lock().unwrap();
        if title.to_lowercase().contains(&guard.query_lc) {
            guard.best = Some(title);
            return BOOL(0);
        }
        BOOL(1)
    }

    let ptr = &ctx as *const Mutex<Ctx> as isize;
    unsafe {
        let _ = EnumWindows(Some(cb), LPARAM(ptr));
    }
    ctx.into_inner().ok().and_then(|c| c.best)
}

#[cfg(not(target_os = "windows"))]
fn resolve_window_title(_query: &str) -> Option<String> {
    None
}

pub fn spawn_exit_watcher(app: AppHandle, game_id: String, game_name: String) {
    std::thread::spawn(move || {
        use tauri::Emitter;

        let appear_deadline = Instant::now() + Duration::from_secs(120);
        let mut appeared = false;
        loop {
            let exists = resolve_window_title(&game_name).is_some();
            if exists {
                appeared = true;
                std::thread::sleep(Duration::from_secs(2));
                continue;
            }
            if appeared {
                let _ = app.emit(
                    "zephyr_game_exited",
                    serde_json::json!({ "gameId": game_id, "gameName": game_name }),
                );
                return;
            }
            if Instant::now() >= appear_deadline {
                return;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    });
}

fn wait_for_window(query: &str, timeout: Duration) -> Result<String> {
    #[cfg(not(target_os = "windows"))]
    {
        let _ = timeout;
        return Ok(query.to_string());
    }
    #[cfg(target_os = "windows")]
    {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(title) = resolve_window_title(query) {
                return Ok(title);
            }
            if Instant::now() >= deadline {
                eyre::bail!(
                    "game window '{}' did not appear within {:?}",
                    query,
                    timeout
                );
            }
            std::thread::sleep(Duration::from_millis(400));
        }
    }
}

fn target_height(quality: &str) -> u32 {
    QUALITY_HEIGHTS
        .iter()
        .find(|(name, _)| *name == quality)
        .map(|(_, h)| *h)
        .unwrap_or(1080)
}

fn make_session_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    let salt = (nanos as u64).wrapping_mul(2654435761) ^ (pid as u64).wrapping_mul(2246822519);
    format!("{:016x}{:08x}", salt, pid)
}

#[command]
pub fn plugin_recording_start(args: StartArgs, app: AppHandle) -> CmdResult<StartResult> {
    ensure_ffmpeg().map_err(|e| eyre!(e))?;

    let safe = safe_filename(&args.filename)?;
    let dir = plugin_files_dir(&args.plugin_id);
    std::fs::create_dir_all(&dir).context("create plugin storage dir")?;
    let output_path = dir.join(&safe);

    let height = target_height(&args.quality);
    let scale = format!("scale=-2:{}", height);

    let resolved_title = match args.window_title.as_deref() {
        Some(q) if !q.is_empty() => Some(wait_for_window(q, Duration::from_secs(60))?),
        _ => None,
    };

    let video_input = match resolved_title.as_deref() {
        Some(title) if !title.is_empty() => format!("title={}", title),
        _ => "desktop".to_string(),
    };

    info!("recording target: {}", video_input);

    let mut cmd = FfmpegCommand::new();

    #[cfg(target_os = "windows")]
    {
        cmd.format("gdigrab")
            .args(["-framerate", &args.fps.to_string()])
            .input(&video_input);

        if args.with_audio {
            cmd.format("dshow")
                .input("audio=virtual-audio-capturer");
        }
    }

    #[cfg(target_os = "macos")]
    {
        let _ = &video_input;
        cmd.format("avfoundation")
            .args(["-framerate", &args.fps.to_string()])
            .input(if args.with_audio { "1:0" } else { "1:none" });
    }

    #[cfg(target_os = "linux")]
    {
        let _ = &video_input;
        cmd.format("x11grab")
            .args(["-framerate", &args.fps.to_string()])
            .input(":0.0");
        if args.with_audio {
            cmd.format("pulse").input("default");
        }
    }

    cmd.args(["-vf", &scale])
        .args(["-c:v", "libx264"])
        .args(["-preset", "ultrafast"])
        .args(["-pix_fmt", "yuv420p"])
        .args(["-y"])
        .output(output_path.to_string_lossy().as_ref());

    let child = cmd
        .as_inner_mut()
        .stdin(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("spawn ffmpeg")?;

    let session_id = make_session_id();
    let state = app.state::<RecordingState>();
    state
        .sessions
        .lock()
        .map_err(|_| eyre!("recording state poisoned"))?
        .insert(
            session_id.clone(),
            Session {
                filename: safe.clone(),
                output_path,
                child,
            },
        );

    info!("recording started: session={} file={}", session_id, safe);
    Ok(StartResult {
        session_id,
        filename: safe,
    })
}

#[command]
pub fn plugin_recording_stop(session_id: String, app: AppHandle) -> CmdResult<StopResult> {
    let state = app.state::<RecordingState>();
    let mut session = state
        .sessions
        .lock()
        .map_err(|_| eyre!("recording state poisoned"))?
        .remove(&session_id)
        .ok_or_else(|| eyre!("unknown session id"))?;

    if let Some(stdin) = session.child.stdin.as_mut() {
        let _ = stdin.write_all(b"q");
        let _ = stdin.flush();
    }

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match session.child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(100));
            }
            _ => {
                warn!("ffmpeg did not exit cleanly within 5s, killing");
                let _ = session.child.kill();
                let _ = session.child.wait();
                break;
            }
        }
    }

    let size = std::fs::metadata(&session.output_path)
        .map(|m| m.len())
        .unwrap_or(0);

    info!(
        "recording stopped: session={} path={} bytes={}",
        session_id,
        session.output_path.display(),
        size
    );

    if size == 0 {
        let _ = std::fs::remove_file(&session.output_path);
        return Err(eyre!(
            "recording produced no data. Check the Zephyr console for ffmpeg errors. Most likely the target window was not visible or gdigrab failed."
        )
        .into());
    }

    Ok(StopResult {
        filename: session.filename,
        size,
    })
}
