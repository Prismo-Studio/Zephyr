use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{self, BufReader, BufWriter, Read},
    path::{Path, PathBuf},
};

use eyre::Result;
#[cfg(target_os = "linux")]
use eyre::Context;

use serde::{de::DeserializeOwned, Serialize};
use tracing::warn;
use walkdir::WalkDir;
use zip::ZipArchive;

use super::error::IoResultExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overwrite {
    Yes,
    No,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseLinks {
    Yes,
    No,
}

pub fn copy_dir(
    src: impl AsRef<Path>,
    dest: impl AsRef<Path>,
    overwrite: Overwrite,
    use_links: UseLinks,
) -> eyre::Result<()> {
    let dest = dest.as_ref();

    fs::create_dir_all(dest).fs_context("creating root directory", dest)?;
    copy_contents(src, dest, overwrite, use_links)
}

pub fn copy_contents(
    src: impl AsRef<Path>,
    dest: impl AsRef<Path>,
    overwrite: Overwrite,
    use_links: UseLinks,
) -> eyre::Result<()> {
    let src = src.as_ref();
    let dest = dest.as_ref();

    for entry in src.read_dir().fs_context("reading directory", src)? {
        let entry = entry?;
        let entry_path = entry.path();

        let Some(file_name) = entry_path.file_name() else {
            warn!(
                "skipping directory entry at {} because it has no name",
                entry_path.display()
            );
            continue;
        };

        let entry_type = entry
            .file_type()
            .fs_context("determining file type", &entry_path)?;

        let new_path = dest.join(file_name);

        if entry_type.is_dir() {
            if !new_path.exists() {
                fs::create_dir(&new_path).fs_context("creating directory", &new_path)?;
            }

            copy_contents(&entry_path, &new_path, overwrite, use_links)?;
        } else {
            if new_path.exists() && overwrite == Overwrite::No {
                continue;
            }

            match use_links {
                UseLinks::Yes => {
                    fs::hard_link(&entry_path, &new_path).fs_context("linking file", &new_path)?;
                }
                UseLinks::No => {
                    fs::copy(&entry_path, &new_path).fs_context("copying file", &new_path)?;
                }
            };
        }
    }

    Ok(())
}

pub fn get_directory_size(path: impl AsRef<Path>) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| match entry.file_type().is_file() {
            true => entry.metadata().map(|meta| meta.len()).unwrap_or(0),
            false => 0,
        })
        .sum()
}

pub fn read_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> eyre::Result<T> {
    let string = fs::read_to_string(path)?;
    let result = serde_json::from_str(&string)?;

    Ok(result)
}

pub fn open_zip(path: impl AsRef<Path>) -> eyre::Result<ZipArchive<BufReader<File>>> {
    let reader = File::open(path).map(BufReader::new)?;
    let archive = ZipArchive::new(reader)?;

    Ok(archive)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonStyle {
    Pretty,
    Compact,
}

pub fn write_json<T: Serialize + ?Sized>(
    path: impl AsRef<Path>,
    value: &T,
    style: JsonStyle,
) -> eyre::Result<()> {
    let writer = File::create(path).map(BufWriter::new)?;

    if style == JsonStyle::Pretty {
        serde_json::to_writer_pretty(writer, value)?;
    } else {
        serde_json::to_writer(writer, value)?;
    }

    Ok(())
}

pub fn file_name_owned(path: impl AsRef<Path>) -> String {
    path.as_ref()
        .file_name()
        .expect("file should have name")
        .to_string_lossy()
        .into_owned()
}

/// Replace every char that isn't ASCII alphanumeric, `_`, `-`, or listed in
/// `extra_allowed` with `_`. Does not trim, strip path separators, or apply
/// any empty-string fallback — callers handle that when needed.
pub fn sanitize_filename_chars(name: &str, extra_allowed: &[char]) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' || extra_allowed.contains(&c) {
                c
            } else {
                '_'
            }
        })
        .collect()
}

pub fn is_enclosed(path: impl AsRef<Path>) -> bool {
    use std::path::Component;

    if path
        .as_ref()
        .as_os_str()
        .to_str()
        .is_some_and(|str| str.contains('\0'))
    {
        return false;
    }

    let mut depth = 0usize;
    for component in path.as_ref().components() {
        match component {
            Component::Prefix(_) | Component::RootDir => return false,
            Component::ParentDir => match depth.checked_sub(1) {
                Some(new_depth) => depth = new_depth,
                None => return false,
            },
            Component::Normal(_) => depth += 1,
            Component::CurDir => (),
        }
    }

    true
}

pub trait PathExt: Sized {
    fn exists_or_none(self) -> Option<Self>;
    fn add_ext(&mut self, extension: impl AsRef<OsStr>);
}

impl PathExt for PathBuf {
    fn exists_or_none(self) -> Option<PathBuf> {
        match self.exists() {
            true => Some(self),
            false => None,
        }
    }

    fn add_ext(&mut self, extension: impl AsRef<OsStr>) {
        match self.extension() {
            Some(ext) => {
                let mut ext = ext.to_os_string();
                ext.push(".");
                ext.push(extension.as_ref());
                self.set_extension(ext)
            }
            None => self.set_extension(extension.as_ref()),
        };
    }
}

pub fn checksum(path: &Path) -> io::Result<blake3::Hash> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 4096];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(hasher.finalize())
}

/// Opens a file or directory in the system's default handler.
/// On Linux, spawns `xdg-open` detached to avoid blocking issues with the `open` crate.
pub fn open_path(path: impl AsRef<Path>) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path.as_ref())
            .spawn()
            .context("failed to open path with xdg-open")?;
    }
    #[cfg(not(target_os = "linux"))]
    {
        open::that(path.as_ref()).map_err(|err| eyre::eyre!(err))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── is_enclosed ──

    #[test]
    fn is_enclosed_accepts_relative() {
        assert!(is_enclosed("foo/bar"));
        assert!(is_enclosed("foo"));
    }

    #[test]
    fn is_enclosed_accepts_curdir() {
        assert!(is_enclosed("./foo"));
    }

    #[test]
    fn is_enclosed_rejects_absolute() {
        assert!(!is_enclosed("/etc/passwd"));
    }

    #[test]
    fn is_enclosed_rejects_parent_traversal() {
        assert!(!is_enclosed("../escape"));
        assert!(!is_enclosed("foo/../../escape"));
    }

    #[test]
    fn is_enclosed_rejects_null_bytes() {
        assert!(!is_enclosed("foo\0bar"));
    }

    // ── file_name_owned ──

    #[test]
    fn file_name_owned_extracts_name() {
        assert_eq!(file_name_owned("a/b/c.txt"), "c.txt");
        assert_eq!(file_name_owned("file.rs"), "file.rs");
    }

    // ── PathExt ──

    #[test]
    fn add_ext_to_path_without_extension() {
        let mut p = PathBuf::from("file");
        p.add_ext("bak");
        assert_eq!(p, PathBuf::from("file.bak"));
    }

    #[test]
    fn add_ext_to_path_with_extension() {
        let mut p = PathBuf::from("file.txt");
        p.add_ext("bak");
        assert_eq!(p, PathBuf::from("file.txt.bak"));
    }

    #[test]
    fn exists_or_none_returns_none_for_missing() {
        let p = PathBuf::from("/this/does/not/exist/ever");
        assert!(p.exists_or_none().is_none());
    }

    // ── copy_dir + get_directory_size ──

    #[test]
    fn copy_dir_and_get_size() {
        let src = tempfile::tempdir().unwrap();
        let dest = tempfile::tempdir().unwrap();

        fs::write(src.path().join("a.txt"), "hello").unwrap();
        fs::create_dir(src.path().join("sub")).unwrap();
        fs::write(src.path().join("sub/b.txt"), "world").unwrap();

        copy_dir(src.path(), dest.path().join("out"), Overwrite::Yes, UseLinks::No).unwrap();

        assert!(dest.path().join("out/a.txt").exists());
        assert!(dest.path().join("out/sub/b.txt").exists());
        assert_eq!(
            fs::read_to_string(dest.path().join("out/a.txt")).unwrap(),
            "hello"
        );

        let size = get_directory_size(dest.path().join("out"));
        assert_eq!(size, 10); // "hello" (5) + "world" (5)
    }

    #[test]
    fn copy_contents_no_overwrite_preserves_existing() {
        let src = tempfile::tempdir().unwrap();
        let dest = tempfile::tempdir().unwrap();

        fs::write(src.path().join("f.txt"), "new").unwrap();
        fs::write(dest.path().join("f.txt"), "original").unwrap();

        copy_contents(src.path(), dest.path(), Overwrite::No, UseLinks::No).unwrap();

        assert_eq!(
            fs::read_to_string(dest.path().join("f.txt")).unwrap(),
            "original"
        );
    }

    #[test]
    fn copy_contents_overwrite_replaces() {
        let src = tempfile::tempdir().unwrap();
        let dest = tempfile::tempdir().unwrap();

        fs::write(src.path().join("f.txt"), "new").unwrap();
        fs::write(dest.path().join("f.txt"), "original").unwrap();

        copy_contents(src.path(), dest.path(), Overwrite::Yes, UseLinks::No).unwrap();

        assert_eq!(
            fs::read_to_string(dest.path().join("f.txt")).unwrap(),
            "new"
        );
    }

    #[test]
    fn get_directory_size_empty() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(get_directory_size(dir.path()), 0);
    }

    // ── JSON roundtrip ──

    #[test]
    fn json_roundtrip() {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct TestData {
            name: String,
            value: u32,
        }

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.json");

        let data = TestData {
            name: "hello".into(),
            value: 42,
        };

        write_json(&path, &data, JsonStyle::Pretty).unwrap();
        let back: TestData = read_json(&path).unwrap();
        assert_eq!(back, data);
    }
}
