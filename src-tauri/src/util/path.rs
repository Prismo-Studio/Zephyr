use std::path::PathBuf;

pub const APP_GUID: &str = "com.prismo-studio.zephyr";

pub fn default_app_config_dir() -> PathBuf {
    app_dir("config", dirs_next::config_dir())
}

pub fn default_app_data_dir() -> PathBuf {
    app_dir("data", dirs_next::data_dir())
}

fn app_dir(id: &str, base: Option<PathBuf>) -> PathBuf {
    let mut path = base.unwrap_or_else(|| panic!("failed to resolve {id} dir"));
    path.push(APP_GUID);
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_guid_value() {
        assert_eq!(APP_GUID, "com.prismo-studio.zephyr");
    }

    #[test]
    fn config_dir_contains_guid() {
        let dir = default_app_config_dir();
        assert!(dir.ends_with(APP_GUID));
    }

    #[test]
    fn data_dir_contains_guid() {
        let dir = default_app_data_dir();
        assert!(dir.ends_with(APP_GUID));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn config_and_data_dirs_differ() {
        assert_ne!(default_app_config_dir(), default_app_data_dir());
    }
}
