//! Waveshaping components.

use super::audionode::*;
use super::math::*;
use super::signal::*;
use super::*;
use numeric_array::typenum::*;
use std::marker::PhantomData;

/// Waveshaper from a closure.
pub struct ShaperFn<T, S> {
    f: S,
    _marker: PhantomData<T>,
}

impl<T, S> ShaperFn<T, S>
where
    T: Float,
    S: Fn(T) -> T,
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
    S: Fn(T) -> T,
{
    const ID: u64 = 37;
    type Sample = T;
    type Inputs = U1;
    type Outputs = U1;

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

    fn route(&self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].distort(0.0);
        output
    }
}

/// Waveshaping modes.
pub enum Shape<T: Real> {
    /// Clip signal to -1...1.
    Clip,
    /// Clip signal between the two arguments.
    ClipTo(T, T),
    /// Apply `tanh` waveshaping with configurable hardness.
    /// Argument to `tanh` is multiplied by the hardness value.
    Tanh(T),
    /// Apply `softsign` waveshaping with configurable hardness.
    /// Argument to `softsign` is multiplied by the hardness value.
    Softsign(T),
}

/// Waveshaper with various shaping modes.
pub struct Shaper<T: Real> {
    shape: Shape<T>,
    _marker: PhantomData<T>,
}

impl<T: Real> Shaper<T> {
    pub fn new(shape: Shape<T>) -> Self {
        Self {
            shape,
            _marker: PhantomData::default(),
        }
    }
}

impl<T: Real> AudioNode for Shaper<T> {
    const ID: u64 = 42;
    type Sample = T;
    type Inputs = U1;
    type Outputs = U1;

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
                for i in 0..size {
                    output[i] = clamp11(input[i]);
                }
            }
            Shape::ClipTo(min, max) => {
                for i in 0..size {
                    output[i] = clamp(min, max, input[i]);
                }
            }
            Shape::Tanh(hardness) => {
                for i in 0..size {
                    output[i] = tanh(input[i] * hardness);
                }
            }
            Shape::Softsign(hardness) => {
                for i in 0..size {
                    output[i] = softsign(input[i] * hardness);
                }
            }
        }
    }

    fn route(&self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].distort(0.0);
        output
    }
}
