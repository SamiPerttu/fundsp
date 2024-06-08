//! FIR filters.

use super::audionode::*;
use super::combinator::*;
use super::signal::*;
use super::*;
use num_complex::Complex64;
use numeric_array::typenum::*;

/// FIR filter.
/// - Input 0: input signal
/// - Output 0: filtered signal
#[derive(Clone)]
pub struct Fir<N: Size<f32>> {
    w: Frame<f32, N>,
    v: Frame<f32, N>,
    sample_rate: f64,
}

impl<N: Size<f32>> Fir<N> {
    /// Create new FIR filter from weights.
    pub fn new<W: ConstantFrame<Sample = f32, Size = N>>(weights: W) -> Self {
        Self {
            w: weights.frame(),
            v: Frame::default(),
            sample_rate: DEFAULT_SR,
        }
    }

    /// Return filter weights.
    #[inline]
    pub fn weights(&self) -> Frame<f32, N> {
        self.w.clone()
    }

    /// Set filter weights.
    #[inline]
    pub fn set_weights<W: ConstantFrame<Sample = f32, Size = N>>(&mut self, weights: W) {
        self.w = weights.frame();
    }
}

impl<N: Size<f32>> AudioNode for Fir<N> {
    const ID: u64 = 52;
    type Inputs = U1;
    type Outputs = U1;

    fn reset(&mut self) {
        self.v = Frame::default();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        self.v = Frame::generate(|i| {
            if i + 1 < N::USIZE {
                self.v[i + 1]
            } else {
                input[0]
            }
        });
        let mut output = 0.0;
        for (i1, i2) in self.w[..N::USIZE].iter().zip(self.v[..N::USIZE].iter()) {
            output += *i1 * *i2;
        }
        [output].into()
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = SignalFrame::new(self.outputs());
        output.set(
            0,
            input.at(0).filter(0.0, |r| {
                let z1 = Complex64::from_polar(1.0, -f64::TAU * frequency / self.sample_rate);
                let mut z = Complex64::new(1.0, 0.0);
                let mut x = Complex64::default();
                for i in 0..N::USIZE {
                    x += Complex64::new(self.w[N::USIZE - 1 - i].to_f64(), 0.0) * z;
                    z *= z1;
                }
                r * x
            }),
        );
        output
    }
}
