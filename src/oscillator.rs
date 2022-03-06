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
        }
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

/// Discrete summation formula. Returns sum, of i in [0, n], of r ** i * sin(f + i * d).
fn dsf<T: Real>(f: T, d: T, r: T, n: T) -> T {
    // Note: beware of division by zero, which results when r = 1 and d = 0.
    // Formula is from Moorer, J. A., The Synthesis of Complex Audio Spectra by Means of Discrete Summation Formulae, 1976.
    (sin(f)
        - r * sin(f - d)
        - pow(r, n + T::one()) * (sin(f + (n + T::one()) * d) - r * sin(f + n * d)))
        / (T::one() + r * r - T::new(2) * r * cos(d))
}

/// DSF oscillator.
pub struct Dsf<T: Real> {
    phase: T,
    roughness: T,
    harmonic_spacing: T,
    sample_duration: T,
    hash: u64,
}

impl<T: Real> Dsf<T> {
    pub fn new(sample_rate: f64, harmonic_spacing: T, roughness: T) -> Self {
        let roughness = clamp(T::from_f64(0.0001), T::from_f64(0.9999), roughness);
        let mut node = Dsf {
            phase: T::zero(),
            roughness,
            harmonic_spacing,
            sample_duration: T::zero(),
            hash: 0,
        };
        node.reset(Some(sample_rate));
        node
    }
}

impl<T: Real> AudioNode for Dsf<T> {
    const ID: u64 = 55;
    type Sample = T;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.phase = T::from_f64(rnd(self.hash as i64));
        if let Some(sr) = sample_rate {
            self.sample_duration = T::from_f64(1.0 / sr);
        }
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        self.phase = (self.phase + input[0] * self.sample_duration).fract();
        let n = floor(T::new(22_050) / input[0] / self.harmonic_spacing);
        Frame::from([dsf(
            self.phase * T::from_f64(TAU),
            self.phase * T::from_f64(TAU) * self.harmonic_spacing,
            self.roughness,
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
