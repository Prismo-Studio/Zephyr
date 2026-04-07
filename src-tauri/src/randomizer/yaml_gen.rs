use eyre::Result;
use serde_yaml::{Mapping, Value as YamlValue};

use super::types::{GameSchema, OptionDef, OptionType, RandomizerConfig, Value};

/// Option ids that Archipelago expects at the YAML root, not inside the game block.
/// See `Options.CommonOptions` in the Archipelago source.
const COMMON_OPTION_IDS: &[&str] = &["progression_balancing", "accessibility"];

fn value_to_yaml(value: &Value) -> YamlValue {
    match value {
        Value::Bool(b) => YamlValue::Bool(*b),
        Value::Int(i) => YamlValue::Number((*i).into()),
        Value::Float(f) => serde_yaml::to_value(f).unwrap_or(YamlValue::Null),
        Value::String(s) => YamlValue::String(s.clone()),
        Value::List(items) => YamlValue::Sequence(items.iter().map(value_to_yaml).collect()),
    }
}

fn default_for(opt: &OptionDef) -> Value {
    match &opt.option_type {
        OptionType::Toggle { default } => Value::Bool(*default),
        OptionType::Range { default, .. } => Value::Int(*default),
        OptionType::Select { default, .. } => Value::String(default.clone()),
        OptionType::MultiSelect { defaults, .. } => {
            Value::List(defaults.iter().cloned().map(Value::String).collect())
        }
        OptionType::Text { default, .. } => Value::String(default.clone()),
    }
}

/// Generate an Archipelago-style YAML document from a config + schema.
///
/// Layout:
/// ```yaml
/// name: PlayerName
/// description: ...
/// game: <Game Name>
/// <Game Name>:
///   option_id: value
///   ...
/// ```
pub fn generate(schema: &GameSchema, config: &RandomizerConfig) -> Result<String> {
    let mut root = Mapping::new();

    let player_name = config
        .player_name
        .clone()
        .unwrap_or_else(|| "Player1".to_string());
    root.insert(YamlValue::from("name"), YamlValue::from(player_name));

    root.insert(
        YamlValue::from("game"),
        YamlValue::from(schema.name.clone()),
    );

    if let Some(seed) = &config.seed {
        if !seed.is_empty() {
            root.insert(YamlValue::from("seed"), YamlValue::from(seed.clone()));
        }
    }

    let mut game_section = Mapping::new();
    for opt in &schema.options {
        let value = config
            .values
            .get(&opt.id)
            .cloned()
            .unwrap_or_else(|| default_for(opt));
        let yaml_value = value_to_yaml(&value);

        if COMMON_OPTION_IDS.contains(&opt.id.as_str()) {
            // Common options live at the YAML root, not in the game section
            root.insert(YamlValue::from(opt.id.clone()), yaml_value);
        } else {
            game_section.insert(YamlValue::from(opt.id.clone()), yaml_value);
        }
    }

    root.insert(
        YamlValue::from(schema.name.clone()),
        YamlValue::Mapping(game_section),
    );

    let yaml = serde_yaml::to_string(&YamlValue::Mapping(root))?;
    Ok(yaml)
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct LintIssue {
    pub level: String, // "error" | "warning" | "info"
    pub message: String,
}

/// Re-parse the YAML and check structural integrity for the Archipelago format.
/// Returns issues found, empty vec means clean.
pub fn lint(yaml: &str, schema: &GameSchema) -> Vec<LintIssue> {
    let mut issues = Vec::new();

    let parsed: YamlValue = match serde_yaml::from_str(yaml) {
        Ok(v) => v,
        Err(err) => {
            issues.push(LintIssue {
                level: "error".into(),
                message: format!("invalid YAML: {err}"),
            });
            return issues;
        }
    };

    let map = match parsed.as_mapping() {
        Some(m) => m,
        None => {
            issues.push(LintIssue {
                level: "error".into(),
                message: "root is not a mapping".into(),
            });
            return issues;
        }
    };

    let required_root = ["name", "game"];
    for key in required_root {
        if !map.contains_key(YamlValue::from(key)) {
            issues.push(LintIssue {
                level: "error".into(),
                message: format!("missing required root key '{key}'"),
            });
        }
    }

    if let Some(name) = map
        .get(YamlValue::from("name"))
        .and_then(|v| v.as_str())
    {
        if name.trim().is_empty() {
            issues.push(LintIssue {
                level: "warning".into(),
                message: "player name is empty".into(),
            });
        } else if name.len() > 16 {
            issues.push(LintIssue {
                level: "warning".into(),
                message: format!("player name is {} chars (Archipelago limit is 16)", name.len()),
            });
        }
    }

    let game_section = map
        .get(YamlValue::from(schema.name.clone()))
        .and_then(|v| v.as_mapping());

    match game_section {
        None => issues.push(LintIssue {
            level: "error".into(),
            message: format!("missing game section '{}'", schema.name),
        }),
        Some(section) => {
            let known: std::collections::HashSet<&str> =
                schema.options.iter().map(|o| o.id.as_str()).collect();
            for (k, _) in section {
                if let Some(key) = k.as_str() {
                    if !known.contains(key) {
                        issues.push(LintIssue {
                            level: "warning".into(),
                            message: format!("unknown option '{key}' in game section"),
                        });
                    }
                }
            }
        }
    }

    issues
}
