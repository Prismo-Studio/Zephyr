use std::path::PathBuf;

use tauri::{command, AppHandle, Manager};

use super::{
    ap_runner::{
        self, GenerateOutcome, PlayerFile, PythonStatus, SeedFile, ServerState, ServerStatus,
    },
    schema,
    types::*,
    validation, yaml_gen,
    yaml_gen::LintIssue,
};
use crate::util::cmd::Result;

#[command]
pub fn list_supported_games(app: AppHandle) -> Result<Vec<GameSummary>> {
    let dir = schema::schemas_dir(&app);
    let schemas = schema::load_all_schemas(&dir)?;
    Ok(schemas.iter().map(schema::summarize).collect())
}

#[command]
pub fn get_game_schema(app: AppHandle, game_id: String) -> Result<GameSchema> {
    let dir = schema::schemas_dir(&app);
    let schema = schema::load_schema_by_id(&dir, &game_id)?;
    Ok(schema)
}

#[command]
pub fn generate_yaml(app: AppHandle, config: RandomizerConfig) -> Result<String> {
    let dir = schema::schemas_dir(&app);
    let schema = schema::load_schema_by_id(&dir, &config.game_id)?;
    let yaml = yaml_gen::generate(&schema, &config)?;
    Ok(yaml)
}

#[command]
pub fn validate_config(app: AppHandle, config: RandomizerConfig) -> Result<Vec<ValidationError>> {
    let dir = schema::schemas_dir(&app);
    let schema = schema::load_schema_by_id(&dir, &config.game_id)?;
    Ok(validation::validate(&schema, &config))
}

#[command]
pub fn lint_yaml(app: AppHandle, game_id: String, yaml: String) -> Result<Vec<LintIssue>> {
    let dir = schema::schemas_dir(&app);
    let schema = schema::load_schema_by_id(&dir, &game_id)?;
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
