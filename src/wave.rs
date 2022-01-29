//! Multichannel wave abstraction.

use crate::audionode::*;
use crate::combinator::*;
use crate::math::*;
use crate::*;
use rsor::Slice;

/// Multichannel wave.
pub struct Wave<F: Float> {
    /// Vector of channels. Each channel is stored in its own vector.
    vec: Vec<Vec<F>>,
    /// Sample rate of the wave.
    sr: f64,
}

impl<F: Float> Wave<F> {
    /// Creates an empty wave. `channels` > 0.
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

    /// Creates an empty wave with the given `capacity`. `channels` > 0.
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

    /// Number of channels in this wave.
    pub fn channels(&self) -> usize {
        self.vec.len()
    }

    /// Returns a reference to the requested channel.
    pub fn channel(&self, channel: usize) -> &Vec<F> {
        &self.vec[channel]
    }

    /// Sample accessor.
    pub fn at(&self, channel: usize, index: usize) -> F {
        self.vec[channel][index]
    }

    /// Sets sample to value.
    pub fn set(&mut self, channel: usize, index: usize, value: F) {
        self.vec[channel][index] = value;
    }

    /// Length of the wave in samples.
    pub fn length(&self) -> usize {
        self.vec[0].len()
    }

    /// Duration of the wave in seconds.
    pub fn duration(&self) -> f64 {
        self.length() as f64 / self.sample_rate()
    }

    /// Resizes the wave in-place. Any new samples are set to zero.
    pub fn resize(&mut self, length: usize) {
        if length != self.length() {
            for channel in 0..self.channels() {
                self.vec[channel].resize(length, F::zero());
            }
        }
    }

    /// Render wave from a generator `node`.
    /// Does not reset `node` or remove pre-delay.
    pub fn render<T>(sample_rate: f64, duration: f64, node: &mut An<T>) -> Self
    where
        T: AudioNode<Sample = F>,
    {
        assert!(node.inputs() == 0);
        assert!(node.outputs() > 0);
        let length = (duration * sample_rate).round() as usize;
        let mut wave = Wave::<F>::with_capacity(node.outputs(), sample_rate, length);
        let mut i = 0;
        let mut buffer = Wave::<F>::new(node.outputs(), sample_rate);
        let mut reusable_slice = Slice::<[F]>::with_capacity(node.outputs());
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

    /// Filter this wave with `node` and return the resulting wave.
    // TODO. What about pre-delay. Maybe make another filter method that discards pre-delay.
    // TODO. What about reseting the node and setting the sample rate?
    pub fn filter<T>(&self, node: &mut An<T>) -> Self
    where
        T: AudioNode<Sample = F>,
    {
        assert!(node.inputs() == self.channels());
        assert!(node.outputs() > 0);
        let length = self.length();
        let mut wave = Wave::<F>::with_capacity(node.outputs(), self.sample_rate(), length);
        let mut i = 0;
        let mut input_buffer = Wave::<F>::new(self.channels(), self.sample_rate());
        let mut reusable_input_slice = Slice::<[F]>::with_capacity(self.channels());
        let mut output_buffer = Wave::<F>::new(node.outputs(), self.sample_rate());
        let mut reusable_output_slice = Slice::<[F]>::with_capacity(node.outputs());
        while i < length {
            let n = min(length - i, MAX_BUFFER_SIZE);
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
        wave
    }
}
