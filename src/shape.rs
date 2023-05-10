//! Waveshaping components.

use super::audionode::*;
use super::math::*;
use super::signal::*;
use super::*;
use numeric_array::typenum::*;
use std::marker::PhantomData;

/// Waveshaper from a closure.
#[derive(Clone)]
pub struct ShaperFn<T, S> {
    f: S,
    _marker: PhantomData<T>,
}

impl<T, S> ShaperFn<T, S>
where
    T: Float,
    S: Fn(T) -> T + Clone,
{
    pub fn new(f: S) -> Self {
        Self {
            f,
            _marker: PhantomData::default(),
        }
    }
}

impl<T, S> AudioNode for ShaperFn<T, S>
where
    T: Float,
    S: Fn(T) -> T + Clone,
{
    const ID: u64 = 37;
    type Sample = T;
    type Inputs = U1;
    type Outputs = U1;
    type Setting = ();

    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        [(self.f)(input[0])].into()
    }

    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        let input = input[0];
        let output = &mut *output[0];
        for i in 0..size {
            output[i] = (self.f)(input[i]);
        }
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].distort(0.0);
        output
    }
}

/// Waveshaping modes.
#[derive(Clone)]
pub enum Shape<T: Real> {
    /// Clip signal to -1...1.
    Clip,
    /// Clip signal between the two arguments.
    ClipTo(T, T),
    /// Apply `tanh` distortion with configurable hardness.
    /// Argument to `tanh` is multiplied by the hardness value.
    Tanh(T),
    /// Apply `softsign` distortion with configurable hardness.
    /// Argument to `softsign` is multiplied by the hardness value.
    Softsign(T),
    /// Apply a staircase function with configurable number of levels per unit.
    Crush(T),
    /// Apply a smooth staircase function with configurable number of levels per unit.
    SoftCrush(T),
    /// Adaptive normalizing `tanh` distortion with smoothing timescale and hardness as parameters.
    /// Smoothing timescale is specified in seconds.
    /// It is the time it takes for level estimation to move halfway to a new level.
    /// The argument to `tanh` is divided by the RMS level of the signal and multiplied by hardness.
    /// Minimum estimated signal level for adaptive distortion is approximately -60 dB.
    AdaptiveTanh(T, T),
}

/// Waveshaper with various shaping modes.
#[derive(Clone)]
pub struct Shaper<T: Real> {
    shape: Shape<T>,
    /// Per-sample smoothing factor.
    smoothing: T,
    state: T,
}

impl<T: Real> Shaper<T> {
    pub fn new(shape: Shape<T>) -> Self {
        let mut shaper = Self {
            shape,
            smoothing: T::zero(),
            state: T::zero(),
        };
        shaper.set_sample_rate(DEFAULT_SR);
        shaper
    }
}

impl<T: Real> AudioNode for Shaper<T> {
    const ID: u64 = 42;
    type Sample = T;
    type Inputs = U1;
    type Outputs = U1;
    type Setting = ();

    fn reset(&mut self) {
        self.state = T::zero();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        if let Shape::AdaptiveTanh(timescale, _) = self.shape {
            self.smoothing = T::from_f64(pow(0.5, 1.0 / (timescale.to_f64() * sample_rate)));
        }
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let input = input[0];
        match self.shape {
            Shape::Clip => [clamp11(input)].into(),
            Shape::ClipTo(min, max) => [clamp(min, max, input)].into(),
            Shape::Tanh(hardness) => [tanh(input * hardness)].into(),
            Shape::Softsign(hardness) => [softsign(input * hardness)].into(),
            Shape::Crush(levels) => [round(input * levels) / levels].into(),
            Shape::SoftCrush(levels) => {
                let x = input * levels;
                let y = floor(x);
                [(y + smooth9(smooth9(x - y))) / levels].into()
            }
            Shape::AdaptiveTanh(_timescale, hardness) => {
                self.state = self.smoothing * self.state
                    + (T::one() - self.smoothing) * (T::from_f32(1.0e-6) + squared(input));
                [tanh(input * hardness / sqrt(self.state))].into()
            }
        }
    }

    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        let input = input[0];
        let output = &mut *output[0];
        match self.shape {
            Shape::Clip => {
                for (x, y) in output[0..size].iter_mut().zip(input[0..size].iter()) {
                    *x = clamp11(*y);
                }
            }
            Shape::ClipTo(min, max) => {
                for (x, y) in output[0..size].iter_mut().zip(input[0..size].iter()) {
                    *x = clamp(min, max, *y);
                }
            }
            Shape::Tanh(hardness) => {
                for (x, y) in output[0..size].iter_mut().zip(input[0..size].iter()) {
                    *x = tanh(*y * hardness);
                }
            }
            Shape::Softsign(hardness) => {
                for (x, y) in output[0..size].iter_mut().zip(input[0..size].iter()) {
                    *x = softsign(*y * hardness);
                }
            }
            Shape::Crush(levels) => {
                for (x, y) in output[0..size].iter_mut().zip(input[0..size].iter()) {
                    *x = round(*y * levels) / levels;
                }
            }
            Shape::SoftCrush(levels) => {
                for (x, y) in output[0..size].iter_mut().zip(input[0..size].iter()) {
                    let a = *y * levels;
                    let b = floor(a);
                    *x = (b + smooth9(smooth9(a - b))) / levels;
                }
            }
            Shape::AdaptiveTanh(_timescale, hardness) => {
                for (x, y) in output[0..size].iter_mut().zip(input[0..size].iter()) {
                    self.state = self.smoothing * self.state
                        + (T::one() - self.smoothing) * (T::from_f32(1.0e-6) + squared(*y));
                    *x = tanh(*y * hardness / sqrt(self.state));
                }
            }
        }
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].distort(0.0);
        output
    }
}
