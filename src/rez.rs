//! Resonant two-pole filter by Paul Kellett.

use super::audionode::*;
use super::math::*;
use super::setting::*;
use super::signal::*;
use super::*;
use numeric_array::*;

#[derive(Default, Clone)]
pub struct Rez<F, N> {
    buf0: F,
    buf1: F,
    f: F,
    fb: F,
    cutoff: F,
    q: F,
    sample_rate: F,
    bandpass: F,
    _marker: core::marker::PhantomData<N>,
}

impl<F: Real, N: Size<f32>> Rez<F, N> {
    /// Create new resonant filter. The `bandpass` mode selector is 0 for a lowpass and 1 for a bandpass.
    pub fn new(bandpass: F, cutoff: F, q: F) -> Self {
        let mut node = Self {
            buf0: F::zero(),
            buf1: F::zero(),
            f: F::one(),
            fb: F::one(),
            cutoff,
            q,
            sample_rate: convert(DEFAULT_SR),
            bandpass,
            _marker: core::marker::PhantomData,
        };
        node.set_cutoff_q(cutoff, q);
        node
    }
    #[inline]
    pub fn set_cutoff_q(&mut self, cutoff: F, q: F) {
        self.cutoff = cutoff;
        self.f = F::new(2) * sin(F::PI * cutoff / self.sample_rate);
        self.q = q;
        self.fb = q + q / (F::one() - self.f);
    }
}

impl<F: Real, N: Size<f32>> AudioNode for Rez<F, N> {
    const ID: u64 = 75;
    type Inputs = N;
    type Outputs = typenum::U1;

    fn reset(&mut self) {
        self.buf0 = F::zero();
        self.buf1 = F::zero();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = convert(sample_rate);
        self.set_cutoff_q(self.cutoff, self.q);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        if N::USIZE > 1 {
            let cutoff: F = convert(input[1]);
            let q = convert(input[2]);
            if cutoff != self.cutoff || q != self.q {
                self.set_cutoff_q(cutoff, q);
            }
        }
        let hp: F = convert::<f32, F>(input[0]) - self.buf0;
        let bp = self.buf0 - self.buf1;
        self.buf0 += self.f * (hp + self.fb * tanh(bp));
        self.buf1 += self.f * (self.buf0 - self.buf1);

        [convert(self.buf1 - self.bandpass * self.buf0)].into()
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
