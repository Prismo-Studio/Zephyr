use std::fs;

use tempfile::tempdir;

use super::{move_contents, pinned_files, pointer, reclaim_pinned_entries, DirPref};

fn write_file(path: impl AsRef<std::path::Path>) {
    let path = path.as_ref();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, "x").unwrap();
}

/// The database the prefs are written to lives in the data directory. If a move
/// took it along, the new path would be saved into a file that is never opened
/// again and the setting would look like it reset itself on the next launch.
#[test]
fn set_leaves_pinned_files_behind() {
    let root = tempdir().unwrap();
    let old = root.path().join("old");
    let new = root.path().join("new");

    write_file(old.join("data.sqlite3"));
    write_file(old.join("latest.log"));
    write_file(old.join("among-us/profiles/Default/mod.dll"));
    fs::create_dir_all(&new).unwrap();

    let mut pref = DirPref {
        value: old.clone(),
        keep_files: vec!["data.sqlite3", "latest.log"],
    };

    assert!(pref.set(new.clone()).unwrap());

    assert_eq!(pref.get(), new);
    assert!(old.join("data.sqlite3").exists());
    assert!(old.join("latest.log").exists());
    assert!(!new.join("data.sqlite3").exists());
    assert!(new.join("among-us/profiles/Default/mod.dll").exists());
    assert!(!old.join("among-us").exists());
}

#[test]
fn set_removes_old_dir_when_nothing_is_pinned() {
    let root = tempdir().unwrap();
    let old = root.path().join("old");
    let new = root.path().join("new");

    write_file(old.join("mods/a.zip"));
    fs::create_dir_all(&new).unwrap();

    let mut pref = DirPref::new(old.clone());

    assert!(pref.set(new.clone()).unwrap());

    assert!(!old.exists());
    assert!(new.join("mods/a.zip").exists());
}

#[test]
fn set_is_a_noop_for_the_same_path() {
    let root = tempdir().unwrap();
    let dir = root.path().join("dir");
    fs::create_dir_all(&dir).unwrap();

    let mut pref = DirPref::new(dir.clone());

    assert!(!pref.set(dir.clone()).unwrap());
    assert_eq!(pref.get(), dir);
}

#[test]
fn set_rejects_non_empty_target_without_changing_the_value() {
    let root = tempdir().unwrap();
    let old = root.path().join("old");
    let new = root.path().join("new");

    write_file(old.join("a.txt"));
    write_file(new.join("someone-elses-file.txt"));

    let mut pref = DirPref::new(old.clone());

    assert!(pref.set(new).is_err());
    assert_eq!(pref.get(), old);
    assert!(old.join("a.txt").exists());
}

#[test]
fn set_rejects_a_subdirectory_of_the_current_dir() {
    let root = tempdir().unwrap();
    let old = root.path().join("old");
    let new = old.join("inner");

    fs::create_dir_all(&new).unwrap();

    let mut pref = DirPref::new(old.clone());

    assert!(pref.set(new).is_err());
    assert_eq!(pref.get(), old);
}

/// The configured directory may have been deleted while the app wasn't running;
/// there is simply nothing to move in that case.
#[test]
fn set_tolerates_a_missing_source_dir() {
    let root = tempdir().unwrap();
    let old = root.path().join("gone");
    let new = root.path().join("new");

    fs::create_dir_all(&new).unwrap();

    let mut pref = DirPref::new(old);

    assert!(pref.set(new.clone()).unwrap());
    assert_eq!(pref.get(), new);
}

/// Every subsystem that resolves its files from the *default* data dir instead
/// of the pref has to be listed here, or a data folder change moves its files
/// out from under it. The randomizer is the easiest one to forget: it addresses
/// everything through Tauri's `app_data_dir`, including a settings file full of
/// absolute paths.
#[test]
fn pinned_files_cover_everything_addressed_from_the_default_dir() {
    let pinned = pinned_files();

    for name in [
        crate::db::FILE_NAME,
        crate::logger::FILE_NAME,
        crate::game::CACHE_FILE_NAME,
        crate::randomizer::DATA_DIR_NAME,
    ] {
        assert!(pinned.contains(&name), "{name} is not pinned");
    }

    for name in crate::plugins::DATA_FILE_NAMES {
        assert!(pinned.contains(name), "{name} is not pinned");
    }
}

/// A half moved directory would strand the user: the target is no longer empty,
/// so they couldn't even retry the same folder.
#[test]
fn failed_move_puts_everything_back() {
    let root = tempdir().unwrap();
    let old = root.path().join("old");
    let new = root.path().join("new");

    let names = ["a.txt", "b.txt", "zz-conflict"];
    for name in names {
        write_file(old.join(name));
    }

    // a directory where one of the files wants to go makes that move fail
    write_file(new.join("zz-conflict").join("inner.txt"));

    assert!(move_contents(&old, &new, &[]).is_err());

    for name in names {
        assert!(old.join(name).is_file(), "{name} was not moved back");
    }
    assert!(!new.join("a.txt").exists());
    assert!(!new.join("b.txt").exists());
}

/// Versions before pinning existed dragged the plugin and randomizer data along
/// with the data folder, leaving the code that reads them looking at an empty
/// default directory. Those users get their files back on the next launch.
#[test]
fn stranded_pinned_entries_are_reclaimed() {
    let root = tempdir().unwrap();
    let default = root.path().join("default");
    let data = root.path().join("elsewhere");

    write_file(default.join("data.sqlite3"));
    write_file(data.join("randomizer/settings.json"));
    write_file(data.join("plugins/some-plugin/manifest.json"));
    write_file(data.join("among-us/profiles/Default/mod.dll"));

    reclaim_pinned_entries(&data, &default, &["data.sqlite3", "randomizer", "plugins"]);

    assert!(default.join("randomizer/settings.json").exists());
    assert!(default.join("plugins/some-plugin/manifest.json").exists());
    assert!(!data.join("randomizer").exists());
    assert!(!data.join("plugins").exists());

    // anything that isn't pinned genuinely belongs in the data folder
    assert!(data.join("among-us/profiles/Default/mod.dll").exists());
}

/// The copy at the default location is the one actually in use, so a stray one
/// must never overwrite it.
#[test]
fn reclaiming_never_overwrites_the_entry_in_use() {
    let root = tempdir().unwrap();
    let default = root.path().join("default");
    let data = root.path().join("elsewhere");

    fs::create_dir_all(default.join("randomizer")).unwrap();
    fs::write(default.join("randomizer/settings.json"), "in use").unwrap();
    fs::create_dir_all(data.join("randomizer")).unwrap();
    fs::write(data.join("randomizer/settings.json"), "stale").unwrap();

    reclaim_pinned_entries(&data, &default, &["randomizer"]);

    let kept = fs::read_to_string(default.join("randomizer/settings.json")).unwrap();
    assert_eq!(kept, "in use");
    assert!(data.join("randomizer").exists());
}

#[test]
fn reclaiming_is_a_noop_when_the_data_dir_is_the_default() {
    let root = tempdir().unwrap();
    let default = root.path().join("default");

    write_file(default.join("randomizer/settings.json"));

    reclaim_pinned_entries(&default, &default, &pinned_files());

    assert!(default.join("randomizer/settings.json").exists());
}

#[test]
fn pointer_survives_a_write_read_roundtrip() {
    let root = tempdir().unwrap();
    let path = root.path().join("nested").join(pointer::FILE_NAME);

    let written = pointer::DirPointer {
        data_dir: Some("D:/Zephyr".into()),
        cache_dir: Some("D:/Zephyr/cache".into()),
    };

    pointer::write_to(&path, &written).unwrap();

    assert_eq!(pointer::read_from(&path), written);
}

#[test]
fn pointer_is_empty_when_missing_or_corrupt() {
    let root = tempdir().unwrap();

    let missing = root.path().join(pointer::FILE_NAME);
    assert_eq!(pointer::read_from(&missing), pointer::DirPointer::default());

    let corrupt = root.path().join("corrupt.json");
    fs::write(&corrupt, "{ not json").unwrap();
    assert_eq!(pointer::read_from(&corrupt), pointer::DirPointer::default());
}
