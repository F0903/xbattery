use super::BatteryLevel;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BatteryCharge {
    Coarse(BatteryLevel),
    Precise(u8),
    Unknown,
}

impl BatteryCharge {
    pub fn estimated_percent(self) -> Option<u8> {
        match self {
            Self::Coarse(level) => level.estimated_percent(),
            Self::Precise(percent) => Some(percent),
            Self::Unknown => None,
        }
    }

    pub fn is_unknown(self) -> bool {
        self.estimated_percent().is_none()
    }

    #[cfg(debug_assertions)]
    pub fn description(self) -> String {
        match self {
            Self::Coarse(level) => level.estimated_percent().map_or_else(
                || "unknown".to_string(),
                |percent| format!("{level} (~{percent}%)"),
            ),
            Self::Precise(percent) => format!("{}%", percent),
            Self::Unknown => "unknown".to_string(),
        }
    }
}
