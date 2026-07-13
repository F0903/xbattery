use crate::controller::battery::{
    BatteryCharge, BatteryLevel, BatteryReading, BatteryWarning, BatteryWarningLevel,
};

#[derive(Clone, Debug, Eq, PartialEq)]
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
            (BatteryCharge::Coarse(previous), BatteryCharge::Precise(current)) => {
                self.crossed_precise_threshold(previous.estimated_percent()?, current)
            }
            (BatteryCharge::Precise(previous), BatteryCharge::Coarse(current)) => {
                self.crossed_coarse_level_from_percent(previous, current)
            }
            (BatteryCharge::Coarse(previous), BatteryCharge::Coarse(current)) => {
                self.crossed_coarse_level(previous, current)
            }
            _ => None,
        }
    }

    pub(crate) fn warning_for_current(&self, current: BatteryReading) -> Option<BatteryWarning> {
        match current.charge {
            BatteryCharge::Precise(current) => self
                .levels
                .iter()
                .filter(|level| level.has_action())
                .filter_map(|level| {
                    level
                        .precise_threshold_percent()
                        .map(|threshold| (threshold, level))
                })
                .filter(|(threshold, _)| current <= *threshold)
                .min_by_key(|(threshold, _)| *threshold)
                .map(|(threshold, level)| BatteryWarning::precise(threshold, level.clone())),
            BatteryCharge::Coarse(current) if current != BatteryLevel::Unknown => self
                .levels
                .iter()
                .find(|level| level.has_action() && level.coarse_level() == Some(current))
                .map(|level| BatteryWarning::coarse(current, level.clone())),
            BatteryCharge::Coarse(_) | BatteryCharge::Unknown => None,
        }
    }

    fn crossed_precise_threshold(&self, previous: u8, current: u8) -> Option<BatteryWarning> {
        self.levels
            .iter()
            .filter(|level| level.has_action())
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
        self.crossed_coarse_level_from_percent(previous.estimated_percent()?, current)
    }

    fn crossed_coarse_level_from_percent(
        &self,
        previous_percent: u8,
        current: BatteryLevel,
    ) -> Option<BatteryWarning> {
        let current_percent = current.estimated_percent()?;
        if current_percent >= previous_percent {
            return None;
        }

        self.levels
            .iter()
            .find(|level| level.has_action() && level.coarse_level() == Some(current))
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
    use crate::{
        audio::AudioClip,
        controller::battery::{
            BatteryCharge, BatteryKind, BatteryLevel, BatteryReading, BatteryWarningLevel,
        },
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
    fn ignores_unknown_charge_on_either_side() {
        let policy = BatteryWarningPolicy::default();

        assert_eq!(
            policy.warning_between(
                reading(BatteryCharge::Unknown),
                reading(BatteryCharge::Coarse(BatteryLevel::Empty)),
            ),
            None
        );
        assert_eq!(
            policy.warning_between(
                reading(BatteryCharge::Coarse(BatteryLevel::Full)),
                reading(BatteryCharge::Coarse(BatteryLevel::Unknown)),
            ),
            None
        );
    }

    #[test]
    fn warns_when_a_coarse_to_precise_transition_crosses_a_threshold() {
        let policy = BatteryWarningPolicy::default();

        let warning = policy.warning_between(
            reading(BatteryCharge::Coarse(BatteryLevel::Full)),
            reading(BatteryCharge::Precise(40)),
        );

        assert_eq!(warning, Some(BatteryWarning::precise(40, low_level())));
    }

    #[test]
    fn does_not_repeat_a_coarse_boundary_after_switching_to_precise() {
        let policy = BatteryWarningPolicy::default();

        let warning = policy.warning_between(
            reading(BatteryCharge::Coarse(BatteryLevel::Medium)),
            reading(BatteryCharge::Precise(69)),
        );

        assert_eq!(warning, None);
    }

    #[test]
    fn warns_when_a_precise_to_coarse_transition_crosses_a_level() {
        let policy = BatteryWarningPolicy::default();

        let warning = policy.warning_between(
            reading(BatteryCharge::Precise(71)),
            reading(BatteryCharge::Coarse(BatteryLevel::Medium)),
        );

        assert_eq!(
            warning,
            Some(BatteryWarning::coarse(BatteryLevel::Medium, medium_level()))
        );
    }

    #[test]
    fn ignores_levels_without_actions() {
        let policy = BatteryWarningPolicy::new(vec![BatteryWarningLevel::new(
            "full",
            Some(100),
            Some(BatteryLevel::Full),
            false,
            false,
            None,
        )]);

        let warning = policy.warning_between(
            reading(BatteryCharge::Precise(100)),
            reading(BatteryCharge::Precise(99)),
        );

        assert_eq!(warning, None);
    }

    #[test]
    fn warns_for_non_notifying_levels_with_audio() {
        let policy = BatteryWarningPolicy::new(vec![BatteryWarningLevel::new(
            "low",
            Some(40),
            Some(BatteryLevel::Low),
            false,
            false,
            Some(AudioClip::file("low.wav")),
        )]);

        let warning = policy.warning_between(
            reading(BatteryCharge::Precise(41)),
            reading(BatteryCharge::Precise(40)),
        );

        assert_eq!(
            warning,
            Some(BatteryWarning::precise(
                40,
                BatteryWarningLevel::new(
                    "low",
                    Some(40),
                    Some(BatteryLevel::Low),
                    false,
                    false,
                    Some(AudioClip::file("low.wav")),
                )
            ))
        );
    }

    fn reading(charge: BatteryCharge) -> BatteryReading {
        BatteryReading::new(BatteryKind::Unknown, charge)
    }

    fn medium_level() -> BatteryWarningLevel {
        BatteryWarningLevel::new(
            "medium",
            Some(70),
            Some(BatteryLevel::Medium),
            true,
            false,
            None,
        )
    }

    fn low_level() -> BatteryWarningLevel {
        BatteryWarningLevel::new("low", Some(40), Some(BatteryLevel::Low), true, false, None)
    }

    fn empty_level() -> BatteryWarningLevel {
        BatteryWarningLevel::new(
            "empty",
            Some(10),
            Some(BatteryLevel::Empty),
            true,
            true,
            None,
        )
    }
}
