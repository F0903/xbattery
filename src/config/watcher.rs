use std::{
    fs,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, SystemTime},
};

use super::AppConfig;

const WATCH_INTERVAL: Duration = Duration::from_secs(1);
const WRITE_DEBOUNCE: Duration = Duration::from_millis(250);

pub fn watch(path: PathBuf) -> Receiver<AppConfig> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || watch_loop(path, tx));
    rx
}

fn watch_loop(path: PathBuf, tx: Sender<AppConfig>) {
    let mut last_modified = modified_time(&path);

    loop {
        thread::sleep(WATCH_INTERVAL);

        let current_modified = modified_time(&path);
        if current_modified == last_modified {
            continue;
        }

        last_modified = current_modified;
        thread::sleep(WRITE_DEBOUNCE);

        match AppConfig::load_from_path(&path) {
            Ok(config) => {
                if tx.send(config).is_err() {
                    break;
                }
            }
            Err(error) => eprintln!("failed to reload config {}: {error}", path.display()),
        }
    }
}

fn modified_time(path: &Path) -> Option<SystemTime> {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
}
