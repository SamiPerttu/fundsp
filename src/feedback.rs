//! Feedback components.

use super::audionode::*;
use super::audiounit::*;
use super::buffer::*;
use super::math::*;
use super::signal::*;
use super::*;
use duplicate::duplicate_item;
use std::marker::PhantomData;

/// Diffusive Hadamard feedback matrix. The number of channels must be a power of two.
#[derive(Default, Clone)]
pub struct FrameHadamard<N: Size<T>, T: Float> {
    _marker: PhantomData<(N, T)>,
}

impl<N: Size<T>, T: Float> FrameHadamard<N, T> {
    pub fn new() -> FrameHadamard<N, T> {
        assert!(N::USIZE.is_power_of_two());
        FrameHadamard::default()
    }
}

impl<N: Size<T>, T: Float> FrameUnop<N, T> for FrameHadamard<N, T> {
    #[inline]
    fn unop(&self, x: &Frame<T, N>) -> Frame<T, N> {
        let mut output = x.clone();
        let mut h = 1;
        while h < N::USIZE {
            let mut i = 0;
            while i < N::USIZE {
                for j in i..i + h {
                    let x = output[j];
                    let y = output[j + h];
                    output[j] = x + y;
                    output[j + h] = x - y;
                    //let x = unsafe { *output.get_unchecked(j) };
                    //let y = unsafe { *output.get_unchecked(j + h) };
                    //unsafe { *output.get_unchecked_mut(j) = x + y };
                    //unsafe { *output.get_unchecked_mut(j + h) = x - y };
                }
                i += h * 2;
            }
            h *= 2;
        }
        // Normalization for up to 256 channels.
        if N::USIZE >= 256 {
            return output * Frame::splat(T::from_f64(1.0 / 16.0));
        }
        if N::USIZE >= 128 {
            return output * Frame::splat(T::from_f64(1.0 / (SQRT_2 * 8.0)));
        }
        if N::USIZE >= 64 {
            return output * Frame::splat(T::from_f64(1.0 / 8.0));
        }
        if N::USIZE >= 32 {
            return output * Frame::splat(T::from_f64(1.0 / (SQRT_2 * 4.0)));
        }
        if N::USIZE >= 16 {
            return output * Frame::splat(T::from_f64(1.0 / 4.0));
        }
        if N::USIZE >= 8 {
            return output * Frame::splat(T::from_f64(1.0 / (SQRT_2 * 2.0)));
        }
        if N::USIZE >= 4 {
            return output * Frame::splat(T::from_f64(1.0 / 2.0));
        }
        if N::USIZE >= 2 {
            return output * Frame::splat(T::from_f64(1.0 / SQRT_2));
        }
        output
    }
    // Not implemented.
    // TODO: Hadamard is a special op because of interchannel dependencies.
    #[inline]
    fn propagate(&self, _: Signal) -> Signal {
        panic!()
    }
    fn assign(&self, _size: usize, _x: &mut [T]) {
        panic!()
    }
}

/// Mix back output of contained node to its input.
/// The contained node must have an equal number of inputs and outputs.
#[derive(Clone)]
pub struct Feedback<N, T, X, U>
where
    N: Size<T>,
    T: Float,
    X: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    U: FrameUnop<X::Outputs, T>,
{
    x: X,
    // Current feedback value.
    value: Frame<T, N>,
    // Feedback operator.
    #[allow(dead_code)]
    feedback: U,
}

impl<N, T, X, U> Feedback<N, T, X, U>
where
    N: Size<T>,
    T: Float,
    X: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    U: FrameUnop<X::Outputs, T>,
{
    pub fn new(x: X, feedback: U) -> Self {
        let mut node = Feedback {
            x,
            value: Frame::default(),
            feedback,
        };
        let hash = node.ping(true, AttoHash::new(Self::ID));
        node.ping(false, hash);
        node
    }
}

impl<N, T, X, U> AudioNode for Feedback<N, T, X, U>
where
    N: Size<T>,
    T: Float,
    X: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    U: FrameUnop<X::Outputs, T>,
{
    const ID: u64 = 11;
    type Sample = T;
    type Inputs = N;
    type Outputs = N;
    type Setting = ();

    fn reset(&mut self) {
        self.x.reset();
        self.value = Frame::default();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x.set_sample_rate(sample_rate);
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let output = self.x.tick(&(input + self.value.clone()));
        self.value = self.feedback.unop(&output);
        output
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Arbitrary.propagate(input, self.outputs())
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.x.ping(probe, hash.hash(Self::ID))
    }

    fn allocate(&mut self) {
        self.x.allocate();
    }
}

/// Mix back output of contained node `X` to its input, with extra feedback processing `Y`.
/// The contained nodes must have an equal number of inputs and outputs.
#[derive(Clone)]
pub struct Feedback2<N, T, X, Y, U>
where
    N: Size<T>,
    T: Float,
    X: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    Y: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    Y::Inputs: Size<T>,
    Y::Outputs: Size<T>,
    U: FrameUnop<X::Outputs, T>,
{
    x: X,
    /// Feedback processing.
    y: Y,
    /// Current feedback value.
    value: Frame<T, N>,
    /// Feedback operator.
    #[allow(dead_code)]
    feedback: U,
}

impl<N, T, X, Y, U> Feedback2<N, T, X, Y, U>
where
    N: Size<T>,
    T: Float,
    X: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    Y: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    Y::Inputs: Size<T>,
    Y::Outputs: Size<T>,
    U: FrameUnop<X::Outputs, T>,
{
    /// Create new (single sample) feedback node.
    /// It mixes back output of contained node `X` to its input, with extra feedback processing `Y`.
    /// The feedforward path does not include `Y`.
    /// The contained nodes must have an equal number of inputs and outputs.
    pub fn new(x: X, y: Y, feedback: U) -> Self {
        let mut node = Feedback2 {
            x,
            y,
            value: Frame::default(),
            feedback,
        };
        let hash = node.ping(true, AttoHash::new(Self::ID));
        node.ping(false, hash);
        node
    }
}

impl<N, T, X, Y, U> AudioNode for Feedback2<N, T, X, Y, U>
where
    N: Size<T>,
    T: Float,
    X: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    Y: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    Y::Inputs: Size<T>,
    Y::Outputs: Size<T>,
    U: FrameUnop<X::Outputs, T>,
{
    const ID: u64 = 66;
    type Sample = T;
    type Inputs = N;
    type Outputs = N;
    type Setting = ();

    fn reset(&mut self) {
        self.x.reset();
        self.y.reset();
        self.value = Frame::default();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x.set_sample_rate(sample_rate);
        self.y.set_sample_rate(sample_rate);
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let output = self.x.tick(&(input + self.value.clone()));
        self.value = self.feedback.unop(&self.y.tick(&output));
        output
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Arbitrary.propagate(input, self.outputs())
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.y.ping(probe, self.x.ping(probe, hash.hash(Self::ID)))
    }

    fn allocate(&mut self) {
        self.x.allocate();
    }
}

#[duplicate_item(
    f48       Feedback48       AudioUnit48;
    [ f64 ]   [ Feedback64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Feedback32 ]   [ AudioUnit32 ];
)]
/// Feedback unit with integrated delay.
#[derive(Clone)]
pub struct Feedback48 {
    /// Contained feedback loop.
    x: Box<dyn AudioUnit48>,
    /// Number of input and output channels.
    channels: usize,
    /// Current sample rate of the unit.
    sample_rate: f48,
    /// Delay in seconds.
    delay: f48,
    /// Delay in samples.
    samples: usize,
    /// Feedback buffers, one per channel, power-of-two sized.
    feedback: Vec<Vec<f48>>,
    /// Feedback buffer length minus one.
    mask: usize,
    /// Current write index into feedback buffers.
    index: usize,
    /// Buffer for assembling frames.
    tick_buffer: Vec<f48>,
    /// Second buffer for assembling frames.
    tick_buffer2: Vec<f48>,
    /// Buffer for assembling blocks.
    buffer: Buffer<f48>,
}

#[duplicate_item(
    f48       Feedback48       AudioUnit48;
    [ f64 ]   [ Feedback64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Feedback32 ]   [ AudioUnit32 ];
)]
impl Feedback48 {
    /// Create new feedback unit with integrated feedback `delay` in seconds.
    /// The delay amount is rounded up to the nearest sample.
    /// The minimum delay is one sample, which may also be accomplished by setting `delay` to zero.
    /// The feedback unit mixes back delayed output of contained unit `x` to its input.
    pub fn new(delay: f48, x: Box<dyn AudioUnit48>) -> Self {
        let channels = x.inputs();
        assert_eq!(channels, x.outputs());
        let mut unit = Self {
            x,
            channels,
            sample_rate: 0.0,
            delay,
            samples: 0,
            feedback: vec![Vec::new(); channels],
            mask: 0,
            index: 0,
            tick_buffer: vec![0.0; channels],
            tick_buffer2: vec![0.0; channels],
            buffer: Buffer::with_channels(channels),
        };
        unit.set_sample_rate(DEFAULT_SR);
        unit
    }

    /// Calculate read index to delayed sample.
    #[inline]
    fn read_index(&self, delay: usize) -> usize {
        (self.index + self.mask + 1 - delay) & self.mask
    }
}

#[duplicate_item(
    f48       Feedback48       AudioUnit48;
    [ f64 ]   [ Feedback64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Feedback32 ]   [ AudioUnit32 ];
)]
impl AudioUnit48 for Feedback48 {
    fn reset(&mut self) {
        for feedback in self.feedback.iter_mut() {
            feedback.fill(0.0);
        }
        self.x.reset();
        self.index = 0;
    }

    #[allow(clippy::unnecessary_cast)]
    fn set_sample_rate(&mut self, sample_rate: f64) {
        if self.sample_rate != sample_rate as f48 {
            self.sample_rate = sample_rate as f48;
            self.x.set_sample_rate(sample_rate);
            self.samples = ceil(self.delay * sample_rate as f48).max(1.0) as usize;
            let feedback_samples = self.samples.next_power_of_two();
            self.mask = feedback_samples - 1;
            for feedback in self.feedback.iter_mut() {
                feedback.fill(0.0);
                feedback.resize(feedback_samples, 0.0);
            }
            self.index = 0;
        }
    }

    fn tick(&mut self, input: &[f48], output: &mut [f48]) {
        let read_i = self.read_index(self.samples);
        for (channel, (tick, i)) in self.tick_buffer.iter_mut().zip(input.iter()).enumerate() {
            *tick = *i + self.feedback[channel][read_i];
        }
        self.x.tick(&self.tick_buffer, output);
        for (channel, i) in output.iter().enumerate() {
            self.feedback[channel][self.index] = *i;
        }
        self.index = (self.index + 1) & self.mask;
    }

    fn process(&mut self, size: usize, input: &[&[f48]], output: &mut [&mut [f48]]) {
        if size <= self.samples {
            // We have enough feedback samples to process the whole block at once.
            for channel in 0..self.channels {
                let mut read_i = self.read_index(self.samples);
                for (b, i) in self.buffer.mut_at(channel)[0..size]
                    .iter_mut()
                    .zip(input[channel][0..size].iter())
                {
                    *b = *i + self.feedback[channel][read_i];
                    read_i = (read_i + 1) & self.mask;
                }
            }
            self.x.process(size, self.buffer.self_ref(), output);
            for channel in 0..self.channels {
                let mut write_i = self.index;
                for i in output[channel][0..size].iter() {
                    self.feedback[channel][write_i] = *i;
                    write_i = (write_i + 1) & self.mask;
                }
            }
        } else {
            // The feedback delay is small so we proceed sample by sample.
            let mut read_i = self.read_index(self.samples);
            let mut write_i = self.index;
            for i in 0..size {
                for (channel, tick) in self.tick_buffer.iter_mut().enumerate() {
                    *tick = input[channel][i] + self.feedback[channel][read_i];
                }
                self.x.tick(&self.tick_buffer, &mut self.tick_buffer2);
                for (channel, tick) in self.tick_buffer2.iter().enumerate() {
                    output[channel][i] = *tick;
                    self.feedback[channel][write_i] = *tick;
                }
                read_i = (read_i + 1) & self.mask;
                write_i = (write_i + 1) & self.mask;
            }
        }
        self.index = (self.index + size) & self.mask;
    }

    fn inputs(&self) -> usize {
        self.feedback.len()
    }

    fn outputs(&self) -> usize {
        self.feedback.len()
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Arbitrary.propagate(input, self.outputs())
    }

    fn get_id(&self) -> u64 {
        const ID: u64 = 79;
        ID
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.x.ping(probe, hash.hash(self.get_id()))
    }

    fn footprint(&self) -> usize {
        std::mem::size_of::<Self>()
    }

    fn allocate(&mut self) {
        self.x.allocate();
    }
}
