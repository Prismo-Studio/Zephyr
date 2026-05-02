use std::{fs, path::Path};

use eyre::Result;
use itertools::Itertools;
use tracing::{debug, info, trace};

use super::mod_loader::bepinex_preloader_path;
use crate::util::error::IoResultExt;

pub fn is_proton(game_dir: &Path) -> Result<bool> {
    if game_dir.join(".forceproton").exists() {
        debug!(".forceproton file found");
        return Ok(true);
    }

    Ok(game_dir
        .read_dir()?
        .filter_map(Result::ok)
        .any(|entry| entry.path().extension().is_some_and(|ext| ext == "exe")))
}

pub fn ensure_wine_override(steam_id: u64, proxy_dll: &str, game_dir: &Path) -> Result<()> {
    debug!("adding wine dll override to steam compatdata");

    let wine_reg_path = game_dir
        .parent() // common
        .unwrap()
        .parent() // steamapps
        .unwrap()
        .join("compatdata")
        .join(steam_id.to_string())
        .join("pfx")
        .join("user.reg");

    debug!(path = %wine_reg_path.display(), "reading registry file");

    let text =
        fs::read_to_string(&wine_reg_path).fs_context("reading wine registry", &wine_reg_path)?;

    debug!(len = text.len(), "read registry file");

    let new_text = reg_add_in_section(
        &text,
        r#"[Software\\Wine\\DllOverrides]"#,
        proxy_dll,
        "native,builtin",
    );

    if text == new_text {
        info!("wine registry is unchanged");
    } else {
        info!("writing to wine registry file");
        fs::write(&wine_reg_path, new_text).fs_context("writing wine registry", &wine_reg_path)?;
    }

    Ok(())
}

fn reg_add_in_section(reg: &str, section: &str, key: &str, value: &str) -> String {
    let mut lines = reg.lines().collect_vec();

    let mut begin = 0;
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with(section) {
            begin = i + 2; // Skip timestamp line
            break;
        }
    }

    trace!("section begins at line {}", begin);

    let mut end = begin;
    while end < lines.len() && !lines[end].is_empty() {
        end += 1;
    }

    trace!("section ends at line {}", end);

    for i in begin..end {
        if lines[i].starts_with(&format!("\"{}\"", key)) {
            debug!("found existing key in wine registry, replacing it");

            let line = format!("\"{}\"=\"{}\"", key, value);
            lines[i] = &line;

            return lines.join("\n");
        }
    }

    debug!("adding key to wine registry");

    let line = format!("\"{}\"=\"{}\"", key, value);
    lines.insert(end, &line);
    lines.join("\n")
}

// Steam's `-applaunch <appid>` does not reliably forward extra argv to the
// game on Linux (when Steam is already running it dispatches via IPC and
// the new argv is dropped), so the `--doorstop-target-assembly` override
// is lost. Doorstop then falls back to the relative `target_assembly` from
// `doorstop_config.ini`, which points inside the game dir — but BepInEx
// lives in the profile dir, so the preloader isn't found and mods don't
// load. We rewrite the copied config with an absolute Wine path so
// Doorstop reads it on init regardless of argv.
pub fn patch_doorstop_config_for_proton(game_dir: &Path, profile_dir: &Path) -> Result<()> {
    let config_path = game_dir.join("doorstop_config.ini");
    if !config_path.exists() {
        debug!("no doorstop_config.ini in game dir, skipping patch");
        return Ok(());
    }

    let preloader = match bepinex_preloader_path(profile_dir) {
        Ok(p) => p,
        Err(err) => {
            debug!("no BepInEx preloader in profile, skipping patch: {:#}", err);
            return Ok(());
        }
    };

    let wine_target = unix_path_to_wine(&preloader);
    let text = fs::read_to_string(&config_path)
        .fs_context("reading doorstop_config.ini", &config_path)?;
    let new_text = replace_target_assembly(&text, &wine_target);

    if new_text == text {
        debug!("doorstop_config.ini target_assembly already matches profile path");
    } else {
        info!(
            "patching doorstop_config.ini target_assembly to {}",
            wine_target
        );
        fs::write(&config_path, new_text)
            .fs_context("writing doorstop_config.ini", &config_path)?;
    }

    Ok(())
}

fn unix_path_to_wine(path: &Path) -> String {
    let s = path.to_string_lossy();
    format!("Z:{}", s.replace('/', "\\"))
}

fn replace_target_assembly(ini: &str, new_target: &str) -> String {
    ini.split_inclusive('\n')
        .map(|line| {
            let body = line.trim_end_matches(['\n', '\r']);
            let body_trimmed = body.trim_start();
            if let Some(rest) = body_trimmed.strip_prefix("target_assembly") {
                if rest.trim_start().starts_with('=') {
                    let leading = &body[..body.len() - body_trimmed.len()];
                    let line_ending = &line[body.len()..];
                    return format!("{leading}target_assembly={new_target}{line_ending}");
                }
            }
            line.to_string()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_target_assembly_preserving_other_lines() {
        let ini = "[General]\nenabled = true\ntarget_assembly=BepInEx\\core\\BepInEx.Preloader.dll\nredirect_output_log = false\n";
        let out = replace_target_assembly(ini, "Z:\\home\\u\\BepInEx.Preloader.dll");
        assert_eq!(
            out,
            "[General]\nenabled = true\ntarget_assembly=Z:\\home\\u\\BepInEx.Preloader.dll\nredirect_output_log = false\n"
        );
    }

    #[test]
    fn replaces_target_assembly_with_spaces_around_equals() {
        let ini = "target_assembly = old_value\n";
        let out = replace_target_assembly(ini, "new_value");
        assert_eq!(out, "target_assembly=new_value\n");
    }

    #[test]
    fn preserves_crlf_line_endings() {
        let ini = "[General]\r\ntarget_assembly=old\r\nother=keep\r\n";
        let out = replace_target_assembly(ini, "new");
        assert_eq!(out, "[General]\r\ntarget_assembly=new\r\nother=keep\r\n");
    }

    #[test]
    fn no_op_when_key_absent() {
        let ini = "[General]\nenabled = true\n";
        let out = replace_target_assembly(ini, "irrelevant");
        assert_eq!(out, ini);
    }

    #[test]
    fn unix_path_converts_to_wine_z_drive() {
        let p = Path::new("/home/user/profile/BepInEx/core/BepInEx.Preloader.dll");
        assert_eq!(
            unix_path_to_wine(p),
            "Z:\\home\\user\\profile\\BepInEx\\core\\BepInEx.Preloader.dll"
        );
    }
}
