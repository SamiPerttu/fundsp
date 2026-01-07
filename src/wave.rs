//! Multichannel wave abstraction.

use core::usize;

use super::audionode::*;
use super::audiounit::*;
use super::buffer::*;
use super::combinator::*;
use super::math::*;
use super::signal::*;
use super::*;
use numeric_array::typenum::Unsigned;
use numeric_array::*;

extern crate alloc;
use alloc::sync::Arc;
use alloc::vec::Vec;

/// Multichannel wave in 32-bit float precision.
/// Requires memory allocation via `Vec`.
/// Each channel is stored in its own vector of samples.
#[derive(Clone)]
pub struct Wave {
    /// Vector of channels.
    vec: Vec<Vec<f32>>,
    /// Sample rate of the wave.
    sample_rate: f64,
    /// Length of the wave in samples. This is 0 if there are no channels.
    len: usize,
}

impl Wave {
    /// Create an empty wave with the specified number of `channels`.
    ///
    /// ### Example: Create Stereo Wave
    /// ```
    /// use fundsp::wave::*;
    /// let wave = Wave::new(2, 44100.0);
    /// ```
    pub fn new(channels: usize, sample_rate: f64) -> Self {
        let mut vec = Vec::with_capacity(channels);
        for _i in 0..channels {
            vec.push(Vec::new());
        }
        Self {
            vec,
            sample_rate,
            len: 0,
        }
    }

    /// Create an empty wave with the specified number of `channels`
    /// and `capacity` in samples.
    ///
    /// ### Example: Create (Empty) Stereo Wave With Capacity of 1 Second
    /// ```
    /// use fundsp::wave::*;
    /// let wave = Wave::with_capacity(2, 44100.0, 44100);
    /// ```
    pub fn with_capacity(channels: usize, sample_rate: f64, capacity: usize) -> Self {
        let mut vec = Vec::with_capacity(channels);
        for _i in 0..channels {
            vec.push(Vec::with_capacity(capacity));
        }
        Self {
            vec,
            sample_rate,
            len: 0,
        }
    }

    /// Create an all-zeros wave with the given `duration` in seconds
    /// (rounded to the nearest sample) and number of `channels`.
    ///
    /// ### Example
    /// ```
    /// use fundsp::prelude32::*;
    /// let wave = Wave::zero(1, 44100.0, 1.0);
    /// assert!(wave.duration() == 1.0 && wave.amplitude() == 0.0);
    /// ```
    pub fn zero(channels: usize, sample_rate: f64, duration: f64) -> Self {
        let length = (duration * sample_rate).round() as usize;
        assert!(channels > 0 || length == 0);
        let mut vec = Vec::with_capacity(channels);
        for _ in 0..channels {
            let mut v = Vec::with_capacity(length);
            v.resize(length, 0.0);
            vec.push(v);
        }
        Self {
            vec,
            sample_rate,
            len: length,
        }
    }

    /// Create a mono wave from a slice of samples.
    ///
    /// ### Example
    /// ```
    /// use fundsp::prelude32::*;
    /// let wave = Wave::from_samples(44100.0, &[0.0; 22050]);
    /// assert!(wave.channels() == 1 && wave.duration() == 0.5 && wave.amplitude() == 0.0);
    /// ```
    pub fn from_samples(sample_rate: f64, samples: &[f32]) -> Self {
        Self {
            vec: alloc::vec![Vec::from(samples); 1],
            sample_rate,
            len: samples.len(),
        }
    }

    /// The sample rate of the wave.
    #[inline]
    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }

    /// Set the sample rate. No resampling is done.
    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
    }

    /// Number of channels in this wave.
    #[inline]
    pub fn channels(&self) -> usize {
        self.vec.len()
    }

    /// Return a reference to the requested `channel`.
    #[inline]
    pub fn channel(&self, channel: usize) -> &Vec<f32> {
        &self.vec[channel]
    }

    /// Return a mutable slice of `channel`.
    #[inline]
    pub fn channel_mut(&mut self, channel: usize) -> &mut [f32] {
        &mut self.vec[channel]
    }

    /// Add a channel to the wave from a slice of samples.
    /// The length of the wave and the number of samples must match.
    /// If there are no channels yet, then the length of the wave
    /// will become the length of the slice.
    pub fn push_channel(&mut self, samples: &[f32]) {
        assert!(self.channels() == 0 || self.len() == samples.len());
        if self.channels() == 0 {
            self.len = samples.len();
        }
        self.vec.push(samples.into());
    }

    /// Insert a channel to the wave at channel `channel` from a vector of `samples`.
    /// The length of the wave and the number of samples must match.
    /// If there are no channels yet, then the length of the wave
    /// will become the length of the slice.
    pub fn insert_channel(&mut self, channel: usize, samples: &[f32]) {
        assert!(self.channels() == 0 || self.len() == samples.len());
        assert!(channel <= self.channels());
        if self.channels() == 0 {
            self.len = samples.len();
        }
        self.vec.insert(channel, samples.into());
    }

    /// Mix from a vector of `samples` to an existing channel here starting from index `offset`.
    /// The offset may be negative, in which case the first items from `samples` will be ignored.
    /// Does not resize this wave - ignores samples that spill over from either end.
    pub fn mix_channel(&mut self, channel: usize, offset: isize, samples: &[f32]) {
        for i in max(offset, 0)..min(offset + samples.len() as isize, self.len() as isize) {
            self.mix(channel, i as usize, samples[(i - offset) as usize]);
        }
    }

    /// Remove channel `channel` from this wave. Returns the removed channel.
    pub fn remove_channel(&mut self, channel: usize) -> Vec<f32> {
        assert!(channel < self.channels());
        if self.channels() == 1 {
            // This is the last channel, set length to zero.
            self.len = 0;
        }
        self.vec.remove(channel)
    }

    /// Append the contents of the `source` wave to the end of this wave.
    /// The number of channels in `source` and this wave must match.
    /// Any sample rate differences are ignored.
    /// The `source` wave is left unchanged.
    pub fn append(&mut self, source: &Wave) {
        assert!(self.channels() == source.channels());
        let offset = self.length();
        self.resize(self.length() + source.length());
        for i in 0..self.channels() {
            self.mix_channel(i, offset as isize, source.channel(i));
        }
    }

    /// Sample accessor.
    #[inline]
    pub fn at(&self, channel: usize, index: usize) -> f32 {
        self.vec[channel][index]
    }

    /// Set sample to `value`.
    #[inline]
    pub fn set(&mut self, channel: usize, index: usize, value: f32) {
        self.vec[channel][index] = value;
    }

    /// Add `value` to sample.
    #[inline]
    pub fn mix(&mut self, channel: usize, index: usize, value: f32) {
        self.vec[channel][index] += value;
    }

    /// Insert a new frame of samples to the end of the wave.
    /// Pushing a scalar frame, the value is broadcast to any number of channels.
    /// Otherwise, the number of channels must match.
    ///
    /// ### Example
    /// ```
    /// use fundsp::prelude64::*;
    /// let mut wave = Wave::new(2, 44100.0);
    /// wave.push(0.0);
    /// assert!(wave.len() == 1 && wave.amplitude() == 0.0);
    /// wave.push((-0.5, 0.5));
    /// assert!(wave.len() == 2 && wave.amplitude() == 0.5);
    /// ```
    #[inline]
    pub fn push<T: ConstantFrame<Sample = f32>>(&mut self, frame: T) {
        let frame = frame.frame();
        if T::Size::USIZE == 1 {
            for channel in 0..self.channels() {
                self.vec[channel].push(frame[0]);
            }
        } else {
            assert_eq!(self.channels(), T::Size::USIZE);
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
    /// use fundsp::prelude64::*;
    /// let wave = Wave::new(1, 44100.0);
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
    /// use fundsp::prelude64::*;
    /// let wave = Wave::with_capacity(1, 44100.0, 44100);
    /// assert!(wave.duration() == 0.0);
    /// ```
    #[inline]
    pub fn duration(&self) -> f64 {
        self.length() as f64 / self.sample_rate()
    }

    /// Resizes the wave in-place to `length` samples. Any new samples are set to zero.
    /// The wave must have a non-zero number of channels.
    ///
    /// ### Example
    /// ```
    /// use fundsp::prelude64::*;
    /// let mut wave = Wave::new(1, 44100.0);
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

    /// Retain in this wave only samples in `start .. start + length` and discard the rest.
    /// Only the part of the span overlapping the wave is considered.
    /// That is, the offset may be negative and the end of the span may lie beyond the end of the wave.
    pub fn retain(&mut self, start: isize, length: usize) {
        let i0 = max(start, 0) as usize;
        let i1 = min(self.length(), max(0, start + length as isize) as usize);
        if i0 > 0 {
            for channel in 0..self.channels() {
                for i in i0..i1 {
                    self.set(channel, i - i0, self.at(channel, i));
                }
            }
        }
        self.resize(i1 - i0);
    }

    /// Peak amplitude of the wave. An empty wave has zero amplitude.
    ///
    /// ### Example
    /// ```
    /// use fundsp::prelude64::*;
    /// let mut wave = Wave::render(44100.0, 1.0, &mut (sine_hz(60.0)));
    /// let amplitude = wave.amplitude();
    /// assert!(amplitude >= 1.0 - 1.0e-5 && amplitude <= 1.0);
    /// ```
    pub fn amplitude(&self) -> f32 {
        let mut peak = 0.0;
        for channel in 0..self.channels() {
            for i in 0..self.len() {
                peak = peak.max(self.at(channel, i).abs());
            }
        }
        peak
    }

    /// Multiply all samples with `amplitude`.
    pub fn amplify(&mut self, amplitude: f32) {
        for channel in 0..self.channels() {
            for i in 0..self.length() {
                self.set(channel, i, self.at(channel, i) * amplitude);
            }
        }
    }

    /// Scales the wave to the range -1..1. Does nothing if the wave is empty.
    ///
    /// ### Example
    /// ```
    /// use fundsp::prelude64::*;
    /// let mut wave = Wave::render(44100.0, 1.0, &mut (sine_hz(60.0)));
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

    /// Applies a smooth fade-in envelope to the wave with a duration of `time` seconds.
    /// If `time` is greater than the duration of the wave, then it will be set to the duration of the wave.
    ///
    /// ### Example
    ///
    /// ```
    /// use fundsp::prelude64::*;
    /// let mut wave = Wave::render(44100.0, 10.0, &mut(white()));
    /// wave.fade_in(1.0);
    /// ```
    pub fn fade_in(&mut self, time: f64) {
        let time = min(time, self.duration());
        let fade_n = round(time * self.sample_rate());
        for i in 0..fade_n as usize {
            let a = smooth5((i + 1) as f64 / (fade_n + 1.0)) as f32;
            for channel in 0..self.channels() {
                self.set(channel, i, self.at(channel, i) * a);
            }
        }
    }

    /// Applies a smooth fade-out envelope to the wave with a duration of `time` seconds.
    /// If `time` is greater than the duration of the wave, then it will be set to the duration of the wave.
    ///
    /// ### Example
    ///
    /// ```
    /// use fundsp::prelude64::*;
    /// let mut wave = Wave::render(44100.0, 10.0, &mut(brown() | brown()));
    /// wave.fade_out(5.0);
    /// ```
    pub fn fade_out(&mut self, time: f64) {
        let time = min(time, self.duration());
        let fade_n = round(time * self.sample_rate());
        let fade_i = fade_n as usize;
        for i in 0..fade_i {
            let a = smooth5((fade_n - i as f64) / (fade_n + 1.0)) as f32;
            let sample = self.len() - fade_i + i;
            for channel in 0..self.channels() {
                self.set(channel, sample, self.at(channel, sample) * a);
            }
        }
    }

    /// Applies both fade-in and fade-out to the wave with a duration of `time` seconds.
    /// If `time` is greater than the duration of the wave, then it will be set to the duration of the wave.
    ///
    /// ### Example
    ///
    /// ```
    /// use fundsp::prelude64::*;
    /// let mut wave = Wave::render(44100.0, 10.0, &mut(pink() | pink()));
    /// wave.fade(1.0);
    /// ```
    pub fn fade(&mut self, time: f64) {
        self.fade_in(time);
        self.fade_out(time);
    }

    /// Render wave with length `duration` seconds from generator `node`.
    /// Sets the sample rate of `node`.
    /// Does not discard pre-delay.
    ///
    /// ### Example: Render 10 Seconds Of Stereo Brown Noise
    /// ```
    /// use fundsp::prelude64::*;
    /// let wave = Wave::render(44100.0, 10.0, &mut (brown() | brown()));
    /// assert!(wave.sample_rate() == 44100.0 && wave.channels() == 2 && wave.duration() == 10.0);
    /// ```
    pub fn render(sample_rate: f64, duration: f64, node: &mut dyn AudioUnit) -> Self {
        assert_eq!(node.inputs(), 0);
        assert!(node.outputs() > 0);
        assert!(duration >= 0.0);
        node.set_sample_rate(sample_rate);
        let length = (duration * sample_rate).round() as usize;
        let mut wave = Self::with_capacity(node.outputs(), sample_rate, length);
        let mut buffer = BufferVec::new(node.outputs());
        let mut buffer_mut = buffer.buffer_mut();
        wave.len = length;
        let mut i = 0;
        while i < length {
            let n = Num::min(length - i, MAX_BUFFER_SIZE);
            node.process(n, &BufferRef::new(&[]), &mut buffer_mut);
            for channel in 0..node.outputs() {
                for j in 0..n >> SIMD_S {
                    wave.vec[channel].extend_from_slice(buffer_mut.at(channel, j).as_array());
                }
                for j in 0..n & SIMD_M {
                    wave.vec[channel].push(buffer_mut.at_f32(channel, (n & !7) + j));
                }
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
    /// use fundsp::prelude32::*;
    /// let wave = Wave::render_latency(44100.0, 10.0, &mut (lfo(|t| (440.0, exp(-t))) >> dsf_square() >> limiter(0.5, 0.5)));
    /// assert!(wave.amplitude() <= 1.0 && wave.duration() == 10.0 && wave.sample_rate() == 44100.0);
    /// ```
    pub fn render_latency(sample_rate: f64, duration: f64, node: &mut dyn AudioUnit) -> Self {
        assert_eq!(node.inputs(), 0);
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
            let mut wave = Self::zero(node.outputs(), sample_rate, duration);
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
    /// Sets the sample rate of `node`. Does not discard pre-delay.
    /// The `node` must have as many inputs as there are channels in this wave.
    /// All zeros input is used for the rest of the wave if
    /// the duration is greater than the duration of this wave.
    ///
    /// ### Example: Reverberate A Square Wave
    /// ```
    /// use fundsp::prelude32::*;
    /// let wave1 = Wave::render(44100.0, 1.0, &mut (lfo(|t| xerp11(215.0, 225.0, sin_hz(8.0, t))) >> square() >> pan(0.0)));
    /// assert!(wave1.channels() == 2 && wave1.duration() == 1.0);
    /// let mut processor = 0.2 * reverb_stereo(10.0, 1.0, 0.5) & multipass();
    /// let wave2 = wave1.filter(2.0, &mut processor);
    /// assert!(wave2.channels() == 2 && wave2.duration() == 2.0);
    /// ```
    pub fn filter(&self, duration: f64, node: &mut dyn AudioUnit) -> Self {
        assert_eq!(node.inputs(), self.channels());
        assert!(node.outputs() > 0);
        assert!(duration >= 0.0);
        node.set_sample_rate(self.sample_rate());
        let total_length = round(duration * self.sample_rate()) as usize;
        let input_length = min(total_length, self.length());
        let mut wave = Self::with_capacity(node.outputs(), self.sample_rate(), total_length);
        wave.len = total_length;
        let mut i = 0;
        let mut input_buffer = BufferVec::new(self.channels());
        let mut output_buffer = BufferVec::new(node.outputs());
        // Filter from this wave.
        while i < input_length {
            let n = min(input_length - i, MAX_BUFFER_SIZE);
            for channel in 0..self.channels() {
                for j in 0..n {
                    input_buffer.set_f32(channel, j, self.at(channel, i + j));
                }
            }
            node.process(
                n,
                &input_buffer.buffer_ref(),
                &mut output_buffer.buffer_mut(),
            );
            for channel in 0..node.outputs() {
                wave.vec[channel].extend_from_slice(&output_buffer.channel_f32(channel)[0..n]);
            }
            i += n;
        }
        // Filter the rest from a zero input.
        if i < total_length {
            input_buffer.clear();
            while i < total_length {
                let n = min(total_length - i, MAX_BUFFER_SIZE);
                node.process(
                    n,
                    &input_buffer.buffer_ref(),
                    &mut output_buffer.buffer_mut(),
                );
                for channel in 0..node.outputs() {
                    wave.vec[channel].extend_from_slice(&output_buffer.channel_f32(channel)[0..n]);
                }
                i += n;
            }
        }
        wave
    }

    /// Filter hte input waves, channels concatenated, with `node` and return the resulting wave.
    /// Sets the sample rate of `node`. Does not discard pre-delay.
    /// The `node` must have as many inputs as there are channels in this wave.
    /// All zeros input is used for the rest of the wave if
    /// the duration is greater than the duration of this wave.
    ///
    /// TODO: example
    pub fn multifilter<'a>(
        inputs: impl core::iter::Iterator<Item = &'a Self> + Clone,
        duration: f64,
        node: &mut dyn AudioUnit,
    ) -> Self {
        let channels = inputs.clone().map(|a| a.channels()).sum();
        let sample_rate = inputs.clone().next().unwrap().sample_rate();
        assert!(inputs.clone().all(|a| a.sample_rate() == sample_rate));
        assert_eq!(node.inputs(), channels);
        assert!(node.outputs() > 0);
        assert!(duration >= 0.0);
        node.set_sample_rate(sample_rate);
        let total_length = round(duration * sample_rate) as usize;
        let input_length = min(
            total_length,
            inputs.clone().map(|a| a.length()).fold(usize::MAX, min),
        );
        let mut wave = Self::with_capacity(node.outputs(), sample_rate, total_length);
        wave.len = total_length;
        let mut i = 0;
        let mut input_buffer = BufferVec::new(channels);
        let mut output_buffer = BufferVec::new(node.outputs());
        // Filter from this wave.
        while i < input_length {
            let n = min(input_length - i, MAX_BUFFER_SIZE);
            let mut k = 0;
            for input in inputs.clone() {
                for channel in 0..input.channels() {
                    for j in 0..n {
                        input_buffer.set_f32(channel + k, j, input.at(channel, i + j));
                    }
                }
                k += input.channels();
            }
            node.process(
                n,
                &input_buffer.buffer_ref(),
                &mut output_buffer.buffer_mut(),
            );
            for channel in 0..node.outputs() {
                wave.vec[channel].extend_from_slice(&output_buffer.channel_f32(channel)[0..n]);
            }
            i += n;
        }
        // Filter the rest from a zero input.
        if i < total_length {
            input_buffer.clear();
            while i < total_length {
                let n = min(total_length - i, MAX_BUFFER_SIZE);
                node.process(
                    n,
                    &input_buffer.buffer_ref(),
                    &mut output_buffer.buffer_mut(),
                );
                for channel in 0..node.outputs() {
                    wave.vec[channel].extend_from_slice(&output_buffer.channel_f32(channel)[0..n]);
                }
                i += n;
            }
        }
        wave
    }

    /// Filter this wave with `node` and return the resulting wave.
    /// Any pre-delay, as measured by signal latency, is discarded.
    /// Sets the sample rate of `node`.
    /// The `node` must have as many inputs as there are channels in this wave.
    /// All zeros input is used for the rest of the wave if
    /// the `duration` is greater than the duration of this wave.
    pub fn filter_latency(&self, duration: f64, node: &mut dyn AudioUnit) -> Self {
        assert_eq!(node.inputs(), self.channels());
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
            let mut wave = Self::zero(node.outputs(), self.sample_rate(), duration);
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

    /// Filter the input waves, channels concatenated, with `node` and return the resulting wave.
    /// Any pre-delay, as measured by signal latency, is discarded.
    /// Sets the sample rate of `node`.
    /// The `node` must have as many inputs as there are channels in this wave.
    /// All zeros input is used for the rest of the wave if
    /// the `duration` is greater than the duration of this wave.
    pub fn multifilter_latency<'a>(
        inputs: impl core::iter::Iterator<Item = &'a Self> + Clone,
        duration: f64,
        node: &mut dyn AudioUnit,
    ) -> Self {
        let channels = inputs.clone().map(|a| a.channels()).sum();
        let sample_rate = inputs.clone().next().unwrap().sample_rate();
        assert!(inputs.clone().all(|a| a.sample_rate() == sample_rate));
        assert_eq!(node.inputs(), channels);
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
            let latency_wave = Self::multifilter(inputs, duration + latency_duration, node);
            let mut wave = Self::zero(node.outputs(), sample_rate, duration);
            for channel in 0..wave.channels() {
                for i in 0..duration_samples {
                    wave.set(channel, i, latency_wave.at(channel, i + latency_samples));
                }
            }
            wave
        } else {
            Self::multifilter(inputs, duration, node)
        }
    }

    /// Resample this wave using FIR based sinc interpolation.
    /// These source and target rates are supported:
    /// 16 kHz, 22.05 kHz, 32 kHz, 44.1 kHz, 48 kHz, 88.2 kHz, 96 kHz, 176.4 kHz, 192 kHz, 384 kHz.
    pub fn resample_fir(&mut self, target_rate: f64) -> Wave {
        let mut output = Wave::new(self.channels(), target_rate);
        let length = self.length();
        for channel in 0..self.channels() {
            let mut input_wave = Wave::new(0, target_rate);
            input_wave.insert_channel(0, &self.remove_channel(channel));
            {
                let input_arc = Arc::new(input_wave);
                {
                    let mut resampler = super::prelude::resample_fir(
                        self.sample_rate(),
                        target_rate,
                        super::prelude::playwave(&input_arc, 0, None),
                    );
                    while resampler.samples() < length + 16 {
                        let value = resampler.get_mono();
                        output.resize(max(output.length(), resampler.produced()));
                        output.set(channel, resampler.produced() - 1, value);
                    }
                }
                let mut input_wave = Arc::into_inner(input_arc).unwrap();
                self.insert_channel(channel, &input_wave.remove_channel(0));
            }
        }
        output
    }
}

#[derive(Clone)]
pub struct WavePlayer {
    wave: Arc<Wave>,
    channel: usize,
    index: usize,
    start_point: usize,
    end_point: usize,
    loop_point: Option<usize>,
}

impl WavePlayer {
    pub fn new(
        wave: &Arc<Wave>,
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
        }
    }
}

impl AudioNode for WavePlayer {
    const ID: u64 = 65;
    type Inputs = typenum::U0;
    type Outputs = typenum::U1;

    fn reset(&mut self) {
        self.index = self.start_point;
    }

    #[inline]
    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
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
            [0.0].into()
        }
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Generator(0.0).route(input, self.outputs())
    }
}
