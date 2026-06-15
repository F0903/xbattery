use super::BatteryLevel;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BatteryCharge {
    Coarse(BatteryLevel),
    Precise(u8),
    Unknown,
}

impl BatteryCharge {
    pub fn description(self) -> String {
        match self {
            Self::Coarse(level) => format!("{} (~{}%)", level, level.estimated_percent()),
            Self::Precise(percent) => format!("{}%", percent),
            Self::Unknown => "unknown".to_string(),
        }
    }
}
