use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::{
    AppResult,
    audio::{
        AudioClip, AudioEffect, AudioEnvelope, AudioLayer, AudioRecipe, AudioSegment,
        DEFAULT_SAMPLE_RATE, Waveform, note_frequency, render_wav_clip,
    },
    controller::battery::{BatteryLevel, BatteryWarningLevel},
};

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BatteryConfig {
    pub levels: Option<BTreeMap<String, BatteryLevelConfig>>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BatteryLevelConfig {
    pub threshold_percent: Option<u8>,
    pub coarse_level: Option<BatteryLevel>,
    pub notify: Option<bool>,
    pub urgent: bool,
    pub sound_file: Option<PathBuf>,
    pub generated_sound: Option<AudioRecipeConfig>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AudioRecipeConfig {
    pub sample_rate: Option<u32>,
    pub segments: Vec<AudioSegmentConfig>,
    pub roll: Vec<AudioRollEventConfig>,
    pub effects: Vec<AudioEffectConfig>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AudioSegmentConfig {
    pub kind: AudioSegmentKind,
    pub notes: String,
    pub duration_seconds: f32,
    pub volume: Option<f32>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AudioRollEventConfig {
    pub notes: String,
    pub at: Option<f32>,
    pub length: f32,
    pub gain: Option<f32>,
    pub wave: WaveformConfig,
    pub adsr: Option<[f32; 4]>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AudioEffectConfig {
    pub kind: AudioEffectKind,
    pub cutoff_hz: Option<f32>,
    pub delay_seconds: Option<f32>,
    pub feedback: Option<f32>,
    pub room_seconds: Option<f32>,
    pub damping: Option<f32>,
    pub mix: Option<f32>,
    pub drive: Option<f32>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AudioSegmentKind {
    #[default]
    Tone,
    Silence,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WaveformConfig {
    #[default]
    Sine,
    Triangle,
    Square,
    Sawtooth,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AudioEffectKind {
    #[default]
    Delay,
    LowPass,
    Reverb,
    SoftLimiter,
}

impl BatteryConfig {
    pub fn warning_levels(&self) -> AppResult<Vec<BatteryWarningLevel>> {
        match &self.levels {
            Some(levels) => levels
                .iter()
                .map(|(name, config)| config.warning_level(name))
                .collect(),
            None => Ok(BatteryWarningLevel::default_levels()),
        }
    }

    pub(super) fn resolve_relative_paths(&mut self, base_dir: Option<&Path>) {
        let Some(base_dir) = base_dir else {
            return;
        };

        let Some(levels) = &mut self.levels else {
            return;
        };

        for level in levels.values_mut() {
            if let Some(sound_file) = &mut level.sound_file
                && !sound_file.as_os_str().is_empty()
                && sound_file.is_relative()
            {
                *sound_file = base_dir.join(&sound_file);
            }
        }
    }
}

impl BatteryLevelConfig {
    fn warning_level(&self, name: &str) -> AppResult<BatteryWarningLevel> {
        Ok(BatteryWarningLevel::with_notify_and_audio(
            name,
            self.threshold_percent,
            self.coarse_level,
            self.notify.unwrap_or(true),
            self.urgent,
            self.audio_clip(name)?,
        ))
    }

    fn audio_clip(&self, name: &str) -> AppResult<Option<AudioClip>> {
        if let Some(sound_file) = &self.sound_file {
            return Ok(Some(AudioClip::file(sound_file.clone())));
        }

        let Some(generated_sound) = &self.generated_sound else {
            return Ok(None);
        };

        let recipe = generated_sound.recipe(name)?;
        render_wav_clip(&recipe).map(Some)
    }
}

impl AudioRecipeConfig {
    fn recipe(&self, level_name: &str) -> AppResult<AudioRecipe> {
        let sample_rate = self.sample_rate.unwrap_or(DEFAULT_SAMPLE_RATE);
        let effects = self
            .effects
            .iter()
            .map(AudioEffectConfig::effect)
            .collect::<Vec<_>>();

        if !self.roll.is_empty() {
            let layers = self
                .roll
                .iter()
                .enumerate()
                .map(|(index, event)| {
                    event.layer(
                        &format!("battery.levels.{level_name}.generated_sound.roll[{index}]"),
                        sample_rate,
                    )
                })
                .collect::<AppResult<Vec<_>>>()?;
            return Ok(AudioRecipe::with_layers(sample_rate, layers, effects));
        }

        if !self.segments.is_empty() {
            let segments = self
                .segments
                .iter()
                .enumerate()
                .map(|(index, segment)| {
                    segment.segment(
                        &format!("battery.levels.{level_name}.generated_sound.segments[{index}]"),
                        sample_rate,
                    )
                })
                .collect::<AppResult<Vec<_>>>()?;
            return Ok(AudioRecipe::with_segments_and_effects(
                sample_rate,
                segments,
                effects,
            ));
        }

        Err(
            format!("battery.levels.{level_name}.generated_sound must define roll or segments")
                .into(),
        )
    }
}

impl AudioSegmentConfig {
    fn segment(&self, field_path: &str, sample_rate: u32) -> AppResult<AudioSegment> {
        match self.kind {
            AudioSegmentKind::Tone => Ok(AudioSegment::Tone {
                frequencies: resolve_notes(
                    &self.notes,
                    &format!("{field_path}.notes"),
                    sample_rate,
                )?,
                duration_seconds: self.duration_seconds,
                volume: self.volume.unwrap_or(0.25),
            }),
            AudioSegmentKind::Silence if self.notes.trim().is_empty() => {
                Ok(AudioSegment::Silence {
                    duration_seconds: self.duration_seconds,
                })
            }
            AudioSegmentKind::Silence => {
                Err(format!("{field_path}.notes are not valid for silence").into())
            }
        }
    }
}

impl AudioRollEventConfig {
    fn layer(&self, field_path: &str, sample_rate: u32) -> AppResult<AudioLayer> {
        let envelope = self
            .adsr
            .map(|[attack, decay, sustain, release]| {
                AudioEnvelope::new(attack, decay, sustain, release)
            })
            .unwrap_or_default();

        Ok(AudioLayer::with_audio_envelope(
            self.wave.waveform(),
            resolve_notes(&self.notes, &format!("{field_path}.notes"), sample_rate)?,
            self.at.unwrap_or(0.0),
            self.length,
            self.gain.unwrap_or(0.2),
            envelope,
        ))
    }
}

impl WaveformConfig {
    fn waveform(self) -> Waveform {
        match self {
            Self::Sine => Waveform::Sine,
            Self::Triangle => Waveform::Triangle,
            Self::Square => Waveform::Square,
            Self::Sawtooth => Waveform::Sawtooth,
        }
    }
}

impl AudioEffectConfig {
    fn effect(&self) -> AudioEffect {
        match self.kind {
            AudioEffectKind::LowPass => AudioEffect::LowPass {
                cutoff_hz: self.cutoff_hz.unwrap_or(1_600.0),
            },
            AudioEffectKind::Delay => AudioEffect::Delay {
                delay_seconds: self.delay_seconds.unwrap_or(0.06),
                feedback: self.feedback.unwrap_or(0.18),
                mix: self.mix.unwrap_or(0.14),
            },
            AudioEffectKind::Reverb => AudioEffect::Reverb {
                room_seconds: self.room_seconds.unwrap_or(0.22),
                damping: self.damping.unwrap_or(0.42),
                mix: self.mix.unwrap_or(0.10),
            },
            AudioEffectKind::SoftLimiter => AudioEffect::SoftLimiter {
                drive: self.drive.unwrap_or(1.2),
            },
        }
    }
}

pub(super) fn resolve_notes(
    notes: &str,
    field_path: &str,
    sample_rate: u32,
) -> AppResult<Vec<f32>> {
    let notes = notes.split_whitespace().collect::<Vec<_>>();
    if notes.is_empty() {
        return Err(format!("{field_path} must not be empty").into());
    }

    let nyquist_hz = sample_rate as f32 / 2.0;
    notes
        .iter()
        .enumerate()
        .map(|(index, note)| {
            let frequency = note_frequency(note)
                .map_err(|reason| format!("{field_path}[{index}] {note:?} is invalid: {reason}"))?;
            if frequency >= nyquist_hz {
                return Err(format!(
                    "{field_path}[{index}] {note:?} must be below the {nyquist_hz} Hz Nyquist limit"
                )
                .into());
            }

            Ok(frequency)
        })
        .collect()
}
