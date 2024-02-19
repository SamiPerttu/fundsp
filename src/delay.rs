//! Delay components.

use super::audionode::*;
use super::math::*;
use super::signal::*;
use super::*;
use num_complex::Complex64;
use numeric_array::typenum::*;
use std::marker::PhantomData;

/// Single sample delay with `N` channels.
/// - Input(s): input signal.
/// - Output(s): input signal delayed by one sample.
#[derive(Clone, Default)]
pub struct Tick<N: Size<T>, T: Float> {
    buffer: Frame<T, N>,
    sample_rate: f64,
}

impl<N: Size<T>, T: Float> Tick<N, T> {
    /// Create a new single sample delay.
    pub fn new() -> Self {
        Tick {
            buffer: Frame::default(),
            sample_rate: DEFAULT_SR,
        }
    }
}

impl<N: Size<T>, T: Float> AudioNode for Tick<N, T> {
    const ID: u64 = 9;
    type Sample = T;
    type Inputs = N;
    type Outputs = N;
    type Setting = ();

    fn reset(&mut self) {
        self.buffer = Frame::default();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let output = self.buffer.clone();
        self.buffer = input.clone();
        output
    }
    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        for i in 0..self.outputs() {
            output[i] = input[i].filter(1.0, |r| {
                r * Complex64::from_polar(1.0, -TAU * frequency / self.sample_rate)
            });
        }
        output
    }
}

/// Fixed delay.
/// - Allocates: the delay line.
/// - Input 0: input
/// - Output 0: delayed input
#[derive(Clone)]
pub struct Delay<T: Float> {
    buffer: Vec<T>,
    i: usize,
    sample_rate: f64,
    length: f64,
}

impl<T: Float> Delay<T> {
    /// Create a new fixed delay. The `length` of the delay line,
    /// which is specified in seconds, is rounded to the nearest sample.
    /// The minimum delay is one sample.
    pub fn new(length: f64) -> Delay<T> {
        let mut node = Delay {
            buffer: vec![],
            i: 0,
            sample_rate: 0.0,
            length,
        };
        node.set_sample_rate(DEFAULT_SR);
        node
    }
}

impl<T: Float> AudioNode for Delay<T> {
    const ID: u64 = 13;
    type Sample = T;
    type Inputs = U1;
    type Outputs = U1;
    type Setting = ();

    fn reset(&mut self) {
        self.i = 0;
        self.buffer.fill(T::zero());
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        if self.sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            let buffer_length = max(1.0, round(self.length * sample_rate));
            self.buffer.resize(buffer_length as usize, T::zero());
            self.reset();
        }
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let output = self.buffer[self.i];
        self.buffer[self.i] = input[0];
        self.i += 1;
        if self.i >= self.buffer.len() {
            self.i = 0;
        }
        [output].into()
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].filter(self.buffer.len() as f64, |r| {
            r * Complex64::from_polar(
                1.0,
                -TAU * self.buffer.len() as f64 * frequency / self.sample_rate,
            )
        });
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
pub struct Tap<N, T>
where
    T: Float,
    N: Size<T> + Add<U1>,
    <N as Add<U1>>::Output: Size<T>,
{
    buffer: Vec<T>,
    i: usize,
    sample_rate: T,
    min_delay: T,
    max_delay: T,
    _marker: PhantomData<N>,
}

impl<N, T> Tap<N, T>
where
    T: Float,
    N: Size<T> + Add<U1>,
    <N as Add<U1>>::Output: Size<T>,
{
    /// Create a tapped delay line. Minimum and maximum delays are specified in seconds.
    pub fn new(min_delay: T, max_delay: T) -> Self {
        assert!(min_delay >= T::zero());
        assert!(min_delay <= max_delay);
        let mut node = Self {
            buffer: vec![],
            i: 0,
            sample_rate: T::zero(),
            min_delay,
            max_delay,
            _marker: PhantomData,
        };
        node.set_sample_rate(DEFAULT_SR);
        node
    }
}

impl<N, T> AudioNode for Tap<N, T>
where
    T: Float,
    N: Size<T> + Add<U1>,
    <N as Add<U1>>::Output: Size<T>,
{
    const ID: u64 = 50;
    type Sample = T;
    type Inputs = Sum<N, U1>;
    type Outputs = U1;
    type Setting = ();

    fn reset(&mut self) {
        self.i = 0;
        self.buffer.fill(T::zero());
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        let sample_rate = T::from_f64(sample_rate);
        if self.sample_rate != sample_rate {
            let buffer_length = ceil(self.max_delay * sample_rate) + T::new(2);
            let buffer_length = (buffer_length.to_f64() as usize).next_power_of_two();
            self.sample_rate = sample_rate;
            self.buffer.resize(buffer_length, T::zero());
            self.reset();
        }
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let mask = self.buffer.len() - 1;
        let mut output = T::zero();
        for tap_i in 1..N::USIZE + 1 {
            let tap =
                clamp(self.min_delay, self.max_delay, convert(input[tap_i])) * self.sample_rate;
            let tap_floor = unsafe { f32::to_int_unchecked::<usize>(tap.to_f32()) };
            let tap_i1 = self.i + (self.buffer.len() - tap_floor);
            let tap_i0 = (tap_i1 + 1) & mask;
            let tap_i2 = (tap_i1.wrapping_sub(1)) & mask;
            let tap_i3 = (tap_i1.wrapping_sub(2)) & mask;
            let tap_i1 = tap_i1 & mask;
            let tap_d = tap - T::new(tap_floor as i64);
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
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].distort(self.min_delay.to_f64() * self.sample_rate.to_f64());
        output
    }
}

/// Nested allpass where the delay block is replaced by `X`.
/// The number of inputs is `N`, either 1 or 2.
/// - Input 0: input signal
/// - Input 1 (optional): feedforward coefficient
/// - Output 0: filtered signal
#[derive(Clone)]
pub struct AllNest<T, N, X>
where
    T: Float,
    N: Size<T>,
    X: AudioNode<Sample = T, Inputs = U1, Outputs = U1>,
{
    x: X,
    eta: T,
    z: T,
    _marker: PhantomData<N>,
}

impl<T, N, X> AllNest<T, N, X>
where
    T: Float,
    N: Size<T>,
    X: AudioNode<Sample = T, Inputs = U1, Outputs = U1>,
{
    /// Create new nested allpass. Feedforward `coefficient` should
    /// have an absolute value smaller than one to avoid a blowup.
    pub fn new(coefficient: T, x: X) -> Self {
        let mut node = Self {
            x,
            eta: T::zero(),
            z: T::zero(),
            _marker: PhantomData,
        };
        node.set_coefficient(coefficient);
        node
    }

    /// Set feedforward `coefficient`. It should have an absolute
    /// value smaller than one to avoid a blowup.
    #[inline]
    pub fn set_coefficient(&mut self, coefficient: T) {
        self.eta = coefficient;
    }
}

impl<T, N, X> AudioNode for AllNest<T, N, X>
where
    T: Float,
    N: Size<T>,
    X: AudioNode<Sample = T, Inputs = U1, Outputs = U1>,
{
    const ID: u64 = 83;
    type Sample = T;
    type Inputs = N;
    type Outputs = U1;
    type Setting = T;

    fn set(&mut self, setting: Self::Setting) {
        self.set_coefficient(setting);
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x.set_sample_rate(sample_rate);
    }

    fn reset(&mut self) {
        self.z = T::zero();
        self.x.reset();
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        if N::USIZE > 1 {
            self.set_coefficient(input[1]);
        }
        let v = input[0] - self.eta * self.z;
        let y = self.eta * v + self.z;
        self.z = self.x.tick(&[v].into())[0];
        [y].into()
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.x.ping(probe, hash.hash(Self::ID))
    }

    fn allocate(&mut self) {
        self.x.allocate();
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Arbitrary(0.0).propagate(input, self.outputs())
    }
}

/// Variable delay line using linear interpolation.
/// The number of taps is `N`.
/// - Allocates: the delay line.
/// - Input 0: input
/// - Inputs 1...N: delay amount in seconds.
/// - Output 0: delayed input
#[derive(Clone)]
pub struct TapLinear<N, T>
where
    T: Float,
    N: Size<T> + Add<U1>,
    <N as Add<U1>>::Output: Size<T>,
{
    buffer: Vec<T>,
    i: usize,
    sample_rate: T,
    min_delay: T,
    max_delay: T,
    _marker: PhantomData<N>,
}

impl<N, T> TapLinear<N, T>
where
    T: Float,
    N: Size<T> + Add<U1>,
    <N as Add<U1>>::Output: Size<T>,
{
    /// Create a tapped delay line. Minimum and maximum delays are specified in seconds.
    pub fn new(min_delay: T, max_delay: T) -> Self {
        assert!(min_delay >= T::zero());
        assert!(min_delay <= max_delay);
        let mut node = Self {
            buffer: vec![],
            i: 0,
            sample_rate: T::zero(),
            min_delay,
            max_delay,
            _marker: PhantomData,
        };
        node.set_sample_rate(DEFAULT_SR);
        node
    }
}

impl<N, T> AudioNode for TapLinear<N, T>
where
    T: Float,
    N: Size<T> + Add<U1>,
    <N as Add<U1>>::Output: Size<T>,
{
    const ID: u64 = 50;
    type Sample = T;
    type Inputs = Sum<N, U1>;
    type Outputs = U1;
    type Setting = ();

    fn reset(&mut self) {
        self.i = 0;
        self.buffer.fill(T::zero());
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        let sample_rate = T::from_f64(sample_rate);
        if self.sample_rate != sample_rate {
            let buffer_length = ceil(self.max_delay * sample_rate) + T::new(2);
            let buffer_length = (buffer_length.to_f64() as usize).next_power_of_two();
            self.sample_rate = sample_rate;
            self.buffer.resize(buffer_length, T::zero());
            self.reset();
        }
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let mask = self.buffer.len() - 1;
        let mut output = T::zero();
        for tap_i in 1..N::USIZE + 1 {
            let tap =
                clamp(self.min_delay, self.max_delay, convert(input[tap_i])) * self.sample_rate;
            let tap_floor = unsafe { f32::to_int_unchecked::<usize>(tap.to_f32()) };
            let tap_i1 = self.i + (self.buffer.len() - tap_floor);
            let tap_i2 = (tap_i1.wrapping_sub(1)) & mask;
            let tap_i1 = tap_i1 & mask;
            let tap_d = tap - T::new(tap_floor as i64);
            output += lerp(self.buffer[tap_i1], self.buffer[tap_i2], tap_d);
        }
        self.buffer[self.i] = input[0];
        self.i = (self.i + 1) & mask;
        [output].into()
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].distort(self.min_delay.to_f64() * self.sample_rate.to_f64());
        output
    }
}
