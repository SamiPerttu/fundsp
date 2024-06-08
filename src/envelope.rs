//! Subsampled control node.

use super::audionode::*;
use super::buffer::*;
use super::combinator::*;
use super::math::*;
use super::signal::*;
use super::*;
use core::marker::PhantomData;
use numeric_array::*;

/// Sample a time varying function.
/// The return type can be scalar or tuple.
/// It determines the number of output channels.
#[derive(Default, Clone)]
pub struct Envelope<F, E, R>
where
    F: Float,
    E: FnMut(F) -> R + Clone + Send + Sync,
    R: ConstantFrame<Sample = F>,
    R::Size: Size<F> + Size<f32>,
{
    /// Control function.
    envelope: E,
    /// Time at next sample.
    t: F,
    /// Start of current segment.
    t_0: F,
    /// End of current segment.
    t_1: F,
    /// Current time dependent hash.
    t_hash: u64,
    /// Value at start of current segment.
    value_0: Frame<f32, R::Size>,
    /// Value at end of current segment.
    value_1: Frame<f32, R::Size>,
    /// Value at next sample.
    value: Frame<f32, R::Size>,
    /// Value delta per sample.
    value_d: Frame<f32, R::Size>,
    /// Average interval between segments in seconds.
    interval: F,
    /// Sample duration in seconds.
    sample_duration: F,
    /// Deterministic pseudorandom phase.
    hash: u64,
}

impl<F, E, R> Envelope<F, E, R>
where
    F: Float,
    E: FnMut(F) -> R + Clone + Send + Sync,
    R: ConstantFrame<Sample = F>,
    R::Size: Size<F> + Size<f32>,
{
    /// Create new envelope with `interval` seconds between samples.
    pub fn new(interval: F, envelope: E) -> Self {
        assert!(interval > F::zero());
        let mut node = Self {
            envelope,
            t: F::zero(),
            t_0: F::zero(),
            t_1: F::zero(),
            t_hash: 0,
            value_0: Frame::default(),
            value_1: Frame::default(),
            value: Frame::default(),
            value_d: Frame::default(),
            interval,
            sample_duration: F::zero(),
            hash: 0,
        };
        node.set_sample_rate(DEFAULT_SR);
        node.reset();
        node
    }

    /// Move to the next segment.
    fn next_segment(&mut self) {
        self.t_0 = self.t_1;
        self.value_0 = self.value_1.clone();
        // Jitter the next sample point.
        let next_interval = lerp(
            F::from_f32(0.75),
            F::from_f32(1.25),
            convert(rnd1(self.t_hash)),
        ) * self.interval;
        self.t_1 = self.t_0 + next_interval;
        let value_1: Frame<_, _> = (self.envelope)(self.t_1).frame();
        self.value_1 = Frame::generate(|i| convert(value_1[i]));
        self.t_hash = self
            .t_hash
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        let u = delerp(self.t_0, self.t_1, self.t);
        self.value = Frame::generate(|i| lerp(self.value_0[i], self.value_1[i], u.to_f32()));
        let samples = next_interval / self.sample_duration;
        self.value_d = Frame::generate(|i| (self.value_1[i] - self.value_0[i]) / samples.to_f32());
    }
}

impl<F, E, R> AudioNode for Envelope<F, E, R>
where
    F: Float,
    E: FnMut(F) -> R + Clone + Send + Sync,
    R: ConstantFrame<Sample = F>,
    R::Size: Size<F> + Size<f32>,
{
    const ID: u64 = 14;
    type Inputs = typenum::U0;
    type Outputs = R::Size;

    fn reset(&mut self) {
        self.t = F::zero();
        self.t_0 = F::zero();
        self.t_1 = F::zero();
        self.t_hash = self.hash;
        let value_0: Frame<_, _> = (self.envelope)(self.t_0).frame();
        self.value_0 = Frame::generate(|i| convert(value_0[i]));
        self.value_1 = self.value_0.clone();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_duration = convert(1.0 / sample_rate);
    }

    #[inline]
    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        if self.t >= self.t_1 {
            self.next_segment();
        }
        let output = self.value.clone();
        self.value += &self.value_d;
        self.t += self.sample_duration;
        output
    }

    fn process(&mut self, size: usize, _input: &BufferRef, output: &mut BufferMut) {
        if self.t >= self.t_1 {
            self.next_segment();
        }
        let mut i = 0;
        while i < size {
            let segment_samples_left =
                ((self.t_1 - self.t) / self.sample_duration).ceil().to_i64() as usize;
            let loop_samples = Num::min(size - i, segment_samples_left);
            for channel in 0..self.outputs() {
                let mut value = self.value[channel];
                let delta = self.value_d[channel];
                for o in output.channel_f32_mut(channel)[i..i + loop_samples].iter_mut() {
                    *o = value.to_f32();
                    value += delta;
                }
                self.value[channel] = value;
            }
            i += loop_samples;
            self.t += F::new(loop_samples as i64) * self.sample_duration;
            if loop_samples == segment_samples_left {
                self.next_segment();
            }
        }
    }

    fn set_hash(&mut self, hash: u64) {
        self.hash = hash;
        self.t_hash = hash;
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Generator(0.0).route(input, self.outputs())
    }
}

/// Sample a time varying, input dependent function.
/// The return type can be scalar or tuple.
/// It determines the number of output channels.
#[derive(Default, Clone)]
pub struct EnvelopeIn<F, E, I, R>
where
    F: Float,
    E: FnMut(F, &Frame<f32, I>) -> R + Clone + Send + Sync,
    I: Size<f32>,
    R: ConstantFrame<Sample = F>,
    R::Size: Size<F> + Size<f32>,
{
    /// Control function.
    envelope: E,
    /// Time at next sample.
    t: F,
    /// Start of current segment.
    t_0: F,
    /// End of current segment.
    t_1: F,
    /// Current time dependent hash.
    t_hash: u64,
    /// Value at start of current segment.
    value_0: Frame<f32, R::Size>,
    /// Value at end of current segment.
    value_1: Frame<f32, R::Size>,
    /// Value at next sample.
    value: Frame<f32, R::Size>,
    /// Value delta per sample.
    value_d: Frame<f32, R::Size>,
    /// Average interval between segments in seconds.
    interval: F,
    /// Sample duration in seconds.
    sample_duration: F,
    /// Deterministic pseudorandom phase.
    hash: u64,
    _marker: PhantomData<I>,
}

impl<F, E, I, R> EnvelopeIn<F, E, I, R>
where
    F: Float,
    E: FnMut(F, &Frame<f32, I>) -> R + Clone + Send + Sync,
    I: Size<f32>,
    R: ConstantFrame<Sample = F>,
    R::Size: Size<F> + Size<f32>,
{
    /// Create new envelope with `interval` seconds between samples.
    pub fn new(interval: F, envelope: E) -> Self {
        assert!(interval > F::zero());
        let mut node = Self {
            envelope,
            t: F::zero(),
            t_0: F::zero(),
            t_1: F::zero(),
            t_hash: 0,
            value_0: Frame::default(),
            value_1: Frame::default(),
            value: Frame::default(),
            value_d: Frame::default(),
            interval,
            sample_duration: F::zero(),
            hash: 0,
            _marker: PhantomData,
        };
        node.set_sample_rate(DEFAULT_SR);
        node.reset();
        node
    }

    /// Move to the next segment.
    fn next_segment(&mut self, input: &Frame<f32, I>) {
        if self.t_0 == F::zero() && self.t_1 == F::zero() {
            // Get the initial value.
            let value_0: Frame<_, _> = (self.envelope)(self.t_0, input).frame();
            self.value_0 = Frame::generate(|i| convert(value_0[i]));
        } else {
            self.t_0 = self.t_1;
            self.value_0 = self.value_1.clone();
        }
        // Jitter the next sample point.
        let next_interval = lerp(
            F::from_f32(0.75),
            F::from_f32(1.25),
            convert(rnd1(self.t_hash)),
        ) * self.interval;
        self.t_1 = self.t_0 + next_interval;
        let value_1: Frame<_, _> = (self.envelope)(self.t_1, input).frame();
        self.value_1 = Frame::generate(|i| convert(value_1[i]));
        self.t_hash = self
            .t_hash
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        let u = delerp(self.t_0, self.t_1, self.t);
        self.value = Frame::generate(|i| lerp(self.value_0[i], self.value_1[i], u.to_f32()));
        let samples = next_interval / self.sample_duration;
        self.value_d = Frame::generate(|i| (self.value_1[i] - self.value_0[i]) / samples.to_f32());
    }
}

impl<F, E, I, R> AudioNode for EnvelopeIn<F, E, I, R>
where
    F: Float,
    E: FnMut(F, &Frame<f32, I>) -> R + Clone + Send + Sync,
    I: Size<f32>,
    R: ConstantFrame<Sample = F>,
    R::Size: Size<F> + Size<f32>,
{
    const ID: u64 = 53;
    type Inputs = I;
    type Outputs = R::Size;

    fn reset(&mut self) {
        self.t = F::zero();
        self.t_0 = F::zero();
        self.t_1 = F::zero();
        self.t_hash = self.hash;
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_duration = convert(1.0 / sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        if self.t >= self.t_1 {
            self.next_segment(input);
        }
        let output = self.value.clone();
        self.value += &self.value_d;
        self.t += self.sample_duration;
        output
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        if size == 0 {
            return;
        }
        if self.t >= self.t_1 {
            self.next_segment(&Frame::generate(|j| input.at_f32(j, 0)));
        }
        let mut i = 0;
        while i < size {
            let segment_samples_left =
                ceil((self.t_1 - self.t) / self.sample_duration).to_i64() as usize;
            let loop_samples = min(size - i, segment_samples_left);
            for channel in 0..self.outputs() {
                let mut value = self.value[channel];
                let delta = self.value_d[channel];
                for o in output.channel_f32_mut(channel)[i..i + loop_samples].iter_mut() {
                    *o = value.to_f32();
                    value += delta;
                }
                self.value[channel] = value;
            }
            i += loop_samples;
            self.t += F::new(loop_samples as i64) * self.sample_duration;
            if loop_samples == segment_samples_left && i < size {
                self.next_segment(&Frame::generate(|j| input.at_f32(j, i)));
            }
        }
    }

    fn set_hash(&mut self, hash: u64) {
        self.hash = hash;
        self.t_hash = hash;
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Generator(0.0).route(input, self.outputs())
    }
}
