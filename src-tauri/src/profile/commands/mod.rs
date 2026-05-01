//! Tauri commands exposed to the frontend.
//!
//! Split by domain so the public path stays `profile::commands::xxx` (no
//! changes needed in `lib.rs`'s `invoke_handler!`).

mod dirs;
mod game;
mod icons;
mod mgmt;
mod mods;

pub use dirs::*;
pub use game::*;
pub use icons::*;
pub use mgmt::*;
pub use mods::*;
