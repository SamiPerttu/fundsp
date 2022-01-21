use crate::audionode::*;
use crate::math::*;
use crate::*;
use rsor::Slice;

/// Simple multichannel wave object.
pub struct Wave<F: Float> {
    /// Vector of channels. Each channel is stored in its own vector.
    vec: Vec<Vec<F>>,
    /// Sample rate of the wave.
    sr: f64,
}

impl<F: Float> Wave<F> {
    /// Creates an empty wave.
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

    /// Creates an empty wave with the given capacity.
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

    /// Renders wave from node. Does not reset node or remove pre-delay.
    pub fn render<T>(sample_rate: f64, duration: f64, node: &mut T) -> Self
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
            for j in 0..node.outputs() {
                wave.vec[j].extend_from_slice(&buffer.vec[j][..]);
            }
            i += n;
        }
        wave
    }
}
