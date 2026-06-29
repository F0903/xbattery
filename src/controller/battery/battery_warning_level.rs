use std::path::{Path, PathBuf};

use crate::controller::battery::BatteryLevel;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatteryWarningLevel {
    name: String,
    precise_threshold_percent: Option<u8>,
    coarse_level: Option<BatteryLevel>,
    notify: bool,
    urgent: bool,
    sound_file: Option<PathBuf>,
}

impl BatteryWarningLevel {
    pub fn new(
        name: impl Into<String>,
        precise_threshold_percent: Option<u8>,
        coarse_level: Option<BatteryLevel>,
        urgent: bool,
    ) -> Self {
        Self::with_notify_and_file(
            name,
            precise_threshold_percent,
            coarse_level,
            true,
            urgent,
            None,
        )
    }

    pub fn with_notify(
        name: impl Into<String>,
        precise_threshold_percent: Option<u8>,
        coarse_level: Option<BatteryLevel>,
        notify: bool,
        urgent: bool,
    ) -> Self {
        Self::with_notify_and_file(
            name,
            precise_threshold_percent,
            coarse_level,
            notify,
            urgent,
            None,
        )
    }

    pub fn with_notify_and_file(
        name: impl Into<String>,
        precise_threshold_percent: Option<u8>,
        coarse_level: Option<BatteryLevel>,
        notify: bool,
        urgent: bool,
        sound_file: Option<PathBuf>,
    ) -> Self {
        Self {
            name: name.into(),
            precise_threshold_percent,
            coarse_level,
            notify,
            urgent,
            sound_file,
        }
    }

    pub fn default_levels() -> Vec<Self> {
        Self::default_levels_with_urgent_threshold(10)
    }

    pub fn default_levels_with_urgent_threshold(urgent_threshold_percent: u8) -> Vec<Self> {
        vec![
            Self::with_notify(
                "full",
                Some(BatteryLevel::Full.estimated_percent()),
                Some(BatteryLevel::Full),
                false,
                false,
            ),
            Self::with_notify(
                "medium",
                Some(BatteryLevel::Medium.estimated_percent()),
                Some(BatteryLevel::Medium),
                true,
                BatteryLevel::Medium.estimated_percent() <= urgent_threshold_percent,
            ),
            Self::with_notify(
                "low",
                Some(BatteryLevel::Low.estimated_percent()),
                Some(BatteryLevel::Low),
                true,
                BatteryLevel::Low.estimated_percent() <= urgent_threshold_percent,
            ),
            Self::with_notify(
                "empty",
                Some(BatteryLevel::Empty.estimated_percent()),
                Some(BatteryLevel::Empty),
                true,
                BatteryLevel::Empty.estimated_percent() <= urgent_threshold_percent,
            ),
        ]
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn precise_threshold_percent(&self) -> Option<u8> {
        self.precise_threshold_percent
    }

    pub fn coarse_level(&self) -> Option<BatteryLevel> {
        self.coarse_level
    }

    pub fn notify(&self) -> bool {
        self.notify
    }

    pub fn urgent(&self) -> bool {
        self.urgent
    }

    pub fn sound_file(&self) -> Option<&Path> {
        self.sound_file.as_deref()
    }

    pub fn has_action(&self) -> bool {
        self.notify || self.sound_file.is_some()
    }
}
