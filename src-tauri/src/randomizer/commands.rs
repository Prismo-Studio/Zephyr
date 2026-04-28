use std::path::PathBuf;

use tauri::{command, AppHandle, Manager};

use super::{
    ap_runner::{
        self, GenerateOutcome, PlayerFile, PythonStatus, SeedFile, ServerState, ServerStatus,
    },
    apworlds::{self, CustomApworld, RefreshResult},
    patches::{self, PatchFile},
    runtime::{self, RuntimeStatus},
    schema,
    types::*,
    validation, yaml_gen,
    yaml_gen::LintIssue,
};
use crate::util::cmd::Result;

async fn blocking<F, T>(f: F) -> Result<T>
where
    F: FnOnce() -> eyre::Result<T> + Send + 'static,
    T: Send + 'static,
{
    let res = tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| eyre::eyre!("join error: {e}"))?;
    Ok(res?)
}

#[command]
pub async fn list_supported_games(app: AppHandle) -> Result<Vec<GameSummary>> {
    blocking(move || {
        if let Err(err) = apworlds::prune_orphan_schemas(&app) {
            tracing::warn!("prune_orphan_schemas failed: {err:#}");
        }
        let schemas = schema::load_all_schemas_merged(&app)?;
        Ok(schemas.iter().map(schema::summarize).collect())
    })
    .await
}

#[command]
pub async fn get_game_schema(app: AppHandle, game_id: String) -> Result<GameSchema> {
    blocking(move || schema::load_schema_by_id_merged(&app, &game_id)).await
}

#[command]
pub async fn generate_yaml(app: AppHandle, config: RandomizerConfig) -> Result<String> {
    blocking(move || {
        let schema = schema::load_schema_by_id_merged(&app, &config.game_id)?;
        yaml_gen::generate(&schema, &config)
    })
    .await
}

#[command]
pub async fn validate_config(
    app: AppHandle,
    config: RandomizerConfig,
) -> Result<Vec<ValidationError>> {
    blocking(move || {
        let schema = schema::load_schema_by_id_merged(&app, &config.game_id)?;
        Ok(validation::validate(&schema, &config))
    })
    .await
}

#[command]
pub async fn lint_yaml(
    app: AppHandle,
    game_id: String,
    yaml: String,
) -> Result<Vec<LintIssue>> {
    blocking(move || {
        let schema = schema::load_schema_by_id_merged(&app, &game_id)?;
        Ok(yaml_gen::lint(&yaml, &schema))
    })
    .await
}

#[command]
pub async fn check_python(app: AppHandle) -> Result<PythonStatus> {
    blocking(move || Ok(ap_runner::check_python(&app))).await
}

#[command]
pub fn save_player_yaml(app: AppHandle, slot_name: String, yaml: String) -> Result<String> {
    let path = ap_runner::save_player_yaml(&app, &slot_name, &yaml)?;
    Ok(path.to_string_lossy().to_string())
}

#[command]
pub async fn list_player_yamls(app: AppHandle) -> Result<Vec<PlayerFile>> {
    blocking(move || ap_runner::list_player_yamls(&app)).await
}

#[command]
pub fn delete_player_yaml(path: String) -> Result<()> {
    ap_runner::delete_player_yaml(&PathBuf::from(path))?;
    Ok(())
}

#[command]
pub fn rename_player_yaml(path: String, new_name: String) -> Result<String> {
    let new_path = ap_runner::rename_player_yaml(&PathBuf::from(path), &new_name)?;
    Ok(new_path.to_string_lossy().to_string())
}

#[command]
pub async fn generate_seed(app: AppHandle) -> Result<GenerateOutcome> {
    blocking(move || ap_runner::run_generate(&app)).await
}

#[command]
pub fn start_server(
    app: AppHandle,
    multidata: String,
    port: u16,
    password: Option<String>,
) -> Result<ServerStatus> {
    let state = app.state::<ServerState>();
    Ok(state.start(&app, &PathBuf::from(multidata), port, password)?)
}

#[command]
pub fn stop_server(app: AppHandle) -> Result<()> {
    let state = app.state::<ServerState>();
    state.stop()?;
    Ok(())
}

#[command]
pub fn server_status(app: AppHandle) -> ServerStatus {
    let state = app.state::<ServerState>();
    state.status()
}

#[command]
pub fn open_workspace_dir(app: AppHandle) -> Result<()> {
    let dir = ap_runner::workspace_dir(&app);
    std::fs::create_dir_all(&dir)?;
    crate::util::fs::open_path(dir)?;
    Ok(())
}

#[command]
pub async fn list_seeds(app: AppHandle) -> Result<Vec<SeedFile>> {
    blocking(move || ap_runner::list_seeds(&app)).await
}

#[command]
pub fn delete_seed(path: String) -> Result<()> {
    ap_runner::delete_seed(&PathBuf::from(path))?;
    Ok(())
}

#[command]
pub fn rename_seed(path: String, new_name: String) -> Result<String> {
    let new_path = ap_runner::rename_seed(&PathBuf::from(path), &new_name)?;
    Ok(new_path.to_string_lossy().to_string())
}

#[command]
pub fn clear_seeds(app: AppHandle) -> Result<usize> {
    Ok(ap_runner::clear_seeds(&app)?)
}

#[command]
pub fn read_file_base64(path: String) -> Result<String> {
    use base64::Engine;
    let bytes = std::fs::read(&path)
        .map_err(|e| eyre::eyre!("read {path}: {e}"))?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
}

#[derive(serde::Serialize)]
pub struct ArchipelagoGgRoom {
    pub room_id: String,
    pub room_url: String,
    pub tracker_url: Option<String>,
    pub host: String,
    pub port: u16,
}

async fn archipelago_gg_host_inner(path: String) -> eyre::Result<ArchipelagoGgRoom> {
    use eyre::{eyre, Context};

    let bytes = std::fs::read(&path).with_context(|| format!("read {path}"))?;
    let file_name = std::path::Path::new(&path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("seed.archipelago")
        .to_string();

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("Zephyr/1 (archipelago.gg-client)")
        .build()
        .context("build http client")?;

    let part = reqwest::multipart::Part::bytes(bytes)
        .file_name(file_name)
        .mime_str("application/zip")
        .context("mime")?;
    let form = reqwest::multipart::Form::new().part("file", part);

    let resp = client
        .post("https://archipelago.gg/uploads")
        .multipart(form)
        .send()
        .await
        .context("upload request")?;

    if !resp.status().is_redirection() && !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(eyre!("archipelago.gg upload failed: HTTP {status}\n{body}"));
    }

    let upload_location = resp
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| eyre!("archipelago.gg upload: missing Location header"))?
        .to_string();

    let seed_id = upload_location
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .ok_or_else(|| eyre!("cannot parse seed id from {upload_location}"))?
        .to_string();

    let resp = client
        .get(format!("https://archipelago.gg/new_room/{seed_id}"))
        .send()
        .await
        .context("new_room request")?;

    if !resp.status().is_redirection() && !resp.status().is_success() {
        let status = resp.status();
        return Err(eyre!("archipelago.gg new_room failed: HTTP {status}"));
    }

    let room_location = resp
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| eyre!("archipelago.gg new_room: missing Location header"))?
        .to_string();

    let room_id = room_location
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .ok_or_else(|| eyre!("cannot parse room id from {room_location}"))?
        .to_string();

    let (port, tracker_url) = fetch_room_details(&client, &room_id).await;

    Ok(ArchipelagoGgRoom {
        room_url: format!("https://archipelago.gg/room/{room_id}"),
        tracker_url,
        host: "archipelago.gg".to_string(),
        port: port.unwrap_or(0),
        room_id,
    })
}

async fn fetch_room_details(
    client: &reqwest::Client,
    room_id: &str,
) -> (Option<u16>, Option<String>) {
    let mut port = fetch_port_via_api(client, room_id).await;
    let mut tracker_url: Option<String> = None;

    for attempt in 0..2 {
        if attempt > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        let Ok(resp) = client
            .get(format!("https://archipelago.gg/room/{room_id}"))
            .send()
            .await
        else {
            continue;
        };
        let Ok(html) = resp.text().await else {
            continue;
        };
        if port.is_none() {
            port = extract_port_from_html(&html);
        }
        if tracker_url.is_none() {
            tracker_url = extract_tracker_url_from_html(&html);
        }
        if port.is_some() && tracker_url.is_some() {
            break;
        }
    }

    (port, tracker_url)
}

fn extract_tracker_url_from_html(html: &str) -> Option<String> {
    for needle in &["/tracker/", "href=\"/tracker/", "href='/tracker/"] {
        if let Some(idx) = html.find(needle) {
            let start = idx + needle.len() - "/tracker/".len() + 1;
            let rest = &html[start..];
            let tail: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '/'))
                .collect();
            if tail.starts_with("tracker/") && tail.len() > "tracker/".len() + 4 {
                return Some(format!("https://archipelago.gg/{tail}"));
            }
        }
    }
    None
}

async fn fetch_port_via_api(client: &reqwest::Client, room_id: &str) -> Option<u16> {
    let resp = client
        .get(format!("https://archipelago.gg/api/room_status/{room_id}"))
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json: serde_json::Value = resp.json().await.ok()?;
    json.get("last_port")
        .and_then(|v| v.as_u64())
        .or_else(|| json.get("port").and_then(|v| v.as_u64()))
        .and_then(|n| u16::try_from(n).ok())
        .filter(|p| *p > 0)
}

fn extract_port_from_html(html: &str) -> Option<u16> {
    let needles = [
        "archipelago.gg:",
        "archipelago.gg</a>:",
        "archipelago.gg</code>:",
        "archipelago.gg</strong>:",
        "archipelago.gg</span>:",
        "\"last_port\":",
        "\"port\":",
        "last_port=",
    ];
    for needle in &needles {
        if let Some(idx) = html.find(needle) {
            let rest = &html[idx + needle.len()..];
            let digits: String = rest
                .chars()
                .skip_while(|c| !c.is_ascii_digit())
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if let Ok(port) = digits.parse::<u16>() {
                if port >= 1024 {
                    return Some(port);
                }
            }
        }
    }
    None
}

#[command]
pub async fn archipelago_gg_host(path: String) -> Result<ArchipelagoGgRoom> {
    Ok(archipelago_gg_host_inner(path).await?)
}

#[derive(serde::Serialize)]
pub struct ArchipelagoGgRoomInfo {
    pub port: u16,
    pub tracker_url: Option<String>,
}

#[command]
pub async fn archipelago_gg_room_info(room_id: String) -> Result<ArchipelagoGgRoomInfo> {
    let client = reqwest::Client::builder()
        .user_agent("Zephyr/1 (archipelago.gg-client)")
        .build()
        .map_err(|e| eyre::eyre!("build client: {e}"))?;

    let mut port = fetch_port_via_api(&client, &room_id).await;
    let html = if let Ok(resp) = client
        .get(format!("https://archipelago.gg/room/{room_id}"))
        .send()
        .await
    {
        resp.text().await.ok()
    } else {
        None
    };

    if port.is_none() {
        port = html.as_deref().and_then(extract_port_from_html);
    }
    let tracker_url = html.as_deref().and_then(extract_tracker_url_from_html);

    Ok(ArchipelagoGgRoomInfo {
        port: port.unwrap_or(0),
        tracker_url,
    })
}

// --- Custom apworlds ---

#[command]
pub async fn list_custom_apworlds(app: AppHandle) -> Result<Vec<CustomApworld>> {
    blocking(move || apworlds::list_custom_apworlds(&app)).await
}

#[command]
pub fn install_apworld_from_path(app: AppHandle, src_path: String) -> Result<CustomApworld> {
    Ok(apworlds::install_from_path(&app, &PathBuf::from(src_path))?)
}

#[derive(serde::Serialize)]
pub struct ApworldsFolderInstallResult {
    pub installed: Vec<CustomApworld>,
    pub failed: Vec<ApworldInstallFailure>,
}

#[derive(serde::Serialize)]
pub struct ApworldInstallFailure {
    pub file_name: String,
    pub error: String,
}

#[command]
pub fn install_apworlds_from_folder(
    app: AppHandle,
    folder_path: String,
) -> Result<ApworldsFolderInstallResult> {
    let (installed, failed_raw) =
        apworlds::install_from_folder(&app, &PathBuf::from(folder_path))?;
    let failed = failed_raw
        .into_iter()
        .map(|(file_name, error)| ApworldInstallFailure { file_name, error })
        .collect();
    Ok(ApworldsFolderInstallResult { installed, failed })
}

#[command]
pub fn install_apworld_from_bytes(
    app: AppHandle,
    file_name: String,
    bytes_base64: String,
) -> Result<CustomApworld> {
    Ok(apworlds::install_from_bytes(&app, &file_name, &bytes_base64)?)
}

#[command]
pub fn remove_custom_apworld(app: AppHandle, file_name: String) -> Result<()> {
    apworlds::remove_apworld(&app, &file_name)?;
    Ok(())
}

#[command]
pub async fn refresh_apworld_schemas(app: AppHandle) -> Result<RefreshResult> {
    // Offload the blocking Python extraction to a worker thread so the UI
    // event loop can keep painting the loading overlay spinner.
    let res = tauri::async_runtime::spawn_blocking(move || apworlds::refresh_schemas(&app))
        .await
        .map_err(|e| eyre::eyre!("refresh_apworld_schemas join error: {e}"))?;
    Ok(res?)
}

#[command]
pub fn open_custom_worlds_dir(app: AppHandle) -> Result<()> {
    let dir = apworlds::custom_worlds_dir(&app);
    std::fs::create_dir_all(&dir)?;
    crate::util::fs::open_path(dir)?;
    Ok(())
}

// --- Patch files & custom clients ---

#[command]
pub fn list_patches(app: AppHandle) -> Result<Vec<PatchFile>> {
    Ok(patches::list_patches(&app)?)
}

#[command]
pub fn delete_patch(path: String) -> Result<()> {
    patches::delete_patch(&PathBuf::from(path))?;
    Ok(())
}

#[command]
pub fn apply_patch(app: AppHandle, path: String) -> Result<()> {
    patches::apply_and_launch(&app, &PathBuf::from(path))?;
    Ok(())
}

#[command]
pub fn launch_ap_component(app: AppHandle, name: String) -> Result<()> {
    patches::launch_component(&app, &name)?;
    Ok(())
}

#[command]
pub fn get_rom_paths(app: AppHandle) -> Result<std::collections::HashMap<String, String>> {
    Ok(patches::load_rom_paths(&app)?)
}

#[command]
pub fn set_rom_path(app: AppHandle, extension: String, rom_path: String) -> Result<()> {
    patches::set_rom_path(&app, &extension, &rom_path)?;
    Ok(())
}

#[command]
pub fn clear_rom_path(app: AppHandle, extension: String) -> Result<()> {
    patches::clear_rom_path(&app, &extension)?;
    Ok(())
}

// --- Archipelago runtime install ---

#[command]
pub fn runtime_status(app: AppHandle) -> RuntimeStatus {
    runtime::status(&app)
}

#[command]
pub async fn install_runtime(app: AppHandle, url: Option<String>) -> Result<RuntimeStatus> {
    Ok(runtime::install(&app, url).await?)
}

#[command]
pub async fn provision_runtime_venv(app: AppHandle) -> Result<RuntimeStatus> {
    Ok(runtime::provision_venv(&app).await?)
}

#[command]
pub fn remove_runtime(app: AppHandle) -> Result<()> {
    runtime::remove(&app)?;
    Ok(())
}

