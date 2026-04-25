use std::process::Command;

pub trait CommandExt {
    fn no_window(&mut self) -> &mut Self;
}

#[cfg(target_os = "windows")]
impl CommandExt for Command {
    fn no_window(&mut self) -> &mut Self {
        use std::os::windows::process::CommandExt as _;
        self.creation_flags(0x0800_0000 | 0x0000_0200)
    }
}

#[cfg(not(target_os = "windows"))]
impl CommandExt for Command {
    fn no_window(&mut self) -> &mut Self {
        self
    }
}
