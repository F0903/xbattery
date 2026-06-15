use std::fmt;

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
