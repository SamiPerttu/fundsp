//! FIR filters.

use super::audionode::*;
use super::combinator::*;
use super::math::*;
use super::signal::*;
use super::*;
use num_complex::Complex64;
use numeric_array::typenum::*;

/// FIR filter.
/// - Input 0: input signal
/// - Output 0: filtered signal
pub struct Fir<T: Float, N: Size<T>> {
    w: Frame<T, N>,
    v: Frame<T, N>,
    sample_rate: f64,
}

impl<T: Float, N: Size<T>> Fir<T, N> {
    /// Create new FIR filter from weights.
    pub fn new<W: ConstantFrame<Sample = T, Size = N>>(weights: W) -> Self {
        Self {
            w: weights.convert(),
            v: Frame::default(),
            sample_rate: DEFAULT_SR,
        }
    }
}

impl<T: Float, N: Size<T>> AudioNode for Fir<T, N> {
    const ID: u64 = 52;
    type Sample = T;
    type Inputs = U1;
    type Outputs = U1;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.v = Frame::default();
        if let Some(sr) = sample_rate {
            self.sample_rate = sr;
        }
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        self.v = Frame::generate(|i| {
            if i + 1 < N::USIZE {
                self.v[i + 1]
            } else {
                input[0]
            }
        });
        let mut output = T::zero();
        for i in 0..N::USIZE {
            output += self.w[i] * self.v[i];
        }
        [output].into()
    }

    fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].filter(0.0, |r| {
            let z1 = Complex64::from_polar(1.0, -TAU * frequency / self.sample_rate);
            let mut z = Complex64::new(1.0, 0.0);
            let mut x = Complex64::default();
            for i in 0..N::USIZE {
                x += Complex64::new(self.w[N::USIZE - 1 - i].to_f64(), 0.0) * z;
                z *= z1;
            }
            r * x
        });
        output
    }
}
