//! Stable on-disk pointer to the configurable directories.
//!
//! The preferences live in the SQLite database inside the *default* data
//! directory, which is exactly the directory the user is allowed to move. If the
//! database ever ends up somewhere else, the freshly saved paths are written to a
//! file that is never read again on the next launch and the setting appears to
//! reset itself.
//!
//! This file lives in the app *config* directory, which is not configurable and
//! never moves, so the paths can always be recovered.

use std::{
    fs,
    path::{Path, PathBuf},
};

use eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::util::{
    self,
    fs::{write_json, JsonStyle},
};

pub const FILE_NAME: &str = "directories.json";

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct DirPointer {
    pub data_dir: Option<PathBuf>,
    pub cache_dir: Option<PathBuf>,
}

pub fn path() -> PathBuf {
    util::path::default_app_config_dir().join(FILE_NAME)
}

pub fn read() -> DirPointer {
    read_from(path())
}

pub fn read_from(path: impl AsRef<Path>) -> DirPointer {
    let path = path.as_ref();

    if !path.exists() {
        return DirPointer::default();
    }

    util::fs::read_json(path).unwrap_or_else(|err| {
        warn!(
            "failed to read {}, falling back to the paths in the database: {:#}",
            path.display(),
            err
        );

        DirPointer::default()
    })
}

pub fn write(pointer: &DirPointer) -> Result<()> {
    write_to(path(), pointer)
}

pub fn write_to(path: impl AsRef<Path>, pointer: &DirPointer) -> Result<()> {
    let path = path.as_ref();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("failed to create config directory")?;
    }

    write_json(path, pointer, JsonStyle::Pretty)
        .with_context(|| format!("failed to write {}", path.display()))
}
