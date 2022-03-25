//! Moog ladder filter.

use super::audionode::*;
use super::math::*;
use super::signal::*;
use super::*;
use numeric_array::*;

/// Moog resonant lowpass filter.
/// The number of inputs is `N`, either `U1` or `U3`.
/// - Input 0: input signal
/// - Input 1 (optional): cutoff frequency (Hz)
/// - Input 2 (optional): Q
/// - Output 0: filtered signal
#[derive(Default)]
pub struct Moog<T: Float, F: Real, N: Size<T>> {
    _marker: std::marker::PhantomData<(T, N)>,
    q: F,
    cutoff: F,
    sample_rate: F,
    rez: F,
    p: F,
    k: F,
    stage0: F,
    stage1: F,
    stage2: F,
    stage3: F,
    px: F,
    ps0: F,
    ps1: F,
    ps2: F,
}

impl<T: Float, F: Real, N: Size<T>> Moog<T, F, N> {
    pub fn new(sample_rate: f64, cutoff: F, q: F) -> Self {
        let mut node = Self {
            sample_rate: convert(sample_rate),
            ..Self::default()
        };
        node.set_cutoff_q(cutoff, q);
        node
    }
    #[inline]
    pub fn set_cutoff_q(&mut self, cutoff: F, q: F) {
        self.cutoff = cutoff;
        self.q = q;
        let c = F::new(2) * cutoff / self.sample_rate;
        self.p = c * (F::from_f64(1.8) - F::from_f64(0.8) * c);
        self.k = F::new(2) * sin(c * F::from_f64(PI * 0.5)) - F::one();
        let t1 = (F::one() - self.p) * F::from_f64(1.386249);
        let t2 = F::new(12) + t1 * t1;
        self.rez = q * (t2 + F::new(6) * t1) / (t2 - F::new(6) * t1);
    }
}

impl<T: Float, F: Real, N: Size<T>> AudioNode for Moog<T, F, N> {
    const ID: u64 = 60;
    type Sample = T;
    type Inputs = N;
    type Outputs = typenum::U1;

    fn reset(&mut self, sample_rate: Option<f64>) {
        if let Some(sample_rate) = sample_rate {
            self.sample_rate = convert(sample_rate);
            self.set_cutoff_q(self.cutoff, self.q);
        }
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        if N::USIZE > 1 {
            self.set_cutoff_q(convert(input[1]), convert(input[2]));
        }

        let x = -self.rez * self.stage3 + convert(input[0]);

        self.stage0 = (x + self.px) * self.p - self.k * self.stage0;
        self.stage1 = (self.stage0 + self.ps0) * self.p - self.k * self.stage1;
        self.stage2 = (self.stage1 + self.ps1) * self.p - self.k * self.stage2;
        self.stage3 = tanh((self.stage2 + self.ps2) * self.p - self.k * self.stage3);

        self.px = x;
        self.ps0 = self.stage0;
        self.ps1 = self.stage1;
        self.ps2 = self.stage2;

        [convert(self.stage3)].into()
    }

    fn route(&self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].distort(0.0);
        output
    }
}
