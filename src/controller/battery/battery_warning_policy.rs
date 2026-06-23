use crate::controller::battery::{
    BatteryCharge, BatteryLevel, BatteryReading, BatteryWarning, BatteryWarningLevel,
};

#[derive(Clone, Debug)]
pub struct BatteryWarningPolicy {
    levels: Vec<BatteryWarningLevel>,
}

impl BatteryWarningPolicy {
    pub fn new(levels: Vec<BatteryWarningLevel>) -> Self {
        Self { levels }
    }

    pub fn warning_between(
        &self,
        previous: BatteryReading,
        current: BatteryReading,
    ) -> Option<BatteryWarning> {
        match (previous.charge, current.charge) {
            (BatteryCharge::Precise(previous), BatteryCharge::Precise(current)) => {
                self.crossed_precise_threshold(previous, current)
            }
            (BatteryCharge::Coarse(previous), BatteryCharge::Coarse(current)) => {
                self.crossed_coarse_level(previous, current)
            }
            _ => None,
        }
    }

    fn crossed_precise_threshold(&self, previous: u8, current: u8) -> Option<BatteryWarning> {
        self.levels
            .iter()
            .filter(|level| level.notify())
            .filter_map(|level| {
                level
                    .precise_threshold_percent()
                    .map(|threshold| (threshold, level))
            })
            .filter(|(threshold, _)| previous > *threshold && current <= *threshold)
            .min_by_key(|(threshold, _)| *threshold)
            .map(|(threshold, level)| BatteryWarning::precise(threshold, level.clone()))
    }

    fn crossed_coarse_level(
        &self,
        previous: BatteryLevel,
        current: BatteryLevel,
    ) -> Option<BatteryWarning> {
        if current >= previous {
            return None;
        }

        self.levels
            .iter()
            .find(|level| level.notify() && level.coarse_level() == Some(current))
            .map(|level| BatteryWarning::coarse(current, level.clone()))
    }
}

impl Default for BatteryWarningPolicy {
    fn default() -> Self {
        Self::new(BatteryWarningLevel::default_levels())
    }
}

#[cfg(test)]
mod tests {
    use crate::controller::battery::{
        BatteryCharge, BatteryKind, BatteryLevel, BatteryReading, BatteryWarningLevel,
    };

    use super::{BatteryWarning, BatteryWarningPolicy};

    #[test]
    fn warns_when_precise_charge_crosses_threshold() {
        let policy = BatteryWarningPolicy::default();

        let warning = policy.warning_between(
            reading(BatteryCharge::Precise(71)),
            reading(BatteryCharge::Precise(70)),
        );

        assert_eq!(warning, Some(BatteryWarning::precise(70, medium_level())));
    }

    #[test]
    fn warns_when_coarse_charge_drops_to_warning_level() {
        let policy = BatteryWarningPolicy::default();

        let warning = policy.warning_between(
            reading(BatteryCharge::Coarse(BatteryLevel::Full)),
            reading(BatteryCharge::Coarse(BatteryLevel::Medium)),
        );

        assert_eq!(
            warning,
            Some(BatteryWarning::coarse(BatteryLevel::Medium, medium_level()))
        );
    }

    #[test]
    fn reports_most_severe_crossed_precise_threshold() {
        let policy = BatteryWarningPolicy::default();

        let warning = policy.warning_between(
            reading(BatteryCharge::Precise(71)),
            reading(BatteryCharge::Precise(9)),
        );

        assert_eq!(warning, Some(BatteryWarning::precise(10, empty_level())));
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

    #[test]
    fn ignores_non_notifying_levels() {
        let policy = BatteryWarningPolicy::new(vec![BatteryWarningLevel::with_notify(
            "full",
            Some(100),
            Some(BatteryLevel::Full),
            false,
            false,
        )]);

        let warning = policy.warning_between(
            reading(BatteryCharge::Precise(100)),
            reading(BatteryCharge::Precise(99)),
        );

        assert_eq!(warning, None);
    }

    fn reading(charge: BatteryCharge) -> BatteryReading {
        BatteryReading::new(BatteryKind::Unknown, charge)
    }

    fn medium_level() -> BatteryWarningLevel {
        BatteryWarningLevel::new("medium", Some(70), Some(BatteryLevel::Medium), false)
    }

    fn empty_level() -> BatteryWarningLevel {
        BatteryWarningLevel::new("empty", Some(10), Some(BatteryLevel::Empty), true)
    }
}
