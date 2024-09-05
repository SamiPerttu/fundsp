//! Biquad filters with optional nonlinearities by Jatin Chowdhury.

// For more information, see:
// https://github.com/jatinchowdhury18/ComplexNonlinearities entries 4 and 5.
// For some of the biquad formulae, see the Audio EQ Cookbook:
// https://www.w3.org/TR/audio-eq-cookbook/

use super::audionode::*;
use super::math::*;
use super::setting::*;
use super::shape::*;
use super::signal::*;
use super::*;
use core::marker::PhantomData;
use numeric_array::typenum::*;

/// Biquad coefficients in normalized form.
#[derive(Copy, Clone, Debug, Default)]
pub struct BiquadCoefs<F> {
    pub a1: F,
    pub a2: F,
    pub b0: F,
    pub b1: F,
    pub b2: F,
}

impl<F: Real> BiquadCoefs<F> {
    /// Return settings for a Butterworth lowpass filter.
    /// Sample rate is in Hz.
    /// Cutoff is the -3 dB point of the filter in Hz.
    #[inline]
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

    /// Return settings for a constant-gain bandpass resonator.
    /// Sample rate and center frequency are in Hz.
    /// The overall gain of the filter is independent of bandwidth.
    #[inline]
    pub fn resonator(sample_rate: F, center: F, q: F) -> Self {
        let c = F::from_f64;
        let r: F = exp(-F::PI * center / (q * sample_rate));
        let a1: F = c(-2.0) * r * cos(F::TAU * center / sample_rate);
        let a2: F = r * r;
        let b0: F = sqrt(c(1.0) - r * r) * c(0.5);
        let b1: F = c(0.0);
        let b2: F = -b0;
        Self { a1, a2, b0, b1, b2 }
    }

    /// Return settings for a lowpass filter.
    /// Sample rate and cutoff frequency are in Hz.
    #[inline]
    pub fn lowpass(sample_rate: F, cutoff: F, q: F) -> Self {
        let c = F::from_f64;
        let omega = F::TAU * cutoff / sample_rate;
        let alpha = sin(omega) / (c(2.0) * q);
        let beta = cos(omega);
        let a0r = c(1.0) / (c(1.0) + alpha);
        let a1 = c(-2.0) * beta * a0r;
        let a2 = (c(1.0) - alpha) * a0r;
        let b1 = (c(1.0) - beta) * a0r;
        let b0 = b1 * c(0.5);
        let b2 = b0;
        Self { a1, a2, b0, b1, b2 }
    }

    /// Return settings for a highpass filter.
    /// Sample rate and cutoff frequency are in Hz.
    #[inline]
    pub fn highpass(sample_rate: F, cutoff: F, q: F) -> Self {
        let c = F::from_f64;
        let omega = F::TAU * cutoff / sample_rate;
        let alpha = sin(omega) / (c(2.0) * q);
        let beta = cos(omega);
        let a0r = c(1.0) / (c(1.0) + alpha);
        let a1 = c(-2.0) * beta * a0r;
        let a2 = (c(1.0) - alpha) * a0r;
        let b0 = (c(1.0) + beta) * c(0.5) * a0r;
        let b1 = (c(-1.0) - beta) * a0r;
        let b2 = b0;
        Self { a1, a2, b0, b1, b2 }
    }

    /// Return settings for a bell equalizer filter.
    /// Sample rate and center frequencies are in Hz.
    /// Gain is amplitude gain (`gain` > 0).
    #[inline]
    pub fn bell(sample_rate: F, center: F, q: F, gain: F) -> Self {
        let c = F::from_f64;
        let omega = F::TAU * center / sample_rate;
        let alpha = sin(omega) / (c(2.0) * q);
        let beta = cos(omega);
        let a = sqrt(gain);
        let a0r = c(1.0) / (c(1.0) + alpha / a);
        let a1 = c(-2.0) * beta * a0r;
        let a2 = (c(1.0) - alpha / a) * a0r;
        let b0 = (c(1.0) + alpha * a) * a0r;
        let b1 = a1;
        let b2 = (c(1.0) - alpha * a) * a0r;
        Self { a1, a2, b0, b1, b2 }
    }

    /// Arbitrary biquad.
    #[inline]
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
/// - Input 1 (optional): filter center (peak) frequency (Hz)
/// - Input 2 (optional): filter Q
/// - Output 0: filtered signal
#[derive(Clone)]
pub struct Resonator<F: Real, N: Size<f32>> {
    _marker: PhantomData<N>,
    biquad: Biquad<F>,
    sample_rate: F,
    center: F,
    q: F,
}

impl<F: Real, N: Size<f32>> Resonator<F, N> {
    /// Create new resonator bandpass. Initial `center` frequency is specified in Hz.
    pub fn new(center: F, q: F) -> Self {
        let mut node = Resonator {
            _marker: PhantomData,
            biquad: Biquad::new(),
            sample_rate: F::from_f64(DEFAULT_SR),
            center,
            q,
        };
        node.biquad.reset();
        node.set_center_q(center, q);
        node
    }
    pub fn set_center_q(&mut self, center: F, q: F) {
        self.biquad
            .set_coefs(BiquadCoefs::resonator(self.sample_rate, center, q));
        self.center = center;
        self.q = q;
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
        self.set_center_q(self.center, self.q);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        if N::USIZE >= 3 {
            let center: F = convert(input[1]);
            let q: F = convert(input[2]);
            if center != self.center || q != self.q {
                self.biquad
                    .set_coefs(BiquadCoefs::resonator(self.sample_rate, center, q));
                self.center = center;
                self.q = q;
            }
        }
        self.biquad.tick(&[input[0]].into())
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

/// Biquad filter common mode parameters. Filter modes use a subset of these.
#[derive(Clone, Default)]
pub struct BiquadParams<F: Real> {
    /// Sample rate in Hz.
    pub sample_rate: F,
    /// Center or cutoff in Hz.
    pub center: F,
    /// Q value, if applicable.
    pub q: F,
    /// Amplitude gain, if applicable.
    pub gain: F,
}

/// Operation of a filter mode. Retains any extra state needed
/// for efficient operation and can update filter coefficients.
/// The mode uses an optional set of inputs for continuously varying parameters.
/// - Input 0: audio
/// - Input 1: center or cutoff frequency in Hz
/// - Input 2: Q
/// - Input 3: amplitude gain
pub trait BiquadMode<F: Real>: Clone + Default + Sync + Send {
    /// Number of inputs, which includes the audio input.
    type Inputs: Size<F>;

    /// Update coefficients and state from the full set of parameters.
    fn update(&mut self, params: &BiquadParams<F>, coefs: &mut BiquadCoefs<F>);
}

/// Resonator biquad mode.
#[derive(Clone, Default)]
pub struct ResonatorBiquad<F: Real> {
    _marker: PhantomData<F>,
}

impl<F: Real> ResonatorBiquad<F> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<F: Real> BiquadMode<F> for ResonatorBiquad<F> {
    type Inputs = U3;
    #[inline]
    fn update(&mut self, params: &BiquadParams<F>, coefs: &mut BiquadCoefs<F>) {
        *coefs = BiquadCoefs::resonator(params.sample_rate, params.center, params.q);
    }
}

/// Lowpass biquad mode.
#[derive(Clone, Default)]
pub struct LowpassBiquad<F: Real> {
    _marker: PhantomData<F>,
}

impl<F: Real> LowpassBiquad<F> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<F: Real> BiquadMode<F> for LowpassBiquad<F> {
    type Inputs = U3;
    #[inline]
    fn update(&mut self, params: &BiquadParams<F>, coefs: &mut BiquadCoefs<F>) {
        *coefs = BiquadCoefs::lowpass(params.sample_rate, params.center, params.q);
    }
}

/// Highpass biquad mode.
#[derive(Clone, Default)]
pub struct HighpassBiquad<F: Real> {
    _marker: PhantomData<F>,
}

impl<F: Real> HighpassBiquad<F> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<F: Real> BiquadMode<F> for HighpassBiquad<F> {
    type Inputs = U3;
    #[inline]
    fn update(&mut self, params: &BiquadParams<F>, coefs: &mut BiquadCoefs<F>) {
        *coefs = BiquadCoefs::highpass(params.sample_rate, params.center, params.q);
    }
}

/// Bell biquad mode.
#[derive(Clone, Default)]
pub struct BellBiquad<F: Real> {
    _marker: PhantomData<F>,
}

impl<F: Real> BellBiquad<F> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<F: Real> BiquadMode<F> for BellBiquad<F> {
    type Inputs = U4;
    #[inline]
    fn update(&mut self, params: &BiquadParams<F>, coefs: &mut BiquadCoefs<F>) {
        *coefs = BiquadCoefs::bell(params.sample_rate, params.center, params.q, params.gain);
    }
}

#[derive(Clone)]
/// Biquad in transposed direct form II with nonlinear feedback.
pub struct FbBiquad<F: Real, M: BiquadMode<F>, S: Shape> {
    mode: M,
    coefs: BiquadCoefs<F>,
    params: BiquadParams<F>,
    shape: S,
    s1: F,
    s2: F,
}

impl<F: Real, M: BiquadMode<F>, S: Shape> FbBiquad<F, M, S> {
    /// Create new feedback biquad filter.
    pub fn new(mode: M, shape: S) -> Self {
        let mut filter = Self {
            mode,
            coefs: BiquadCoefs::default(),
            params: BiquadParams {
                sample_rate: F::from_f64(DEFAULT_SR),
                center: F::new(440),
                q: F::one(),
                gain: F::one(),
            },
            shape,
            s1: F::zero(),
            s2: F::zero(),
        };
        filter.set_sample_rate(DEFAULT_SR);
        filter
    }
}

impl<F: Real, M: BiquadMode<F>, S: Shape> AudioNode for FbBiquad<F, M, S> {
    const ID: u64 = 88;
    type Inputs = M::Inputs;
    type Outputs = U1;

    fn reset(&mut self) {
        self.s1 = F::zero();
        self.s2 = F::zero();
        self.shape.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.params.sample_rate = F::from_f64(sample_rate);
        self.mode.update(&self.params, &mut self.coefs);
    }

    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        if M::Inputs::USIZE == 2 {
            let center = F::from_f32(input[1]);
            if center != self.params.center {
                self.params.center = center;
                self.mode.update(&self.params, &mut self.coefs);
            }
        }
        if M::Inputs::USIZE == 3 {
            let center = F::from_f32(input[1]);
            let q = F::from_f32(input[2]);
            if squared(center - self.params.center) + squared(q - self.params.q) != F::zero() {
                self.params.center = center;
                self.params.q = q;
                self.mode.update(&self.params, &mut self.coefs);
            }
        }
        if M::Inputs::USIZE == 4 {
            let center = F::from_f32(input[1]);
            let q = F::from_f32(input[2]);
            let gain = F::from_f32(input[3]);
            if squared(center - self.params.center)
                + squared(q - self.params.q)
                + squared(gain - self.params.gain)
                != F::zero()
            {
                self.params.center = center;
                self.params.q = q;
                self.params.gain = gain;
                self.mode.update(&self.params, &mut self.coefs);
            }
        }
        // Transposed Direct Form II with nonlinear feedback is:
        //   y0 = b0 * x0 + s1
        //   s1 = s2 + b1 * x0 - a1 * shape(y0)
        //   s2 = b2 * x0 - a2 * shape(y0)
        let x0 = F::from_f32(input[0]);
        let y0 = self.coefs.b0 * x0 + self.s1;
        let fb = F::from_f32(self.shape.shape(y0.to_f32()));
        self.s1 = self.s2 + self.coefs.b1 * x0 - fb * self.coefs.a1;
        self.s2 = self.coefs.b2 * x0 - fb * self.coefs.a2;
        [y0.to_f32()].into()
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Arbitrary(0.0).route(input, self.outputs())
    }
}

#[derive(Clone)]
/// Biquad in transposed direct form II with nonlinear feedback, fixed parameters.
pub struct FixedFbBiquad<F: Real, M: BiquadMode<F>, S: Shape> {
    mode: M,
    coefs: BiquadCoefs<F>,
    params: BiquadParams<F>,
    shape: S,
    s1: F,
    s2: F,
}

impl<F: Real, M: BiquadMode<F>, S: Shape> FixedFbBiquad<F, M, S> {
    /// Create new feedback biquad filter.
    pub fn new(mode: M, shape: S) -> Self {
        let mut filter = Self {
            mode,
            coefs: BiquadCoefs::default(),
            params: BiquadParams {
                sample_rate: F::from_f64(DEFAULT_SR),
                center: F::new(440),
                q: F::one(),
                gain: F::one(),
            },
            shape,
            s1: F::zero(),
            s2: F::zero(),
        };
        filter.set_sample_rate(DEFAULT_SR);
        filter
    }

    /// Set filter `center` or cutoff frequency in Hz.
    pub fn set_center(&mut self, center: F) {
        self.params.center = center;
        self.mode.update(&self.params, &mut self.coefs);
    }

    /// Set filter Q.
    pub fn set_q(&mut self, q: F) {
        self.params.q = q;
        self.mode.update(&self.params, &mut self.coefs);
    }

    /// Set filter amplitude `gain`.
    pub fn set_gain(&mut self, gain: F) {
        self.params.gain = gain;
        self.mode.update(&self.params, &mut self.coefs);
    }

    /// Set filter `center` or cutoff frequency in Hz and Q.
    pub fn set_center_q(&mut self, center: F, q: F) {
        self.params.center = center;
        self.params.q = q;
        self.mode.update(&self.params, &mut self.coefs);
    }

    /// Set filter `center` or cutoff frequency in Hz, Q and amplitude `gain`.
    pub fn set_center_q_gain(&mut self, center: F, q: F, gain: F) {
        self.params.center = center;
        self.params.q = q;
        self.params.gain = gain;
        self.mode.update(&self.params, &mut self.coefs);
    }
}

impl<F: Real, M: BiquadMode<F>, S: Shape> AudioNode for FixedFbBiquad<F, M, S> {
    const ID: u64 = 90;
    type Inputs = U1;
    type Outputs = U1;

    fn reset(&mut self) {
        self.s1 = F::zero();
        self.s2 = F::zero();
        self.shape.reset();
    }

    fn set(&mut self, setting: Setting) {
        match setting.parameter() {
            Parameter::Center(center) => self.set_center(F::from_f32(*center)),
            Parameter::CenterQ(center, q) => {
                self.set_center_q(F::from_f32(*center), F::from_f32(*q))
            }
            Parameter::CenterQGain(center, q, gain) => {
                self.set_center_q_gain(F::from_f32(*center), F::from_f32(*q), F::from_f32(*gain))
            }
            _ => (),
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.params.sample_rate = F::from_f64(sample_rate);
        self.mode.update(&self.params, &mut self.coefs);
    }

    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let x0 = F::from_f32(input[0]);
        let y0 = self.coefs.b0 * x0 + self.s1;
        let fb = F::from_f32(self.shape.shape(y0.to_f32()));
        self.s1 = self.s2 + self.coefs.b1 * x0 - fb * self.coefs.a1;
        self.s2 = self.coefs.b2 * x0 - fb * self.coefs.a2;
        [y0.to_f32()].into()
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Arbitrary(0.0).route(input, self.outputs())
    }
}

/// Biquad in transposed direct form II with nonlinear state shaping.
#[derive(Clone)]
pub struct DirtyBiquad<F: Real, M: BiquadMode<F>, S: Shape> {
    mode: M,
    coefs: BiquadCoefs<F>,
    params: BiquadParams<F>,
    shape1: S,
    shape2: S,
    s1: F,
    s2: F,
}

impl<F: Real, M: BiquadMode<F>, S: Shape> DirtyBiquad<F, M, S> {
    /// Create new dirty biquad filter.
    pub fn new(mode: M, shape: S) -> Self {
        let shape1 = shape;
        let shape2 = shape1.clone();
        let mut filter = Self {
            mode,
            coefs: BiquadCoefs::default(),
            params: BiquadParams {
                sample_rate: F::from_f64(DEFAULT_SR),
                center: F::new(440),
                q: F::one(),
                gain: F::one(),
            },
            shape1,
            shape2,
            s1: F::zero(),
            s2: F::zero(),
        };
        filter.set_sample_rate(DEFAULT_SR);
        filter
    }
}

impl<F: Real, M: BiquadMode<F>, S: Shape> AudioNode for DirtyBiquad<F, M, S> {
    const ID: u64 = 89;
    type Inputs = M::Inputs;
    type Outputs = typenum::U1;

    fn reset(&mut self) {
        self.s1 = F::zero();
        self.s2 = F::zero();
        self.shape1.reset();
        self.shape2.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.params.sample_rate = F::from_f64(sample_rate);
        self.mode.update(&self.params, &mut self.coefs);
    }

    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        if M::Inputs::USIZE == 2 {
            let center = F::from_f32(input[1]);
            if center != self.params.center {
                self.params.center = center;
                self.mode.update(&self.params, &mut self.coefs);
            }
        }
        if M::Inputs::USIZE == 3 {
            let center = F::from_f32(input[1]);
            let q = F::from_f32(input[2]);
            if squared(center - self.params.center) + squared(q - self.params.q) != F::zero() {
                self.params.center = center;
                self.params.q = q;
                self.mode.update(&self.params, &mut self.coefs);
            }
        }
        if M::Inputs::USIZE == 4 {
            let center = F::from_f32(input[1]);
            let q = F::from_f32(input[2]);
            let gain = F::from_f32(input[3]);
            if squared(center - self.params.center)
                + squared(q - self.params.q)
                + squared(gain - self.params.gain)
                != F::zero()
            {
                self.params.center = center;
                self.params.q = q;
                self.params.gain = gain;
                self.mode.update(&self.params, &mut self.coefs);
            }
        }
        // Transposed Direct Form II with nonlinear state shaping is:
        //   y0 = b0 * x0 + s1
        //   s1 = shape(s2 + b1 * x0 - a1 * y0)
        //   s2 = shape(b2 * x0 - a2 * y0)
        let x0 = F::from_f32(input[0]);
        let y0 = self.coefs.b0 * x0 + self.s1;
        self.s1 = F::from_f32(
            self.shape1
                .shape((self.s2 + self.coefs.b1 * x0 - y0 * self.coefs.a1).to_f32()),
        );
        self.s2 = F::from_f32(
            self.shape2
                .shape((self.coefs.b2 * x0 - y0 * self.coefs.a2).to_f32()),
        );
        [y0.to_f32()].into()
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Arbitrary(0.0).route(input, self.outputs())
    }
}

/// Biquad in transposed direct form II with nonlinear state shaping, fixed parameters.
#[derive(Clone)]
pub struct FixedDirtyBiquad<F: Real, M: BiquadMode<F>, S: Shape> {
    mode: M,
    coefs: BiquadCoefs<F>,
    params: BiquadParams<F>,
    shape1: S,
    shape2: S,
    s1: F,
    s2: F,
}

impl<F: Real, M: BiquadMode<F>, S: Shape> FixedDirtyBiquad<F, M, S> {
    /// Create new dirty biquad filter.
    pub fn new(mode: M, shape: S) -> Self {
        let shape1 = shape;
        let shape2 = shape1.clone();
        let mut filter = Self {
            mode,
            coefs: BiquadCoefs::default(),
            params: BiquadParams {
                sample_rate: F::from_f64(DEFAULT_SR),
                center: F::new(440),
                q: F::one(),
                gain: F::one(),
            },
            shape1,
            shape2,
            s1: F::zero(),
            s2: F::zero(),
        };
        filter.set_sample_rate(DEFAULT_SR);
        filter
    }

    /// Set filter `center` or cutoff frequency in Hz.
    pub fn set_center(&mut self, center: F) {
        self.params.center = center;
        self.mode.update(&self.params, &mut self.coefs);
    }

    /// Set filter Q.
    pub fn set_q(&mut self, q: F) {
        self.params.q = q;
        self.mode.update(&self.params, &mut self.coefs);
    }

    /// Set filter amplitude `gain`.
    pub fn set_gain(&mut self, gain: F) {
        self.params.gain = gain;
        self.mode.update(&self.params, &mut self.coefs);
    }

    /// Set filter `center` or cutoff frequency in Hz and Q.
    pub fn set_center_q(&mut self, center: F, q: F) {
        self.params.center = center;
        self.params.q = q;
        self.mode.update(&self.params, &mut self.coefs);
    }

    /// Set filter `center` or cutoff frequency in Hz, Q and amplitude `gain`.
    pub fn set_center_q_gain(&mut self, center: F, q: F, gain: F) {
        self.params.center = center;
        self.params.q = q;
        self.params.gain = gain;
        self.mode.update(&self.params, &mut self.coefs);
    }
}

impl<F: Real, M: BiquadMode<F>, S: Shape> AudioNode for FixedDirtyBiquad<F, M, S> {
    const ID: u64 = 91;
    type Inputs = U1;
    type Outputs = U1;

    fn reset(&mut self) {
        self.s1 = F::zero();
        self.s2 = F::zero();
        self.shape1.reset();
        self.shape2.reset();
    }

    fn set(&mut self, setting: Setting) {
        match setting.parameter() {
            Parameter::Center(center) => self.set_center(F::from_f32(*center)),
            Parameter::CenterQ(center, q) => {
                self.set_center_q(F::from_f32(*center), F::from_f32(*q))
            }
            Parameter::CenterQGain(center, q, gain) => {
                self.set_center_q_gain(F::from_f32(*center), F::from_f32(*q), F::from_f32(*gain))
            }
            _ => (),
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.params.sample_rate = F::from_f64(sample_rate);
        self.mode.update(&self.params, &mut self.coefs);
    }

    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let x0 = F::from_f32(input[0]);
        let y0 = self.coefs.b0 * x0 + self.s1;
        self.s1 = F::from_f32(
            self.shape1
                .shape((self.s2 + self.coefs.b1 * x0 - y0 * self.coefs.a1).to_f32()),
        );
        self.s2 = F::from_f32(
            self.shape2
                .shape((self.coefs.b2 * x0 - y0 * self.coefs.a2).to_f32()),
        );
        [y0.to_f32()].into()
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Arbitrary(0.0).route(input, self.outputs())
    }
}
