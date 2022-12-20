//! Delay components.

use super::audionode::*;
use super::math::*;
use super::signal::*;
use super::*;
use num_complex::Complex64;
use numeric_array::typenum::*;
use std::marker::PhantomData;

/// Single sample delay.
#[derive(Clone)]
pub struct Tick<N: Size<T>, T: Float> {
    buffer: Frame<T, N>,
    sample_rate: f64,
}

impl<N: Size<T>, T: Float> Tick<N, T> {
    pub fn new(sample_rate: f64) -> Self {
        Tick {
            buffer: Frame::default(),
            sample_rate,
        }
    }
}

impl<N: Size<T>, T: Float> AudioNode for Tick<N, T> {
    const ID: u64 = 9;
    type Sample = T;
    type Inputs = N;
    type Outputs = N;
    type Setting = ();

    #[inline]
    fn reset(&mut self, sample_rate: Option<f64>) {
        if let Some(sample_rate) = sample_rate {
            self.sample_rate = sample_rate;
        }
        self.buffer = Frame::default();
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
    /// Create a new fixed delay. The `length` of the line,
    /// which is specified in seconds, is rounded to the nearest sample.
    pub fn new(length: f64, sample_rate: f64) -> Delay<T> {
        let mut node = Delay {
            buffer: vec![],
            i: 0,
            sample_rate,
            length,
        };
        node.reset(Some(sample_rate));
        node
    }
}

impl<T: Float> AudioNode for Delay<T> {
    const ID: u64 = 13;
    type Sample = T;
    type Inputs = U1;
    type Outputs = U1;
    type Setting = ();

    #[inline]
    fn reset(&mut self, sample_rate: Option<f64>) {
        if let Some(sample_rate) = sample_rate {
            let buffer_length = round(self.length * sample_rate);
            self.sample_rate = sample_rate;
            self.buffer
                .resize(max(1, buffer_length as usize), T::zero());
        }
        self.i = 0;
        for x in self.buffer.iter_mut() {
            *x = T::zero();
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
    pub fn new(sample_rate: f64, min_delay: T, max_delay: T) -> Self {
        let mut node = Tap {
            buffer: vec![],
            i: 0,
            sample_rate: T::from_f64(sample_rate),
            min_delay,
            max_delay,
            _marker: PhantomData::default(),
        };
        node.reset(Some(sample_rate));
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

    #[inline]
    fn reset(&mut self, sample_rate: Option<f64>) {
        if let Some(sample_rate) = sample_rate {
            let buffer_length = ceil(self.max_delay * T::from_f64(sample_rate)) + T::new(2);
            let buffer_length = (buffer_length.to_f64() as usize).next_power_of_two();
            self.sample_rate = T::from_f64(sample_rate);
            self.buffer.resize(buffer_length, T::zero());
        }
        self.i = 0;
        self.buffer.fill(T::zero());
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
            let tap_floor = floor(tap);
            let tap_i1 = self.i + (self.buffer.len() - tap_floor.to_f32() as usize);
            let tap_i0 = (tap_i1 + 1) & mask;
            let tap_i2 = (tap_i1.wrapping_sub(1)) & mask;
            let tap_i3 = (tap_i1.wrapping_sub(2)) & mask;
            let tap_i1 = tap_i1 & mask;
            let tap_d = tap - tap_floor;
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
