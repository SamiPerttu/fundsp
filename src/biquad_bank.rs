//! Bank of parallel biquad filters with SIMD acceleration.

use super::audionode::*;
use super::biquad::*;
use super::setting::*;
use super::signal::*;
use super::*;

/// 2nd order IIR filter bank implemented in normalized Direct Form I and SIMD.
/// - Setting channel `i` coefficients: `Setting::biquad(a1, a2, b0, b1, b2).index(i)`.
/// - Inputs: input signals.
/// - Outputs: filtered signals.
#[derive(Default, Clone)]
pub struct BiquadBank<F>
where
    F: Float,
{
    coefs: BiquadCoefs<F>,
    x1: F,
    x2: F,
    y1: F,
    y2: F,
    sample_rate: f64,
}

impl<F> BiquadBank<F>
where
    F: Float,
{
    pub fn new() -> Self {
        Self {
            sample_rate: DEFAULT_SR,
            ..Default::default()
        }
    }

    pub fn with_coefs(coefs: BiquadCoefs<F>) -> Self {
        Self {
            coefs,
            sample_rate: DEFAULT_SR,
            ..Default::default()
        }
    }

    pub fn coefs(&self) -> &BiquadCoefs<F> {
        &self.coefs
    }

    pub fn set_coefs(&mut self, coefs: BiquadCoefs<F>) {
        self.coefs = coefs;
    }
}

impl<F> AudioNode for BiquadBank<F>
where
    F: Float,
{
    const ID: u64 = 98;
    type Inputs = F::Size;
    type Outputs = F::Size;

    fn reset(&mut self) {
        self.x1 = F::zero();
        self.x2 = F::zero();
        self.y1 = F::zero();
        self.y2 = F::zero();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let x0 = F::from_frame(input);
        let y0 = self.coefs.b0 * x0 + self.coefs.b1 * self.x1 + self.coefs.b2 * self.x2
            - self.coefs.a1 * self.y1
            - self.coefs.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x0;
        self.y2 = self.y1;
        self.y1 = y0;
        y0.to_frame()
    }

    fn set(&mut self, setting: Setting) {
        if let Address::Index(index) = setting.direction() {
            if let Parameter::Biquad(a1, a2, b0, b1, b2) = setting.parameter() {
                self.coefs.a1.set(index, *a1);
                self.coefs.a2.set(index, *a2);
                self.coefs.b0.set(index, *b0);
                self.coefs.b1.set(index, *b1);
                self.coefs.b2.set(index, *b2);
            }
        }
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = SignalFrame::new(self.outputs());
        for i in 0..self.outputs() {
            let coefs = BiquadCoefs::<f32>::arbitrary(
                self.coefs.a1.get(i),
                self.coefs.a2.get(i),
                self.coefs.b0.get(i),
                self.coefs.b1.get(i),
                self.coefs.b2.get(i),
            );
            output.set(
                i,
                input
                    .at(i)
                    .filter(0.0, |r| r * coefs.response(frequency / self.sample_rate)),
            );
        }
        output
    }
}
