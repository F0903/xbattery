use crate::controller::battery::{BatteryLevel, BatteryWarning};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BatteryWarningStage {
    Medium,
    Low,
    Empty,
}

impl BatteryWarningStage {
    pub(crate) fn diagnostic(warning_level: u8) -> Self {
        match warning_level {
            0 | 1 => Self::Medium,
            2 => Self::Low,
            _ => Self::Empty,
        }
    }

    pub(crate) fn for_warning(warning: BatteryWarning) -> Option<Self> {
        match warning {
            BatteryWarning::Precise(percent) if percent <= 10 => Some(Self::Empty),
            BatteryWarning::Precise(percent) if percent <= 25 => Some(Self::Low),
            BatteryWarning::Precise(_) => Some(Self::Medium),
            BatteryWarning::Coarse(BatteryLevel::Empty) => Some(Self::Empty),
            BatteryWarning::Coarse(BatteryLevel::Low) => Some(Self::Low),
            BatteryWarning::Coarse(BatteryLevel::Medium) => Some(Self::Medium),
            BatteryWarning::Coarse(BatteryLevel::Full) => None,
        }
    }
}
