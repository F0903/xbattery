use std::fmt;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BatteryLevel {
    #[default]
    Unknown,
    Empty,
    Low,
    Medium,
    Full,
}

impl BatteryLevel {
    pub fn estimated_percent(self) -> Option<u8> {
        match self {
            Self::Unknown => None,
            Self::Full => Some(100),
            Self::Medium => Some(70),
            Self::Low => Some(40),
            Self::Empty => Some(10),
        }
    }
}

impl fmt::Display for BatteryLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => write!(f, "unknown"),
            Self::Empty => write!(f, "empty"),
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::Full => write!(f, "full"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BatteryLevel;

    #[test]
    fn unknown_is_the_default_without_an_estimated_percentage() {
        let level = BatteryLevel::default();

        assert_eq!(level, BatteryLevel::Unknown);
        assert_eq!(level.estimated_percent(), None);
        assert_eq!(level.to_string(), "unknown");
    }
}
