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

#[command]
pub fn list_supported_games(app: AppHandle) -> Result<Vec<GameSummary>> {
    let schemas = schema::load_all_schemas_merged(&app)?;
    Ok(schemas.iter().map(schema::summarize).collect())
}

#[command]
pub fn get_game_schema(app: AppHandle, game_id: String) -> Result<GameSchema> {
    let schema = schema::load_schema_by_id_merged(&app, &game_id)?;
    Ok(schema)
}

#[command]
pub fn generate_yaml(app: AppHandle, config: RandomizerConfig) -> Result<String> {
    let schema = schema::load_schema_by_id_merged(&app, &config.game_id)?;
    let yaml = yaml_gen::generate(&schema, &config)?;
    Ok(yaml)
}

#[command]
pub fn validate_config(app: AppHandle, config: RandomizerConfig) -> Result<Vec<ValidationError>> {
    let schema = schema::load_schema_by_id_merged(&app, &config.game_id)?;
    Ok(validation::validate(&schema, &config))
}

#[command]
pub fn lint_yaml(app: AppHandle, game_id: String, yaml: String) -> Result<Vec<LintIssue>> {
    let schema = schema::load_schema_by_id_merged(&app, &game_id)?;
    Ok(yaml_gen::lint(&yaml, &schema))
}

#[command]
pub fn check_python(app: AppHandle) -> PythonStatus {
    ap_runner::check_python(&app)
}

#[command]
pub fn save_player_yaml(app: AppHandle, slot_name: String, yaml: String) -> Result<String> {
    let path = ap_runner::save_player_yaml(&app, &slot_name, &yaml)?;
    Ok(path.to_string_lossy().to_string())
}

#[command]
pub fn list_player_yamls(app: AppHandle) -> Result<Vec<PlayerFile>> {
    Ok(ap_runner::list_player_yamls(&app)?)
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
pub fn generate_seed(app: AppHandle) -> Result<GenerateOutcome> {
    Ok(ap_runner::run_generate(&app)?)
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
pub fn list_seeds(app: AppHandle) -> Result<Vec<SeedFile>> {
    Ok(ap_runner::list_seeds(&app)?)
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
    pub tracker_url: String,
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
        .post(format!("https://archipelago.gg/new_room/{seed_id}"))
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

    let room_page = client
        .get(format!("https://archipelago.gg/room/{room_id}"))
        .send()
        .await
        .context("fetch room page")?
        .text()
        .await
        .context("read room page")?;

    let port = room_page
        .split("archipelago.gg:")
        .nth(1)
        .and_then(|rest| {
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            digits.parse::<u16>().ok()
        })
        .unwrap_or(0);

    Ok(ArchipelagoGgRoom {
        room_url: format!("https://archipelago.gg/room/{room_id}"),
        tracker_url: format!("https://archipelago.gg/tracker/{room_id}"),
        host: "archipelago.gg".to_string(),
        port,
        room_id,
    })
}

#[command]
pub async fn archipelago_gg_host(path: String) -> Result<ArchipelagoGgRoom> {
    Ok(archipelago_gg_host_inner(path).await?)
}

// --- Custom apworlds ---

#[command]
pub fn list_custom_apworlds(app: AppHandle) -> Result<Vec<CustomApworld>> {
    Ok(apworlds::list_custom_apworlds(&app)?)
}

#[command]
pub fn install_apworld_from_path(app: AppHandle, src_path: String) -> Result<CustomApworld> {
    Ok(apworlds::install_from_path(&app, &PathBuf::from(src_path))?)
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
pub fn refresh_apworld_schemas(app: AppHandle) -> Result<RefreshResult> {
    Ok(apworlds::refresh_schemas(&app)?)
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

