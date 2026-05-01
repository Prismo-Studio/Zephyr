//! Game-level commands: list/select games, favorite toggle.

use chrono::{DateTime, Utc};
use eyre::OptionExt;
use itertools::Itertools;
use serde::Serialize;
use tauri::{command, AppHandle};

use crate::{
    game::{self, platform::Platform, Game},
    state::ManagerExt,
    util::cmd::Result,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendGame {
    name: &'static str,
    slug: &'static str,
    popular: bool,
    mod_loader: &'static str,
    platforms: Vec<Platform>,
}

impl From<Game> for FrontendGame {
    fn from(value: Game) -> Self {
        let platforms = value.platforms.iter().collect();

        Self {
            name: value.name,
            slug: &*value.slug,
            popular: value.popular,
            mod_loader: value.mod_loader.as_str(),
            platforms,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInfo {
    all: Vec<FrontendGame>,
    last_updated: DateTime<Utc>,
    active: FrontendGame,
    favorites: Vec<&'static str>,
}

#[command]
pub fn get_game_info(app: AppHandle) -> GameInfo {
    let manager = app.lock_manager();

    let favorites = manager
        .games
        .iter()
        .filter_map(|(game, managed_game)| match managed_game.favorite {
            true => Some(&*game.slug),
            false => None,
        })
        .collect();

    GameInfo {
        all: game::list().map_into().collect(),
        last_updated: game::last_updated(),
        active: manager.active_game.into(),
        favorites,
    }
}

#[command]
pub fn favorite_game(slug: String, app: AppHandle) -> Result<()> {
    let prefs = app.lock_prefs();
    let mut manager = app.lock_manager();

    let game = game::from_slug(&slug).ok_or_eyre("unknown game")?;
    let managed_game = manager.ensure_game(game, false, &prefs, app.db())?;
    managed_game.favorite = !managed_game.favorite;

    managed_game.save(&app)?;

    Ok(())
}

#[command]
pub fn set_active_game(slug: &str, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();

    let game = game::from_slug(slug).ok_or_eyre("unknown game")?;

    app.sync_socket().unsubscribe(manager.active_profile());

    let managed_game = manager.set_active_game(game, &app)?;

    app.sync_socket().subscribe(managed_game.active_profile());

    managed_game.update_window_title(&app)?;
    manager.save_all(&app)?;

    Ok(())
}
