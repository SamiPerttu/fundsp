//! Delay components.

use super::audionode::*;
use super::math::*;
use super::signal::*;
use super::*;
use num_complex::Complex64;
use numeric_array::typenum::*;

/// Single sample delay.
pub struct Tick<T: Float, N: Size<T>> {
    buffer: Frame<T, N>,
    sample_rate: f64,
}

impl<T: Float, N: Size<T>> Tick<T, N> {
    pub fn new(sample_rate: f64) -> Self {
        Tick {
            buffer: Frame::default(),
            sample_rate,
        }
    }
}

impl<T: Float, N: Size<T>> AudioNode for Tick<T, N> {
    const ID: u64 = 9;
    type Sample = T;
    type Inputs = N;
    type Outputs = N;

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
    fn propagate(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
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
pub struct Delay<T: Float> {
    buffer: Vec<T>,
    i: usize,
    sample_rate: f64,
    length: f64,
}

impl<T: Float> Delay<T> {
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

    fn propagate(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
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
