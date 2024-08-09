//! Biquad filters with nonlinearities.

use super::audionode::*;
use super::math::*;
use super::setting::*;
use super::shape::*;
use super::signal::*;
use super::*;
use core::marker::PhantomData;

#[derive(Copy, Clone, Debug, Default)]
pub struct BiquadCoefs<F> {
    pub a1: F,
    pub a2: F,
    pub b0: F,
    pub b1: F,
    pub b2: F,
}

impl<F: Real> BiquadCoefs<F> {
    /// Returns settings for a Butterworth lowpass filter.
    /// Cutoff is the -3 dB point of the filter in Hz.
    pub fn butter_lowpass(sample_rate: F, cutoff: F) -> Self {
        let c = F::from_f64;
        let f: F = tan(cutoff * F::PI / sample_rate);
        let a0r: F = c(1.0) / (c(1.0) + F::SQRT_2 * f + f * f);
        let a1: F = (c(2.0) * f * f - c(2.0)) * a0r;
        let a2: F = (c(1.0) - F::SQRT_2 * f + f * f) * a0r;
        let b0: F = f * f * a0r;
        let b1: F = c(2.0) * b0;
        let b2: F = b0;
        Self { a1, a2, b0, b1, b2 }
    }

    /// Returns settings for a constant-gain bandpass resonator.
    /// The center frequency is given in Hz.
    /// Bandwidth is the difference in Hz between -3 dB points of the filter response.
    /// The overall gain of the filter is independent of bandwidth.
    pub fn resonator(sample_rate: F, center: F, bandwidth: F) -> Self {
        let c = F::from_f64;
        let r: F = exp(-F::PI * bandwidth / sample_rate);
        let a1: F = c(-2.0) * r * cos(F::TAU * center / sample_rate);
        let a2: F = r * r;
        let b0: F = sqrt(c(1.0) - r * r) * c(0.5);
        let b1: F = c(0.0);
        let b2: F = -b0;
        Self { a1, a2, b0, b1, b2 }
    }

    /// Arbitrary biquad.
    pub fn arbitrary(a1: F, a2: F, b0: F, b1: F, b2: F) -> Self {
        Self { a1, a2, b0, b1, b2 }
    }

    /// Frequency response at frequency `omega` expressed as fraction of sampling rate.
    pub fn response(&self, omega: f64) -> Complex64 {
        let z1 = Complex64::from_polar(1.0, -f64::TAU * omega);
        let z2 = z1 * z1;
        /// Complex64 with real component `x` and imaginary component zero.
        fn re<T: Float>(x: T) -> Complex64 {
            Complex64::new(x.to_f64(), 0.0)
        }
        (re(self.b0) + re(self.b1) * z1 + re(self.b2) * z2)
            / (re(1.0) + re(self.a1) * z1 + re(self.a2) * z2)
    }
}

/// 2nd order IIR filter implemented in normalized Direct Form I.
/// - Setting: coefficients as tuple Parameter::Biquad(a1, a2, b0, b1, b2).
/// - Input 0: input signal.
/// - Output 0: filtered signal.
#[derive(Default, Clone)]
pub struct Biquad<F> {
    coefs: BiquadCoefs<F>,
    x1: F,
    x2: F,
    y1: F,
    y2: F,
    sample_rate: f64,
}

impl<F: Real> Biquad<F> {
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

impl<F: Real> AudioNode for Biquad<F> {
    const ID: u64 = 15;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;

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
        let x0 = convert(input[0]);
        let y0 = self.coefs.b0 * x0 + self.coefs.b1 * self.x1 + self.coefs.b2 * self.x2
            - self.coefs.a1 * self.y1
            - self.coefs.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x0;
        self.y2 = self.y1;
        self.y1 = y0;
        [convert(y0)].into()
    }

    fn set(&mut self, setting: Setting) {
        if let Parameter::Biquad(a1, a2, b0, b1, b2) = setting.parameter() {
            self.set_coefs(BiquadCoefs::arbitrary(
                F::from_f32(*a1),
                F::from_f32(*a2),
                F::from_f32(*b0),
                F::from_f32(*b1),
                F::from_f32(*b2),
            ));
        }
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = SignalFrame::new(self.outputs());
        output.set(
            0,
            input.at(0).filter(0.0, |r| {
                r * self.coefs().response(frequency / self.sample_rate)
            }),
        );
        output
    }
}

/// Butterworth lowpass filter.
/// Setting: cutoff.
/// Number of inputs is `N`, either `U1` or `U2`.
/// - Input 0: input signal
/// - Input 1 (optional): cutoff frequency (Hz)
/// - Output 0: filtered signal
#[derive(Clone)]
pub struct ButterLowpass<F: Real, N: Size<f32>> {
    _marker: PhantomData<N>,
    biquad: Biquad<F>,
    sample_rate: F,
    cutoff: F,
}

impl<F: Real, N: Size<f32>> ButterLowpass<F, N> {
    /// Create new Butterworth lowpass filter with initial `cutoff` frequency in Hz.
    pub fn new(cutoff: F) -> Self {
        let mut node = ButterLowpass {
            _marker: PhantomData,
            biquad: Biquad::new(),
            sample_rate: F::from_f64(DEFAULT_SR),
            cutoff: F::zero(),
        };
        node.biquad.reset();
        node.set_cutoff(cutoff);
        node
    }
    pub fn set_cutoff(&mut self, cutoff: F) {
        self.biquad
            .set_coefs(BiquadCoefs::butter_lowpass(self.sample_rate, cutoff));
        self.cutoff = cutoff;
    }
}

impl<F: Real, N: Size<f32>> AudioNode for ButterLowpass<F, N> {
    const ID: u64 = 16;
    type Inputs = N;
    type Outputs = typenum::U1;

    fn reset(&mut self) {
        self.biquad.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = convert(sample_rate);
        self.biquad.set_sample_rate(sample_rate);
        self.set_cutoff(self.cutoff);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        if N::USIZE > 1 {
            let cutoff: F = convert(input[1]);
            if cutoff != self.cutoff {
                self.set_cutoff(cutoff);
            }
        }
        self.biquad.tick(&[input[0]].into())
    }

    fn set(&mut self, setting: Setting) {
        if let Parameter::Center(cutoff) = setting.parameter() {
            self.set_cutoff(F::from_f32(*cutoff));
        }
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = SignalFrame::new(self.outputs());
        output.set(
            0,
            input.at(0).filter(0.0, |r| {
                r * self
                    .biquad
                    .coefs()
                    .response(frequency / self.sample_rate.to_f64())
            }),
        );
        output
    }
}

/// Constant-gain bandpass filter (resonator).
/// Filter gain is (nearly) independent of bandwidth.
/// Setting: (center, bandwidth).
/// Number of inputs is `N`, either `U1` or `U3`.
/// - Input 0: input signal
/// - Input 1 (optional): filter center frequency (peak) (Hz)
/// - Input 2 (optional): filter bandwidth (distance) between -3 dB points (Hz)
/// - Output 0: filtered signal
#[derive(Clone)]
pub struct Resonator<F: Real, N: Size<f32>> {
    _marker: PhantomData<N>,
    biquad: Biquad<F>,
    sample_rate: F,
    center: F,
    bandwidth: F,
}

impl<F: Real, N: Size<f32>> Resonator<F, N> {
    /// Create new resonator bandpass. Initial `center` frequency and `bandwidth` are specified in Hz.
    pub fn new(center: F, bandwidth: F) -> Self {
        let mut node = Resonator {
            _marker: PhantomData,
            biquad: Biquad::new(),
            sample_rate: F::from_f64(DEFAULT_SR),
            center,
            bandwidth,
        };
        node.biquad.reset();
        node.set_center_bandwidth(center, bandwidth);
        node
    }
    pub fn set_center_bandwidth(&mut self, center: F, bandwidth: F) {
        self.biquad
            .set_coefs(BiquadCoefs::resonator(self.sample_rate, center, bandwidth));
        self.center = center;
        self.bandwidth = bandwidth;
    }
}

impl<F: Real, N: Size<f32>> AudioNode for Resonator<F, N> {
    const ID: u64 = 17;
    type Inputs = N;
    type Outputs = typenum::U1;

    fn reset(&mut self) {
        self.biquad.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = convert(sample_rate);
        self.set_center_bandwidth(self.center, self.bandwidth);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        if N::USIZE >= 3 {
            let center: F = convert(input[1]);
            let bandwidth: F = convert(input[2]);
            if center != self.center || bandwidth != self.bandwidth {
                self.biquad
                    .set_coefs(BiquadCoefs::resonator(self.sample_rate, center, bandwidth));
                self.center = center;
                self.bandwidth = bandwidth;
            }
        }
        self.biquad.tick(&[input[0]].into())
    }

    fn set(&mut self, setting: Setting) {
        match setting.parameter() {
            Parameter::Center(center) => {
                self.set_center_bandwidth(F::from_f32(*center), self.bandwidth)
            }
            Parameter::CenterBandwidth(center, bandwidth) => {
                self.set_center_bandwidth(F::from_f32(*center), F::from_f32(*bandwidth))
            }
            _ => (),
        }
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = SignalFrame::new(self.outputs());
        output.set(
            0,
            input.at(0).filter(0.0, |r| {
                r * self
                    .biquad
                    .coefs()
                    .response(frequency / self.sample_rate.to_f64())
            }),
        );
        output
    }
}

#[derive(Clone)]
/// Biquad in transposed direct form II with nonlinear feedback.
pub struct BiquadFb<F: Real, S: Shape> {
    shape1: S,
    shape2: S,
    coefs: BiquadCoefs<F>,
    s1: F,
    s2: F,
}

// Transposed Direct Form II would be:
//   y0 = b0 * x0 + s1
//   s1 = s2 + b1 * x0 - a1 * y0
//   s2 = b2 * x0 - a2 * y0

impl<F: Real, S: Shape> BiquadFb<F, S> {
    /*
    inline float process (float x) override
    {
        // process input sample, direct form II transposed
        float y = z[1] + x*b[0];
        z[1] = z[2] + x*b[1] - saturator (y)*a[1];
        z[2] = x*b[2] - saturator (y)*a[2];

        return y;
    }
    */
}

impl<F: Real, S: Shape> AudioNode for BiquadFb<F, S> {
    const ID: u64 = 88;
    /// Input arity.
    type Inputs = typenum::U1;
    /// Output arity.
    type Outputs = typenum::U1;

    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let x0 = F::from_f32(input[0]);
        let y0 = self.coefs.b0 * x0 + self.s1;
        self.s1 = self.s2 + self.coefs.b1 * x0
            - F::from_f32(self.shape1.shape(y0.to_f32())) * self.coefs.a1;
        self.s2 = self.coefs.b2 * x0 - F::from_f32(self.shape2.shape(y0.to_f32())) * self.coefs.a2;
        [y0.to_f32()].into()
    }

    /*
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let x0 = F::from_f32(input[0]);
        let y0 = self.coefs.b0 * x0 + self.s1;
        self.s1 = F::from_f32(self.shape1.shape((self.s2 + self.coefs.b1 * x0 - y0 * self.coefs.a1).to_f32()));
        self.s2 = F::from_f32(self.shape2.shape((self.coefs.b2 * x0 - y0 * self.coefs.a2).to_f32()));
        [y0.to_f32()].into()
    }
    */
}
