//! Oscillator components.

use super::audionode::*;
use super::math::*;
use super::signal::*;
use super::*;
use numeric_array::*;

/// Sine oscillator.
pub struct Sine<T: Real> {
    phase: T,
    sample_duration: T,
    hash: u64,
}

impl<T: Real> Sine<T> {
    pub fn new(sample_rate: f64) -> Sine<T> {
        Sine {
            phase: T::zero(),
            sample_duration: T::from_f64(1.0 / sample_rate),
            hash: 0,
        }
    }
}

impl<T: Real> AudioNode for Sine<T> {
    const ID: u64 = 21;
    type Sample = T;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.phase = T::from_f64(rnd(self.hash as i64));
        if let Some(sr) = sample_rate {
            self.sample_duration = T::from_f64(1.0 / sr);
        };
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        self.phase = (self.phase + input[0] * self.sample_duration).fract();
        [sin(self.phase * T::from_f64(TAU))].into()
    }

    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        for i in 0..size {
            self.phase += input[0][i] * self.sample_duration;
            output[0][i] = sin(self.phase * T::from_f64(TAU));
        }
        self.phase = self.phase.fract();
    }

    #[inline]
    fn set_hash(&mut self, hash: u64) {
        self.hash = hash;
        self.reset(None);
    }

    fn route(&self, _input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = Signal::Latency(0.0);
        output
    }
}
