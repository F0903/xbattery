use std::{
    fs,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::AppResult;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub(super) struct UpdateCheckState {
    last_check_unix_seconds: Option<u64>,
    last_notified_version: Option<String>,
}

impl UpdateCheckState {
    pub(super) fn load(path: &Path) -> Self {
        let Ok(content) = fs::read_to_string(path) else {
            return Self::default();
        };

        toml::from_str(&content).unwrap_or_default()
    }

    pub(super) fn save(&self, path: &Path) -> AppResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, toml::to_string(self)?)?;
        Ok(())
    }

    pub(super) fn is_due(&self, now: SystemTime, interval: Duration) -> bool {
        let Some(last_check) = self.last_check_unix_seconds else {
            return true;
        };

        unix_seconds(now).saturating_sub(last_check) >= interval.as_secs()
    }

    pub(super) fn mark_checked(&mut self, now: SystemTime) {
        self.last_check_unix_seconds = Some(unix_seconds(now));
    }

    pub(super) fn should_notify_for(&self, version: &str) -> bool {
        self.last_notified_version.as_deref() != Some(version)
    }

    pub(super) fn mark_notified(&mut self, version: impl Into<String>) {
        self.last_notified_version = Some(version.into());
    }
}

fn unix_seconds(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use super::UpdateCheckState;

    #[test]
    fn missing_last_check_is_due() {
        assert!(UpdateCheckState::default().is_due(UNIX_EPOCH, Duration::from_secs(60)));
    }

    #[test]
    fn recent_last_check_is_not_due() {
        let mut state = UpdateCheckState::default();
        state.mark_checked(UNIX_EPOCH + Duration::from_secs(100));

        assert!(!state.is_due(
            UNIX_EPOCH + Duration::from_secs(120),
            Duration::from_secs(60)
        ));
    }

    #[test]
    fn old_last_check_is_due() {
        let mut state = UpdateCheckState::default();
        state.mark_checked(UNIX_EPOCH + Duration::from_secs(100));

        assert!(state.is_due(
            UNIX_EPOCH + Duration::from_secs(161),
            Duration::from_secs(60)
        ));
    }

    #[test]
    fn notification_version_is_tracked() {
        let mut state = UpdateCheckState::default();
        assert!(state.should_notify_for("0.2.0"));

        state.mark_notified("0.2.0");

        assert!(!state.should_notify_for("0.2.0"));
        assert!(state.should_notify_for("0.3.0"));
    }
}
