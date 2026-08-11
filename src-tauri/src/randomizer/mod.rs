pub mod ap_runner;
pub mod apworlds;
pub mod commands;
pub mod patches;
pub mod random_value;
pub mod runtime;
pub mod schema;
pub mod settings;
pub mod types;
pub mod validation;
pub mod yaml_gen;

/// Directory under the *default* app data dir that holds everything the
/// randomizer persists: the Archipelago runtime, player slot YAMLs, generated
/// output, user schemas, the base ROM registry and the folder settings.
///
/// Every one of those is addressed through Tauri's `app_data_dir`, which is
/// fixed by the OS and does not follow the `dataDir` pref, so this must stay
/// where it is when the user moves their data folder.
pub const DATA_DIR_NAME: &str = "randomizer";

#[cfg(test)]
mod tests;
