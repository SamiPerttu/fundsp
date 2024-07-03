//! Feedback components.

use super::audionode::*;
use super::audiounit::*;
use super::buffer::*;
use super::math::*;
use super::signal::*;
use super::*;
use core::marker::PhantomData;
extern crate alloc;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

/// Diffusive Hadamard feedback matrix. The number of channels must be a power of two.
#[derive(Default, Clone)]
pub struct FrameHadamard<N: Size<f32>> {
    _marker: PhantomData<N>,
}

impl<N: Size<f32>> FrameHadamard<N> {
    pub fn new() -> FrameHadamard<N> {
        assert!(N::USIZE.is_power_of_two());
        FrameHadamard::default()
    }
}

impl<N: Size<f32>> FrameUnop<N> for FrameHadamard<N> {
    fn unop(&self, _x: F32x) -> F32x {
        // Not implemented.
        panic!()
    }
    #[inline]
    fn frame(&self, x: &Frame<f32, N>) -> Frame<f32, N> {
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
                    // Note. This unsafe version is not any faster.
                    //let x = unsafe { *output.get_unchecked(j) };
                    //let y = unsafe { *output.get_unchecked(j + h) };
                    //unsafe { *output.get_unchecked_mut(j) = x + y };
                    //unsafe { *output.get_unchecked_mut(j + h) = x - y };
                }
                i += h * 2;
            }
            h *= 2;
        }
        output * Frame::splat((1.0 / sqrt(N::I32 as f64)) as f32)
    }
    // Not implemented.
    // TODO: Hadamard is a special op because of interchannel dependencies.
    fn route(&self, _: Signal) -> Signal {
        panic!()
    }
    fn assign(&self, _size: usize, _x: &mut [f32]) {
        panic!()
    }
}

/// Mix back output of contained node to its input.
/// The contained node must have an equal number of inputs and outputs.
#[derive(Clone)]
pub struct Feedback<N, X, U>
where
    N: Size<f32>,
    X: AudioNode<Inputs = N, Outputs = N>,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
    U: FrameUnop<X::Outputs>,
{
    x: X,
    // Current feedback value.
    value: Frame<f32, N>,
    // Feedback operator.
    #[allow(dead_code)]
    feedback: U,
}

impl<N, X, U> Feedback<N, X, U>
where
    N: Size<f32>,
    X: AudioNode<Inputs = N, Outputs = N>,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
    U: FrameUnop<X::Outputs>,
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

impl<N, X, U> AudioNode for Feedback<N, X, U>
where
    N: Size<f32>,
    X: AudioNode<Inputs = N, Outputs = N>,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
    U: FrameUnop<X::Outputs>,
{
    const ID: u64 = 11;
    type Inputs = N;
    type Outputs = N;

    fn reset(&mut self) {
        self.x.reset();
        self.value = Frame::default();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x.set_sample_rate(sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let output = self.x.tick(&(input + self.value.clone()));
        self.value = self.feedback.frame(&output);
        output
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        for i in 0..size {
            let input_frame =
                Frame::generate(|channel| input.at_f32(channel, i) + self.value[channel]);
            let output_frame = self.x.tick(&input_frame);
            self.value = self.feedback.frame(&output_frame);
            for channel in 0..self.outputs() {
                output.set_f32(channel, i, output_frame[channel]);
            }
        }
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Arbitrary(0.0).route(input, self.outputs())
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
pub struct Feedback2<N, X, Y, U>
where
    N: Size<f32>,
    X: AudioNode<Inputs = N, Outputs = N>,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
    Y: AudioNode<Inputs = N, Outputs = N>,
    Y::Inputs: Size<f32>,
    Y::Outputs: Size<f32>,
    U: FrameUnop<X::Outputs>,
{
    x: X,
    /// Feedback processing.
    y: Y,
    /// Current feedback value.
    value: Frame<f32, N>,
    /// Feedback operator.
    #[allow(dead_code)]
    feedback: U,
}

impl<N, X, Y, U> Feedback2<N, X, Y, U>
where
    N: Size<f32>,
    X: AudioNode<Inputs = N, Outputs = N>,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
    Y: AudioNode<Inputs = N, Outputs = N>,
    Y::Inputs: Size<f32>,
    Y::Outputs: Size<f32>,
    U: FrameUnop<X::Outputs>,
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

impl<N, X, Y, U> AudioNode for Feedback2<N, X, Y, U>
where
    N: Size<f32>,
    X: AudioNode<Inputs = N, Outputs = N>,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
    Y: AudioNode<Inputs = N, Outputs = N>,
    Y::Inputs: Size<f32>,
    Y::Outputs: Size<f32>,
    U: FrameUnop<X::Outputs>,
{
    const ID: u64 = 66;
    type Inputs = N;
    type Outputs = N;

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
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let output = self.x.tick(&(input + self.value.clone()));
        self.value = self.feedback.frame(&self.y.tick(&output));
        output
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        for i in 0..size {
            let input_frame =
                Frame::generate(|channel| input.at_f32(channel, i) + self.value[channel]);
            let output_frame = self.x.tick(&input_frame);
            self.value = self.feedback.frame(&self.y.tick(&output_frame));
            for channel in 0..self.outputs() {
                output.set_f32(channel, i, output_frame[channel]);
            }
        }
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Arbitrary(0.0).route(input, self.outputs())
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.y.ping(probe, self.x.ping(probe, hash.hash(Self::ID)))
    }

    fn allocate(&mut self) {
        self.x.allocate();
    }
}

/// Feedback unit with integrated delay.
#[derive(Clone)]
pub struct FeedbackUnit {
    /// Contained feedback loop.
    x: Box<dyn AudioUnit>,
    /// Number of input and output channels.
    channels: usize,
    /// Current sample rate of the unit.
    sample_rate: f64,
    /// Delay in seconds.
    delay: f64,
    /// Delay in samples.
    samples: usize,
    /// Feedback buffers, one per channel, power-of-two sized.
    feedback: Vec<Vec<f32>>,
    /// Feedback buffer length minus one.
    mask: usize,
    /// Current write index into feedback buffers.
    index: usize,
    /// Buffer for assembling frames.
    tick_buffer: Vec<f32>,
    /// Second buffer for assembling frames.
    tick_buffer2: Vec<f32>,
    /// Buffer for assembling blocks.
    buffer: BufferVec,
}

impl FeedbackUnit {
    /// Create new feedback unit with integrated feedback `delay` in seconds.
    /// The delay amount is rounded to the nearest sample.
    /// The minimum delay is one sample, which may also be accomplished by setting `delay` to zero.
    /// The feedback unit mixes back delayed output of contained unit `x` to its input.
    pub fn new(delay: f64, x: Box<dyn AudioUnit>) -> Self {
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
            buffer: BufferVec::new(channels),
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

impl AudioUnit for FeedbackUnit {
    fn reset(&mut self) {
        for feedback in self.feedback.iter_mut() {
            feedback.fill(0.0);
        }
        self.x.reset();
        self.index = 0;
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        if self.sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            self.x.set_sample_rate(sample_rate);
            self.samples = round(self.delay * sample_rate).max(1.0) as usize;
            let feedback_samples = self.samples.next_power_of_two();
            self.mask = feedback_samples - 1;
            for feedback in self.feedback.iter_mut() {
                feedback.fill(0.0);
                feedback.resize(feedback_samples, 0.0);
            }
            self.index = 0;
        }
    }

    fn tick(&mut self, input: &[f32], output: &mut [f32]) {
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

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        if size <= self.samples {
            // We have enough feedback samples to process the whole block at once.
            for channel in 0..self.channels {
                let mut read_i = self.read_index(self.samples);
                for (b, i) in self.buffer.channel_mut_f32(channel)[0..size]
                    .iter_mut()
                    .zip(input.channel_f32(channel)[0..size].iter())
                {
                    *b = *i + self.feedback[channel][read_i];
                    read_i = (read_i + 1) & self.mask;
                }
            }
            self.x.process(size, &self.buffer.buffer_ref(), output);
            for channel in 0..self.channels {
                let mut write_i = self.index;
                for i in output.channel_f32(channel)[0..size].iter() {
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
                    *tick = input.at_f32(channel, i) + self.feedback[channel][read_i];
                }
                self.x.tick(&self.tick_buffer, &mut self.tick_buffer2);
                for (channel, tick) in self.tick_buffer2.iter().enumerate() {
                    output.set_f32(channel, i, *tick);
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
        Routing::Arbitrary(0.0).route(input, self.outputs())
    }

    fn get_id(&self) -> u64 {
        const ID: u64 = 79;
        ID
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.x.ping(probe, hash.hash(self.get_id()))
    }

    fn footprint(&self) -> usize {
        core::mem::size_of::<Self>()
    }

    fn allocate(&mut self) {
        self.x.allocate();
    }
}
