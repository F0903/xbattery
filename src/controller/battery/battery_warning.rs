use crate::controller::battery::{BatteryLevel, BatteryWarningLevel};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatteryWarning {
    reading: BatteryWarningReading,
    level: BatteryWarningLevel,
}

impl BatteryWarning {
    pub fn precise(percent: u8, level: BatteryWarningLevel) -> Self {
        Self {
            reading: BatteryWarningReading::Precise(percent),
            level,
        }
    }

    pub fn coarse(coarse_level: BatteryLevel, level: BatteryWarningLevel) -> Self {
        Self {
            reading: BatteryWarningReading::Coarse(coarse_level),
            level,
        }
    }

    pub fn reading(&self) -> BatteryWarningReading {
        self.reading
    }

    pub fn level(&self) -> &BatteryWarningLevel {
        &self.level
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BatteryWarningReading {
    Precise(u8),
    Coarse(BatteryLevel),
}
