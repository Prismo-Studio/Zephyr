//! Filesystem-related commands: open directories, log files, create shortcuts,
//! load mod-local markdown.

use eyre::{Context, OptionExt};
use tauri::{command, AppHandle};
use uuid::Uuid;

use crate::{
    state::ManagerExt,
    thunderstore::cache::MarkdownKind,
    util::cmd::Result,
};

#[command]
pub fn open_profile_dir(app: AppHandle) -> Result<()> {
    let manager = app.lock_manager();

    let path = &manager.active_profile().path;
    crate::util::fs::open_path(path).context("failed to open directory")?;

    Ok(())
}

#[command]
pub fn open_mod_dir(uuid: Uuid, app: AppHandle) -> Result<()> {
    let manager = app.lock_manager();

    manager.active_profile().open_mod_dir(uuid)?;

    Ok(())
}

#[command]
pub fn open_game_log(app: AppHandle) -> Result<()> {
    let manager = app.lock_manager();

    let path = manager.active_profile().log_path()?;
    open::that_detached(path).context("failed to open log file")?;

    Ok(())
}

#[command]
pub fn create_desktop_shortcut(app: AppHandle) -> Result<()> {
    let manager = app.lock_manager();

    manager.active_game().create_desktop_shortcut()?;

    Ok(())
}

#[command]
pub async fn get_local_markdown(
    uuid: Uuid,
    kind: MarkdownKind,
    app: AppHandle,
) -> Result<Option<String>> {
    let manager = app.lock_manager();
    let profile = manager.active_profile();

    let local_mod = profile.get_mod(uuid).and_then(|profile_mod| {
        profile_mod
            .as_local()
            .map(|(local_mod, _)| local_mod)
            .ok_or_eyre("mod is not local")
    })?;

    let str = match kind {
        MarkdownKind::Readme => &local_mod.readme,
        MarkdownKind::Changelog => &local_mod.changelog,
    };

    Ok(str.clone())
}
