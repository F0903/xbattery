use std::time::Duration;

use crate::controller::battery::BatteryWarningStage;

use super::RumbleStep;

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

    pub fn pattern_for_stage(&self, stage: BatteryWarningStage) -> &RumblePattern {
        match stage {
            BatteryWarningStage::Medium => &self.medium,
            BatteryWarningStage::Low => &self.low,
            BatteryWarningStage::Empty => &self.empty,
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

    pub fn pattern_for_stage(&self, stage: BatteryWarningStage) -> &RumblePattern {
        self.patterns.pattern_for_stage(stage)
    }

    pub(super) fn steps_for_stage(&self, stage: BatteryWarningStage) -> Vec<RumbleStep> {
        self.steps_for_pattern(self.pattern_for_stage(stage))
    }

    pub(super) fn steps_for_pattern(&self, pattern: &RumblePattern) -> Vec<RumbleStep> {
        let mut steps = Vec::new();

        for (index, group) in pattern.groups.iter().enumerate() {
            self.push_group_steps(&mut steps, group);

            if index + 1 < pattern.groups.len() {
                steps.push(RumbleStep::gap(self.group_gap_duration));
            }
        }

        steps
    }

    fn push_group_steps(&self, steps: &mut Vec<RumbleStep>, group: &[RumbleJolt]) {
        for (index, jolt) in group.iter().enumerate() {
            push_gradient_jolt(steps, *jolt);

            if index + 1 < group.len() {
                steps.push(RumbleStep::gap(self.jolt_gap_duration));
            }
        }
    }
}

fn push_gradient_jolt(steps: &mut Vec<RumbleStep>, jolt: RumbleJolt) {
    let handle = rumble_float(jolt.handle_strength_percent);
    let trigger = rumble_float(jolt.trigger_strength_percent);

    steps.push(RumbleStep::active(
        handle,
        handle,
        0.0,
        0.0,
        jolt.handle_phase_duration,
    ));
    steps.push(RumbleStep::active(
        handle * 0.10,
        handle * 0.05,
        trigger,
        trigger,
        jolt.trigger_phase_duration,
    ));
}

#[cfg(test)]
pub(super) fn motor_speed(percent: u8) -> u16 {
    ((percent.min(100) as u32 * u16::MAX as u32) / 100) as u16
}

fn rumble_float(percent: u8) -> f32 {
    percent.min(100) as f32 / 100.0
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
