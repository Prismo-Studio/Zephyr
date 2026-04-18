use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use eyre::{Context, Result};
use tauri::{AppHandle, Manager};
use tracing::warn;

use super::types::{GameSchema, GameSummary};

/// Directory where the schema extractor writes schemas for user-installed
/// `.apworld` files. Overlays (and takes precedence over) the bundled dir.
pub fn user_schemas_dir(app: &AppHandle) -> PathBuf {
    let base = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir());
    base.join("randomizer").join("schemas")
}

/// Resolve the directory containing schema JSON files.
///
/// Search order:
/// 1. `<resource_dir>/data/randomizer/schemas`
/// 2. `<cwd>/data/randomizer/schemas`
/// 3. `<cwd>/../data/randomizer/schemas` (when running from `src-tauri`)
pub fn schemas_dir(app: &AppHandle) -> PathBuf {
    if let Ok(resource_dir) = app.path().resource_dir() {
        let candidates = [
            resource_dir.join("schemas"),
            resource_dir.join("data/randomizer/schemas"),
            resource_dir.join("randomizer/schemas"),
        ];
        for c in &candidates {
            tracing::debug!("schemas_dir: trying resource path {}", c.display());
            if c.exists() {
                tracing::info!("schemas_dir: found at {}", c.display());
                return c.clone();
            }
        }
        tracing::warn!("schemas_dir: no resource path found, resource_dir={}", resource_dir.display());
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let candidates = [
        cwd.join("data/randomizer/schemas"),
        cwd.join("../data/randomizer/schemas"),
    ];
    for c in &candidates {
        if c.exists() {
            tracing::info!("schemas_dir: found at cwd path {}", c.display());
            return c.clone();
        }
    }

    tracing::warn!("schemas_dir: no path found, falling back to cwd");
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

/// Load schemas from the bundled dir AND the user overlay, with user entries
/// taking precedence by id. Used by the catalog + configurator commands so
/// freshly refreshed custom apworlds show up without touching bundled files.
pub fn load_all_schemas_merged(app: &AppHandle) -> Result<Vec<GameSchema>> {
    let primary = schemas_dir(app);
    let overlay = user_schemas_dir(app);

    let mut by_id: HashMap<String, GameSchema> = HashMap::new();
    // Bundled first, overlay overwrites.
    for schema in load_all_schemas(&primary)? {
        by_id.insert(schema.id.clone(), schema);
    }
    if overlay.exists() {
        for schema in load_all_schemas(&overlay)? {
            by_id.insert(schema.id.clone(), schema);
        }
    }

    let mut out: Vec<GameSchema> = by_id.into_values().collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// Try the user overlay first (newer / custom worlds), then fall back to
/// the bundled schemas dir.
pub fn load_schema_by_id_merged(app: &AppHandle, game_id: &str) -> Result<GameSchema> {
    let overlay = user_schemas_dir(app);
    let overlay_path = overlay.join(format!("{game_id}.json"));
    if overlay_path.exists() {
        return load_schema_file(&overlay_path);
    }
    load_schema_by_id(&schemas_dir(app), game_id)
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
