use std::{
    fs,
    path::{Path, PathBuf},
};

use eyre::{Context, Result};
use tauri::{AppHandle, Manager};
use tracing::warn;

use super::types::{GameSchema, GameSummary};

/// Resolve the directory containing schema JSON files.
///
/// Search order:
/// 1. `<resource_dir>/data/randomizer/schemas`
/// 2. `<cwd>/data/randomizer/schemas`
/// 3. `<cwd>/../data/randomizer/schemas` (when running from `src-tauri`)
pub fn schemas_dir(app: &AppHandle) -> PathBuf {
    if let Ok(resource_dir) = app.path().resource_dir() {
        let candidate = resource_dir.join("data/randomizer/schemas");
        if candidate.exists() {
            return candidate;
        }
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let candidate = cwd.join("data/randomizer/schemas");
    if candidate.exists() {
        return candidate;
    }

    let candidate = cwd.join("../data/randomizer/schemas");
    if candidate.exists() {
        return candidate;
    }

    cwd.join("data/randomizer/schemas")
}

pub fn load_schema_file(path: &Path) -> Result<GameSchema> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read schema file {}", path.display()))?;
    let schema: GameSchema = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse schema {}", path.display()))?;
    Ok(schema)
}

pub fn load_all_schemas(dir: &Path) -> Result<Vec<GameSchema>> {
    let mut out = Vec::new();
    if !dir.exists() {
        warn!("randomizer schemas dir not found: {}", dir.display());
        return Ok(out);
    }

    for entry in fs::read_dir(dir).context("failed to read schemas dir")? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        match load_schema_file(&path) {
            Ok(schema) => out.push(schema),
            Err(err) => warn!("skipping invalid schema {}: {:#}", path.display(), err),
        }
    }

    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

pub fn load_schema_by_id(dir: &Path, game_id: &str) -> Result<GameSchema> {
    let path = dir.join(format!("{game_id}.json"));
    load_schema_file(&path)
}

pub fn summarize(schema: &GameSchema) -> GameSummary {
    GameSummary {
        id: schema.id.clone(),
        name: schema.name.clone(),
        version: schema.version.clone(),
        description: schema.description.clone(),
        option_count: schema.options.len(),
        preset_count: schema.presets.len(),
        icon: schema.meta.icon.clone(),
    }
}
