use std::{
    fs,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use super::{AppConfig, ConfigIssue};

const WATCH_INTERVAL: Duration = Duration::from_secs(1);
const WRITE_DEBOUNCE: Duration = Duration::from_millis(250);

type FileRevision = Vec<u8>;

#[derive(Clone, Debug)]
pub enum ConfigWatchEvent {
    Loaded { path: PathBuf, config: AppConfig },
    Rejected(ConfigIssue),
}

pub(crate) type ConfigWatchEvents = Receiver<ConfigWatchEvent>;

pub(super) struct PreparedConfigWatch {
    path: PathBuf,
    initial_revision: Option<FileRevision>,
}

impl PreparedConfigWatch {
    pub(super) fn new(path: PathBuf) -> Self {
        let initial_revision = file_revision(&path);
        Self {
            path,
            initial_revision,
        }
    }

    pub(super) fn start(self) -> ConfigWatchEvents {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || watch_loop(self.path, self.initial_revision, tx));
        rx
    }
}

fn watch_loop(
    path: PathBuf,
    mut last_revision: Option<FileRevision>,
    tx: Sender<ConfigWatchEvent>,
) {
    loop {
        thread::sleep(WATCH_INTERVAL);

        let current_revision = file_revision(&path);
        if current_revision == last_revision {
            continue;
        }

        thread::sleep(WRITE_DEBOUNCE);
        last_revision = file_revision(&path);

        if tx.send(load_event(&path)).is_err() {
            break;
        }
    }
}

fn load_event(path: &Path) -> ConfigWatchEvent {
    match AppConfig::load_from_path(path) {
        Ok(config) => ConfigWatchEvent::Loaded {
            path: path.to_path_buf(),
            config,
        },
        Err(error) => ConfigWatchEvent::Rejected(ConfigIssue::new(path.to_path_buf(), error)),
    }
}

fn file_revision(path: &Path) -> Option<FileRevision> {
    fs::read(path).ok()
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{Duration, SystemTime},
    };

    use super::{ConfigWatchEvent, PreparedConfigWatch, load_event};

    #[test]
    fn prepared_watcher_observes_a_correction_before_thread_start() {
        let temp_dir = unique_temp_dir("xbattery-config-watch-race-test");
        fs::create_dir_all(&temp_dir).unwrap();
        let path = temp_dir.join("xbattery.toml");
        fs::write(&path, "[monitor\npoll_interval_seconds = 5").unwrap();
        let prepared_watch = PreparedConfigWatch::new(path.clone());

        fs::write(
            &path,
            r#"
            [notifications]
            app_id = "corrected-immediately"
            "#,
        )
        .unwrap();
        let events = prepared_watch.start();

        match events.recv_timeout(Duration::from_secs(3)).unwrap() {
            ConfigWatchEvent::Loaded { config, .. } => {
                assert_eq!(config.notifications.app_id, "corrected-immediately");
            }
            ConfigWatchEvent::Rejected(issue) => {
                panic!("corrected config was rejected: {issue}")
            }
        }

        drop(events);
        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn reload_events_report_rejection_then_accept_a_corrected_file() {
        let temp_dir = unique_temp_dir("xbattery-config-event-test");
        fs::create_dir_all(&temp_dir).unwrap();
        let path = temp_dir.join("xbattery.toml");
        fs::write(&path, "[monitor\npoll_interval_seconds = 5").unwrap();

        match load_event(&path) {
            ConfigWatchEvent::Rejected(issue) => assert_eq!(issue.path(), path),
            ConfigWatchEvent::Loaded { .. } => panic!("invalid config was accepted"),
        }

        fs::write(
            &path,
            r#"
            [notifications]
            app_id = "corrected"
            "#,
        )
        .unwrap();

        match load_event(&path) {
            ConfigWatchEvent::Loaded {
                path: loaded_path,
                config,
            } => {
                assert_eq!(loaded_path, path);
                assert_eq!(config.notifications.app_id, "corrected");
            }
            ConfigWatchEvent::Rejected(issue) => {
                panic!("corrected config was rejected: {issue}")
            }
        }

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
