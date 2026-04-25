//! Cross-platform extension trait for `std::process::Command` that hides the
//! console window on Windows. Without `CREATE_NO_WINDOW` every Python / pip /
//! PowerShell spawn flashes a black cmd window during runtime install and
//! seed generation.

use std::process::Command;

pub trait CommandExt {
    /// Suppress the console window on Windows. No-op on other platforms.
    fn no_window(&mut self) -> &mut Self;
}

#[cfg(target_os = "windows")]
impl CommandExt for Command {
    fn no_window(&mut self) -> &mut Self {
        use std::os::windows::process::CommandExt as _;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        self.creation_flags(CREATE_NO_WINDOW)
    }
}

#[cfg(not(target_os = "windows"))]
impl CommandExt for Command {
    fn no_window(&mut self) -> &mut Self {
        self
    }
}
