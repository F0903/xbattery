use std::time::Duration;

use crate::controller::{
    battery::BatteryWarningPolicy, event::ControllerNotificationPolicy,
    rumble::ControllerRumbleConfig,
};

#[derive(Clone, Debug)]
pub struct ControllerServiceConfig {
    poll_interval: Duration,
    control_wait_slice: Duration,
    warning_policy: BatteryWarningPolicy,
    notification_policy: ControllerNotificationPolicy,
    rumble_config: ControllerRumbleConfig,
}

impl ControllerServiceConfig {
    pub fn new(
        poll_interval: Duration,
        control_wait_slice: Duration,
        warning_policy: BatteryWarningPolicy,
        notification_policy: ControllerNotificationPolicy,
        rumble_config: ControllerRumbleConfig,
    ) -> Self {
        Self {
            poll_interval,
            control_wait_slice,
            warning_policy,
            notification_policy,
            rumble_config,
        }
    }

    pub(super) fn poll_interval(&self) -> Duration {
        self.poll_interval
    }

    pub(super) fn control_wait_slice(&self) -> Duration {
        self.control_wait_slice
    }

    pub(super) fn warning_policy(&self) -> &BatteryWarningPolicy {
        &self.warning_policy
    }

    pub(super) fn notification_policy(&self) -> &ControllerNotificationPolicy {
        &self.notification_policy
    }

    pub(super) fn rumble_config(&self) -> &ControllerRumbleConfig {
        &self.rumble_config
    }
}

impl Default for ControllerServiceConfig {
    fn default() -> Self {
        Self::new(
            Duration::from_secs(60),
            Duration::from_millis(250),
            BatteryWarningPolicy::default(),
            ControllerNotificationPolicy::default(),
            ControllerRumbleConfig::default(),
        )
    }
}
