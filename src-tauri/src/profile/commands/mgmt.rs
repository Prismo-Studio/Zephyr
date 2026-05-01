//! Profile lifecycle commands: list, switch, create/delete/rename/duplicate,
//! plus path management and forget.

use std::path::PathBuf;

use eyre::eyre;
use tauri::{command, AppHandle};

use crate::{
    profile::FrontendManagedGame,
    state::ManagerExt,
    util::cmd::Result,
};

#[command]
pub fn get_profile_info(app: AppHandle) -> FrontendManagedGame {
    let manager = app.lock_manager();

    manager.active_game().to_frontend()
}

#[command]
pub fn get_all_sync_ids(app: AppHandle) -> Vec<String> {
    let manager = app.lock_manager();
    manager
        .games
        .values()
        .flat_map(|game| game.profiles.iter())
        .filter_map(|profile| profile.sync.as_ref().map(|s| s.id().to_string()))
        .collect()
}

#[command]
pub async fn set_active_profile(index: usize, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();

    let game = manager.active_game_mut();

    app.sync_socket().unsubscribe(game.active_profile());

    game.set_active_profile(index)?;

    app.sync_socket().subscribe(game.active_profile());

    game.save(&app)?;
    game.update_window_title(&app)?;

    Ok(())
}

#[command]
pub async fn reorder_profile(from: usize, to: usize, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();
    let game = manager.active_game_mut();

    if from >= game.profiles.len() || to >= game.profiles.len() || from == to {
        return Ok(());
    }

    let profile = game.profiles.remove(from);
    game.profiles.insert(to, profile);

    game.save(&app)?;
    Ok(())
}

#[command]
pub fn create_profile(name: String, override_path: Option<PathBuf>, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();
    let game = manager.active_game_mut();

    let profile = game.create_profile(name, override_path, app.db())?;

    profile.save(&app, false)?;
    game.save(&app)?;

    game.update_window_title(&app)?;

    Ok(())
}

#[command]
pub fn delete_profile(id: i64, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();
    let game = manager.active_game_mut();

    game.delete_profile(id, false, app.db())?;
    game.save(&app)?;

    game.update_window_title(&app)?;

    Ok(())
}

#[command]
pub fn rename_profile(name: String, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();
    let game = manager.active_game_mut();

    let profile = game.active_profile_mut();

    profile.rename(name)?;
    profile.save(&app, true)?;

    game.update_window_title(&app)?;

    Ok(())
}

#[command]
pub fn duplicate_profile(name: String, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();
    let game = manager.active_game_mut();

    let profile = game.duplicate_profile(name, game.active_profile_id, app.db())?;

    profile.save(&app, false)?;
    game.save(&app)?;

    game.update_window_title(&app)?;

    Ok(())
}

#[command]
pub fn set_custom_args(custom_args: Vec<String>, enabled: bool, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();
    let profile = manager.active_profile_mut();
    profile.custom_args = custom_args;
    profile.custom_args_enabled = enabled;
    profile.save(&app, false)?;
    manager.save_active_game(&app)?;
    Ok(())
}

#[command]
pub fn set_profile_path(new_path: PathBuf, profile_id: i64, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();
    let game = manager.active_game_mut();

    if !new_path.is_dir() {
        return Err(eyre!("provided path is not a directory").into());
    }

    let profile = game.profile_mut(profile_id)?;
    profile.path = new_path;
    profile.missing = false;

    profile.save(&app, true)?;

    Ok(())
}

#[command]
pub fn forget_profile(profile_id: i64, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();
    let game = manager.active_game_mut();

    game.forget_profile(profile_id, app.db())?;

    game.save(&app)?;

    Ok(())
}
