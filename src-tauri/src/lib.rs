use std::env;

use itertools::Itertools;
use state::ManagerExt;
use tauri::{App, AppHandle, RunEvent};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_dialog::DialogExt;
use tracing::{error, info, warn};

#[cfg(target_os = "linux")]
extern crate webkit2gtk;
#[cfg(target_os = "linux")]
use gtk::prelude::GtkWindowExt;

mod cli;
mod config;
mod console;
mod db;
mod deep_link;
mod game;
mod icon_cache;
mod logger;
mod prefs;
mod profile;
mod randomizer;
pub mod source;
mod state;
mod thunderstore;
mod util;

fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "zephyr v{} running on {}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
    );

    // On Linux (Wayland + X11), GTK can ignore the `decorations: false` config and
    // draw its own title bar. Calling set_decorated(false) on the underlying GtkWindow
    // directly is the reliable fix.
    #[cfg(target_os = "linux")]
    {
        use tauri::Manager;
        if let Some(window) = app.get_webview_window("main") {
            if let Ok(gtk_window) = window.gtk_window() {
                gtk_window.set_decorated(false);
            }
        }
    }

    use tauri::Manager;
    app.manage(crate::randomizer::ap_runner::ServerState::default());

    if let Err(err) = state::setup(app.handle()) {
        error!("setup error: {:?}", err);

        app.dialog()
            .message(format!("Failed to launch Zephyr: {err:?}"))
            .blocking_show();

        return Err(err.into());
    }

    if let Err(err) = app.deep_link().register("ror2mm") {
        warn!("failed to register ror2mm deep link protocol: {:#}", err);
    }

    if let Err(err) = app.deep_link().register("zephyr") {
        warn!("failed to register zephyr deep link protocol: {:#}", err);
    }

    // Apple Events only matter on macOS where browsers deliver custom URL
    // schemes via NSAppleEventManager rather than argv. On Windows and Linux,
    // tauri-plugin-single-instance already handles the deep link via argv,
    // so registering on_open_url there would double-fire the callback.
    #[cfg(target_os = "macos")]
    {
        let handle = app.handle().to_owned();
        app.deep_link().on_open_url(move |event| {
            for url in event.urls() {
                deep_link::handle(&handle, vec![String::from("zephyr"), url.to_string()]);
            }
        });
    }

    let args = env::args().collect_vec();
    if !args.is_empty() && !deep_link::handle(app.handle(), args.clone()) {
        cli::run(args, app.handle());
    }

    let handle = app.handle().to_owned();
    tauri::async_runtime::spawn(async move {
        tokio::task::spawn_blocking(move || {
            handle
                .db()
                .evict_outdated_cache()
                .unwrap_or_else(|err| warn!("failed to evict outdated cache: {err:#}"))
        })
        .await
    });

    let handle = app.handle().to_owned();
    tauri::async_runtime::spawn(async move {
        if let Err(err) = game::update_list_task(&handle).await {
            warn!("failed to update games list: {err}");
        }
    });

    #[cfg(debug_assertions)]
    {
        use tauri::Manager;
        let handle = app.handle().to_owned();
        tauri::async_runtime::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                if let Some(w) = handle.get_webview_window("main") {
                    w.close_devtools();
                }
            }
        });
    }

    info!("setup done");

    Ok(())
}

fn event_handler(app: &AppHandle, event: RunEvent) {
    if let RunEvent::ExitRequested { api, .. } = event {
        if !app.install_queue().handle().is_processing() {
            return;
        }

        api.prevent_exit();

        tauri::async_runtime::spawn(profile::install::handle_exit(app.to_owned()));
    }
}

fn handle_single_instance(app: &AppHandle, args: Vec<String>, _cwd: String) {
    if !deep_link::handle(app, args.clone()) {
        cli::run(args, app);
    }
}

pub fn run() {
    // Workaround for WebKitGTK issues on Linux (Wayland crash, blank window on GPU)
    // These env vars fix rendering on CachyOS, Hyprland, and similar compositors
    #[cfg(target_os = "linux")]
    {
        // Fix blank/empty WebView on newer WebKitGTK with DMA-BUF renderer
        if env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_err() {
            env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
        // Disable GPU compositing which can cause rendering issues
        if env::var("WEBKIT_DISABLE_COMPOSITING_MODE").is_err() {
            env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        }
        // Disable GTK client-side decorations (CSD) so the native title bar
        // doesn't appear on Wayland/X11 when decorations: false is set in tauri.conf.json.
        // Without this, GTK falls back to drawing its own CSD title bar.
        if env::var("GTK_CSD").is_err() {
            env::set_var("GTK_CSD", "0");
        }
        // Force GDK to use the X11 backend on Wayland (via XWayland).
        // WebKitGTK's native Wayland backend has known issues rendering images/logos,
        // causing blank/missing visuals. XWayland provides a more stable rendering path.
        if env::var("GDK_BACKEND").is_err() {
            env::set_var("GDK_BACKEND", "x11");
        }
    }

    logger::setup().unwrap_or_else(|err| {
        eprintln!("failed to set up logger: {err:#}");
    });

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            logger::open_zephyr_log,
            logger::log_err,
            source::commands::get_sources,
            source::commands::search_sources,
            source::commands::get_source_mod_description,
            source::commands::get_source_mod_changelog,
            source::commands::get_nexusmods_games,
            source::commands::get_curseforge_games,
            state::is_first_run,
            thunderstore::commands::query_thunderstore,
            thunderstore::commands::stop_querying_thunderstore,
            thunderstore::commands::get_markdown,
            thunderstore::commands::set_thunderstore_token,
            thunderstore::commands::has_thunderstore_token,
            thunderstore::commands::clear_thunderstore_token,
            thunderstore::commands::trigger_mod_fetch,
            prefs::commands::get_prefs,
            prefs::commands::set_prefs,
            prefs::commands::zoom_window,
            prefs::commands::set_dpi_scale,
            prefs::commands::get_system_fonts,
            prefs::commands::open_dir,
            prefs::commands::upload_custom_background,
            prefs::commands::probe_custom_background,
            icon_cache::get_cached_icon,
            icon_cache::clear_icon_cache,
            profile::commands::get_game_info,
            profile::commands::favorite_game,
            profile::commands::set_active_game,
            profile::commands::get_profile_info,
            profile::commands::get_all_sync_ids,
            profile::commands::set_active_profile,
            profile::commands::reorder_profile,
            profile::commands::is_mod_installed,
            profile::commands::query_profile,
            profile::commands::get_dependants,
            profile::commands::create_profile,
            profile::commands::delete_profile,
            profile::commands::rename_profile,
            profile::commands::duplicate_profile,
            profile::commands::remove_mod,
            profile::commands::force_remove_mods,
            profile::commands::toggle_mod,
            profile::commands::force_toggle_mods,
            profile::commands::reorder_mod,
            profile::commands::set_all_mods_state,
            profile::commands::remove_disabled_mods,
            profile::commands::open_profile_dir,
            profile::commands::open_mod_dir,
            profile::commands::open_game_log,
            profile::commands::create_desktop_shortcut,
            profile::commands::get_local_markdown,
            profile::commands::set_custom_args,
            profile::commands::set_profile_path,
            profile::commands::forget_profile,
            profile::commands::set_profile_icon,
            profile::commands::set_profile_icon_url,
            profile::commands::upload_profile_icon,
            profile::commands::remove_profile_icon,
            profile::launch::commands::launch_game,
            profile::launch::commands::get_launch_args,
            profile::launch::commands::open_game_dir,
            profile::launch::commands::get_game_dir,
            profile::launch::commands::launch_vanilla,
            profile::install::commands::install_all_mods,
            profile::install::commands::install_mod,
            profile::install::commands::cancel_all_installs,
            profile::install::commands::has_pending_installations,
            profile::install::commands::clear_download_cache,
            profile::install::commands::get_download_size,
            profile::update::commands::change_mod_version,
            profile::update::commands::update_mods,
            profile::update::commands::ignore_update,
            profile::import::commands::import_profile,
            profile::import::commands::read_profile_code,
            profile::import::commands::read_profile_file,
            profile::import::commands::read_profile_base64,
            profile::import::commands::import_local_mod,
            profile::import::commands::import_local_mod_base64,
            profile::import::commands::get_r2modman_info,
            profile::import::commands::import_r2modman,
            profile::export::commands::export_code,
            profile::export::commands::export_file,
            profile::export::commands::export_pack,
            profile::export::commands::upload_pack,
            profile::export::commands::get_pack_args,
            profile::export::commands::set_pack_args,
            profile::export::commands::generate_changelog,
            profile::export::commands::copy_dependency_strings,
            profile::export::commands::copy_debug_info,
            profile::sync::commands::read_sync_profile,
            profile::sync::commands::create_sync_profile,
            profile::sync::commands::disconnect_sync_profile,
            profile::sync::commands::delete_sync_profile,
            profile::sync::commands::push_sync_profile,
            profile::sync::commands::clone_sync_profile,
            profile::sync::commands::pull_sync_profile,
            profile::sync::commands::fetch_sync_profile,
            profile::sync::commands::get_owned_sync_profiles,
            profile::sync::commands::login,
            profile::sync::commands::logout,
            profile::sync::commands::get_user,
            config::commands::get_config_files,
            config::commands::set_config_entry,
            config::commands::reset_config_entry,
            config::commands::open_config_file,
            config::commands::delete_config_file,
            randomizer::commands::list_supported_games,
            randomizer::commands::get_game_schema,
            randomizer::commands::generate_yaml,
            randomizer::commands::validate_config,
            randomizer::commands::lint_yaml,
            randomizer::commands::check_python,
            randomizer::commands::save_player_yaml,
            randomizer::commands::list_player_yamls,
            randomizer::commands::delete_player_yaml,
            randomizer::commands::rename_player_yaml,
            randomizer::commands::generate_seed,
            randomizer::commands::start_server,
            randomizer::commands::stop_server,
            randomizer::commands::server_status,
            randomizer::commands::open_workspace_dir,
            randomizer::commands::list_seeds,
            randomizer::commands::delete_seed,
            randomizer::commands::rename_seed,
            randomizer::commands::clear_seeds,
            randomizer::commands::read_file_base64,
            randomizer::commands::remote_upload_seed,
            randomizer::commands::remote_request,
            randomizer::commands::list_custom_apworlds,
            randomizer::commands::install_apworld_from_path,
            randomizer::commands::install_apworld_from_bytes,
            randomizer::commands::remove_custom_apworld,
            randomizer::commands::refresh_apworld_schemas,
            randomizer::commands::open_custom_worlds_dir,
            randomizer::commands::runtime_status,
            randomizer::commands::install_runtime,
            randomizer::commands::provision_runtime_venv,
            randomizer::commands::remove_runtime,
            randomizer::commands::list_patches,
            randomizer::commands::delete_patch,
            randomizer::commands::apply_patch,
            randomizer::commands::launch_ap_component,
            randomizer::commands::get_rom_paths,
            randomizer::commands::set_rom_path,
            randomizer::commands::clear_rom_path,
            console::commands::console_server_send_stdin,
            console::commands::console_server_recent_log,
            console::commands::open_console_window,
        ])
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        // TODO .plugin(tauri_plugin_oauth::Builder)
        .plugin(tauri_plugin_single_instance::init(handle_single_instance))
        .setup(setup)
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(event_handler);
}
