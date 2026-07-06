use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};

use super::audio_exporter::AudioExporter;
use crate::AppResult;

const PCM_FORMAT: u16 = 1;
const CHANNELS: u16 = 1;
const BITS_PER_SAMPLE: u16 = 16;
const FORMAT_CHUNK_SIZE: u32 = 16;
const DATA_HEADER_BYTES: usize = 36;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct WavExporter;

impl AudioExporter for WavExporter {
    fn export(&self, path: &Path, sample_rate: u32, samples: &[i16]) -> AppResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let data_size = samples
            .len()
            .checked_mul(size_of::<i16>())
            .ok_or("generated sound is too large")?;
        let riff_size = u32::try_from(
            DATA_HEADER_BYTES
                .checked_add(data_size)
                .ok_or("WAV is too large")?,
        )
        .map_err(|_| "WAV is too large")?;
        let data_size = u32::try_from(data_size).map_err(|_| "WAV is too large")?;

        let mut writer = BufWriter::new(File::create(path)?);
        write_header(&mut writer, sample_rate, riff_size, data_size)?;

        for sample in samples {
            writer.write_all(&sample.to_le_bytes())?;
        }

        Ok(())
    }
}

fn write_header(
    writer: &mut impl Write,
    sample_rate: u32,
    riff_size: u32,
    data_size: u32,
) -> AppResult<()> {
    writer.write_all(b"RIFF")?;
    writer.write_all(&riff_size.to_le_bytes())?;
    writer.write_all(b"WAVE")?;
    writer.write_all(b"fmt ")?;
    writer.write_all(&FORMAT_CHUNK_SIZE.to_le_bytes())?;
    writer.write_all(&PCM_FORMAT.to_le_bytes())?;
    writer.write_all(&CHANNELS.to_le_bytes())?;
    writer.write_all(&sample_rate.to_le_bytes())?;
    writer.write_all(&(sample_rate * bytes_per_frame() as u32).to_le_bytes())?;
    writer.write_all(&bytes_per_frame().to_le_bytes())?;
    writer.write_all(&BITS_PER_SAMPLE.to_le_bytes())?;
    writer.write_all(b"data")?;
    writer.write_all(&data_size.to_le_bytes())?;

    Ok(())
}

fn bytes_per_frame() -> u16 {
    CHANNELS * (BITS_PER_SAMPLE / 8)
}
