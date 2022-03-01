//! Oscillator components.

use super::audionode::*;
use super::math::*;
use super::signal::*;
use super::*;
use numeric_array::*;

/// Sine oscillator.
#[derive(Default)]
pub struct Sine<T: Real> {
    phase: T,
    sample_duration: T,
    hash: u64,
}

impl<T: Real> Sine<T> {
    pub fn new(sample_rate: f64) -> Self {
        let mut sine = Sine::default();
        sine.reset(Some(sample_rate));
        sine
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

/// Discrete summation formula. Returns sum, of i in [0, n], of a ** i * sin(f + i * d).
fn dsf<T: Real>(f: T, d: T, a: T, n: T) -> T {
    // Note: beware of division by zero, which results when a = 1 and d = 0.
    // Formula is from Moorer, J. A., The Synthesis of Complex Audio Spectra by Means of Discrete Summation Formulae, 1976.
    (sin(f)
        - a * sin(f - d)
        - pow(a, n + T::one()) * (sin(f + (n + T::one()) * d) - a * sin(f + n * d)))
        / (T::one() + a * a - T::new(2) * a * cos(d))
}

/// Buzz oscillator. WIP.
pub struct Buzz<T: Real> {
    phase: T,
    attenuation: T,
    sample_duration: T,
    hash: u64,
}

impl<T: Real> Buzz<T> {
    pub fn new(sample_rate: f64, attenuation: T) -> Self {
        let mut buzz = Buzz {
            phase: T::zero(),
            attenuation,
            sample_duration: T::zero(),
            hash: 0,
        };
        buzz.reset(Some(sample_rate));
        buzz
    }
}

impl<T: Real> AudioNode for Buzz<T> {
    const ID: u64 = 55;
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
        let n = ceil(T::new(22_050) / input[0]);
        Frame::from([dsf(
            self.phase * T::from_f64(TAU),
            self.phase * T::from_f64(TAU),
            self.attenuation,
            n,
        )])
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
