use std::path::PathBuf;

use tauri::{command, AppHandle, Manager};

use super::{
    ap_runner::{
        self, GenerateOutcome, PlayerFile, PythonStatus, SeedFile, ServerState, ServerStatus,
    },
    apworlds::{self, CustomApworld, RefreshResult},
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

#[command]
pub async fn remote_upload_seed(path: String, remote_url: String) -> Result<String> {
    let bytes = std::fs::read(&path)
        .map_err(|e| eyre::eyre!("read {path}: {e}"))?;
    let file_name = std::path::Path::new(&path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("seed.archipelago")
        .to_string();

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{remote_url}/upload"))
        .header("Content-Type", "application/octet-stream")
        .header("X-Filename", &file_name)
        .body(bytes)
        .send()
        .await
        .map_err(|e| eyre::eyre!("upload failed: {e}"))?;

    let body = resp.text().await.map_err(|e| eyre::eyre!("read response: {e}"))?;
    Ok(body)
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
pub fn remove_runtime(app: AppHandle) -> Result<()> {
    runtime::remove(&app)?;
    Ok(())
}

#[command]
pub async fn remote_request(remote_url: String, endpoint: String, method: String, body: Option<String>) -> Result<String> {
    let client = reqwest::Client::new();
    let url = format!("{remote_url}{endpoint}");
    let req = match method.as_str() {
        "POST" => {
            let mut r = client.post(&url);
            if let Some(b) = body {
                r = r.header("Content-Type", "application/json").body(b);
            }
            r
        }
        _ => client.get(&url),
    };
    let resp = req.send().await.map_err(|e| eyre::eyre!("request failed: {e}"))?;
    let text = resp.text().await.map_err(|e| eyre::eyre!("read response: {e}"))?;
    Ok(text)
}
