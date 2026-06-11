use std::fmt;

pub mod warning;
pub use warning::{BatteryWarning, BatteryWarningPolicy};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BatteryKind {
    Disconnected,
    Wired,
    Alkaline,
    Nimh,
    Unknown,
}

impl fmt::Display for BatteryKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disconnected => write!(f, "disconnected"),
            Self::Wired => write!(f, "wired"),
            Self::Alkaline => write!(f, "alkaline"),
            Self::Nimh => write!(f, "NiMH"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
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
            Self::Medium => 50,
            Self::Low => 25,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BatteryReading {
    pub kind: BatteryKind,
    pub charge: BatteryCharge,
}

impl BatteryReading {
    pub fn new(kind: BatteryKind, charge: BatteryCharge) -> Self {
        Self { kind, charge }
    }

    pub fn description(self) -> String {
        match (self.kind, self.charge) {
            (BatteryKind::Wired, _) => "wired".to_string(),
            (_, BatteryCharge::Unknown) => self.kind.to_string(),
            (_, charge) => charge.description(),
        }
    }
}
