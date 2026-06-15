use std::{
    env, fs,
    path::{Path, PathBuf},
};

use super::{AppConfig, LoadedAppConfig};
use crate::AppResult;

const DEFAULT_CONFIG_FILE_NAME: &str = "xbattery.toml";
const CONFIG_ENV_VAR: &str = "XBATTERY_CONFIG";

fn default_config_paths() -> AppResult<Vec<PathBuf>> {
    let mut paths = vec![env::current_dir()?.join(DEFAULT_CONFIG_FILE_NAME)];

    if let Ok(exe_path) = env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        let exe_config = exe_dir.join(DEFAULT_CONFIG_FILE_NAME);
        if !paths.iter().any(|path| path == &exe_config) {
            paths.push(exe_config);
        }
    }

    Ok(paths)
}

pub(super) fn load() -> AppResult<AppConfig> {
    Ok(load_with_source()?.config)
}

pub(super) fn load_with_source() -> AppResult<LoadedAppConfig> {
    if let Some(path) = env::var_os(CONFIG_ENV_VAR).map(PathBuf::from) {
        let config = load_from_path(&path)?;
        return Ok(LoadedAppConfig::new(config, Some(path)));
    }

    for path in default_config_paths()? {
        if path.exists() {
            let config = load_from_path(&path)?;
            return Ok(LoadedAppConfig::new(config, Some(path)));
        }
    }

    Ok(LoadedAppConfig::new(AppConfig::default(), None))
}

pub(super) fn load_from_path(path: impl AsRef<Path>) -> AppResult<AppConfig> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {}", path.display(), error))?;
    let config = toml::from_str::<AppConfig>(&content)
        .map_err(|error| format!("failed to parse {}: {}", path.display(), error))?;

    config.validate()?;
    Ok(config)
}
