//! Multichannel wave abstraction.

use crate::audionode::*;
use crate::combinator::*;
use crate::math::*;
use crate::*;
use rsor::Slice;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

/// Write a 32-bit value to a WAV file.
fn write32(file: &mut File, x: u32) -> std::io::Result<()> {
    // WAV files are little endian.
    file.write_all(&[x as u8])?;
    file.write_all(&[(x >> 8) as u8])?;
    file.write_all(&[(x >> 16) as u8])?;
    file.write_all(&[(x >> 24) as u8])?;
    std::io::Result::Ok(())
}

/// Write a 16-bit value to a WAV file.
fn write16(file: &mut File, x: u16) -> std::io::Result<()> {
    file.write_all(&[x as u8])?;
    file.write_all(&[(x >> 8) as u8])?;
    std::io::Result::Ok(())
}

/// Multichannel wave.
pub struct Wave<T: Float> {
    /// Vector of channels. Each channel is stored in its own vector.
    vec: Vec<Vec<T>>,
    /// Sample rate of the wave.
    sr: f64,
}

impl<T: Float> Wave<T> {
    /// Creates an empty wave with the specified number of channels (`channels` > 0).
    pub fn new(channels: usize, sample_rate: f64) -> Self {
        assert!(channels > 0);
        let mut vec = Vec::with_capacity(channels);
        for _i in 0..channels {
            vec.push(Vec::new());
        }
        Wave {
            vec,
            sr: sample_rate,
        }
    }

    /// Creates an empty wave with the given `capacity` in samples
    /// and number of channels (`channels` > 0).
    pub fn with_capacity(channels: usize, sample_rate: f64, capacity: usize) -> Self {
        assert!(channels > 0);
        let mut vec = Vec::with_capacity(channels);
        for _i in 0..channels {
            vec.push(Vec::with_capacity(capacity));
        }
        Wave {
            vec,
            sr: sample_rate,
        }
    }

    /// Sample rate of this wave.
    pub fn sample_rate(&self) -> f64 {
        self.sr
    }

    /// Set the sample rate.
    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sr = sample_rate;
    }

    /// Number of channels in this wave.
    pub fn channels(&self) -> usize {
        self.vec.len()
    }

    /// Returns a reference to the requested channel.
    pub fn channel(&self, channel: usize) -> &Vec<T> {
        &self.vec[channel]
    }

    /// Returns a mutable reference to the requested channel.
    pub fn channel_mut(&mut self, channel: usize) -> &mut Vec<T> {
        &mut self.vec[channel]
    }

    /// Sample accessor.
    pub fn at(&self, channel: usize, index: usize) -> T {
        self.vec[channel][index]
    }

    /// Set sample to value.
    pub fn set(&mut self, channel: usize, index: usize, value: T) {
        self.vec[channel][index] = value;
    }

    /// Length of the wave in samples.
    pub fn length(&self) -> usize {
        self.vec[0].len()
    }

    /// Length of the wave in samples.
    pub fn len(&self) -> usize {
        self.vec[0].len()
    }

    /// Returns whether this wave contains no samples.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Duration of the wave in seconds.
    pub fn duration(&self) -> f64 {
        self.length() as f64 / self.sample_rate()
    }

    /// Resizes the wave in-place. Any new samples are set to zero.
    pub fn resize(&mut self, length: usize) {
        if length != self.length() {
            for channel in 0..self.channels() {
                self.vec[channel].resize(length, T::zero());
            }
        }
    }

    /// Peak amplitude of the wave.
    pub fn amplitude(&self) -> T {
        let mut peak = T::zero();
        for channel in 0..self.channels() {
            for i in 0..self.len() {
                peak = max(peak, abs(self.at(channel, i)));
            }
        }
        peak
    }

    /// Scales the wave to the range -1...1.
    pub fn normalize(&mut self) {
        let a = self.amplitude();
        if a == T::zero() || a == T::one() {
            return;
        }
        let z = T::one() / a;
        for channel in 0..self.channels() {
            for i in 0..self.len() {
                self.set(channel, i, self.at(channel, i) * z);
            }
        }
    }

    /// Render wave from a generator `node`.
    /// Resets `node` and sets its sample rate.
    /// Does not discard pre-delay.
    pub fn render<X>(sample_rate: f64, duration: f64, node: &mut An<X>) -> Self
    where
        X: AudioNode<Sample = T>,
    {
        assert!(node.inputs() == 0);
        assert!(node.outputs() > 0);
        node.reset(Some(sample_rate));
        let length = (duration * sample_rate).round() as usize;
        let mut wave = Wave::<T>::with_capacity(node.outputs(), sample_rate, length);
        let mut i = 0;
        let mut buffer = Wave::<T>::new(node.outputs(), sample_rate);
        let mut reusable_slice = Slice::<[T]>::with_capacity(node.outputs());
        while i < length {
            let n = min(length - i, MAX_BUFFER_SIZE);
            buffer.resize(n);
            node.process(n, &[], reusable_slice.from_muts(&mut buffer.vec));
            for channel in 0..node.outputs() {
                wave.vec[channel].extend_from_slice(&buffer.vec[channel][..]);
            }
            i += n;
        }
        wave
    }

    /// Render wave from a generator `node`.
    /// Any pre-delay, as measured by signal latency, is discarded.
    /// Resets `node` and sets its sample rate.
    pub fn render_latency<X>(sample_rate: f64, duration: f64, node: &mut An<X>) -> Self
    where
        X: AudioNode<Sample = T>,
    {
        assert!(node.inputs() == 0);
        assert!(node.outputs() > 0);
        let latency = node.latency().unwrap_or_default();
        // Round latency down to nearest sample.
        let latency_samples = floor(latency) as usize;
        let latency_duration = latency_samples as f64 / sample_rate;
        // Round duration to nearest sample.
        let duration_samples = round(duration * sample_rate) as usize;
        let duration = duration_samples as f64 / sample_rate;
        if latency_samples > 0 {
            let latency_wave = Wave::render(sample_rate, duration + latency_duration, node);
            let mut wave = Wave::with_capacity(node.outputs(), sample_rate, duration_samples);
            wave.resize(duration_samples);
            for channel in 0..wave.channels() {
                for i in 0..duration_samples {
                    wave.set(channel, i, latency_wave.at(channel, i + latency_samples));
                }
            }
            wave
        } else {
            Wave::render(sample_rate, duration, node)
        }
    }

    /// Filter this wave with `node` and return the resulting wave.
    /// Resets `node` and sets its sample rate. Does not discard pre-delay.
    /// The `node` must have as many inputs as there are channels in this wave.
    /// All zeros input is used for the rest of the wave if
    /// the duration is greater than the duration of this wave.
    pub fn filter<X>(&self, duration: f64, node: &mut An<X>) -> Self
    where
        X: AudioNode<Sample = T>,
    {
        assert!(node.inputs() == self.channels());
        assert!(node.outputs() > 0);
        node.reset(Some(self.sample_rate()));
        let total_length = round(duration * self.sample_rate()) as usize;
        let input_length = min(total_length, self.length());
        let mut wave = Wave::<T>::with_capacity(node.outputs(), self.sample_rate(), total_length);
        let mut i = 0;
        let mut input_buffer = Wave::<T>::new(self.channels(), self.sample_rate());
        let mut reusable_input_slice = Slice::<[T]>::with_capacity(self.channels());
        let mut output_buffer = Wave::<T>::new(node.outputs(), self.sample_rate());
        let mut reusable_output_slice = Slice::<[T]>::with_capacity(node.outputs());
        // Filter from this wave.
        while i < input_length {
            let n = min(input_length - i, MAX_BUFFER_SIZE);
            input_buffer.resize(n);
            output_buffer.resize(n);
            for channel in 0..self.channels() {
                for j in 0..n {
                    input_buffer.set(channel, j, self.at(channel, i + j));
                }
            }
            node.process(
                n,
                reusable_input_slice.from_refs(&input_buffer.vec),
                reusable_output_slice.from_muts(&mut output_buffer.vec),
            );
            for channel in 0..node.outputs() {
                wave.vec[channel].extend_from_slice(&output_buffer.vec[channel][..]);
            }
            i += n;
        }
        // Filter the rest from a zero input.
        if i < total_length {
            input_buffer.resize(MAX_BUFFER_SIZE);
            for channel in 0..self.channels() {
                for j in 0..MAX_BUFFER_SIZE {
                    input_buffer.set(channel, j, T::zero());
                }
            }
            while i < total_length {
                let n = min(total_length - i, MAX_BUFFER_SIZE);
                input_buffer.resize(n);
                output_buffer.resize(n);
                node.process(
                    n,
                    reusable_input_slice.from_refs(&input_buffer.vec),
                    reusable_output_slice.from_muts(&mut output_buffer.vec),
                );
                for channel in 0..node.outputs() {
                    wave.vec[channel].extend_from_slice(&output_buffer.vec[channel][..]);
                }
                i += n;
            }
        }
        wave
    }

    /// Filter this wave with `node` and return the resulting wave.
    /// Any pre-delay, as measured by signal latency, is discarded.
    /// Resets `node` and sets its sample rate.
    /// The `node` must have as many inputs as there are channels in this wave.
    /// All zeros input is used for the rest of the wave if
    /// the duration is greater than the duration of this wave.
    pub fn filter_latency<X>(&self, duration: f64, node: &mut An<X>) -> Self
    where
        X: AudioNode<Sample = T>,
    {
        assert!(node.inputs() == self.channels());
        assert!(node.outputs() > 0);
        let latency = node.latency().unwrap_or_default();
        // Round latency down to nearest sample.
        let latency_samples = floor(latency) as usize;
        let latency_duration = latency_samples as f64 / self.sample_rate();
        // Round duration to nearest sample.
        let duration_samples = round(duration * self.sample_rate()) as usize;
        let duration = duration_samples as f64 / self.sample_rate();
        if latency_samples > 0 {
            let latency_wave = self.filter(duration + latency_duration, node);
            let mut wave =
                Wave::with_capacity(node.outputs(), self.sample_rate(), duration_samples);
            wave.resize(duration_samples);
            for channel in 0..wave.channels() {
                for i in 0..duration_samples {
                    wave.set(channel, i, latency_wave.at(channel, i + latency_samples));
                }
            }
            wave
        } else {
            self.filter(duration, node)
        }
    }

    /// Saves the wave as a 16-bit WAV file.
    /// Individual samples are clipped to the range -1...1.
    pub fn save_wav(&self, path: &Path) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(b"RIFF")?;
        let data_length = 2 * self.channels() * self.length();
        write32(&mut file, data_length as u32 + 36)?;
        file.write_all(b"WAVE")?;
        file.write_all(b"fmt ")?;
        // Length of fmt block.
        write32(&mut file, 16)?;
        // Audio data format 1 = WAVE_FORMAT_PCM.
        write16(&mut file, 1)?;
        write16(&mut file, self.channels() as u16)?;
        let sample_rate = round(self.sample_rate()) as u32;
        write32(&mut file, sample_rate)?;
        // Data rate in bytes per second.
        write32(&mut file, sample_rate * self.channels() as u32 * 2)?;
        // Sample frame length in bytes.
        write16(&mut file, self.channels() as u16 * 2)?;
        // Bits per sample.
        write16(&mut file, 16)?;
        file.write_all(b"data")?;
        // Length of data block.
        write32(&mut file, data_length as u32)?;
        for i in 0..self.length() {
            for channel in 0..self.channels() {
                let sample = round(clamp11(self.at(channel, i)) * T::from_f64(32767.49));
                write16(&mut file, sample.to_i64() as u16)?;
            }
        }
        std::io::Result::Ok(())
    }

    /// Saves the wave as a 32-bit float WAV file.
    /// Samples are not clipped to any range but some
    /// applications may expect the range to be -1...1.
    pub fn save_wav_float(&self, path: &Path) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(b"RIFF")?;
        let data_length = 4 * self.channels() * self.length();
        write32(&mut file, data_length as u32 + 36)?;
        file.write_all(b"WAVE")?;
        file.write_all(b"fmt ")?;
        // Length of fmt block.
        write32(&mut file, 16)?;
        // Audio data format 3 = WAVE_FORMAT_IEEE_FLOAT.
        write16(&mut file, 3)?;
        write16(&mut file, self.channels() as u16)?;
        let sample_rate = round(self.sample_rate()) as u32;
        write32(&mut file, sample_rate)?;
        // Data rate in bytes per second.
        write32(&mut file, sample_rate * self.channels() as u32 * 4)?;
        // Sample frame length in bytes.
        write16(&mut file, self.channels() as u16 * 4)?;
        // Bits per sample.
        write16(&mut file, 32)?;
        file.write_all(b"data")?;
        // Length of data block.
        write32(&mut file, data_length as u32)?;
        for i in 0..self.length() {
            for channel in 0..self.channels() {
                let sample = self.at(channel, i);
                file.write_all(&sample.to_f32().to_le_bytes())?;
            }
        }
        std::io::Result::Ok(())
    }
}
