use super::{BatteryCharge, BatteryKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BatteryReading {
    pub kind: BatteryKind,
    pub charge: BatteryCharge,
}

impl BatteryReading {
    pub fn new(kind: BatteryKind, charge: BatteryCharge) -> Self {
        Self { kind, charge }
    }

    #[cfg(debug_assertions)]
    pub fn description(self) -> String {
        if self.kind == BatteryKind::Wired {
            return "wired".to_string();
        }

        if self.charge.is_unknown() {
            return self.kind.to_string();
        }

        self.charge.description()
    }
}
