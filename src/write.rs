//! WAV file writing.
use super::math::*;
use super::wave::*;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::Path;

/// Write a 32-bit value to a WAV file.
#[inline]
fn write32<W: Write>(writer: &mut W, x: u32) -> std::io::Result<()> {
    // WAV files are little endian.
    writer.write_all(&[x as u8, (x >> 8) as u8, (x >> 16) as u8, (x >> 24) as u8])?;
    std::io::Result::Ok(())
}

/// Write a 16-bit value to a WAV file.
#[inline]
fn write16<W: Write>(writer: &mut W, x: u16) -> std::io::Result<()> {
    writer.write_all(&[x as u8, (x >> 8) as u8])?;
    std::io::Result::Ok(())
}

// Write WAV header, including the header of the data block.
fn write_wav_header<W: Write>(
    writer: &mut W,
    data_length: usize,
    format: u16,
    channels: usize,
    sample_rate: usize,
) -> std::io::Result<()> {
    writer.write_all(b"RIFF")?;
    write32(writer, data_length as u32 + 36)?;
    writer.write_all(b"WAVE")?;
    writer.write_all(b"fmt ")?;
    // Length of fmt block.
    write32(writer, 16)?;
    // Audio data format 1 = WAVE_FORMAT_PCM, 3 = WAVE_FORMAT_IEEE_FLOAT.
    write16(writer, format)?;
    write16(writer, channels as u16)?;
    write32(writer, sample_rate as u32)?;
    // Data rate in bytes per second.
    let sample_bytes = if format == 1 { 2 } else { 4 };
    write32(writer, (sample_rate * channels) as u32 * sample_bytes)?;
    // Sample frame length in bytes.
    write16(writer, channels as u16 * sample_bytes as u16)?;
    // Bits per sample.
    write16(writer, sample_bytes as u16 * 8)?;
    writer.write_all(b"data")?;
    // Length of data block.
    write32(writer, data_length as u32)?;
    std::io::Result::Ok(())
}

impl Wave {
    /// Write the wave as a 16-bit WAV to a buffer.
    /// Individual samples are clipped to the range -1...1.
    pub fn write_wav16<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        assert!(self.channels() > 0);
        let mut writer = BufWriter::new(writer);
        write_wav_header(
            &mut writer,
            2 * self.channels() * self.length(),
            1,
            self.channels(),
            round(self.sample_rate()) as usize,
        )?;
        for i in 0..self.length() {
            for channel in 0..self.channels() {
                let sample = round(clamp11(self.at(channel, i)) * 32767.49);
                write16(&mut writer, (sample as i16) as u16)?;
            }
        }
        std::io::Result::Ok(())
    }

    /// Write the wave as a 32-bit float WAV to a buffer.
    /// Samples are not clipped to any range but some
    /// applications may expect the range to be -1...1.
    pub fn write_wav32<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        assert!(self.channels() > 0);
        let mut writer = BufWriter::new(writer);
        write_wav_header(
            &mut writer,
            4 * self.channels() * self.length(),
            3,
            self.channels(),
            round(self.sample_rate()) as usize,
        )?;
        for i in 0..self.length() {
            for channel in 0..self.channels() {
                let sample = self.at(channel, i);
                writer.write_all(&sample.to_le_bytes())?;
            }
        }
        std::io::Result::Ok(())
    }

    /// Save the wave as a 16-bit WAV file.
    /// Individual samples are clipped to the range -1...1.
    pub fn save_wav16<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        assert!(self.channels() > 0);
        let mut file = File::create(path.as_ref())?;
        self.write_wav16(&mut file)
    }

    /// Save the wave as a 32-bit float WAV file.
    /// Samples are not clipped to any range but some
    /// applications may expect the range to be -1...1.
    pub fn save_wav32<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        assert!(self.channels() > 0);
        let mut file = File::create(path.as_ref())?;
        self.write_wav32(&mut file)
    }
}
