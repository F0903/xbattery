#[derive(Clone, Debug)]
pub enum GeneratedSoundSegment {
    Tone {
        frequencies: Vec<f32>,
        duration_seconds: f32,
        volume: f32,
    },
    Silence {
        duration_seconds: f32,
    },
}
