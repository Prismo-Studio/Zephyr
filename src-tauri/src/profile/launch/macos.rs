use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use eyre::{Context, Result};
use tracing::{debug, info, warn};

use crate::util::error::IoResultExt;

/// Prepares BepInEx Doorstop environment for macOS.
/// On macOS, `start_game_bepinex.sh` doesn't set DYLD variables,
/// so we need to generate a wrapper script that does.
pub fn prepare_doorstop_env(command: &mut Command, profile_dir: &Path) -> Result<()> {
    let doorstop_libs = profile_dir.join("doorstop_libs");
    if !doorstop_libs.exists() {
        debug!("no doorstop_libs directory found, skipping macOS Doorstop setup");
        return Ok(());
    }

    // Find the dylib in doorstop_libs
    let dylib = find_doorstop_dylib(&doorstop_libs);

    if let Some(dylib_name) = dylib {
        info!(
            "setting macOS Doorstop env: DYLD_LIBRARY_PATH={}, DYLD_INSERT_LIBRARIES={}",
            doorstop_libs.display(),
            dylib_name
        );

        // DYLD_LIBRARY_PATH tells the loader where to find the dylib
        command.env("DYLD_LIBRARY_PATH", &doorstop_libs);
        // DYLD_INSERT_LIBRARIES injects the Doorstop hook into the process
        command.env("DYLD_INSERT_LIBRARIES", &dylib_name);
    } else {
        warn!("doorstop_libs exists but no .dylib found inside");
    }

    Ok(())
}

/// Generates a wrapper shell script that sources start_game_bepinex.sh
/// with the correct DYLD environment variables set.
/// Returns the path to the wrapper script.
pub fn create_bepinex_wrapper(profile_dir: &Path) -> Result<Option<PathBuf>> {
    let bepinex_script = profile_dir.join("start_game_bepinex.sh");
    if !bepinex_script.exists() {
        return Ok(None);
    }

    let doorstop_libs = profile_dir.join("doorstop_libs");
    let dylib_name = find_doorstop_dylib(&doorstop_libs).unwrap_or("libdoorstop_x64.dylib".into());

    let wrapper_path = profile_dir.join("_zephyr_macos_launch.sh");
    let wrapper_content = format!(
        "#!/bin/sh\n\
         export DYLD_LIBRARY_PATH=\"{}:${{DYLD_LIBRARY_PATH}}\"\n\
         if [ -z \"${{DYLD_INSERT_LIBRARIES}}\" ]; then\n\
         \texport DYLD_INSERT_LIBRARIES=\"{}\"\n\
         else\n\
         \texport DYLD_INSERT_LIBRARIES=\"{}:${{DYLD_INSERT_LIBRARIES}}\"\n\
         fi\n\
         source \"{}\" \"$@\"\n",
        doorstop_libs.to_string_lossy(),
        dylib_name,
        dylib_name,
        bepinex_script.to_string_lossy()
    );

    fs::write(&wrapper_path, wrapper_content)
        .fs_context("writing macOS BepInEx wrapper", &wrapper_path)?;

    // chmod +x
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&wrapper_path, fs::Permissions::from_mode(0o755))
            .fs_context("setting permissions on wrapper", &wrapper_path)?;
    }

    info!("created macOS BepInEx wrapper at {}", wrapper_path.display());
    Ok(Some(wrapper_path))
}

fn find_doorstop_dylib(doorstop_libs: &Path) -> Option<String> {
    doorstop_libs
        .read_dir()
        .ok()?
        .filter_map(|e| e.ok())
        .find(|entry| {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            name_str.contains("libdoorstop") && name_str.ends_with(".dylib")
        })
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
}
