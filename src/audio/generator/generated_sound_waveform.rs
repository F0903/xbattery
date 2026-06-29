#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GeneratedSoundWaveform {
    Sine,
    Triangle,
    Square,
    Sawtooth,
}

impl GeneratedSoundWaveform {
    pub(crate) fn sample(self, phase: f32) -> f32 {
        match self {
            Self::Sine => (std::f32::consts::TAU * phase).sin(),
            Self::Triangle => 4.0 * (phase.fract() - 0.5).abs() - 1.0,
            Self::Square => {
                if phase.fract() < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Self::Sawtooth => 2.0 * phase.fract() - 1.0,
        }
    }
}
