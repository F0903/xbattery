use std::{
    env, fs,
    path::{Path, PathBuf},
};

use crate::AppResult;

const EXE_NAME: &str = "xbattery.exe";
const CONFIG_NAME: &str = "xbattery.toml";

#[derive(Clone, Debug)]
pub(super) struct StartupPaths {
    pub(super) install_dir: PathBuf,
    pub(super) installed_exe: PathBuf,
    pub(super) installed_config: PathBuf,
}

impl StartupPaths {
    pub(super) fn current_user() -> AppResult<Self> {
        let install_dir = install_dir()?;

        Ok(Self {
            installed_exe: install_dir.join(EXE_NAME),
            installed_config: install_dir.join(CONFIG_NAME),
            install_dir,
        })
    }

    pub(super) fn source_config_path() -> AppResult<Option<PathBuf>> {
        let current_dir_config = env::current_dir()?.join(CONFIG_NAME);
        if current_dir_config.exists() {
            return Ok(Some(current_dir_config));
        }

        let exe_dir_config = env::current_exe()?
            .parent()
            .map(|path| path.join(CONFIG_NAME))
            .filter(|path| path.exists());

        Ok(exe_dir_config)
    }

    pub(super) fn same_path(left: &Path, right: &Path) -> bool {
        match (fs::canonicalize(left), fs::canonicalize(right)) {
            (Ok(left), Ok(right)) => left == right,
            _ => left == right,
        }
    }
}

fn install_dir() -> AppResult<PathBuf> {
    let local_app_data = env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .ok_or("LOCALAPPDATA is not set")?;

    Ok(local_app_data.join("Programs").join("xbattery"))
}
