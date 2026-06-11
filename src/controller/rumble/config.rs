use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RumbleJolt {
    pub handle_strength_percent: u8,
    pub trigger_strength_percent: u8,
    pub handle_phase_duration: Duration,
    pub trigger_phase_duration: Duration,
}

impl RumbleJolt {
    pub fn new(
        handle_strength_percent: u8,
        trigger_strength_percent: u8,
        handle_phase_duration: Duration,
        trigger_phase_duration: Duration,
    ) -> Self {
        Self {
            handle_strength_percent,
            trigger_strength_percent,
            handle_phase_duration,
            trigger_phase_duration,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RumblePattern {
    pub groups: Vec<Vec<RumbleJolt>>,
}

impl RumblePattern {
    pub fn new(groups: Vec<Vec<RumbleJolt>>) -> Self {
        Self { groups }
    }
}

#[derive(Clone, Debug)]
pub struct RumblePatternSet {
    medium: RumblePattern,
    low: RumblePattern,
    empty: RumblePattern,
}

impl RumblePatternSet {
    pub fn new(medium: RumblePattern, low: RumblePattern, empty: RumblePattern) -> Self {
        Self { medium, low, empty }
    }

    pub fn pattern_for_stage(&self, stage: super::pattern::BatteryWarningStage) -> &RumblePattern {
        match stage {
            super::pattern::BatteryWarningStage::Medium => &self.medium,
            super::pattern::BatteryWarningStage::Low => &self.low,
            super::pattern::BatteryWarningStage::Empty => &self.empty,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ControllerRumbleConfig {
    pub enabled: bool,
    pub jolt_gap_duration: Duration,
    pub group_gap_duration: Duration,
    patterns: RumblePatternSet,
}

impl ControllerRumbleConfig {
    pub fn custom(
        enabled: bool,
        jolt_gap_duration: Duration,
        group_gap_duration: Duration,
        patterns: RumblePatternSet,
    ) -> Self {
        Self {
            enabled,
            jolt_gap_duration,
            group_gap_duration,
            patterns,
        }
    }

    pub fn pattern_for_stage(&self, stage: super::pattern::BatteryWarningStage) -> &RumblePattern {
        self.patterns.pattern_for_stage(stage)
    }
}

impl Default for ControllerRumbleConfig {
    fn default() -> Self {
        let quick = RumbleJolt::new(
            100,
            75,
            Duration::from_millis(35),
            Duration::from_millis(50),
        );
        let strong = RumbleJolt::new(
            100,
            100,
            Duration::from_millis(75),
            Duration::from_millis(100),
        );

        Self::custom(
            false,
            Duration::from_millis(45),
            Duration::from_millis(200),
            RumblePatternSet::new(
                RumblePattern::new(vec![vec![quick, quick]]),
                RumblePattern::new(vec![vec![quick, quick, strong]]),
                RumblePattern::new(vec![vec![quick, quick, strong], vec![quick, quick, strong]]),
            ),
        )
    }
}
