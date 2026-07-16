use std::time::Duration;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct MonitorConfig {
    pub(crate) poll_interval_seconds: u64,
    pub(crate) control_wait_slice_millis: u64,
}

impl MonitorConfig {
    pub(crate) fn poll_interval(&self) -> Duration {
        Duration::from_secs(self.poll_interval_seconds)
    }

    pub(crate) fn control_wait_slice(&self) -> Duration {
        Duration::from_millis(self.control_wait_slice_millis)
    }
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            poll_interval_seconds: 60,
            control_wait_slice_millis: 250,
        }
    }
}
