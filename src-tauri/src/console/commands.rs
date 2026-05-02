//! IPC commands for the Zephyr Console.

use tauri::{command, AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::randomizer::ap_runner::ServerState;
use crate::util::cmd::Result;

/// Write a line to the running MultiServer's stdin. The caller is expected to
/// have already prefixed it correctly (e.g. `"/players"`). A trailing `\n` is
/// appended automatically if missing.
///
/// Fails if no server is running or the pipe has been closed. Error is a plain
/// string, surfaced as a toast on the frontend.
#[command]
pub fn console_server_send_stdin(app: AppHandle, line: String) -> Result<()> {
    let state = app.state::<ServerState>();
    state.send_line(&line)?;
    Ok(())
}

/// Snapshot the most recent log lines (oldest first, capped internally at ~80).
/// Used by the Console when it first mounts to backfill the feed. After that,
/// live updates arrive via the `"console://server-log"` Tauri event.
#[command]
pub fn console_server_recent_log(app: AppHandle) -> Vec<String> {
    let state = app.state::<ServerState>();
    state.recent_log_snapshot()
}

/// Open the Zephyr Console in a **standalone companion window**, Archipelago
/// TextClient-style. The window uses native OS decorations (a real title bar)
/// unlike the main Zephyr window, because this one is a secondary tool surface
///. It needs to be close/minimize/resize-familiar, not pretty.
///
/// Single-instance: calling this while the window is already open focuses the
/// existing one instead of spawning a duplicate.
///
/// The window's URL ends with `?standalone=1` so the front-end knows to hide
/// the main-app sidebar/chrome and render the Console full-bleed.
#[command]
pub async fn open_console_window(app: AppHandle) -> Result<()> {
    const LABEL: &str = "console";
    if let Some(existing) = app.get_webview_window(LABEL) {
        let _ = existing.show();
        let _ = existing.unminimize();
        let _ = existing.set_focus();
        return Ok(());
    }

    let url = WebviewUrl::App("/randomizer/console?standalone=1".into());
    let builder = WebviewWindowBuilder::new(&app, LABEL, url)
        .title("Zephyr Console")
        .inner_size(1100.0, 700.0)
        .min_inner_size(780.0, 440.0)
        // Match the main window: custom Zephyr titlebar, not the OS one. The
        // Titlebar Svelte component flips this back on if the user has the
        // "native titlebar" preference enabled.
        .decorations(false)
        .resizable(true);

    builder
        .build()
        .map_err(|e| eyre::eyre!("failed to open console window: {e}"))?;
    Ok(())
}
