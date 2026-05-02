//! Per-profile mod commands: query, toggle, remove, reorder, dependants.

use eyre::eyre;
use itertools::Itertools;
use serde::Serialize;
use tauri::{command, AppHandle};
use tracing::warn;
use uuid::Uuid;

use crate::profile::{actions::ActionResult, Dependant, Profile};
use crate::{
    state::ManagerExt,
    thunderstore::{query::QueryModsArgs, FrontendProfileMod, Thunderstore, VersionIdent},
    util::cmd::Result,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendAvailableUpdate {
    full_name: VersionIdent,
    ignore: bool,
    package_uuid: Uuid,
    version_uuid: Uuid,
    old: semver::Version,
    new: semver::Version,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileQuery {
    mods: Vec<FrontendProfileMod>,
    total_mod_count: usize,
    filtered_mod_count: usize,
    updates: Vec<FrontendAvailableUpdate>,
    unknown_mods: Vec<Dependant>,
}

#[command]
pub async fn query_profile(args: QueryModsArgs, app: AppHandle) -> Result<ProfileQuery> {
    tauri::async_runtime::spawn_blocking(move || {
        let manager = app.lock_manager();
        let thunderstore = app.lock_thunderstore();
        let install_queue = app.install_queue().handle();

        let profile = manager.active_profile();

        let (mods, unknown_mods) = profile.query_mods(&args, &thunderstore);
        let total_mod_count = profile.mods.len();

        let mut count_args = args.clone();
        count_args.max_count = usize::MAX;
        let (all_matching, _) = profile.query_mods(&count_args, &thunderstore);
        let filtered_mod_count = all_matching.len();

        let updates = profile
            .mods
            .iter()
            .filter_map(|profile_mod| {
                profile
                    .check_update(profile_mod.uuid(), false, &thunderstore, &install_queue)
                    .transpose()
            })
            .map_ok(|update| {
                let ignore = profile.ignored_updates.contains(&update.latest.uuid);

                FrontendAvailableUpdate {
                    full_name: update.latest.ident.clone(),
                    package_uuid: update.package.uuid,
                    version_uuid: update.latest.uuid,
                    old: update.current.parsed_version(),
                    new: update.latest.parsed_version(),
                    ignore,
                }
            })
            .collect::<eyre::Result<Vec<_>>>()
            .unwrap_or_else(|err| {
                warn!("failed to check for updates: {:#}", err);
                Vec::new()
            });

        Ok(ProfileQuery {
            mods,
            total_mod_count,
            filtered_mod_count,
            updates,
            unknown_mods,
        })
    })
    .await
    .map_err(|err| eyre!("query_profile join error: {err}"))?
}

#[command]
pub fn is_mod_installed(uuid: Uuid, app: AppHandle) -> Result<bool> {
    let manager = app.lock_manager();
    let profile = manager.active_profile();

    let result = profile.has_mod(uuid) || app.install_queue().handle().has_mod(uuid, profile.id);

    Ok(result)
}

#[command]
pub fn remove_mod(uuid: Uuid, app: AppHandle) -> Result<ActionResult> {
    mod_action_command(app, |profile, thunderstore| {
        profile.remove_mod(uuid, thunderstore)
    })
}

#[command]
pub fn toggle_mod(uuid: Uuid, app: AppHandle) -> Result<ActionResult> {
    mod_action_command(app, |profile, thunderstore| {
        profile.toggle_mod(uuid, thunderstore)
    })
}

fn mod_action_command<F>(app: AppHandle, action: F) -> Result<ActionResult>
where
    F: FnOnce(&mut Profile, &Thunderstore) -> eyre::Result<ActionResult>,
{
    let mut manager = app.lock_manager();
    let thunderstore = app.lock_thunderstore();

    let profile = manager.active_profile_mut();
    let response = action(profile, &thunderstore)?;

    if let ActionResult::Done = response {
        profile.save(&app, true)?;
    }

    Ok(response)
}

#[command]
pub fn force_remove_mods(uuids: Vec<Uuid>, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();

    let profile = manager.active_profile_mut();
    for package_uuid in uuids {
        profile.force_remove_mod(package_uuid)?;
    }

    profile.save(&app, true)?;

    Ok(())
}

#[command]
pub fn set_all_mods_state(enable: bool, app: AppHandle) -> Result<usize> {
    let mut manager = app.lock_manager();

    let profile = manager.active_profile_mut();
    let uuids = profile
        .mods
        .iter()
        .filter(|profile_mod| profile_mod.enabled != enable)
        .map(|profile_mod| profile_mod.uuid())
        .collect_vec();

    let count = uuids.len();

    for uuid in uuids {
        profile.force_toggle_mod(uuid)?;
    }

    profile.save(&app, true)?;

    Ok(count)
}

#[command]
pub fn remove_disabled_mods(app: AppHandle) -> Result<usize> {
    let mut manager = app.lock_manager();

    let profile = manager.active_profile_mut();
    let uuids = profile
        .mods
        .iter()
        .filter(|profile_mod| !profile_mod.enabled)
        .map(|profile_mod| profile_mod.uuid())
        .collect_vec();

    let len = uuids.len();

    for uuid in uuids {
        profile.force_remove_mod(uuid)?;
    }

    profile.save(&app, true)?;

    Ok(len)
}

#[command]
pub fn force_toggle_mods(uuids: Vec<Uuid>, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();

    let profile = manager.active_profile_mut();
    for package_uuid in uuids {
        profile.force_toggle_mod(package_uuid)?;
    }

    profile.save(&app, true)?;

    Ok(())
}

#[command]
pub fn reorder_mod(uuid: Uuid, delta: i32, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();
    let profile = manager.active_profile_mut();
    profile.reorder_mod(uuid, delta)?;
    profile.save(&app, true)?;
    Ok(())
}

#[command]
pub fn get_dependants(uuid: Uuid, app: AppHandle) -> Result<Vec<VersionIdent>> {
    let manager = app.lock_manager();
    let thunderstore = app.lock_thunderstore();

    let dependants = manager
        .active_profile()
        .dependants(uuid, &thunderstore)
        .map(|profile_mod| profile_mod.ident().into_owned())
        .collect();

    Ok(dependants)
}
