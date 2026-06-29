use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::{
    AppResult,
    audio::{
        AudioGenerator, DEFAULT_SAMPLE_RATE, GeneratedSound, GeneratedSoundEffect,
        GeneratedSoundLayer, GeneratedSoundSegment, GeneratedSoundWaveform,
    },
    controller::battery::{BatteryLevel, BatteryWarningLevel},
};

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BatteryConfig {
    pub levels: Option<BTreeMap<String, BatteryLevelConfig>>,
    pub precise_warning_thresholds: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BatteryLevelConfig {
    pub threshold_percent: Option<u8>,
    pub coarse_level: Option<BatteryLevel>,
    pub notify: Option<bool>,
    pub urgent: bool,
    pub sound_file: Option<PathBuf>,
    pub generated_sound: Option<GeneratedSoundConfig>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GeneratedSoundConfig {
    pub file: Option<PathBuf>,
    pub sample_rate: Option<u32>,
    pub segments: Vec<GeneratedSoundSegmentConfig>,
    pub layers: Vec<GeneratedSoundLayerConfig>,
    pub effects: Vec<GeneratedSoundEffectConfig>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GeneratedSoundSegmentConfig {
    pub kind: GeneratedSoundSegmentKind,
    pub frequencies: Vec<f32>,
    pub duration_seconds: f32,
    pub volume: Option<f32>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GeneratedSoundLayerConfig {
    pub waveform: GeneratedSoundWaveformConfig,
    pub frequencies: Vec<f32>,
    pub start_seconds: Option<f32>,
    pub duration_seconds: f32,
    pub volume: Option<f32>,
    pub attack_seconds: Option<f32>,
    pub decay_seconds: Option<f32>,
    pub sustain_level: Option<f32>,
    pub release_seconds: Option<f32>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GeneratedSoundEffectConfig {
    pub kind: GeneratedSoundEffectKind,
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
pub enum GeneratedSoundSegmentKind {
    #[default]
    Tone,
    Silence,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GeneratedSoundWaveformConfig {
    #[default]
    Sine,
    Triangle,
    Square,
    Sawtooth,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedSoundEffectKind {
    #[default]
    Delay,
    LowPass,
    Reverb,
    SoftLimiter,
}

impl BatteryConfig {
    pub fn warning_levels(
        &self,
        legacy_urgent_threshold_percent: Option<u8>,
    ) -> Vec<BatteryWarningLevel> {
        if let Some(thresholds) = &self.precise_warning_thresholds {
            return legacy_warning_levels(
                thresholds,
                legacy_urgent_threshold_percent.unwrap_or(10),
            );
        }

        match &self.levels {
            Some(levels) => levels
                .iter()
                .map(|(name, config)| config.warning_level(name))
                .collect(),
            None => BatteryWarningLevel::default_levels_with_urgent_threshold(
                legacy_urgent_threshold_percent.unwrap_or(10),
            ),
        }
    }

    pub(super) fn resolve_relative_paths(&mut self, base_dir: Option<&Path>) {
        let Some(base_dir) = base_dir else {
            return;
        };

        let Some(levels) = &mut self.levels else {
            return;
        };

        for (name, level) in levels {
            level.resolve_generated_sound_file(name);

            if let Some(sound_file) = &mut level.sound_file
                && !sound_file.as_os_str().is_empty()
                && sound_file.is_relative()
            {
                *sound_file = base_dir.join(&sound_file);
            }

            if let Some(generated_sound) = &mut level.generated_sound
                && let Some(file) = &mut generated_sound.file
                && !file.as_os_str().is_empty()
                && file.is_relative()
            {
                *file = base_dir.join(&file);
            }
        }
    }

    pub(super) fn generate_sounds(&self) -> AppResult<Vec<PathBuf>> {
        let mut generated = Vec::new();

        let Some(levels) = &self.levels else {
            return Ok(generated);
        };

        let generator = AudioGenerator::new();

        for (name, level) in levels {
            let Some(sound) = &level.generated_sound else {
                continue;
            };

            let path = sound.file_for_level(name);
            let generated_sound = sound.generated_sound(name)?;
            generator.write_wav(&path, &generated_sound)?;
            generated.push(path);
        }

        Ok(generated)
    }

    pub fn generated_sound_files(&self) -> Vec<PathBuf> {
        self.levels
            .iter()
            .flat_map(|levels| levels.iter())
            .filter_map(|(name, level)| {
                level
                    .generated_sound
                    .as_ref()
                    .map(|sound| sound.file_for_level(name))
            })
            .collect()
    }
}

impl BatteryLevelConfig {
    pub fn warning_level(&self, name: &str) -> BatteryWarningLevel {
        BatteryWarningLevel::with_notify_and_file(
            name,
            self.threshold_percent,
            self.coarse_level,
            self.notify.unwrap_or(true),
            self.urgent,
            self.effective_sound_file(name),
        )
    }

    pub fn effective_sound_file(&self, name: &str) -> Option<PathBuf> {
        self.sound_file.clone().or_else(|| {
            self.generated_sound
                .as_ref()
                .map(|sound| sound.file_for_level(name))
        })
    }

    fn resolve_generated_sound_file(&mut self, name: &str) {
        let Some(sound) = &mut self.generated_sound else {
            return;
        };

        if sound.file.is_none() {
            sound.file = Some(PathBuf::from("sounds").join(format!("{name}.wav")));
        }
    }
}

impl GeneratedSoundConfig {
    pub fn file_for_level(&self, name: &str) -> PathBuf {
        self.file
            .clone()
            .unwrap_or_else(|| PathBuf::from("sounds").join(format!("{name}.wav")))
    }

    pub fn generated_sound(&self, level_name: &str) -> AppResult<GeneratedSound> {
        let sample_rate = self.sample_rate.unwrap_or(DEFAULT_SAMPLE_RATE);
        let effects = self
            .effects
            .iter()
            .map(GeneratedSoundEffectConfig::effect)
            .collect::<Vec<_>>();

        if !self.layers.is_empty() {
            let layers = self
                .layers
                .iter()
                .map(GeneratedSoundLayerConfig::layer)
                .collect::<Vec<_>>();
            return Ok(GeneratedSound::with_layers(sample_rate, layers, effects));
        }

        if !self.segments.is_empty() {
            let segments = self
                .segments
                .iter()
                .map(GeneratedSoundSegmentConfig::segment)
                .collect::<Vec<_>>();
            return Ok(GeneratedSound::with_segments_and_effects(
                sample_rate,
                segments,
                effects,
            ));
        }

        Err(
            format!("battery.levels.{level_name}.generated_sound must define layers or segments")
                .into(),
        )
    }
}

impl GeneratedSoundSegmentConfig {
    pub fn segment(&self) -> GeneratedSoundSegment {
        match self.kind {
            GeneratedSoundSegmentKind::Tone => GeneratedSoundSegment::Tone {
                frequencies: self.frequencies.clone(),
                duration_seconds: self.duration_seconds,
                volume: self.volume.unwrap_or(0.25),
            },
            GeneratedSoundSegmentKind::Silence => GeneratedSoundSegment::Silence {
                duration_seconds: self.duration_seconds,
            },
        }
    }
}

impl GeneratedSoundLayerConfig {
    pub fn layer(&self) -> GeneratedSoundLayer {
        GeneratedSoundLayer::with_decay_envelope(
            self.waveform.waveform(),
            self.frequencies.clone(),
            self.start_seconds.unwrap_or(0.0),
            self.duration_seconds,
            self.volume.unwrap_or(0.2),
            self.attack_seconds.unwrap_or(0.008),
            self.decay_seconds.unwrap_or(0.0),
            self.sustain_level.unwrap_or(1.0),
            self.release_seconds.unwrap_or(0.028),
        )
    }
}

impl GeneratedSoundWaveformConfig {
    pub fn waveform(self) -> GeneratedSoundWaveform {
        match self {
            Self::Sine => GeneratedSoundWaveform::Sine,
            Self::Triangle => GeneratedSoundWaveform::Triangle,
            Self::Square => GeneratedSoundWaveform::Square,
            Self::Sawtooth => GeneratedSoundWaveform::Sawtooth,
        }
    }
}

impl GeneratedSoundEffectConfig {
    pub fn effect(&self) -> GeneratedSoundEffect {
        match self.kind {
            GeneratedSoundEffectKind::LowPass => GeneratedSoundEffect::LowPass {
                cutoff_hz: self.cutoff_hz.unwrap_or(1_600.0),
            },
            GeneratedSoundEffectKind::Delay => GeneratedSoundEffect::Delay {
                delay_seconds: self.delay_seconds.unwrap_or(0.06),
                feedback: self.feedback.unwrap_or(0.18),
                mix: self.mix.unwrap_or(0.14),
            },
            GeneratedSoundEffectKind::Reverb => GeneratedSoundEffect::Reverb {
                room_seconds: self.room_seconds.unwrap_or(0.22),
                damping: self.damping.unwrap_or(0.42),
                mix: self.mix.unwrap_or(0.10),
            },
            GeneratedSoundEffectKind::SoftLimiter => GeneratedSoundEffect::SoftLimiter {
                drive: self.drive.unwrap_or(1.2),
            },
        }
    }
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self {
            levels: None,
            precise_warning_thresholds: None,
        }
    }
}

fn legacy_warning_levels(
    thresholds: &[u8],
    urgent_threshold_percent: u8,
) -> Vec<BatteryWarningLevel> {
    let mut thresholds = thresholds.to_vec();
    thresholds.sort_unstable_by(|left, right| right.cmp(left));
    thresholds.dedup();

    let mut levels = thresholds
        .into_iter()
        .map(|threshold| {
            BatteryWarningLevel::with_notify(
                format!("{threshold}%"),
                Some(threshold),
                None,
                true,
                threshold <= urgent_threshold_percent,
            )
        })
        .collect::<Vec<_>>();

    levels.extend(
        BatteryWarningLevel::default_levels_with_urgent_threshold(urgent_threshold_percent)
            .into_iter()
            .map(|level| {
                BatteryWarningLevel::with_notify_and_file(
                    level.name(),
                    None,
                    level.coarse_level(),
                    level.notify(),
                    level.urgent(),
                    level.sound_file().map(PathBuf::from),
                )
            }),
    );

    levels
}
