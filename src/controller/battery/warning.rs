use crate::controller::battery::{BatteryCharge, BatteryLevel, BatteryReading};

const DEFAULT_PRECISE_THRESHOLDS: [u8; 3] = [50, 25, 10];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BatteryWarning {
    Precise(u8),
    Coarse(BatteryLevel),
}

#[derive(Clone, Debug)]
pub struct BatteryWarningPolicy {
    precise_thresholds: Vec<u8>,
}

impl BatteryWarningPolicy {
    pub fn new(mut precise_thresholds: Vec<u8>) -> Self {
        precise_thresholds.sort_unstable_by(|left, right| right.cmp(left));
        precise_thresholds.dedup();

        Self { precise_thresholds }
    }

    pub fn warning_between(
        &self,
        previous: BatteryReading,
        current: BatteryReading,
    ) -> Option<BatteryWarning> {
        match (previous.charge, current.charge) {
            (BatteryCharge::Precise(previous), BatteryCharge::Precise(current)) => self
                .crossed_precise_threshold(previous, current)
                .map(BatteryWarning::Precise),
            (BatteryCharge::Coarse(previous), BatteryCharge::Coarse(current)) => self
                .crossed_coarse_level(previous, current)
                .map(BatteryWarning::Coarse),
            _ => None,
        }
    }

    fn crossed_precise_threshold(&self, previous: u8, current: u8) -> Option<u8> {
        self.precise_thresholds
            .iter()
            .rev()
            .copied()
            .find(|threshold| previous > *threshold && current <= *threshold)
    }

    fn crossed_coarse_level(
        &self,
        previous: BatteryLevel,
        current: BatteryLevel,
    ) -> Option<BatteryLevel> {
        if current < previous && current.is_warning_level() {
            Some(current)
        } else {
            None
        }
    }
}

impl Default for BatteryWarningPolicy {
    fn default() -> Self {
        Self::new(DEFAULT_PRECISE_THRESHOLDS.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use crate::controller::battery::{BatteryCharge, BatteryKind, BatteryLevel, BatteryReading};

    use super::{BatteryWarning, BatteryWarningPolicy};

    #[test]
    fn warns_when_precise_charge_crosses_threshold() {
        let policy = BatteryWarningPolicy::default();

        let warning = policy.warning_between(
            reading(BatteryCharge::Precise(51)),
            reading(BatteryCharge::Precise(50)),
        );

        assert_eq!(warning, Some(BatteryWarning::Precise(50)));
    }

    #[test]
    fn warns_when_coarse_charge_drops_to_warning_level() {
        let policy = BatteryWarningPolicy::default();

        let warning = policy.warning_between(
            reading(BatteryCharge::Coarse(BatteryLevel::Full)),
            reading(BatteryCharge::Coarse(BatteryLevel::Medium)),
        );

        assert_eq!(warning, Some(BatteryWarning::Coarse(BatteryLevel::Medium)));
    }

    #[test]
    fn ignores_charge_recovery() {
        let policy = BatteryWarningPolicy::default();

        let warning = policy.warning_between(
            reading(BatteryCharge::Coarse(BatteryLevel::Low)),
            reading(BatteryCharge::Coarse(BatteryLevel::Full)),
        );

        assert_eq!(warning, None);
    }

    fn reading(charge: BatteryCharge) -> BatteryReading {
        BatteryReading::new(BatteryKind::Unknown, charge)
    }
}
