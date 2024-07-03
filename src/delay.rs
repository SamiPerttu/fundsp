//! Delay components.

use super::audionode::*;
use super::math::*;
use super::setting::*;
use super::signal::*;
use super::*;
use core::marker::PhantomData;
use num_complex::Complex64;
use numeric_array::typenum::*;
extern crate alloc;
use alloc::vec::Vec;

/// Single sample delay with `N` channels.
/// - Input(s): input signal.
/// - Output(s): input signal delayed by one sample.
#[derive(Clone, Default)]
pub struct Tick<N: Size<f32>> {
    buffer: Frame<f32, N>,
    sample_rate: f64,
}

impl<N: Size<f32>> Tick<N> {
    /// Create a new single sample delay.
    pub fn new() -> Self {
        Tick {
            buffer: Frame::default(),
            sample_rate: DEFAULT_SR,
        }
    }
}

impl<N: Size<f32>> AudioNode for Tick<N> {
    const ID: u64 = 9;
    type Inputs = N;
    type Outputs = N;

    fn reset(&mut self) {
        self.buffer = Frame::default();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let output = self.buffer.clone();
        self.buffer = input.clone();
        output
    }
    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = SignalFrame::new(self.outputs());
        for i in 0..self.outputs() {
            output.set(
                i,
                input.at(i).filter(1.0, |r| {
                    r * Complex64::from_polar(1.0, -f64::TAU * frequency / self.sample_rate)
                }),
            );
        }
        output
    }
}

/// Fixed delay.
/// - Allocates: the delay line.
/// - Input 0: input
/// - Output 0: delayed input
#[derive(Clone)]
pub struct Delay {
    buffer: Vec<f32>,
    i: usize,
    sample_rate: f64,
    length: f64,
}

impl Delay {
    /// Create a new fixed delay. The `length` of the delay line,
    /// which is specified in seconds, is rounded to the nearest sample.
    /// The minimum delay is one sample.
    pub fn new(length: f64) -> Delay {
        let mut node = Delay {
            buffer: Vec::new(),
            i: 0,
            sample_rate: 0.0,
            length,
        };
        node.set_sample_rate(DEFAULT_SR);
        node
    }
}

impl AudioNode for Delay {
    const ID: u64 = 13;
    type Inputs = U1;
    type Outputs = U1;

    fn reset(&mut self) {
        self.i = 0;
        self.buffer.fill(0.0);
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        if self.sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            let buffer_length = max(1.0, round(self.length * sample_rate));
            self.buffer.resize(buffer_length as usize, 0.0);
            self.reset();
        }
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let output = self.buffer[self.i];
        self.buffer[self.i] = input[0];
        self.i += 1;
        if self.i >= self.buffer.len() {
            self.i = 0;
        }
        [output].into()
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = SignalFrame::new(self.outputs());
        output.set(
            0,
            input.at(0).filter(self.buffer.len() as f64, |r| {
                r * Complex64::from_polar(
                    1.0,
                    -f64::TAU * self.buffer.len() as f64 * frequency / self.sample_rate,
                )
            }),
        );
        output
    }
}

/// Variable delay line using cubic interpolation.
/// The number of taps is `N`.
/// - Allocates: the delay line.
/// - Input 0: input
/// - Inputs 1...N: delay amount in seconds.
/// - Output 0: delayed input
#[derive(Clone)]
pub struct Tap<N>
where
    N: Size<f32> + Add<U1>,
    <N as Add<U1>>::Output: Size<f32>,
{
    buffer: Vec<f32>,
    i: usize,
    sample_rate: f32,
    min_delay: f32,
    max_delay: f32,
    _marker: PhantomData<N>,
}

impl<N> Tap<N>
where
    N: Size<f32> + Add<U1>,
    <N as Add<U1>>::Output: Size<f32>,
{
    /// Create a tapped delay line. Minimum and maximum delays are specified in seconds.
    pub fn new(min_delay: f32, max_delay: f32) -> Self {
        assert!(min_delay >= 0.0);
        assert!(min_delay <= max_delay);
        let mut node = Self {
            buffer: Vec::new(),
            i: 0,
            sample_rate: 0.0,
            min_delay,
            max_delay,
            _marker: PhantomData,
        };
        node.set_sample_rate(DEFAULT_SR);
        node
    }
}

impl<N> AudioNode for Tap<N>
where
    N: Size<f32> + Add<U1>,
    <N as Add<U1>>::Output: Size<f32>,
{
    const ID: u64 = 50;
    type Inputs = Sum<N, U1>;
    type Outputs = U1;

    fn reset(&mut self) {
        self.i = 0;
        self.buffer.fill(0.0);
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        let sample_rate = sample_rate as f32;
        if self.sample_rate != sample_rate {
            let buffer_length = ceil(self.max_delay * sample_rate) + 2.0;
            let buffer_length = (buffer_length.to_f64() as usize).next_power_of_two();
            self.sample_rate = sample_rate;
            self.buffer.resize(buffer_length, 0.0);
            self.reset();
        }
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let mask = self.buffer.len() - 1;
        let mut output = 0.0;
        for tap_i in 1..N::USIZE + 1 {
            let tap =
                clamp(self.min_delay, self.max_delay, convert(input[tap_i])) * self.sample_rate;
            // Safety: the value has been clamped.
            let tap_floor = unsafe { f32::to_int_unchecked::<usize>(tap.to_f32()) };
            let tap_i1 = self.i + (self.buffer.len() - tap_floor);
            let tap_i0 = (tap_i1 + 1) & mask;
            let tap_i2 = (tap_i1.wrapping_sub(1)) & mask;
            let tap_i3 = (tap_i1.wrapping_sub(2)) & mask;
            let tap_i1 = tap_i1 & mask;
            let tap_d = tap - tap_floor as f32;
            output += spline(
                self.buffer[tap_i0],
                self.buffer[tap_i1],
                self.buffer[tap_i2],
                self.buffer[tap_i3],
                tap_d,
            );
        }
        self.buffer[self.i] = input[0];
        self.i = (self.i + 1) & mask;
        [output].into()
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = SignalFrame::new(self.outputs());
        output.set(
            0,
            input
                .at(0)
                .distort(self.min_delay.to_f64() * self.sample_rate.to_f64()),
        );
        output
    }
}

/// Nested allpass where the delay block is replaced by `X`.
/// The number of inputs is `N`, either 1 or 2.
/// - Input 0: input signal
/// - Input 1 (optional): feedforward coefficient
/// - Output 0: filtered signal
#[derive(Clone)]
pub struct AllNest<N, X>
where
    N: Size<f32>,
    X: AudioNode<Inputs = U1, Outputs = U1>,
{
    x: X,
    eta: f32,
    z: f32,
    _marker: PhantomData<N>,
}

impl<N, X> AllNest<N, X>
where
    N: Size<f32>,
    X: AudioNode<Inputs = U1, Outputs = U1>,
{
    /// Create new nested allpass. Feedforward `coefficient` should
    /// have an absolute value smaller than one to avoid a blowup.
    pub fn new(coefficient: f32, x: X) -> Self {
        let mut node = Self {
            x,
            eta: 0.0,
            z: 0.0,
            _marker: PhantomData,
        };
        node.set_coefficient(coefficient);
        node
    }

    /// Set feedforward `coefficient`. It should have an absolute
    /// value smaller than one to avoid a blowup.
    #[inline]
    pub fn set_coefficient(&mut self, coefficient: f32) {
        self.eta = coefficient;
    }
}

impl<N, X> AudioNode for AllNest<N, X>
where
    N: Size<f32>,
    X: AudioNode<Inputs = U1, Outputs = U1>,
{
    const ID: u64 = 83;
    type Inputs = N;
    type Outputs = U1;

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x.set_sample_rate(sample_rate);
    }

    fn reset(&mut self) {
        self.z = 0.0;
        self.x.reset();
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        if N::USIZE > 1 {
            self.set_coefficient(input[1]);
        }
        let v = input[0] - self.eta * self.z;
        let y = self.eta * v + self.z;
        self.z = self.x.tick(&[v].into())[0];
        [y].into()
    }

    fn set(&mut self, setting: Setting) {
        if let Parameter::Coefficient(value) = setting.parameter() {
            self.set_coefficient(*value);
        }
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.x.ping(probe, hash.hash(Self::ID))
    }

    fn allocate(&mut self) {
        self.x.allocate();
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Arbitrary(0.0).route(input, self.outputs())
    }
}

/// Variable delay line using linear interpolation.
/// The number of taps is `N`.
/// - Allocates: the delay line.
/// - Input 0: input
/// - Inputs 1...N: delay amount in seconds.
/// - Output 0: delayed input
#[derive(Clone)]
pub struct TapLinear<N>
where
    N: Size<f32> + Add<U1>,
    <N as Add<U1>>::Output: Size<f32>,
{
    buffer: Vec<f32>,
    i: usize,
    sample_rate: f32,
    min_delay: f32,
    max_delay: f32,
    _marker: PhantomData<N>,
}

impl<N> TapLinear<N>
where
    N: Size<f32> + Add<U1>,
    <N as Add<U1>>::Output: Size<f32>,
{
    /// Create a tapped delay line. Minimum and maximum delays are specified in seconds.
    pub fn new(min_delay: f32, max_delay: f32) -> Self {
        assert!(min_delay >= 0.0);
        assert!(min_delay <= max_delay);
        let mut node = Self {
            buffer: Vec::new(),
            i: 0,
            sample_rate: 0.0,
            min_delay,
            max_delay,
            _marker: PhantomData,
        };
        node.set_sample_rate(DEFAULT_SR);
        node
    }
}

impl<N> AudioNode for TapLinear<N>
where
    N: Size<f32> + Add<U1>,
    <N as Add<U1>>::Output: Size<f32>,
{
    const ID: u64 = 50;
    type Inputs = Sum<N, U1>;
    type Outputs = U1;

    fn reset(&mut self) {
        self.i = 0;
        self.buffer.fill(0.0);
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        let sample_rate = sample_rate as f32;
        if self.sample_rate != sample_rate {
            let buffer_length = ceil(self.max_delay * sample_rate) + 2.0;
            let buffer_length = (buffer_length.to_f64() as usize).next_power_of_two();
            self.sample_rate = sample_rate;
            self.buffer.resize(buffer_length, 0.0);
            self.reset();
        }
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let mask = self.buffer.len() - 1;
        let mut output = 0.0;
        for tap_i in 1..N::USIZE + 1 {
            let tap =
                clamp(self.min_delay, self.max_delay, convert(input[tap_i])) * self.sample_rate;
            // Safety: the value has been clamped.
            let tap_floor = unsafe { f32::to_int_unchecked::<usize>(tap.to_f32()) };
            let tap_i1 = self.i + (self.buffer.len() - tap_floor);
            let tap_i2 = (tap_i1.wrapping_sub(1)) & mask;
            let tap_i1 = tap_i1 & mask;
            let tap_d = tap - tap_floor as f32;
            output += lerp(self.buffer[tap_i1], self.buffer[tap_i2], tap_d);
        }
        self.buffer[self.i] = input[0];
        self.i = (self.i + 1) & mask;
        [output].into()
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = SignalFrame::new(self.outputs());
        output.set(
            0,
            input
                .at(0)
                .distort(self.min_delay.to_f64() * self.sample_rate.to_f64()),
        );
        output
    }
}
