use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// JSON / YAML scalar value used for option values, defaults, and dependency comparisons.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Choice {
    pub value: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OptionType {
    Toggle {
        default: bool,
    },
    Range {
        min: i64,
        max: i64,
        #[serde(default = "default_step")]
        step: i64,
        default: i64,
    },
    Select {
        choices: Vec<Choice>,
        default: String,
    },
    MultiSelect {
        choices: Vec<Choice>,
        #[serde(default)]
        defaults: Vec<String>,
    },
    Text {
        #[serde(default)]
        default: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
    },
}

fn default_step() -> i64 {
    1
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Condition {
    Equals { value: Value },
    NotEquals { value: Value },
    In { values: Vec<Value> },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Dependency {
    pub option_id: String,
    pub condition: Condition,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OptionDef {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub description: String,
    /// general | items | entrances | logic | goals | cosmetic | advanced
    pub category: String,
    #[serde(rename = "type")]
    pub option_type: OptionType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<Dependency>>,
    #[serde(default)]
    pub advanced: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Preset {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub values: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GameMeta {
    #[serde(default)]
    pub rom_required: bool,
    #[serde(default)]
    pub supported_versions: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wiki_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// True for worlds bundled with Archipelago (i.e. present on
    /// archipelago.gg). False for user-installed `.apworld` files whose
    /// tutorials/setup docs are only served by a local webhost. Defaults to
    /// `true` for backwards compatibility with schemas generated before this
    /// field existed — all pre-existing bundled schemas are official.
    #[serde(default = "default_true")]
    pub is_official: bool,
    /// Author-defined English tutorial link (e.g. "setup/en",
    /// "multiworld/en"). Combined with the game name by the frontend to build
    /// an archipelago.gg/tutorial/... URL. `None` when the world declares no
    /// tutorial, in which case the UI hides the setup-guide button.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tutorial_path: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameSchema {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub options: Vec<OptionDef>,
    #[serde(default)]
    pub presets: Vec<Preset>,
    #[serde(default)]
    pub meta: GameMeta,
    /// All item names for this game, used for start_inventory autocomplete.
    /// Empty for schemas generated before this field was added.
    #[serde(default)]
    pub items: Vec<String>,
}

/// Lightweight summary for the game catalog.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameSummary {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub option_count: usize,
    pub preset_count: usize,
    pub icon: Option<String>,
}

/// User-facing config state for one game.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RandomizerConfig {
    pub game_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<String>,
    #[serde(default)]
    pub values: HashMap<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_name: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ValidationError {
    pub option_id: String,
    pub message: String,
}
