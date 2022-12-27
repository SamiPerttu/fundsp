//! Multichannel wave abstraction.

use super::audionode::*;
use super::audiounit::*;
use super::combinator::*;
use super::math::*;
use super::*;
use duplicate::duplicate_item;
use numeric_array::typenum::Unsigned;
use numeric_array::*;
use rsor::Slice;
use std::fs::File;
use std::io::prelude::*;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;

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

/// Multichannel wave.
#[duplicate_item(
    f48       Wave48       AudioUnit48;
    [ f64 ]   [ Wave64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Wave32 ]   [ AudioUnit32 ];
)]
pub struct Wave48 {
    /// Vector of channels. Each channel is stored in its own vector.
    vec: Vec<Vec<f48>>,
    /// Sample rate of the wave.
    sr: f64,
    /// Length of the wave in samples. This is 0 if there are no channels.
    len: usize,
    /// Slice of references. This is only allocated if it is used.
    slice: Slice<[f48]>,
}

#[duplicate_item(
    f48       Wave48       AudioUnit48;
    [ f64 ]   [ Wave64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Wave32 ]   [ AudioUnit32 ];
)]
impl Clone for Wave48 {
    fn clone(&self) -> Self {
        Self {
            vec: self.vec.clone(),
            sr: self.sr,
            len: self.len,
            slice: Slice::new(),
        }
    }
}

#[duplicate_item(
    f48       Wave48       AudioUnit48;
    [ f64 ]   [ Wave64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Wave32 ]   [ AudioUnit32 ];
)]
impl Wave48 {
    /// Create an empty wave with the specified number of `channels`.
    ///
    /// ### Example: Create Stereo Wave
    /// ```
    /// use fundsp::hacker::*;
    /// let wave = Wave64::new(2, 44100.0);
    /// ```
    pub fn new(channels: usize, sample_rate: f64) -> Self {
        let mut vec = Vec::with_capacity(channels);
        for _i in 0..channels {
            vec.push(Vec::new());
        }
        Self {
            vec,
            sr: sample_rate,
            len: 0,
            slice: Slice::new(),
        }
    }

    /// Create an empty wave with the given `capacity` in samples
    /// and number of `channels`.
    ///
    /// ### Example: Create Stereo Wave
    /// ```
    /// use fundsp::hacker::*;
    /// let wave = Wave64::with_capacity(2, 44100.0, 44100);
    /// assert!(wave.channels() == 2 && wave.length() == 0);
    /// ```
    pub fn with_capacity(channels: usize, sample_rate: f64, capacity: usize) -> Self {
        let mut vec = Vec::with_capacity(channels);
        for _i in 0..channels {
            vec.push(Vec::with_capacity(capacity));
        }
        Self {
            vec,
            sr: sample_rate,
            len: 0,
            slice: Slice::new(),
        }
    }

    /// Create an all-zeros wave with the given `duration` in seconds and number of `channels`.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker32::*;
    /// let wave = Wave32::silence(1, 44100.0, 1.0);
    /// assert!(wave.duration() == 1.0 && wave.amplitude() == 0.0);
    /// ```
    pub fn silence(channels: usize, sample_rate: f64, duration: f64) -> Self {
        let length = round(duration * sample_rate) as usize;
        assert!(channels > 0 || length == 0);
        let mut vec = Vec::with_capacity(channels);
        for _i in 0..channels {
            vec.push(vec![0.0; length]);
        }
        Self {
            vec,
            sr: sample_rate,
            len: length,
            slice: Slice::new(),
        }
    }

    /// Create a mono wave from a slice of samples.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker32::*;
    /// let wave = Wave32::from_samples(44100.0, &[0.0; 22050]);
    /// assert!(wave.channels() == 1 && wave.duration() == 0.5 && wave.amplitude() == 0.0);
    /// ```
    pub fn from_samples(sample_rate: f64, samples: &[f48]) -> Self {
        Self {
            vec: vec![Vec::from(samples)],
            sr: sample_rate,
            len: samples.len(),
            slice: Slice::new(),
        }
    }

    /// Sample rate of this wave.
    #[inline]
    pub fn sample_rate(&self) -> f64 {
        self.sr
    }

    /// Set the sample rate.
    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sr = sample_rate;
    }

    /// Number of channels in this wave.
    #[inline]
    pub fn channels(&self) -> usize {
        self.vec.len()
    }

    /// Return a reference to the requested channel.
    #[inline]
    pub fn channel(&self, channel: usize) -> &Vec<f48> {
        &self.vec[channel]
    }

    /// Add a channel to the wave from a slice of samples.
    /// The length of the wave and the number of samples must match.
    pub fn push_channel(&mut self, samples: &[f48]) {
        assert!(self.channels() == 0 || self.len() == samples.len());
        if self.channels() == 0 {
            self.len = samples.len();
        }
        self.vec.push(samples.into());
    }

    /// Insert a channel to the wave at channel `channel` from a vector of `samples`.
    /// The length of the wave and the number of samples must match.
    pub fn insert_channel(&mut self, channel: usize, samples: &[f48]) {
        assert!(self.channels() == 0 || self.len() == samples.len());
        assert!(channel <= self.channels());
        if self.channels() == 0 {
            self.len = samples.len();
        }
        self.vec.insert(channel, samples.into());
    }

    /// Remove channel `channel` from this wave. Returns the removed channel.
    pub fn remove_channel(&mut self, channel: usize) -> Vec<f48> {
        assert!(channel < self.channels());
        self.vec.remove(channel)
    }

    /// Return a reference to the channels vector as a slice of slices.
    pub fn channels_ref(&mut self) -> &[&[f48]] {
        self.slice.from_refs(&self.vec)
    }

    /// Return a mutable reference to the channels vector as a slice of slices.
    pub fn channels_mut(&mut self) -> &mut [&mut [f48]] {
        self.slice.from_muts(&mut self.vec)
    }

    /// Sample accessor.
    #[inline]
    pub fn at(&self, channel: usize, index: usize) -> f48 {
        self.vec[channel][index]
    }

    /// Set sample to value.
    #[inline]
    pub fn set(&mut self, channel: usize, index: usize, value: f48) {
        self.vec[channel][index] = value;
    }

    /// Insert a new frame of samples to the end of the wave.
    /// Pushing a scalar frame, the value is broadcast to any number of channels.
    /// Otherwise, the number of channels must match.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// let mut wave = Wave64::new(2, 44100.0);
    /// wave.push(0.0);
    /// assert!(wave.len() == 1 && wave.amplitude() == 0.0);
    /// wave.push((-0.5, 0.5));
    /// assert!(wave.len() == 2 && wave.amplitude() == 0.5);
    /// ```
    #[inline]
    pub fn push<T: ConstantFrame<Sample = f48>>(&mut self, frame: T) {
        let frame = frame.convert();
        if T::Size::USIZE == 1 {
            for channel in 0..self.channels() {
                self.vec[channel].push(frame[0]);
            }
        } else {
            assert!(self.channels() == T::Size::USIZE);
            for channel in 0..self.channels() {
                self.vec[channel].push(frame[channel]);
            }
        }
        if self.channels() > 0 {
            self.len += 1;
        }
    }

    /// Length of the wave in samples.
    #[inline]
    pub fn length(&self) -> usize {
        self.len
    }

    /// Length of the wave in samples.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns whether this wave contains no samples.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// let wave = Wave64::new(1, 44100.0);
    /// assert!(wave.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Duration of the wave in seconds.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// let wave = Wave64::with_capacity(1, 44100.0, 44100);
    /// assert!(wave.duration() == 0.0);
    /// ```
    #[inline]
    pub fn duration(&self) -> f64 {
        self.length() as f64 / self.sample_rate()
    }

    /// Resizes the wave in-place. Any new samples are set to zero.
    /// The wave must have a non-zero number of channels.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// let mut wave = Wave64::new(1, 44100.0);
    /// wave.resize(44100);
    /// assert!(wave.duration() == 1.0 && wave.amplitude() == 0.0);
    /// ```
    pub fn resize(&mut self, length: usize) {
        assert!(self.channels() > 0);
        if length != self.length() {
            for channel in 0..self.channels() {
                self.vec[channel].resize(length, 0.0);
            }
        }
        self.len = length;
    }

    /// Peak amplitude of the wave. An empty wave has zero amplitude.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// let mut wave = Wave64::render(44100.0, 1.0, &mut (sine_hz(60.0)));
    /// let amplitude = wave.amplitude();
    /// assert!(amplitude > 1.0 - 1.0e-5 && amplitude <= 1.0);
    /// ```
    pub fn amplitude(&self) -> f48 {
        let mut peak = 0.0;
        for channel in 0..self.channels() {
            for i in 0..self.len() {
                peak = max(peak, abs(self.at(channel, i)));
            }
        }
        peak
    }

    /// Scales the wave to the range -1..1. Does nothing if the wave is empty.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// let mut wave = Wave64::render(44100.0, 1.0, &mut (sine_hz(60.0)));
    /// wave.normalize();
    /// assert!(wave.amplitude() == 1.0);
    /// ```
    pub fn normalize(&mut self) {
        let a = self.amplitude();
        if a == 0.0 || a == 1.0 {
            return;
        }
        let z = 1.0 / a;
        for channel in 0..self.channels() {
            for i in 0..self.len() {
                self.set(channel, i, self.at(channel, i) * z);
            }
        }
    }

    /// Applies a fade-in envelope to the wave with a duration of `time` seconds.
    /// The duration may not exceed the duration of the wave.
    ///
    /// ### Example
    ///
    /// ```
    /// use fundsp::hacker::*;
    /// let mut wave = Wave64::render(44100.0, 10.0, &mut(white()));
    /// wave.fade_in(1.0);
    /// ```
    pub fn fade_in(&mut self, time: f64) {
        assert!(time <= self.duration());
        let fade_n = round(time * self.sample_rate());
        for i in 0..fade_n as usize {
            let a = smooth5((i + 1) as f64 / (fade_n + 1.0)) as f48;
            for channel in 0..self.channels() {
                self.set(channel, i, self.at(channel, i) * a);
            }
        }
    }

    /// Applies a fade-out envelope to the wave with a duration of `time` seconds.
    /// The duration may not exceed the duration of the wave.
    ///
    /// ### Example
    ///
    /// ```
    /// use fundsp::hacker::*;
    /// let mut wave = Wave64::render(44100.0, 10.0, &mut(brown() | brown()));
    /// wave.fade_out(5.0);
    /// ```
    pub fn fade_out(&mut self, time: f64) {
        assert!(time <= self.duration());
        let fade_n = round(time * self.sample_rate());
        let fade_i = fade_n as usize;
        for i in 0..fade_i {
            let a = smooth5((fade_n - i as f64) / (fade_n + 1.0)) as f48;
            for channel in 0..self.channels() {
                self.set(channel, self.len() - fade_i + i, self.at(channel, i) * a);
            }
        }
    }

    /// Applies both fade-in and fade-out to the wave with a duration of `time` seconds.
    /// The duration may not exceed the duration of the wave.
    ///
    /// ### Example
    ///
    /// ```
    /// use fundsp::hacker::*;
    /// let mut wave = Wave64::render(44100.0, 10.0, &mut(pink() | pink()));
    /// wave.fade(1.0);
    /// ```
    pub fn fade(&mut self, time: f64) {
        self.fade_in(time);
        self.fade_out(time);
    }

    /// Render wave with length `duration` seconds from generator `node`.
    /// Resets `node` and sets its sample rate.
    /// Does not discard pre-delay.
    ///
    /// ### Example: Render 10 Seconds Of Stereo Brown Noise
    /// ```
    /// use fundsp::hacker32::*;
    /// let wave = Wave32::render(44100.0, 10.0, &mut (brown() | brown()));
    /// assert!(wave.sample_rate() == 44100.0 && wave.channels() == 2 && wave.duration() == 10.0);
    /// ```
    pub fn render(sample_rate: f64, duration: f64, node: &mut dyn AudioUnit48) -> Self {
        assert!(node.inputs() == 0);
        assert!(node.outputs() > 0);
        assert!(duration >= 0.0);
        node.reset(Some(sample_rate));
        let length = (duration * sample_rate).round() as usize;
        let mut wave = Self::with_capacity(node.outputs(), sample_rate, length);
        wave.len = length;
        let mut i = 0;
        let mut buffer = Self::new(node.outputs(), sample_rate);
        let mut reusable_slice = Slice::<[f48]>::with_capacity(node.outputs());
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

    /// Render wave with length `duration` seconds from generator `node`.
    /// Any pre-delay, as measured by signal latency, is discarded.
    /// Resets `node` and sets its sample rate.
    ///
    /// ### Example: Render 10 Seconds Of Square-Like Wave With Look-Ahead Limiter
    /// ```
    /// use fundsp::hacker32::*;
    /// let wave = Wave32::render_latency(44100.0, 10.0, &mut (lfo(|t| (440.0, exp(-t))) >> dsf_square() >> limiter(0.5)));
    /// assert!(wave.amplitude() <= 1.0 && wave.duration() == 10.0 && wave.sample_rate() == 44100.0);
    /// ```
    pub fn render_latency(sample_rate: f64, duration: f64, node: &mut dyn AudioUnit48) -> Self {
        assert!(node.inputs() == 0);
        assert!(node.outputs() > 0);
        assert!(duration >= 0.0);
        let latency = node.latency().unwrap_or_default();
        // Round latency down to nearest sample.
        let latency_samples = floor(latency) as usize;
        let latency_duration = latency_samples as f64 / sample_rate;
        // Round duration to nearest sample.
        let duration_samples = round(duration * sample_rate) as usize;
        let duration = duration_samples as f64 / sample_rate;
        if latency_samples > 0 {
            let latency_wave = Self::render(sample_rate, duration + latency_duration, node);
            let mut wave = Self::silence(node.outputs(), sample_rate, duration);
            for channel in 0..wave.channels() {
                for i in 0..duration_samples {
                    wave.set(channel, i, latency_wave.at(channel, i + latency_samples));
                }
            }
            wave
        } else {
            Self::render(sample_rate, duration, node)
        }
    }

    /// Filter this wave with `node` and return the resulting wave.
    /// Resets `node` and sets its sample rate. Does not discard pre-delay.
    /// The `node` must have as many inputs as there are channels in this wave.
    /// All zeros input is used for the rest of the wave if
    /// the duration is greater than the duration of this wave.
    ///
    /// ### Example: Reverberate A Pulse Wave
    /// ```
    /// use fundsp::hacker32::*;
    /// let wave1 = Wave32::render(44100.0, 2.0, &mut (lfo(|t| (220.0, lerp11(0.01, 0.99, sin_hz(0.5, t)))) >> pulse() >> pan(0.0)));
    /// assert!(wave1.channels() == 2 && wave1.duration() == 2.0);
    /// let wave2 = wave1.filter(3.0, &mut (multipass() & 0.2 * reverb_stereo(10.0, 1.0)));
    /// assert!(wave2.channels() == 2 && wave2.duration() == 3.0 && wave2.amplitude() > wave1.amplitude());
    /// ```
    pub fn filter(&self, duration: f64, node: &mut dyn AudioUnit48) -> Self {
        assert!(node.inputs() == self.channels());
        assert!(node.outputs() > 0);
        assert!(duration >= 0.0);
        node.reset(Some(self.sample_rate()));
        let total_length = round(duration * self.sample_rate()) as usize;
        let input_length = min(total_length, self.length());
        let mut wave = Self::with_capacity(node.outputs(), self.sample_rate(), total_length);
        wave.len = total_length;
        let mut i = 0;
        let mut input_buffer = Self::new(self.channels(), self.sample_rate());
        let mut reusable_input_slice = Slice::<[f48]>::with_capacity(self.channels());
        let mut output_buffer = Self::new(node.outputs(), self.sample_rate());
        let mut reusable_output_slice = Slice::<[f48]>::with_capacity(node.outputs());
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
                    input_buffer.set(channel, j, 0.0);
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
    /// the `duration` is greater than the duration of this wave.
    pub fn filter_latency(&self, duration: f64, node: &mut dyn AudioUnit48) -> Self {
        assert!(node.inputs() == self.channels());
        assert!(node.outputs() > 0);
        assert!(duration >= 0.0);
        let latency = node.latency().unwrap_or_default();
        // Round latency down to nearest sample.
        let latency_samples = floor(latency) as usize;
        let latency_duration = latency_samples as f64 / self.sample_rate();
        // Round duration to nearest sample.
        let duration_samples = round(duration * self.sample_rate()) as usize;
        let duration = duration_samples as f64 / self.sample_rate();
        if latency_samples > 0 {
            let latency_wave = self.filter(duration + latency_duration, node);
            let mut wave = Self::silence(node.outputs(), self.sample_rate(), duration);
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

    /// Write the wave as a 16-bit WAV to a buffer.
    /// Individual samples are clipped to the range -1...1.
    pub fn write_wav16<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        assert!(self.channels() > 0);
        write_wav_header(
            writer,
            2 * self.channels() * self.length(),
            1,
            self.channels(),
            round(self.sample_rate()) as usize,
        )?;
        for i in 0..self.length() {
            for channel in 0..self.channels() {
                let sample = round(clamp11(self.at(channel, i)) * 32767.49);
                write16(writer, sample.to_i64() as u16)?;
            }
        }
        std::io::Result::Ok(())
    }

    /// Write the wave as a 32-bit float WAV to a buffer.
    /// Samples are not clipped to any range but some
    /// applications may expect the range to be -1...1.
    pub fn write_wav32<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        assert!(self.channels() > 0);
        write_wav_header(
            writer,
            4 * self.channels() * self.length(),
            3,
            self.channels(),
            round(self.sample_rate()) as usize,
        )?;
        for i in 0..self.length() {
            for channel in 0..self.channels() {
                let sample = self.at(channel, i);
                writer.write_all(&sample.to_f32().to_le_bytes())?;
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

/// Play back one channel of a wave.
/// - Output 0: wave
#[duplicate_item(
    f48       Wave48       Wave48Player;
    [ f64 ]   [ Wave64 ]   [ Wave64Player ];
    [ f32 ]   [ Wave32 ]   [ Wave32Player ];
)]
#[derive(Clone)]
pub struct Wave48Player<T: Float> {
    wave: Arc<Wave48>,
    channel: usize,
    index: usize,
    start_point: usize,
    end_point: usize,
    loop_point: Option<usize>,
    _marker: PhantomData<T>,
}

#[duplicate_item(
    f48       Wave48       Wave48Player;
    [ f64 ]   [ Wave64 ]   [ Wave64Player ];
    [ f32 ]   [ Wave32 ]   [ Wave32Player ];
)]
impl<T: Float> Wave48Player<T> {
    pub fn new(
        wave: &Arc<Wave48>,
        channel: usize,
        start_point: usize,
        end_point: usize,
        loop_point: Option<usize>,
    ) -> Self {
        assert!(channel < wave.channels());
        assert!(end_point <= wave.length());
        Self {
            wave: wave.clone(),
            channel,
            index: start_point,
            start_point,
            end_point,
            loop_point,
            _marker: PhantomData::default(),
        }
    }
}

#[duplicate_item(
    f48       Wave48       Wave48Player;
    [ f64 ]   [ Wave64 ]   [ Wave64Player ];
    [ f32 ]   [ Wave32 ]   [ Wave32Player ];
)]
impl<T: Float> AudioNode for Wave48Player<T> {
    const ID: u64 = 65;
    type Sample = T;
    type Inputs = typenum::U0;
    type Outputs = typenum::U1;
    type Setting = ();

    fn reset(&mut self, _sample_rate: Option<f64>) {
        self.index = self.start_point;
    }

    #[inline]
    fn tick(
        &mut self,
        _input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        if self.index < self.end_point {
            let value = self.wave.at(self.channel, self.index);
            self.index += 1;
            if self.index == self.end_point {
                if let Some(point) = self.loop_point {
                    self.index = point;
                }
            }
            [convert(value)].into()
        } else {
            [T::zero()].into()
        }
    }
}
