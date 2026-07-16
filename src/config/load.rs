use std::{
    env, fs,
    path::{Path, PathBuf},
};

use super::{
    AppConfig, ConfigIssue, LoadedAppConfig,
    watcher::{ConfigWatchEvents, PreparedConfigWatch},
};
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

pub(super) fn load_with_source() -> AppResult<LoadedAppConfig> {
    let Some(path) = selected_config_path()? else {
        return Ok(LoadedAppConfig::new(AppConfig::default(), None, None));
    };

    let config = load_from_path(&path)?;
    Ok(LoadedAppConfig::new(config, Some(path), None))
}

pub(super) fn load_for_monitor() -> AppResult<(LoadedAppConfig, Option<ConfigWatchEvents>)> {
    let Some(path) = selected_config_path()? else {
        return Ok((LoadedAppConfig::new(AppConfig::default(), None, None), None));
    };

    let (loaded, prepared_watch) = prepare_monitor_path(path);
    Ok((loaded, Some(prepared_watch.start())))
}

fn selected_config_path() -> AppResult<Option<PathBuf>> {
    if let Some(path) = env::var_os(CONFIG_ENV_VAR).map(PathBuf::from) {
        return Ok(Some(path));
    }

    Ok(default_config_paths()?
        .into_iter()
        .find(|path| path.exists()))
}

fn load_path_for_monitor(path: PathBuf) -> LoadedAppConfig {
    match load_from_path(&path) {
        Ok(config) => LoadedAppConfig::new(config, Some(path), None),
        Err(error) => LoadedAppConfig::new(
            AppConfig::default(),
            Some(path.clone()),
            Some(ConfigIssue::new(path, error)),
        ),
    }
}

fn prepare_monitor_path(path: PathBuf) -> (LoadedAppConfig, PreparedConfigWatch) {
    prepare_monitor_path_with(path, || {})
}

fn prepare_monitor_path_with(
    path: PathBuf,
    after_initial_load: impl FnOnce(),
) -> (LoadedAppConfig, PreparedConfigWatch) {
    // Capture the revision before reading. A save that lands during or directly
    // after the read will then differ from the watcher's baseline.
    let prepared_watch = PreparedConfigWatch::new(path.clone());
    let loaded = load_path_for_monitor(path);
    after_initial_load();
    (loaded, prepared_watch)
}

pub(super) fn load_from_path(path: impl AsRef<Path>) -> AppResult<AppConfig> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {}", path.display(), error))?;
    let mut config = toml::from_str::<AppConfig>(&content)
        .map_err(|error| format!("failed to parse {}: {}", path.display(), error))?;

    config.resolve_relative_paths(path.parent());
    config.validate()?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{Duration, SystemTime},
    };

    use crate::config::ConfigWatchEvent;

    use super::{load_from_path, load_path_for_monitor, prepare_monitor_path_with};

    #[test]
    fn monitor_falls_back_without_changing_an_outdated_config() {
        let temp_dir = unique_temp_dir("xbattery-outdated-config-test");
        fs::create_dir_all(&temp_dir).unwrap();
        let path = temp_dir.join("xbattery.toml");
        let outdated = r#"
            [battery]
            precise_warning_thresholds = [50, 25, 10]

            [notifications]
            urgent_precise_threshold_percent = 10

            [rumble]
            enabled = false
        "#;
        fs::write(&path, outdated).unwrap();

        assert!(load_from_path(&path).is_err());

        let loaded = load_path_for_monitor(path.clone());

        assert_eq!(loaded.path.as_deref(), Some(path.as_path()));
        assert!(loaded.issue.is_some());
        assert!(loaded.config.battery.levels.is_none());
        assert_eq!(fs::read_to_string(&path).unwrap(), outdated);

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn monitor_falls_back_for_a_config_that_fails_validation() {
        let temp_dir = unique_temp_dir("xbattery-invalid-config-test");
        fs::create_dir_all(&temp_dir).unwrap();
        let path = temp_dir.join("xbattery.toml");
        fs::write(
            &path,
            r#"
            [monitor]
            poll_interval_seconds = 0
            "#,
        )
        .unwrap();

        let loaded = load_path_for_monitor(path.clone());
        let issue = loaded.issue.unwrap();

        assert_eq!(loaded.path.as_deref(), Some(path.as_path()));
        assert!(loaded.config.battery.levels.is_none());
        assert!(issue.message().contains("poll_interval_seconds"));

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn monitor_observes_a_correction_between_initial_load_and_watch_start() {
        let temp_dir = unique_temp_dir("xbattery-config-load-watch-race-test");
        fs::create_dir_all(&temp_dir).unwrap();
        let path = temp_dir.join("xbattery.toml");
        fs::write(&path, "[monitor\npoll_interval_seconds = 5").unwrap();

        let corrected_path = path.clone();
        let (loaded, prepared_watch) = prepare_monitor_path_with(path.clone(), || {
            fs::write(
                &corrected_path,
                r#"
                [notifications]
                app_id = "corrected-during-startup"
                "#,
            )
            .unwrap();
        });

        assert!(loaded.issue.is_some());
        let events = prepared_watch.start();
        match events.recv_timeout(Duration::from_secs(3)).unwrap() {
            ConfigWatchEvent::Loaded { config, .. } => {
                assert_eq!(config.notifications.app_id, "corrected-during-startup");
            }
            ConfigWatchEvent::Rejected(issue) => {
                panic!("corrected config was rejected: {issue}")
            }
        }

        drop(events);
        fs::remove_dir_all(temp_dir).unwrap();
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "{prefix}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }
}
