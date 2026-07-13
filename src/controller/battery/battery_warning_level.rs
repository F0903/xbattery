use crate::{audio::AudioClip, controller::battery::BatteryLevel};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatteryWarningLevel {
    name: String,
    precise_threshold_percent: Option<u8>,
    coarse_level: Option<BatteryLevel>,
    notify: bool,
    urgent: bool,
    audio: Option<AudioClip>,
}

impl BatteryWarningLevel {
    pub fn with_notify(
        name: impl Into<String>,
        precise_threshold_percent: Option<u8>,
        coarse_level: Option<BatteryLevel>,
        notify: bool,
        urgent: bool,
    ) -> Self {
        Self::with_notify_and_audio(
            name,
            precise_threshold_percent,
            coarse_level,
            notify,
            urgent,
            None,
        )
    }

    pub fn with_notify_and_audio(
        name: impl Into<String>,
        precise_threshold_percent: Option<u8>,
        coarse_level: Option<BatteryLevel>,
        notify: bool,
        urgent: bool,
        audio: Option<AudioClip>,
    ) -> Self {
        Self {
            name: name.into(),
            precise_threshold_percent,
            coarse_level,
            notify,
            urgent,
            audio,
        }
    }

    pub fn default_levels() -> Vec<Self> {
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
                false,
            ),
            Self::with_notify(
                "low",
                Some(BatteryLevel::Low.estimated_percent()),
                Some(BatteryLevel::Low),
                true,
                false,
            ),
            Self::with_notify(
                "empty",
                Some(BatteryLevel::Empty.estimated_percent()),
                Some(BatteryLevel::Empty),
                true,
                true,
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

    pub fn audio(&self) -> Option<&AudioClip> {
        self.audio.as_ref()
    }

    pub fn has_action(&self) -> bool {
        self.notify || self.audio.is_some()
    }
}
