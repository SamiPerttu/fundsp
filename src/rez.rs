//! Resonant two-pole filter by Paul Kellett.

use super::audionode::*;
use super::math::*;
use super::signal::*;
use super::*;
use numeric_array::*;

#[derive(Default, Clone)]
pub struct Rez<T, F, N> {
    buf0: F,
    buf1: F,
    f: F,
    fb: F,
    cutoff: F,
    q: F,
    sample_rate: F,
    bandpass: F,
    _marker: std::marker::PhantomData<(T, N)>,
}

impl<T: Float, F: Real, N: Size<T>> Rez<T, F, N> {
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
            _marker: std::marker::PhantomData::default(),
        };
        node.set_cutoff_q(cutoff, q);
        node
    }
    #[inline]
    pub fn set_cutoff_q(&mut self, cutoff: F, q: F) {
        self.cutoff = cutoff;
        self.f = F::new(2) * sin(F::from_f64(PI) * cutoff / self.sample_rate);
        self.q = q;
        self.fb = q + q / (F::one() - self.f);
    }
}

impl<T: Float, F: Real, N: Size<T>> AudioNode for Rez<T, F, N> {
    const ID: u64 = 75;
    type Sample = T;
    type Inputs = N;
    type Outputs = typenum::U1;
    type Setting = (F, F);

    fn set(&mut self, (cutoff, q): (F, F)) {
        self.set_cutoff_q(cutoff, q);
    }

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
            let cutoff: F = convert(input[1]);
            let q = convert(input[2]);
            if cutoff != self.cutoff || q != self.q {
                self.set_cutoff_q(cutoff, q);
            }
        }
        let hp: F = convert::<T, F>(input[0]) - self.buf0;
        let bp = self.buf0 - self.buf1;
        self.buf0 += self.f * (hp + self.fb * tanh(bp));
        self.buf1 += self.f * (self.buf0 - self.buf1);

        [convert(self.buf1 - self.bandpass * self.buf0)].into()
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].distort(0.0);
        output
    }
}
