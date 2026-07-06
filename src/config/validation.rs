use std::{collections::HashSet, path::Path};

use super::{
    AppConfig,
    battery_config::{
        AudioEffectConfig, AudioEffectKind, AudioLayerConfig, AudioSegmentConfig, AudioSegmentKind,
    },
};
use crate::AppResult;

pub(super) fn validate(config: &AppConfig) -> AppResult<()> {
    if config.monitor.poll_interval_seconds == 0 {
        return Err("monitor.poll_interval_seconds must be greater than zero".into());
    }

    if config.monitor.control_wait_slice_millis == 0 {
        return Err("monitor.control_wait_slice_millis must be greater than zero".into());
    }

    validate_battery_config(config)?;

    if config
        .notifications
        .urgent_precise_threshold_percent
        .is_some_and(|threshold| threshold > 100)
    {
        return Err(
            "notifications.urgent_precise_threshold_percent must be between 0 and 100".into(),
        );
    }

    if config.notifications.app_id.trim().is_empty() {
        return Err("notifications.app_id must not be empty".into());
    }

    if config.updates.repo_owner.trim().is_empty() {
        return Err("updates.repo_owner must not be empty".into());
    }

    if config.updates.repo_name.trim().is_empty() {
        return Err("updates.repo_name must not be empty".into());
    }

    if config.updates.asset_identifier.trim().is_empty() {
        return Err("updates.asset_identifier must not be empty".into());
    }

    if config.updates.bin_path_in_archive.trim().is_empty() {
        return Err("updates.bin_path_in_archive must not be empty".into());
    }

    if config.updates.check_interval_hours == 0 {
        return Err("updates.check_interval_hours must be greater than zero".into());
    }

    Ok(())
}

fn validate_battery_config(config: &AppConfig) -> AppResult<()> {
    if let Some(thresholds) = &config.battery.precise_warning_thresholds {
        if thresholds.is_empty() {
            return Err("battery.precise_warning_thresholds must not be empty".into());
        }

        if thresholds.iter().any(|threshold| *threshold > 100) {
            return Err(
                "battery.precise_warning_thresholds values must be between 0 and 100".into(),
            );
        }
    }

    let Some(levels) = &config.battery.levels else {
        return Ok(());
    };

    if levels.is_empty() {
        return Err("battery.levels must not be empty".into());
    }

    let mut threshold_percents = HashSet::new();
    let mut coarse_levels = HashSet::new();

    for (name, level) in levels {
        let level_path = format!("battery.levels.{name}");

        if name.trim().is_empty() {
            return Err("battery level names must not be empty".into());
        }

        if level.threshold_percent.is_none() && level.coarse_level.is_none() {
            return Err(
                format!("{level_path} must define threshold_percent or coarse_level").into(),
            );
        }

        if level
            .threshold_percent
            .is_some_and(|threshold| threshold > 100)
        {
            return Err(format!("{level_path}.threshold_percent must be between 0 and 100").into());
        }

        if let Some(threshold) = level.threshold_percent
            && !threshold_percents.insert(threshold)
        {
            return Err(format!(
                "battery warning threshold {threshold}% is configured more than once"
            )
            .into());
        }

        if let Some(coarse) = level.coarse_level
            && !coarse_levels.insert(coarse)
        {
            return Err(
                format!("battery coarse level {coarse} is configured more than once").into(),
            );
        }

        if level.sound_file.is_some() && level.generated_sound.is_some() {
            return Err(format!(
                "{level_path} must not define both sound_file and generated_sound"
            )
            .into());
        }

        if let Some(sound_file) = &level.sound_file {
            validate_sound_file(&format!("{level_path}.sound_file"), sound_file)?;
        }

        if let Some(generated_sound) = &level.generated_sound {
            validate_generated_sound(&level_path, name, generated_sound)?;
        }
    }

    Ok(())
}

fn validate_sound_file(field_path: &str, sound_file: &Path) -> AppResult<()> {
    if sound_file.as_os_str().is_empty() {
        return Err(format!("{field_path} must not be empty").into());
    }

    let is_wav = sound_file
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("wav"));

    if !is_wav {
        return Err(format!("{field_path} must point to a .wav file").into());
    }

    Ok(())
}

fn validate_generated_sound(
    level_path: &str,
    name: &str,
    sound: &super::battery_config::AudioRecipeConfig,
) -> AppResult<()> {
    let sound_path = format!("{level_path}.generated_sound");
    let file = sound.file_for_level(name);
    validate_sound_file(&format!("{sound_path}.file"), &file)?;

    if let Some(sample_rate) = sound.sample_rate
        && !(8_000..=192_000).contains(&sample_rate)
    {
        return Err(format!("{sound_path}.sample_rate must be between 8000 and 192000").into());
    }

    if sound.layers.is_empty() && sound.segments.is_empty() {
        return Err(format!("{sound_path} must define layers or segments").into());
    }

    if !sound.layers.is_empty() && !sound.segments.is_empty() {
        return Err(format!("{sound_path} must not define both layers and segments").into());
    }

    for (index, segment) in sound.segments.iter().enumerate() {
        validate_audio_segment(&sound_path, index, segment)?;
    }

    for (index, layer) in sound.layers.iter().enumerate() {
        validate_audio_layer(&sound_path, index, layer)?;
    }

    for (index, effect) in sound.effects.iter().enumerate() {
        validate_audio_effect(&sound_path, index, effect)?;
    }

    Ok(())
}

fn validate_audio_segment(
    sound_path: &str,
    index: usize,
    segment: &AudioSegmentConfig,
) -> AppResult<()> {
    let segment_path = format!("{sound_path}.segments[{index}]");

    validate_duration_seconds(
        &format!("{segment_path}.duration_seconds"),
        segment.duration_seconds,
    )?;

    if let Some(volume) = segment.volume {
        validate_unit_interval(&format!("{segment_path}.volume"), volume, false)?;
    }

    if segment.kind == AudioSegmentKind::Tone {
        if segment.frequencies.is_empty() {
            return Err(format!("{segment_path}.frequencies must not be empty").into());
        }

        validate_frequencies(&format!("{segment_path}.frequencies"), &segment.frequencies)?;
    }

    Ok(())
}

fn validate_audio_layer(sound_path: &str, index: usize, layer: &AudioLayerConfig) -> AppResult<()> {
    let layer_path = format!("{sound_path}.layers[{index}]");

    if layer.frequencies.is_empty() {
        return Err(format!("{layer_path}.frequencies must not be empty").into());
    }

    validate_frequencies(&format!("{layer_path}.frequencies"), &layer.frequencies)?;

    if let Some(start_seconds) = layer.start_seconds
        && (!start_seconds.is_finite() || !(0.0..=30.0).contains(&start_seconds))
    {
        return Err(format!("{layer_path}.start_seconds must be between 0 and 30").into());
    }

    validate_duration_seconds(
        &format!("{layer_path}.duration_seconds"),
        layer.duration_seconds,
    )?;

    if let Some(volume) = layer.volume {
        validate_unit_interval(&format!("{layer_path}.volume"), volume, false)?;
    }

    if let Some(attack_seconds) = layer.attack_seconds {
        validate_envelope_seconds(&format!("{layer_path}.attack_seconds"), attack_seconds)?;
    }

    if let Some(decay_seconds) = layer.decay_seconds {
        validate_envelope_seconds(&format!("{layer_path}.decay_seconds"), decay_seconds)?;
    }

    if let Some(sustain_level) = layer.sustain_level {
        validate_unit_interval(&format!("{layer_path}.sustain_level"), sustain_level, true)?;
    }

    if let Some(release_seconds) = layer.release_seconds {
        validate_envelope_seconds(&format!("{layer_path}.release_seconds"), release_seconds)?;
    }

    Ok(())
}

fn validate_audio_effect(
    sound_path: &str,
    index: usize,
    effect: &AudioEffectConfig,
) -> AppResult<()> {
    let effect_path = format!("{sound_path}.effects[{index}]");

    match effect.kind {
        AudioEffectKind::LowPass => {
            if effect.delay_seconds.is_some()
                || effect.feedback.is_some()
                || effect.room_seconds.is_some()
                || effect.damping.is_some()
                || effect.mix.is_some()
                || effect.drive.is_some()
            {
                return Err(format!(
                    "{effect_path}.delay_seconds, feedback, room_seconds, damping, mix, and drive are not valid for low_pass"
                )
                .into());
            }

            if let Some(cutoff_hz) = effect.cutoff_hz
                && (!cutoff_hz.is_finite() || !(80.0..=20_000.0).contains(&cutoff_hz))
            {
                return Err(format!("{effect_path}.cutoff_hz must be between 80 and 20000").into());
            }
        }
        AudioEffectKind::Delay => {
            if effect.cutoff_hz.is_some() {
                return Err(format!("{effect_path}.cutoff_hz is only valid for low_pass").into());
            }

            if effect.drive.is_some() {
                return Err(format!("{effect_path}.drive is only valid for soft_limiter").into());
            }

            if effect.room_seconds.is_some() || effect.damping.is_some() {
                return Err(format!(
                    "{effect_path}.room_seconds and damping are only valid for reverb"
                )
                .into());
            }

            if let Some(delay_seconds) = effect.delay_seconds
                && (!delay_seconds.is_finite() || delay_seconds <= 0.0 || delay_seconds > 2.0)
            {
                return Err(format!(
                    "{effect_path}.delay_seconds must be greater than 0 and at most 2"
                )
                .into());
            }

            if let Some(feedback) = effect.feedback {
                validate_unit_interval(&format!("{effect_path}.feedback"), feedback, true)?;
            }

            if let Some(mix) = effect.mix {
                validate_unit_interval(&format!("{effect_path}.mix"), mix, true)?;
            }
        }
        AudioEffectKind::Reverb => {
            if effect.cutoff_hz.is_some()
                || effect.delay_seconds.is_some()
                || effect.feedback.is_some()
                || effect.drive.is_some()
            {
                return Err(format!(
                    "{effect_path}.cutoff_hz, delay_seconds, feedback, and drive are not valid for reverb"
                )
                .into());
            }

            if let Some(room_seconds) = effect.room_seconds
                && (!room_seconds.is_finite() || room_seconds <= 0.0 || room_seconds > 3.0)
            {
                return Err(format!(
                    "{effect_path}.room_seconds must be greater than 0 and at most 3"
                )
                .into());
            }

            if let Some(damping) = effect.damping {
                validate_unit_interval(&format!("{effect_path}.damping"), damping, true)?;
            }

            if let Some(mix) = effect.mix {
                validate_unit_interval(&format!("{effect_path}.mix"), mix, true)?;
            }
        }
        AudioEffectKind::SoftLimiter => {
            if effect.cutoff_hz.is_some()
                || effect.delay_seconds.is_some()
                || effect.feedback.is_some()
                || effect.room_seconds.is_some()
                || effect.damping.is_some()
                || effect.mix.is_some()
            {
                return Err(format!(
                    "{effect_path}.cutoff_hz, delay_seconds, feedback, room_seconds, damping, and mix are not valid for soft_limiter"
                )
                .into());
            }

            if let Some(drive) = effect.drive
                && (!drive.is_finite() || drive <= 0.0 || drive > 8.0)
            {
                return Err(
                    format!("{effect_path}.drive must be greater than 0 and at most 8").into(),
                );
            }
        }
    }

    Ok(())
}

fn validate_duration_seconds(field_path: &str, duration_seconds: f32) -> AppResult<()> {
    if !duration_seconds.is_finite() || duration_seconds <= 0.0 || duration_seconds > 5.0 {
        return Err(format!("{field_path} must be greater than 0 and at most 5").into());
    }

    Ok(())
}

fn validate_envelope_seconds(field_path: &str, seconds: f32) -> AppResult<()> {
    if !seconds.is_finite() || !(0.0..=2.0).contains(&seconds) {
        return Err(format!("{field_path} must be between 0 and 2").into());
    }

    Ok(())
}

fn validate_unit_interval(field_path: &str, value: f32, allow_zero: bool) -> AppResult<()> {
    let lower_bound_valid = if allow_zero {
        value >= 0.0
    } else {
        value > 0.0
    };
    if !value.is_finite() || !lower_bound_valid || value > 1.0 {
        let message = if allow_zero {
            format!("{field_path} must be between 0 and 1")
        } else {
            format!("{field_path} must be greater than 0 and at most 1")
        };
        return Err(message.into());
    }

    Ok(())
}

fn validate_frequencies(field_path: &str, frequencies: &[f32]) -> AppResult<()> {
    for frequency in frequencies {
        if !frequency.is_finite() || *frequency <= 0.0 || *frequency > 20_000.0 {
            return Err(
                format!("{field_path} values must be greater than 0 and at most 20000").into(),
            );
        }
    }

    Ok(())
}
