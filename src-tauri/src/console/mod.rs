//! Zephyr Console. Backend surface.
//!
//! Phase 1 scope: wrap the existing `ap_runner::ServerState` with two
//! additional IPC commands so the Svelte Console can (a) write admin commands
//! to `MultiServer.py`'s stdin and (b) backfill its feed with the most recent
//! log lines on mount. Live log streaming is done via the
//! `"console://server-log"` Tauri event emitted from `ap_runner::start`. No
//! new channel needed here.
//!
//! Phase 2 will add `client.rs` (a WebSocket AP client) and `protocol.rs`
//! (Rust-side AP packet types). They're intentionally absent from this slice.

pub mod commands;
