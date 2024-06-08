//! Moog ladder filter.

use super::audionode::*;
use super::math::*;
use super::setting::*;
use super::signal::*;
use super::*;
use numeric_array::*;

/// Moog resonant lowpass filter.
/// The number of inputs is `N`, either `U1` or `U3`.
/// - Input 0: input signal
/// - Input 1 (optional): cutoff frequency (Hz)
/// - Input 2 (optional): Q
/// - Output 0: filtered signal
#[derive(Default, Clone)]
pub struct Moog<F: Real, N: Size<f32>> {
    _marker: core::marker::PhantomData<N>,
    q: F,
    cutoff: F,
    sample_rate: F,
    rez: F,
    p: F,
    k: F,
    s0: F,
    s1: F,
    s2: F,
    s3: F,
    px: F,
    ps0: F,
    ps1: F,
    ps2: F,
}

impl<F: Real, N: Size<f32>> Moog<F, N> {
    pub fn new(cutoff: F, q: F) -> Self {
        let mut node = Self {
            sample_rate: convert(DEFAULT_SR),
            ..Self::default()
        };
        node.set_cutoff_q(cutoff, q);
        node
    }

    /// Set cutoff frequency (in Hz) and Q.
    /// This has no effect if the filter has cutoff and Q inputs.
    #[inline]
    pub fn set_cutoff_q(&mut self, cutoff: F, q: F) {
        self.cutoff = cutoff;
        self.q = q;
        let c = F::new(2) * cutoff / self.sample_rate;
        self.p = c * (F::from_f64(1.8) - F::from_f64(0.8) * c);
        self.k = F::new(2) * sin(c * F::PI * F::from_f64(0.5)) - F::one();
        let t1 = (F::one() - self.p) * F::from_f64(1.386249);
        let t2 = F::new(12) + t1 * t1;
        self.rez = q * (t2 + F::new(6) * t1) / (t2 - F::new(6) * t1);
    }
}

impl<F: Real, N: Size<f32>> AudioNode for Moog<F, N> {
    const ID: u64 = 60;
    type Inputs = N;
    type Outputs = typenum::U1;

    fn reset(&mut self) {
        self.s0 = F::zero();
        self.s1 = F::zero();
        self.s2 = F::zero();
        self.s3 = F::zero();
        self.px = F::zero();
        self.ps0 = F::zero();
        self.ps1 = F::zero();
        self.ps2 = F::zero();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = convert(sample_rate);
        self.set_cutoff_q(self.cutoff, self.q);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        if N::USIZE > 1 {
            self.set_cutoff_q(convert(input[1]), convert(input[2]));
        }

        let x = -self.rez * self.s3 + convert(input[0]);

        self.s0 = (x + self.px) * self.p - self.k * self.s0;
        self.s1 = (self.s0 + self.ps0) * self.p - self.k * self.s1;
        self.s2 = (self.s1 + self.ps1) * self.p - self.k * self.s2;
        self.s3 = tanh((self.s2 + self.ps2) * self.p - self.k * self.s3);

        self.px = x;
        self.ps0 = self.s0;
        self.ps1 = self.s1;
        self.ps2 = self.s2;

        [convert(self.s3)].into()
    }

    fn set(&mut self, setting: Setting) {
        match setting.parameter() {
            Parameter::Center(cutoff) => self.set_cutoff_q(F::from_f32(*cutoff), self.q),
            Parameter::CenterQ(cutoff, q) => {
                self.set_cutoff_q(F::from_f32(*cutoff), F::from_f32(*q))
            }
            _ => (),
        }
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = SignalFrame::new(self.outputs());
        output.set(0, input.at(0).distort(0.0));
        output
    }
}
