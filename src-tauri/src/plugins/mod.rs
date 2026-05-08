use serde::{Deserialize, Serialize};

pub mod commands;
pub mod registry;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegistryAuthor {
    pub name: String,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PluginType {
    Feature,
    Theme,
    Game,
    Mod,
}

/// Entry as it lives in `registry.json` on the plugins repo.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegistryEntry {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(rename = "type")]
    pub kind: PluginType,
    pub author: RegistryAuthor,
    pub description: String,
    pub icon: String,
    #[serde(default)]
    pub default_installed: bool,
    #[serde(default = "default_true")]
    pub removable: bool,
    pub path: String,
}

fn default_true() -> bool {
    true
}

/// Plugin entry as exposed to the frontend.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PluginEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub icon: String,
    pub kind: PluginType,
    pub built_in: bool,
    pub removable: bool,
    pub enabled: bool,
}

/// Feature ids whose runtime code is shipped in the Zephyr binary. These
/// are pre-installed on first launch and toggle in-place. Anything in the
/// registry that isn't listed here would need an external install pipeline,
/// which is not implemented yet.
pub const BUILT_IN_FEATURE_IDS: &[&str] = &["archipelago"];

pub fn is_built_in_feature(entry: &RegistryEntry) -> bool {
    matches!(entry.kind, PluginType::Feature)
        && BUILT_IN_FEATURE_IDS.contains(&entry.id.as_str())
}

/// Used when the network fetch and the local cache both fail (first launch
/// offline, etc.). Returning an empty list means the Plugins page is empty
/// in that degraded state, which makes the dependency on the remote
/// registry visible during testing.
pub fn fallback_registry() -> Vec<RegistryEntry> {
    vec![]
}
