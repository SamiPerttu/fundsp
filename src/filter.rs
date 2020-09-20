use num_complex::Complex64;

use crate::prelude::*;
use crate::audiocomponent::AudioComponent;
use crate::lti::Lti;

use utd::prelude::cast;

// NEXT: add enum here for filter parameters and modify below accordingly.

#[derive(Copy, Clone)]
pub struct BiquadCoefs<F: AudioFloat = f64> {
    a1 : F,
    a2 : F,
    b0 : F,
    b1 : F,
    b2 : F,
}

impl<F: AudioFloat> BiquadCoefs<F> {

    /// Returns settings for a Butterworth lowpass filter.
    /// Cutoff is the -3 dB point of the filter in Hz.
    pub fn butter_lowpass(sample_rate: F, cutoff: F) -> BiquadCoefs<F> {
        let f: F = tan(cutoff * cast(PI) / sample_rate);
        let a0r: F = cast(1.0) / (cast(1.0) + cast(SQRT_2) * f + f * f);
        let a1: F = (cast(2.0) * f * f - cast(2.0)) * a0r;
        let a2: F = (cast(1.0) - cast(SQRT_2) * f + f * f) * a0r;
        let b0: F = f * f * a0r;
        let b1: F = cast(2.0) * b0;
        let b2: F = b0;
        BiquadCoefs::<F> { a1, a2, b0, b1, b2 }
    }

    /// Returns settings for a constant-gain bandpass resonator.
    /// The center frequency is given in Hz.
    /// Bandwidth is the difference in Hz between -3 dB points of the filter response.
    /// The overall gain of the filter is independent of bandwidth.
    pub fn resonator(sample_rate: F, center: F, bandwidth: F) -> BiquadCoefs<F> {
        let r: F = exp(cast(PI) * bandwidth / sample_rate);
        let a1: F = -r * cast(2.0) * cos(cast(TAU) * center / sample_rate);
        let a2: F = r * r;
        let b0: F = sqrt(cast(1.0) - r * r) * cast(0.5);
        let b1: F = cast(0.0);
        let b2: F = -b0;
        BiquadCoefs::<F> { a1, a2, b0, b1, b2 }
    }
}

/// 2nd order IIR filter implemented in normalized Direct Form I.
#[derive(Copy, Clone, Default)]
pub struct Biquad<F: AudioFloat = f64> {
    a1 : F,
    a2 : F,
    b0 : F,
    b1 : F,
    b2 : F,
    x1 : F,
    x2 : F,
    y1 : F,
    y2 : F,
}

impl<F: AudioFloat> Biquad<F> {
    pub fn new() -> Self { Default::default() }
    pub fn set_coefs(&mut self, coefs: BiquadCoefs<F>) {
        self.a1 = coefs.a1;
        self.a2 = coefs.a2;
        self.b0 = coefs.b0;
        self.b1 = coefs.b1;
        self.b2 = coefs.b2;
    }
}

impl<F: AudioFloat> Lti for Biquad<F> {
    fn response(&self, omega : f64) -> Complex64
    {
        let e1 = Complex64::from_polar(1.0, -TAU * omega);
        let e2 = Complex64::from_polar(1.0, -2.0 * TAU * omega);
        (re(self.b0) + re(self.b1) * e1 + re(self.b2) * e2) / (re(1.0) + re(self.a1) * e1 + re(self.a2) * e2)
    }
}

impl<F: AudioFloat> AudioComponent<1, 1> for Biquad<F> {

    fn reset(&mut self, sample_rate : Option<f64>)
    {
        self.x1 = F::zero();
        self.x2 = F::zero();
        self.y1 = F::zero();
        self.y2 = F::zero();
    }

    fn tick(&mut self, input : [F32; 1]) -> [F32; 1]
    {
        // Best practices: use unto() to convert between buffer and processing types.
        let x0 = cast(input[0]);
        let y0 = self.b0 * x0 + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x0;
        self.y2 = self.y1;
        self.y1 = y0;
        [cast(y0)]
        // Transposed Direct Form II would be:
        //   y0 = b0 * x0 + s1
        //   s1 = s2 + b1 * x0 - a1 * y0
        //   s2 = b2 * x0 - a2 * y0
    }
}

/// Butterworth lowpass filter.
/// Input 0: input signal
/// Input 1: cutoff frequency (Hz)
/// Output 0: filtered signal
#[derive(Copy, Clone)]
pub struct ButterLowpass<F: AudioFloat = f64> {
    biquad: Biquad<F>,
    sample_rate: F,
    cutoff: F,
}

impl<F: AudioFloat> ButterLowpass<F> {
    pub fn new(sample_rate: F) -> ButterLowpass<F> {
        ButterLowpass::<F> { biquad: Biquad::<F>::new(), sample_rate, cutoff: F::zero() }
    }
}

impl<F: AudioFloat> AudioComponent<2, 1> for ButterLowpass<F> {

    fn reset(&mut self, sample_rate: Option<f64>)
    {
        self.biquad.reset(sample_rate);
    }

    fn tick(&mut self, input : [F32; 2]) -> [F32; 1]
    {
        let cutoff: F = cast(input[1]);
        if cutoff != self.cutoff {
            self.biquad.set_coefs(BiquadCoefs::<F>::butter_lowpass(self.sample_rate, cutoff));
            self.cutoff = cutoff;
        }
        self.biquad.tick([input[0]])
    }
}

/// Constant-gain bandpass filter (resonator).
/// Filter gain is (nearly) independent of bandwidth.
/// Input 0: input signal
/// Input 1: filter center frequency (peak) (Hz)
/// Input 2: filter bandwidth (distance) between -3 dB points (Hz)
/// Output 0: filtered signal
#[derive(Copy, Clone)]
pub struct Resonator<F: AudioFloat = f64> {
    biquad: Biquad<F>,
    sample_rate: F,
    center: F,
    bandwidth: F,
}

impl<F: AudioFloat> Resonator<F> {
    pub fn new(sample_rate: F) -> Resonator<F> {
        Resonator::<F> { biquad: Biquad::<F>::new(), sample_rate, center: F::zero(), bandwidth: F::zero() }
    }
}

impl<F: AudioFloat> AudioComponent<3, 1> for Resonator<F> {

    fn reset(&mut self, sample_rate: Option<f64>)
    {
        self.biquad.reset(sample_rate);
    }

    fn tick(&mut self, input : [F32; 3]) -> [F32; 1]
    {
        let center: F = cast(input[1]);
        let bandwidth: F = cast(input[2]);
        if center != self.center || bandwidth != self.bandwidth {
            self.biquad.set_coefs(BiquadCoefs::<F>::resonator(self.sample_rate, center, bandwidth));
            self.center = center;
            self.bandwidth = bandwidth;
        }
        self.biquad.tick([input[0]])
    }
}

// TODO. These should be AudioUnits...

pub fn resonator_sr(sample_rate: f64) -> Resonator<f64> { Resonator::<f64>::new(sample_rate) }
pub fn resonator32_sr(sample_rate: f32) -> Resonator<f32> { Resonator::<f32>::new(sample_rate) }
pub fn resonator64_sr(sample_rate: f64) -> Resonator<f64> { Resonator::<f64>::new(sample_rate) }

pub fn lowpass_sr(sample_rate: f64) -> ButterLowpass<f64> { ButterLowpass::<f64>::new(sample_rate) }
pub fn lowpass32_sr(sample_rate: f32) -> ButterLowpass<f32> { ButterLowpass::<f32>::new(sample_rate) }
pub fn lowpass64_sr(sample_rate: f64) -> ButterLowpass<f64> { ButterLowpass::<f64>::new(sample_rate) }

pub fn resonator() -> Resonator<f64> { resonator_sr(DEFAULT_SR) }
pub fn resonator32() -> Resonator<f32> { resonator32_sr(DEFAULT_SR as f32) }
pub fn resonator64() -> Resonator<f64> { resonator64_sr(DEFAULT_SR) }

pub fn lowpass() -> ButterLowpass<f64> { lowpass_sr(DEFAULT_SR) }
pub fn lowpass32() -> ButterLowpass<f32> { lowpass32_sr(DEFAULT_SR as f32) }
pub fn lowpass64() -> ButterLowpass<f64> { lowpass64_sr(DEFAULT_SR) }

