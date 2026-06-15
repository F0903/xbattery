use super::{
    RumbleStep,
    config::{ControllerRumbleConfig, RumbleJolt, RumblePattern},
};

pub(super) fn rumble_sequence(
    pattern: &RumblePattern,
    config: &ControllerRumbleConfig,
) -> Vec<RumbleStep> {
    let mut steps = Vec::new();

    for (index, group) in pattern.groups.iter().enumerate() {
        push_group(&mut steps, group, config);

        if index + 1 < pattern.groups.len() {
            steps.push(RumbleStep::gap(config.group_gap_duration));
        }
    }

    steps
}

fn push_group(steps: &mut Vec<RumbleStep>, group: &[RumbleJolt], config: &ControllerRumbleConfig) {
    for (index, jolt) in group.iter().enumerate() {
        push_gradient_jolt(steps, *jolt);

        if index + 1 < group.len() {
            steps.push(RumbleStep::gap(config.jolt_gap_duration));
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
