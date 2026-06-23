use std::fmt;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum BatteryLevel {
    Empty,
    Low,
    Medium,
    Full,
}

impl BatteryLevel {
    pub fn estimated_percent(self) -> u8 {
        match self {
            Self::Full => 100,
            Self::Medium => 70,
            Self::Low => 40,
            Self::Empty => 10,
        }
    }

    pub fn is_warning_level(self) -> bool {
        !matches!(self, Self::Full)
    }
}

impl fmt::Display for BatteryLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty"),
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::Full => write!(f, "full"),
        }
    }
}
