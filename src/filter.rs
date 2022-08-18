//! Various filters.

use super::audionode::*;
use super::combinator::*;
use super::math::*;
use super::signal::*;
use super::*;
use num_complex::Complex64;
use numeric_array::typenum::*;
use numeric_array::*;

/// Complex64 with real component `x` and imaginary component zero.
fn re<T: Float>(x: T) -> Complex64 {
    Complex64::new(x.to_f64(), 0.0)
}

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
        let f: F = tan(cutoff * c(PI) / sample_rate);
        let a0r: F = c(1.0) / (c(1.0) + c(SQRT_2) * f + f * f);
        let a1: F = (c(2.0) * f * f - c(2.0)) * a0r;
        let a2: F = (c(1.0) - c(SQRT_2) * f + f * f) * a0r;
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
        let r: F = exp(c(-PI) * bandwidth / sample_rate);
        let a1: F = c(-2.0) * r * cos(c(TAU) * center / sample_rate);
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
        let z1 = Complex64::from_polar(1.0, -TAU * omega);
        let z2 = z1 * z1;
        (re(self.b0) + re(self.b1) * z1 + re(self.b2) * z2)
            / (re(1.0) + re(self.a1) * z1 + re(self.a2) * z2)
    }
}

/// 2nd order IIR filter implemented in normalized Direct Form I.
#[derive(Default)]
pub struct Biquad<T, F> {
    _marker: std::marker::PhantomData<T>,
    coefs: BiquadCoefs<F>,
    x1: F,
    x2: F,
    y1: F,
    y2: F,
    sample_rate: f64,
}

impl<T: Float, F: Real> Biquad<T, F> {
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

impl<T: Float, F: Real> AudioNode for Biquad<T, F> {
    const ID: u64 = 15;
    type Sample = T;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x1 = F::zero();
        self.x2 = F::zero();
        self.y1 = F::zero();
        self.y2 = F::zero();
        if let Some(sr) = sample_rate {
            self.sample_rate = sr;
        }
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let x0 = convert(input[0]);
        let y0 = self.coefs.b0 * x0 + self.coefs.b1 * self.x1 + self.coefs.b2 * self.x2
            - self.coefs.a1 * self.y1
            - self.coefs.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x0;
        self.y2 = self.y1;
        self.y1 = y0;
        [convert(y0)].into()

        // Transposed Direct Form II would be:
        //   y0 = b0 * x0 + s1
        //   s1 = s2 + b1 * x0 - a1 * y0
        //   s2 = b2 * x0 - a2 * y0
    }

    fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].filter(0.0, |r| {
            r * self.coefs().response(frequency / self.sample_rate)
        });
        output
    }
}

/// Butterworth lowpass filter.
/// Number of inputs is `N`, either `U1` or `U2`.
/// - Input 0: input signal
/// - Input 1 (optional): cutoff frequency (Hz)
/// - Output 0: filtered signal
pub struct ButterLowpass<T: Float, F: Real, N: Size<T>> {
    _marker: std::marker::PhantomData<N>,
    biquad: Biquad<T, F>,
    sample_rate: F,
    cutoff: F,
}

impl<T: Float, F: Real, N: Size<T>> ButterLowpass<T, F, N> {
    pub fn new(sample_rate: f64, cutoff: F) -> Self {
        let mut node = ButterLowpass {
            _marker: std::marker::PhantomData::default(),
            biquad: Biquad::new(),
            sample_rate: F::from_f64(sample_rate),
            cutoff: F::zero(),
        };
        node.biquad.reset(Some(sample_rate));
        node.set_cutoff(cutoff);
        node
    }
    pub fn set_cutoff(&mut self, cutoff: F) {
        self.biquad
            .set_coefs(BiquadCoefs::butter_lowpass(self.sample_rate, cutoff));
        self.cutoff = cutoff;
    }
}

impl<T: Float, F: Real, N: Size<T>> AudioNode for ButterLowpass<T, F, N> {
    const ID: u64 = 16;
    type Sample = T;
    type Inputs = N;
    type Outputs = typenum::U1;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.biquad.reset(sample_rate);
        if let Some(sample_rate) = sample_rate {
            self.sample_rate = convert(sample_rate);
            self.set_cutoff(self.cutoff);
        }
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        if N::USIZE > 1 {
            let cutoff: F = convert(input[1]);
            if cutoff != self.cutoff {
                self.set_cutoff(cutoff);
            }
        }
        self.biquad.tick(&[input[0]].into())
    }

    fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].filter(0.0, |r| {
            r * self
                .biquad
                .coefs()
                .response(frequency / self.sample_rate.to_f64())
        });
        output
    }
}

/// Constant-gain bandpass filter (resonator).
/// Filter gain is (nearly) independent of bandwidth.
/// Number of inputs is `N`, either `U1` or `U3` (sic).
/// - Input 0: input signal
/// - Input 1 (optional): filter center frequency (peak) (Hz)
/// - Input 2 (optional): filter bandwidth (distance) between -3 dB points (Hz)
/// - Output 0: filtered signal
pub struct Resonator<T: Float, F: Real, N: Size<T>> {
    _marker: std::marker::PhantomData<N>,
    biquad: Biquad<T, F>,
    sample_rate: F,
    center: F,
    bandwidth: F,
}

impl<T: Float, F: Real, N: Size<T>> Resonator<T, F, N> {
    pub fn new(sample_rate: f64, center: F, bandwidth: F) -> Self {
        let mut node = Resonator {
            _marker: std::marker::PhantomData::default(),
            biquad: Biquad::new(),
            sample_rate: F::from_f64(sample_rate),
            center,
            bandwidth,
        };
        node.biquad.reset(Some(sample_rate));
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

impl<T: Float, F: Real, N: Size<T>> AudioNode for Resonator<T, F, N> {
    const ID: u64 = 17;
    type Sample = T;
    type Inputs = N;
    type Outputs = typenum::U1;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.biquad.reset(sample_rate);
        if let Some(sr) = sample_rate {
            self.sample_rate = convert(sr);
            self.set_center_bandwidth(self.center, self.bandwidth);
        }
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        if N::USIZE > 2 {
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

    fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].filter(0.0, |r| {
            r * self
                .biquad
                .coefs()
                .response(frequency / self.sample_rate.to_f64())
        });
        output
    }
}

/// One-pole lowpass filter.
/// The number of inputs is `N`, either `U1` or `U2`.
/// - Input 0: input signal
/// - Input 1 (optional): cutoff frequency (Hz)
/// - Output 0: filtered signal
#[derive(Default)]
pub struct Lowpole<T: Float, F: Real, N: Size<T>> {
    _marker: std::marker::PhantomData<(T, N)>,
    value: F,
    coeff: F,
    cutoff: F,
    sample_rate: F,
}

impl<T: Float, F: Real, N: Size<T>> Lowpole<T, F, N> {
    pub fn new(sample_rate: f64, cutoff: F) -> Self {
        let mut node = Lowpole {
            _marker: std::marker::PhantomData::default(),
            value: F::zero(),
            coeff: F::zero(),
            cutoff,
            sample_rate: convert(sample_rate),
        };
        node.set_cutoff(cutoff);
        node
    }
    pub fn set_cutoff(&mut self, cutoff: F) {
        self.cutoff = cutoff;
        self.coeff = exp(F::from_f64(-TAU) * cutoff / self.sample_rate);
    }
}

impl<T: Float, F: Real, N: Size<T>> AudioNode for Lowpole<T, F, N> {
    const ID: u64 = 18;
    type Sample = T;
    type Inputs = N;
    type Outputs = typenum::U1;

    fn reset(&mut self, sample_rate: Option<f64>) {
        if let Some(sample_rate) = sample_rate {
            self.sample_rate = convert(sample_rate);
            self.set_cutoff(self.cutoff);
        }
        self.value = F::zero();
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        if N::USIZE > 1 {
            let cutoff: F = convert(input[1]);
            if cutoff != self.cutoff {
                self.set_cutoff(cutoff);
            }
        }
        let x = convert(input[0]);
        self.value = (F::one() - self.coeff) * x + self.coeff * self.value;
        [convert(self.value)].into()
    }

    fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].filter(0.0, |r| {
            let c = self.coeff.to_f64();
            let f = frequency * TAU / self.sample_rate.to_f64();
            let z1 = Complex64::from_polar(1.0, -f);
            r * ((1.0 - c) / (1.0 - c * z1))
        });
        output
    }
}

/// DC blocking filter.
/// - Input 0: input signal
/// - Output 0: zero centered signal
#[derive(Default)]
pub struct DCBlock<T: Float, F: Real> {
    _marker: std::marker::PhantomData<T>,
    x1: F,
    y1: F,
    cutoff: F,
    coeff: F,
    sample_rate: F,
}

impl<T: Float, F: Real> DCBlock<T, F> {
    pub fn new(sample_rate: f64, cutoff: F) -> Self {
        let mut node = DCBlock::<T, F> {
            cutoff,
            ..Default::default()
        };
        node.reset(Some(sample_rate));
        node
    }
}

impl<T: Float, F: Real> AudioNode for DCBlock<T, F> {
    const ID: u64 = 22;
    type Sample = T;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;

    fn reset(&mut self, sample_rate: Option<f64>) {
        if let Some(sample_rate) = sample_rate {
            self.sample_rate = convert(sample_rate);
            self.coeff = F::one() - (F::from_f64(TAU / sample_rate) * self.cutoff);
        }
        self.x1 = F::zero();
        self.y1 = F::zero();
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let x = convert(input[0]);
        let y0 = x - self.x1 + self.coeff * self.y1;
        self.x1 = x;
        self.y1 = y0;
        [convert(y0)].into()
    }

    fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].filter(0.0, |r| {
            let c = self.coeff.to_f64();
            let f = frequency * TAU / self.sample_rate.to_f64();
            let z1 = Complex64::from_polar(1.0, -f);
            r * ((1.0 - z1) / (1.0 - c * z1))
        });
        output
    }
}

/// Logistic sigmoid.
#[inline]
fn logistic<T: Real>(x: T) -> T {
    T::one() / (T::one() + exp(T::zero() - x))
}

fn halfway_coeff<F: Real>(samples: F) -> F {
    // This approximation is accurate to 0.5% when 1 <= response_samples <= 500_000.
    let r0 = log(max(F::one(), samples)) - F::from_f64(0.861624594696583);
    let r1 = logistic(r0);
    let r2 = r1 * F::from_f64(1.13228543863477) - F::from_f64(0.1322853859);
    min(F::one(), r2)
}

/// Smoothing filter with adjustable edge response time.
/// - Input 0: input signal
/// - Output 0: smoothed signal
#[derive(Default)]
pub struct Follow<T: Float, F: Real> {
    v3: F,
    v2: F,
    v1: F,
    coeff: F,
    /// Halfway response time.
    response_time: F,
    sample_rate: F,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Float, F: Real> Follow<T, F> {
    /// Create new smoothing filter.
    /// Response time is how long it takes for the follower to reach halfway to the new value.
    pub fn new(sample_rate: f64, response_time: F) -> Self {
        let mut node = Follow::<T, F> {
            response_time,
            ..Follow::default()
        };
        node.reset(Some(sample_rate));
        node
    }

    /// Response time in seconds.
    pub fn response_time(&self) -> F {
        self.response_time
    }

    /// Set response time in seconds.
    pub fn set_response_time(&mut self, response_time: F) {
        self.response_time = response_time;
        self.coeff = halfway_coeff(response_time * self.sample_rate);
    }

    /// Current response.
    pub fn value(&self) -> F {
        self.v3
    }

    /// Jump to `x` immediately.
    pub fn set_value(&mut self, x: F) {
        self.v3 = x;
        self.v2 = x;
        self.v1 = x;
    }
}

impl<T: Float, F: Real> AudioNode for Follow<T, F> {
    const ID: u64 = 24;
    type Sample = T;
    type Inputs = U1;
    type Outputs = U1;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.v3 = F::zero();
        self.v2 = F::zero();
        self.v1 = F::zero();
        if let Some(sample_rate) = sample_rate {
            self.sample_rate = F::from_f64(sample_rate);
            self.set_response_time(self.response_time);
        }
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        // Three 1-pole filters in series.
        let rcoeff = F::one() - self.coeff;
        self.v1 = rcoeff * convert(input[0]) + self.coeff * self.v1;
        self.v2 = rcoeff * self.v1 + self.coeff * self.v2;
        self.v3 = rcoeff * self.v2 + self.coeff * self.v3;
        [convert(self.v3)].into()
    }

    fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].filter(0.0, |r| {
            let c = self.coeff.to_f64();
            let f = frequency * TAU / self.sample_rate.to_f64();
            let z1 = Complex64::from_polar(1.0, -f);
            let pole = (1.0 - c) / (1.0 - c * z1);
            r * pole * pole * pole
        });
        output
    }
}

/// Smoothing filter with adjustable edge response times for attack and release.
/// - Input 0: input signal
/// - Output 0: smoothed signal
#[derive(Default)]
pub struct AFollow<T: Float, F: Real, S: ScalarOrPair<Sample = F>> {
    v3: F,
    v2: F,
    v1: F,
    acoeff: F,
    rcoeff: F,
    /// Response times.
    time: S,
    sample_rate: F,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Float, F: Real, S: ScalarOrPair<Sample = F>> AFollow<T, F, S> {
    /// Create new smoothing filter.
    /// Response time is how long it takes for the follower to reach halfway to the new value.
    pub fn new(sample_rate: f64, time: S) -> Self {
        let mut node = AFollow::<T, F, S> {
            time,
            ..AFollow::default()
        };
        node.reset(Some(sample_rate));
        node
    }

    /// Attack time in seconds.
    pub fn attack_time(&self) -> F {
        self.time.broadcast().0
    }

    /// Release time in seconds.
    pub fn release_time(&self) -> F {
        self.time.broadcast().1
    }

    /// Set attack/release time in seconds.
    pub fn set_time(&mut self, time: S) {
        self.time = time;
        self.acoeff = halfway_coeff(self.attack_time() * self.sample_rate);
        self.rcoeff = halfway_coeff(self.release_time() * self.sample_rate);
    }

    /// Current response.
    pub fn value(&self) -> F {
        self.v3
    }

    /// Jump to `x` immediately.
    pub fn set_value(&mut self, x: F) {
        self.v3 = x;
        self.v2 = x;
        self.v1 = x;
    }
}

impl<T: Float, F: Real, S: ScalarOrPair<Sample = F>> AudioNode for AFollow<T, F, S> {
    const ID: u64 = 29;
    type Sample = T;
    type Inputs = U1;
    type Outputs = U1;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.v3 = F::zero();
        self.v2 = F::zero();
        self.v1 = F::zero();
        if let Some(sample_rate) = sample_rate {
            self.sample_rate = F::from_f64(sample_rate);
            // Recalculate coefficients.
            self.set_time(self.time.clone());
        }
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        // Three 1-pole filters in series.
        let afactor = F::one() - self.acoeff;
        let rfactor = F::one() - self.rcoeff;
        let v0: F = convert(input[0]);
        self.v1 = self.time.filter_pole(v0, self.v1, afactor, rfactor);
        self.v2 = self.time.filter_pole(self.v1, self.v2, afactor, rfactor);
        self.v3 = self.time.filter_pole(self.v2, self.v3, afactor, rfactor);
        [convert(self.v3)].into()
    }

    fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        // The frequency response exists only in symmetric mode, as the asymmetric mode is nonlinear.
        if self.acoeff == self.rcoeff {
            output[0] = input[0].filter(0.0, |r| {
                let c = self.acoeff.to_f64();
                let f = frequency * TAU / self.sample_rate.to_f64();
                let z1 = Complex64::from_polar(1.0, -f);
                let pole = (1.0 - c) / (1.0 - c * z1);
                r * pole * pole * pole
            });
        } else {
            output[0] = input[0].distort(0.0);
        }
        output
    }
}

/// Pinking filter (3 dB/octave lowpass).
/// - Input 0: input signal
/// - Output 0: filtered signal
#[derive(Default)]
pub struct Pinkpass<T: Float, F: Float> {
    // Algorithm by Paul Kellett. +-0.05 dB accuracy above 9.2 Hz @ 44.1 kHz.
    b0: F,
    b1: F,
    b2: F,
    b3: F,
    b4: F,
    b5: F,
    b6: F,
    _marker: std::marker::PhantomData<T>,
    sample_rate: F,
}

impl<T: Float, F: Float> Pinkpass<T, F> {
    /// Create pinking filter.
    pub fn new(sample_rate: f64) -> Self {
        Pinkpass::<T, F> {
            sample_rate: convert(sample_rate),
            ..Pinkpass::default()
        }
    }
}

impl<T: Float, F: Float> AudioNode for Pinkpass<T, F> {
    const ID: u64 = 26;
    type Sample = T;
    type Inputs = U1;
    type Outputs = U1;

    #[inline]
    fn reset(&mut self, _sample_rate: Option<f64>) {
        self.b0 = F::zero();
        self.b1 = F::zero();
        self.b2 = F::zero();
        self.b3 = F::zero();
        self.b4 = F::zero();
        self.b5 = F::zero();
        self.b6 = F::zero();
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let x: F = convert(input[0]);
        self.b0 = F::from_f64(0.99886) * self.b0 + x * F::from_f64(0.0555179);
        self.b1 = F::from_f64(0.99332) * self.b1 + x * F::from_f64(0.0750759);
        self.b2 = F::from_f64(0.96900) * self.b2 + x * F::from_f64(0.1538520);
        self.b3 = F::from_f64(0.86650) * self.b3 + x * F::from_f64(0.3104856);
        self.b4 = F::from_f64(0.55000) * self.b4 + x * F::from_f64(0.5329522);
        self.b5 = F::from_f64(-0.7616) * self.b5 - x * F::from_f64(0.0168980);
        let out = (self.b0
            + self.b1
            + self.b2
            + self.b3
            + self.b4
            + self.b5
            + self.b6
            + x * F::from_f64(0.5362))
            * F::from_f64(0.115830421);
        self.b6 = x * F::from_f64(0.115926);
        [convert(out)].into()
    }

    fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].filter(0.0, |r| {
            let f = frequency * TAU / self.sample_rate.to_f64();
            let z1 = Complex64::from_polar(1.0, -f);
            let pole0 = 0.0555179 / (1.0 - 0.99886 * z1);
            let pole1 = 0.0750759 / (1.0 - 0.99332 * z1);
            let pole2 = 0.1538520 / (1.0 - 0.96900 * z1);
            let pole3 = 0.3104856 / (1.0 - 0.86650 * z1);
            let pole4 = 0.5329522 / (1.0 - 0.55000 * z1);
            let pole5 = -0.016898 / (1.0 + 0.7616 * z1);
            r * (pole0 + pole1 + pole2 + pole3 + pole4 + pole5 + 0.115926 * z1 + 0.5362)
                * 0.115830421
        });
        output
    }
}

/// 1st order allpass filter.
/// The number of inputs is `N`, either `U1` or `U2`.
/// - Input 0: input signal
/// - Input 1 (optional): delay in samples (delay > 0)
/// - Output 0: filtered signal
#[derive(Default)]
pub struct Allpole<T: Float, F: Float, N: Size<T>> {
    _marker: std::marker::PhantomData<(T, N)>,
    eta: F,
    x1: F,
    y1: F,
    sample_rate: F,
}

impl<T: Float, F: Float, N: Size<T>> Allpole<T, F, N> {
    pub fn new(sample_rate: f64, delay: F) -> Self {
        assert!(delay > F::zero());
        let mut node = Allpole {
            _marker: std::marker::PhantomData::default(),
            eta: F::zero(),
            x1: F::zero(),
            y1: F::zero(),
            sample_rate: convert(sample_rate),
        };
        node.set_delay(delay);
        node
    }

    /// Set delay in samples.
    #[inline]
    pub fn set_delay(&mut self, delay: F) {
        self.eta = (F::one() - delay) / (F::one() + delay);
    }
}

impl<T: Float, F: Float, N: Size<T>> AudioNode for Allpole<T, F, N> {
    const ID: u64 = 46;
    type Sample = T;
    type Inputs = N;
    type Outputs = typenum::U1;

    fn reset(&mut self, sample_rate: Option<f64>) {
        if let Some(sample_rate) = sample_rate {
            self.sample_rate = convert(sample_rate);
        }
        self.x1 = F::zero();
        self.y1 = F::zero();
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        if N::USIZE > 1 {
            self.set_delay(convert(input[1]));
        }
        let x0 = convert(input[0]);
        let y0 = self.eta * (x0 - self.y1) + self.x1;
        self.x1 = x0;
        self.y1 = y0;
        [convert(y0)].into()
    }

    fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].filter(0.0, |r| {
            let eta = self.eta.to_f64();
            let z1 = Complex64::from_polar(1.0, -frequency * TAU / self.sample_rate.to_f64());
            r * (eta + z1) / (1.0 + eta * z1)
        });
        output
    }
}

/// One-pole, one-zero highpass filter.
/// The number of inputs is `N`, either `U1` or `U2`.
/// - Input 0: input signal
/// - Input 1 (optional): cutoff frequency (Hz)
/// - Output 0: filtered signal
#[derive(Default)]
pub struct Highpole<T: Float, F: Real, N: Size<T>> {
    _marker: std::marker::PhantomData<(T, N)>,
    x1: F,
    y1: F,
    coeff: F,
    cutoff: F,
    sample_rate: F,
}

impl<T: Float, F: Real, N: Size<T>> Highpole<T, F, N> {
    pub fn new(sample_rate: f64, cutoff: F) -> Self {
        let mut node = Highpole {
            _marker: std::marker::PhantomData::default(),
            x1: F::zero(),
            y1: F::zero(),
            coeff: F::zero(),
            cutoff,
            sample_rate: convert(sample_rate),
        };
        node.set_cutoff(cutoff);
        node
    }
    pub fn set_cutoff(&mut self, cutoff: F) {
        self.cutoff = cutoff;
        self.coeff = exp(F::from_f64(-TAU) * cutoff / self.sample_rate);
    }
}

impl<T: Float, F: Real, N: Size<T>> AudioNode for Highpole<T, F, N> {
    const ID: u64 = 47;
    type Sample = T;
    type Inputs = N;
    type Outputs = typenum::U1;

    fn reset(&mut self, sample_rate: Option<f64>) {
        if let Some(sample_rate) = sample_rate {
            self.sample_rate = convert(sample_rate);
            self.set_cutoff(self.cutoff);
        }
        self.x1 = F::zero();
        self.y1 = F::zero();
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        if N::USIZE > 1 {
            let cutoff: F = convert(input[1]);
            if cutoff != self.cutoff {
                self.set_cutoff(cutoff);
            }
        }
        let x0 = convert(input[0]);
        let y0 = self.coeff * (self.y1 + x0 - self.x1);
        self.x1 = x0;
        self.y1 = y0;
        [convert(y0)].into()
    }

    fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].filter(0.0, |r| {
            let c = self.coeff.to_f64();
            let f = frequency * TAU / self.sample_rate.to_f64();
            let z1 = Complex64::from_polar(1.0, -f);
            r * (c * (1.0 - z1) / (1.0 - c * z1))
        });
        output
    }
}
