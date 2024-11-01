//! Andrew Simper's state variable filters.
//!
//! See <https://cytomic.com/files/dsp/SvfLinearTrapOptimised2.pdf>.

use super::audionode::*;
use super::buffer::*;
use super::math::*;
use super::setting::*;
use super::signal::*;
use super::*;
use core::marker::PhantomData;
use num_complex::Complex64;
use numeric_array::typenum::*;

/// State variable filter coefficients, generic formulation.
#[derive(Clone, Default)]
pub struct SvfCoefs<F: Real> {
    pub a1: F,
    pub a2: F,
    pub a3: F,
    pub m0: F,
    pub m1: F,
    pub m2: F,
}

impl<F: Real> SvfCoefs<F> {
    /// Calculate coefficients for a lowpass filter.
    pub fn lowpass(sample_rate: F, cutoff: F, q: F) -> Self {
        let g = tan(F::from_f64(f64::PI) * cutoff / sample_rate);
        let k = F::one() / q;
        let a1 = F::one() / (F::one() + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        let m0 = F::zero();
        let m1 = F::zero();
        let m2 = F::one();

        SvfCoefs {
            a1,
            a2,
            a3,
            m0,
            m1,
            m2,
        }
    }

    /// Calculate coefficients for a highpass filter.
    pub fn highpass(sample_rate: F, cutoff: F, q: F) -> Self {
        let g = tan(F::from_f64(f64::PI) * cutoff / sample_rate);
        let k = F::one() / q;
        let a1 = F::one() / (F::one() + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        let m0 = F::one();
        let m1 = -k;
        let m2 = -F::one();

        SvfCoefs {
            a1,
            a2,
            a3,
            m0,
            m1,
            m2,
        }
    }

    /// Calculate coefficients for a bandpass filter.
    pub fn bandpass(sample_rate: F, cutoff: F, q: F) -> Self {
        let g = tan(F::from_f64(f64::PI) * cutoff / sample_rate);
        let k = F::one() / q;
        let a1 = F::one() / (F::one() + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        let m0 = F::zero();
        let m1 = F::one();
        let m2 = F::zero();

        SvfCoefs {
            a1,
            a2,
            a3,
            m0,
            m1,
            m2,
        }
    }

    /// Calculate coefficients for a notch filter.
    pub fn notch(sample_rate: F, cutoff: F, q: F) -> Self {
        let g = tan(F::from_f64(f64::PI) * cutoff / sample_rate);
        let k = F::one() / q;
        let a1 = F::one() / (F::one() + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        let m0 = F::one();
        let m1 = -k;
        let m2 = F::zero();

        SvfCoefs {
            a1,
            a2,
            a3,
            m0,
            m1,
            m2,
        }
    }

    /// Calculate coefficients for a peak filter.
    pub fn peak(sample_rate: F, cutoff: F, q: F) -> Self {
        let g = tan(F::from_f64(f64::PI) * cutoff / sample_rate);
        let k = F::one() / q;
        let a1 = F::one() / (F::one() + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        let m0 = F::one();
        let m1 = -k;
        let m2 = F::new(-2);

        SvfCoefs {
            a1,
            a2,
            a3,
            m0,
            m1,
            m2,
        }
    }

    /// Calculate coefficients for an allpass filter.
    pub fn allpass(sample_rate: F, cutoff: F, q: F) -> Self {
        let g = tan(F::from_f64(f64::PI) * cutoff / sample_rate);
        let k = F::one() / q;
        let a1 = F::one() / (F::one() + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        let m0 = F::one();
        let m1 = F::new(-2) * k;
        let m2 = F::zero();

        SvfCoefs {
            a1,
            a2,
            a3,
            m0,
            m1,
            m2,
        }
    }

    /// Calculate coefficients for a bell filter.
    /// Gain is amplitude gain (`gain` > 0).
    pub fn bell(sample_rate: F, cutoff: F, q: F, gain: F) -> Self {
        let a = sqrt(gain);
        let g = tan(F::from_f64(f64::PI) * cutoff / sample_rate);
        let k = F::one() / (q * a);
        let a1 = F::one() / (F::one() + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        let m0 = F::one();
        let m1 = k * (a * a - F::one());
        let m2 = F::zero();

        SvfCoefs {
            a1,
            a2,
            a3,
            m0,
            m1,
            m2,
        }
    }

    /// Calculate coefficients for a low shelf filter.
    /// Gain is amplitude gain (`gain` > 0).
    pub fn lowshelf(sample_rate: F, cutoff: F, q: F, gain: F) -> Self {
        let a = sqrt(gain);
        let g = tan(F::from_f64(f64::PI) * cutoff / sample_rate) / sqrt(a);
        let k = F::one() / q;
        let a1 = F::one() / (F::one() + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        let m0 = F::one();
        let m1 = k * (a - F::one());
        let m2 = a * a - F::one();

        SvfCoefs {
            a1,
            a2,
            a3,
            m0,
            m1,
            m2,
        }
    }

    /// Calculate coefficients for a high shelf filter.
    /// Gain is amplitude gain (`gain` > 0).
    pub fn highshelf(sample_rate: F, cutoff: F, q: F, gain: F) -> Self {
        let a = sqrt(gain);
        let g = tan(F::from_f64(f64::PI) * cutoff / sample_rate) * sqrt(a);
        let k = F::one() / q;
        let a1 = F::one() / (F::one() + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        let m0 = a * a;
        let m1 = k * (F::one() - a) * a;
        let m2 = F::one() - a * a;

        SvfCoefs {
            a1,
            a2,
            a3,
            m0,
            m1,
            m2,
        }
    }
}

/// Operation of a filter mode. Retains any extra state needed
/// for efficient operation and can update filter coefficients.
/// The mode uses an optional set of inputs for continuously varying parameters.
/// The definition of each input is mode dependent.
pub trait SvfMode<F: Real>: Clone + Default + Sync + Send {
    /// Number of inputs, which includes the audio input. Equal to the number of continuous parameters plus one.
    type Inputs: Size<F>;

    /// Update coefficients and state from the full set of parameters.
    fn update(&mut self, params: &SvfParams<F>, coefs: &mut SvfCoefs<F>);

    /// Update coefficients and state from sample rate and/or cutoff.
    /// Other parameters are untouched.
    fn update_frequency(&mut self, params: &SvfParams<F>, coefs: &mut SvfCoefs<F>) {
        // Do a bulk update by default.
        self.update(params, coefs);
    }

    /// Update coefficients and state from Q.
    /// Other parameters are untouched.
    fn update_q(&mut self, params: &SvfParams<F>, coefs: &mut SvfCoefs<F>) {
        // Do a bulk update by default.
        self.update(params, coefs);
    }

    /// Update coefficients and state from gain. Gain is given as amplitude.
    /// Other parameters are untouched.
    /// Only equalizing modes (bell and shelf) support gain. It is ignored by default.
    #[allow(unused_variables)]
    fn update_gain(&mut self, params: &SvfParams<F>, coefs: &mut SvfCoefs<F>) {}

    /// Update coefficients, parameters and state from input.
    #[allow(unused_variables)]
    fn update_inputs(
        &mut self,
        input: &Frame<f32, Self::Inputs>,
        params: &mut SvfParams<F>,
        coefs: &mut SvfCoefs<F>,
    );

    /// Response function.
    fn response(&self, params: &SvfParams<F>, frequency: f64) -> Complex64;
}

#[derive(Clone, Default)]
pub struct SvfParams<F: Real> {
    pub sample_rate: F,
    pub cutoff: F,
    pub q: F,
    pub gain: F,
}

/// Lowpass filter with cutoff and Q inputs.
/// - Input 0: audio
/// - Input 1: cutoff in Hz
/// - Input 2: Q
/// - Output 0: audio
#[derive(Clone, Default)]
pub struct LowpassMode<F: Real> {
    _marker: PhantomData<F>,
}

impl<F: Real> LowpassMode<F> {
    pub fn new() -> Self {
        LowpassMode::default()
    }
}

impl<F: Real> SvfMode<F> for LowpassMode<F> {
    type Inputs = U3;

    #[inline]
    fn update(&mut self, params: &SvfParams<F>, coefs: &mut SvfCoefs<F>) {
        *coefs = SvfCoefs::lowpass(params.sample_rate, params.cutoff, params.q);
    }
    #[inline]
    fn update_inputs(
        &mut self,
        input: &Frame<f32, Self::Inputs>,
        params: &mut SvfParams<F>,
        coefs: &mut SvfCoefs<F>,
    ) {
        let cutoff = F::from_f32(input[1]);
        let q = F::from_f32(input[2]);
        if cutoff != params.cutoff || q != params.q {
            //if squared(cutoff - params.cutoff) + squared(q - params.q) > F::zero() {
            params.cutoff = cutoff;
            params.q = q;
            self.update(params, coefs);
        }
    }

    fn response(&self, params: &SvfParams<F>, frequency: f64) -> Complex64 {
        let g = tan(f64::PI * params.cutoff.to_f64() / params.sample_rate.to_f64());
        let k = 1.0 / params.q.to_f64();
        let f = frequency * f64::TAU / params.sample_rate.to_f64();
        let z = Complex64::from_polar(1.0, f);
        (g * g * (1.0 + z) * (1.0 + z))
            / ((z - 1.0) * (z - 1.0) + g * g * (1.0 + z) * (1.0 + z) + g * k * (z * z - 1.0))
    }
}

/// Highpass filter with cutoff and Q inputs.
/// - Input 0: audio
/// - Input 1: cutoff in Hz
/// - Input 2: Q
/// - Output 0: audio
#[derive(Clone, Default)]
pub struct HighpassMode<F: Real> {
    _marker: PhantomData<F>,
}

impl<F: Real> HighpassMode<F> {
    pub fn new() -> Self {
        HighpassMode::default()
    }
}

impl<F: Real> SvfMode<F> for HighpassMode<F> {
    type Inputs = U3;

    #[inline]
    fn update(&mut self, params: &SvfParams<F>, coefs: &mut SvfCoefs<F>) {
        *coefs = SvfCoefs::highpass(params.sample_rate, params.cutoff, params.q);
    }
    #[inline]
    fn update_inputs(
        &mut self,
        input: &Frame<f32, Self::Inputs>,
        params: &mut SvfParams<F>,
        coefs: &mut SvfCoefs<F>,
    ) {
        let cutoff = F::from_f32(input[1]);
        let q = F::from_f32(input[2]);
        if cutoff != params.cutoff || q != params.q {
            params.cutoff = cutoff;
            params.q = q;
            self.update(params, coefs);
        }
    }

    fn response(&self, params: &SvfParams<F>, frequency: f64) -> Complex64 {
        let g = tan(f64::PI * params.cutoff.to_f64() / params.sample_rate.to_f64());
        let k = 1.0 / params.q.to_f64();
        let f = frequency * f64::TAU / params.sample_rate.to_f64();
        let z = Complex64::from_polar(1.0, f);
        ((z - 1.0) * (z - 1.0))
            / ((z - 1.0) * (z - 1.0) + g * g * (1.0 + z) * (1.0 + z) + g * k * (z * z - 1.0))
    }
}

/// Bandpass filter with center and Q inputs.
/// - Input 0: audio
/// - Input 1: center in Hz
/// - Input 2: Q
/// - Output 0: audio
#[derive(Clone, Default)]
pub struct BandpassMode<F: Real> {
    _marker: PhantomData<F>,
}

impl<F: Real> BandpassMode<F> {
    pub fn new() -> Self {
        BandpassMode::default()
    }
}

impl<F: Real> SvfMode<F> for BandpassMode<F> {
    type Inputs = U3;

    #[inline]
    fn update(&mut self, params: &SvfParams<F>, coefs: &mut SvfCoefs<F>) {
        *coefs = SvfCoefs::bandpass(params.sample_rate, params.cutoff, params.q);
    }
    #[inline]
    fn update_inputs(
        &mut self,
        input: &Frame<f32, Self::Inputs>,
        params: &mut SvfParams<F>,
        coefs: &mut SvfCoefs<F>,
    ) {
        let cutoff = F::from_f32(input[1]);
        let q = F::from_f32(input[2]);
        if cutoff != params.cutoff || q != params.q {
            params.cutoff = cutoff;
            params.q = q;
            self.update(params, coefs);
        }
    }

    fn response(&self, params: &SvfParams<F>, frequency: f64) -> Complex64 {
        let g = tan(f64::PI * params.cutoff.to_f64() / params.sample_rate.to_f64());
        let k = 1.0 / params.q.to_f64();
        let f = frequency * f64::TAU / params.sample_rate.to_f64();
        let z = Complex64::from_polar(1.0, f);
        (g * (z * z - 1.0))
            / ((z - 1.0) * (z - 1.0) + g * g * (1.0 + z) * (1.0 + z) + g * k * (z * z - 1.0))
    }
}

/// Notch filter with center and Q inputs.
/// - Input 0: audio
/// - Input 1: center in Hz
/// - Input 2: Q
/// - Output 0: audio
#[derive(Clone, Default)]
pub struct NotchMode<F: Real> {
    _marker: PhantomData<F>,
}

impl<F: Real> NotchMode<F> {
    pub fn new() -> Self {
        NotchMode::default()
    }
}

impl<F: Real> SvfMode<F> for NotchMode<F> {
    type Inputs = U3;

    #[inline]
    fn update(&mut self, params: &SvfParams<F>, coefs: &mut SvfCoefs<F>) {
        *coefs = SvfCoefs::notch(params.sample_rate, params.cutoff, params.q);
    }
    #[inline]
    fn update_inputs(
        &mut self,
        input: &Frame<f32, Self::Inputs>,
        params: &mut SvfParams<F>,
        coefs: &mut SvfCoefs<F>,
    ) {
        let cutoff = F::from_f32(input[1]);
        let q = F::from_f32(input[2]);
        if cutoff != params.cutoff || q != params.q {
            params.cutoff = cutoff;
            params.q = q;
            self.update(params, coefs);
        }
    }

    fn response(&self, params: &SvfParams<F>, frequency: f64) -> Complex64 {
        let g = tan(f64::PI * params.cutoff.to_f64() / params.sample_rate.to_f64());
        let k = 1.0 / params.q.to_f64();
        let f = frequency * f64::TAU / params.sample_rate.to_f64();
        let z = Complex64::from_polar(1.0, f);
        ((z - 1.0) * (z - 1.0) + g * g * (1.0 + z) * (1.0 + z))
            / ((z - 1.0) * (z - 1.0) + g * g * (1.0 + z) * (1.0 + z) + g * k * (z * z - 1.0))
    }
}

/// Peak filter with center and Q inputs.
/// - Input 0: audio
/// - Input 1: center in Hz
/// - Input 2: Q
/// - Output 0: audio
#[derive(Clone, Default)]
pub struct PeakMode<F: Real> {
    _marker: PhantomData<F>,
}

impl<F: Real> PeakMode<F> {
    pub fn new() -> Self {
        PeakMode::default()
    }
}

impl<F: Real> SvfMode<F> for PeakMode<F> {
    type Inputs = U3;

    #[inline]
    fn update(&mut self, params: &SvfParams<F>, coefs: &mut SvfCoefs<F>) {
        *coefs = SvfCoefs::peak(params.sample_rate, params.cutoff, params.q);
    }
    #[inline]
    fn update_inputs(
        &mut self,
        input: &Frame<f32, Self::Inputs>,
        params: &mut SvfParams<F>,
        coefs: &mut SvfCoefs<F>,
    ) {
        let cutoff = F::from_f32(input[1]);
        let q = F::from_f32(input[2]);
        if cutoff != params.cutoff || q != params.q {
            params.cutoff = cutoff;
            params.q = q;
            self.update(params, coefs);
        }
    }

    fn response(&self, params: &SvfParams<F>, frequency: f64) -> Complex64 {
        let g = tan(f64::PI * params.cutoff.to_f64() / params.sample_rate.to_f64());
        let k = 1.0 / params.q.to_f64();
        let f = frequency * f64::TAU / params.sample_rate.to_f64();
        let z = Complex64::from_polar(1.0, f);
        // Note: this is the negation of the transfer function reported in the derivation.
        -((1.0 + g + (g - 1.0) * z) * (-1.0 + g + z + g * z))
            / ((z - 1.0) * (z - 1.0) + g * g * (1.0 + z) * (1.0 + z) + g * k * (z * z - 1.0))
    }
}

/// Allpass filter with center and Q inputs.
/// - Input 0: audio
/// - Input 1: center in Hz
/// - Input 2: Q
/// - Output 0: audio
#[derive(Clone, Default)]
pub struct AllpassMode<F: Real> {
    _marker: PhantomData<F>,
}

impl<F: Real> AllpassMode<F> {
    pub fn new() -> Self {
        AllpassMode::default()
    }
}

impl<F: Real> SvfMode<F> for AllpassMode<F> {
    type Inputs = U3;

    #[inline]
    fn update(&mut self, params: &SvfParams<F>, coefs: &mut SvfCoefs<F>) {
        *coefs = SvfCoefs::allpass(params.sample_rate, params.cutoff, params.q);
    }
    #[inline]
    fn update_inputs(
        &mut self,
        input: &Frame<f32, Self::Inputs>,
        params: &mut SvfParams<F>,
        coefs: &mut SvfCoefs<F>,
    ) {
        let cutoff = F::from_f32(input[1]);
        let q = F::from_f32(input[2]);
        if cutoff != params.cutoff || q != params.q {
            params.cutoff = cutoff;
            params.q = q;
            self.update(params, coefs);
        }
    }

    fn response(&self, params: &SvfParams<F>, frequency: f64) -> Complex64 {
        let g = tan(f64::PI * params.cutoff.to_f64() / params.sample_rate.to_f64());
        let k = 1.0 / params.q.to_f64();
        let f = frequency * f64::TAU / params.sample_rate.to_f64();
        let z = Complex64::from_polar(1.0, f);
        ((z - 1.0) * (z - 1.0) + g * g * (1.0 + z) * (1.0 + z) + g * (k - k * z * z))
            / ((z - 1.0) * (z - 1.0) + g * g * (1.0 + z) * (1.0 + z) + g * k * (z * z - 1.0))
    }
}

/// Bell filter with center, Q and gain inputs.
/// - Input 0: audio
/// - Input 1: center in Hz
/// - Input 2: Q
/// - Input 3: amplitude gain (gain > 0)
/// - Output 0: audio
#[derive(Clone, Default)]
pub struct BellMode<F: Real> {
    _marker: PhantomData<F>,
}

impl<F: Real> BellMode<F> {
    pub fn new() -> Self {
        BellMode::default()
    }
}

impl<F: Real> SvfMode<F> for BellMode<F> {
    type Inputs = U4;

    #[inline]
    fn update(&mut self, params: &SvfParams<F>, coefs: &mut SvfCoefs<F>) {
        *coefs = SvfCoefs::bell(params.sample_rate, params.cutoff, params.q, params.gain);
    }
    #[inline]
    fn update_inputs(
        &mut self,
        input: &Frame<f32, Self::Inputs>,
        params: &mut SvfParams<F>,
        coefs: &mut SvfCoefs<F>,
    ) {
        let cutoff = F::from_f32(input[1]);
        let q = F::from_f32(input[2]);
        let gain = F::from_f32(input[3]);
        if cutoff != params.cutoff || q != params.q || gain != params.gain {
            params.cutoff = cutoff;
            params.q = q;
            params.gain = gain;
            self.update(params, coefs);
        }
    }

    fn response(&self, params: &SvfParams<F>, frequency: f64) -> Complex64 {
        let a = sqrt(params.gain.to_f64());
        let g = tan(f64::PI * params.cutoff.to_f64() / params.sample_rate.to_f64());
        let k = 1.0 / params.q.to_f64();
        let z = Complex64::from_polar(1.0, frequency * f64::TAU / params.sample_rate.to_f64());
        (g * k * (z * z - 1.0)
            + a * (g * (1.0 + z) * ((a * a - 1.0) * k / a * (z - 1.0))
                + ((z - 1.0) * (z - 1.0) + g * g * (1.0 + z) * (1.0 + z))))
            / (g * k * (z * z - 1.0) + a * ((z - 1.0) * (z - 1.0) + g * g * (z + 1.0) * (z + 1.0)))
    }
}

/// Low shelf filter with center, Q and gain inputs.
/// - Input 0: audio
/// - Input 1: center in Hz
/// - Input 2: Q
/// - Input 3: amplitude gain (gain > 0)
/// - Output 0: audio
#[derive(Clone, Default)]
pub struct LowshelfMode<F: Real> {
    _marker: PhantomData<F>,
}

impl<F: Real> LowshelfMode<F> {
    pub fn new() -> Self {
        LowshelfMode::default()
    }
}

impl<F: Real> SvfMode<F> for LowshelfMode<F> {
    type Inputs = U4;

    #[inline]
    fn update(&mut self, params: &SvfParams<F>, coefs: &mut SvfCoefs<F>) {
        *coefs = SvfCoefs::lowshelf(params.sample_rate, params.cutoff, params.q, params.gain);
    }
    #[inline]
    fn update_inputs(
        &mut self,
        input: &Frame<f32, Self::Inputs>,
        params: &mut SvfParams<F>,
        coefs: &mut SvfCoefs<F>,
    ) {
        let cutoff = F::from_f32(input[1]);
        let q = F::from_f32(input[2]);
        let gain = F::from_f32(input[3]);
        if cutoff != params.cutoff || q != params.q || gain != params.gain {
            params.cutoff = cutoff;
            params.q = q;
            params.gain = gain;
            self.update(params, coefs);
        }
    }

    fn response(&self, params: &SvfParams<F>, frequency: f64) -> Complex64 {
        let a = sqrt(params.gain.to_f64());
        let g = tan(f64::PI * params.cutoff.to_f64() / params.sample_rate.to_f64());
        let k = 1.0 / params.q.to_f64();
        let z = Complex64::from_polar(1.0, frequency * f64::TAU / params.sample_rate.to_f64());
        let sqrt_a = sqrt(a);
        (a * (z - 1.0) * (z - 1.0)
            + g * g * a * a * (z + 1.0) * (z + 1.0)
            + sqrt_a * g * a * k * (z * z - 1.0))
            / (a * (z - 1.0) * (z - 1.0)
                + g * g * (1.0 + z) * (1.0 + z)
                + sqrt_a * g * k * (z * z - 1.0))
    }
}

/// High shelf filter with center, Q and gain inputs.
/// - Input 0: audio
/// - Input 1: center in Hz
/// - Input 2: Q
/// - Input 3: amplitude gain (gain > 0)
/// - Output 0: audio
#[derive(Clone, Default)]
pub struct HighshelfMode<F: Real> {
    _marker: PhantomData<F>,
}

impl<F: Real> HighshelfMode<F> {
    pub fn new() -> Self {
        HighshelfMode::default()
    }
}

impl<F: Real> SvfMode<F> for HighshelfMode<F> {
    type Inputs = U4;

    #[inline]
    fn update(&mut self, params: &SvfParams<F>, coefs: &mut SvfCoefs<F>) {
        *coefs = SvfCoefs::highshelf(params.sample_rate, params.cutoff, params.q, params.gain);
    }
    #[inline]
    fn update_inputs(
        &mut self,
        input: &Frame<f32, Self::Inputs>,
        params: &mut SvfParams<F>,
        coefs: &mut SvfCoefs<F>,
    ) {
        let cutoff = F::from_f32(input[1]);
        let q = F::from_f32(input[2]);
        let gain = F::from_f32(input[3]);
        if cutoff != params.cutoff || q != params.q || gain != params.gain {
            params.cutoff = cutoff;
            params.q = q;
            params.gain = gain;
            self.update(params, coefs);
        }
    }

    fn response(&self, params: &SvfParams<F>, frequency: f64) -> Complex64 {
        let a = sqrt(params.gain.to_f64());
        let g = tan(f64::PI * params.cutoff.to_f64() / params.sample_rate.to_f64());
        let k = 1.0 / params.q.to_f64();
        let z = Complex64::from_polar(1.0, frequency * f64::TAU / params.sample_rate.to_f64());
        let sqrt_a = sqrt(a);
        (sqrt_a
            * g
            * (1.0 + z)
            * (-(a - 1.0) * a * k * (z - 1.0) + sqrt_a * g * (1.0 - a * a) * (1.0 + z))
            + a * a
                * ((z - 1.0) * (z - 1.0)
                    + a * g * g * (1.0 + z) * (1.0 + z)
                    + sqrt_a * g * k * (z * z - 1.0)))
            / ((z - 1.0) * (z - 1.0)
                + a * g * g * (1.0 + z) * (1.0 + z)
                + sqrt_a * g * k * (z * z - 1.0))
    }
}

/// Simper SVF.
/// - Inputs: see descriptions of the filter modes.
/// - Output 0: filtered audio
#[derive(Default, Clone)]
pub struct Svf<F, M>
where
    F: Real,
    M: SvfMode<F>,
    M::Inputs: Size<f32>,
{
    mode: M,
    params: SvfParams<F>,
    coefs: SvfCoefs<F>,
    ic1eq: F,
    ic2eq: F,
    period: u32,
    counter: u32,
}

impl<F, M> Svf<F, M>
where
    F: Real,
    M: SvfMode<F>,
    M::Inputs: Size<f32>,
{
    pub fn new(mode: M, params: &SvfParams<F>) -> Self {
        let params = params.clone();
        let mut coefs = SvfCoefs::default();
        let mut mode = mode;
        mode.update(&params, &mut coefs);
        Svf {
            mode,
            params,
            coefs,
            ic1eq: F::zero(),
            ic2eq: F::zero(),
            period: 1,
            counter: 0,
        }
    }

    /// Sample rate in Hz.
    #[inline]
    pub fn sample_rate(&self) -> F {
        self.params.sample_rate
    }
    /// Filter cutoff in Hz. Synonymous with `center`.
    #[inline]
    pub fn cutoff(&self) -> F {
        self.params.cutoff
    }
    /// Filter center in Hz. Synonymous with `cutoff`.
    #[inline]
    pub fn center(&self) -> F {
        self.params.cutoff
    }
    /// Filter Q.
    #[inline]
    pub fn q(&self) -> F {
        self.params.q
    }
    /// Filter gain. Only equalization modes support gain; others ignore it.
    #[inline]
    pub fn gain(&self) -> F {
        self.params.gain
    }
}

impl<F, M> AudioNode for Svf<F, M>
where
    F: Real,
    M: SvfMode<F>,
    M::Inputs: Size<f32>,
{
    const ID: u64 = 36;
    type Inputs = M::Inputs;
    type Outputs = U1;

    fn reset(&mut self) {
        self.ic1eq = F::zero();
        self.ic2eq = F::zero();
        self.counter = 0;
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.params.sample_rate = convert(sample_rate);
        self.mode.update_frequency(&self.params, &mut self.coefs);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        self.counter += 1;
        if self.counter >= self.period {
            self.counter = 0;
            // Update parameters from input.
            self.mode
                .update_inputs(input, &mut self.params, &mut self.coefs);
        }
        let v0 = F::from_f32(input[0]);
        let v3 = v0 - self.ic2eq;
        let v1 = self.coefs.a1 * self.ic1eq + self.coefs.a2 * v3;
        let v2 = self.ic2eq + self.coefs.a2 * self.ic1eq + self.coefs.a3 * v3;
        self.ic1eq = F::new(2) * v1 - self.ic1eq;
        self.ic2eq = F::new(2) * v2 - self.ic2eq;
        [convert(
            self.coefs.m0 * v0 + self.coefs.m1 * v1 + self.coefs.m2 * v2,
        )]
        .into()
    }

    fn set(&mut self, setting: Setting) {
        if let Parameter::Subsample(period) = setting.parameter() {
            self.period = max(1, *period);
            self.counter = 0;
        }
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = SignalFrame::new(self.outputs());
        output.set(
            0,
            input
                .at(0)
                .filter(0.0, |r| r * self.mode.response(&self.params, frequency)),
        );
        output
    }
}

/// Simper SVF with fixed parameters.
/// - Input 0: audio
/// - Output 0: filtered audio
#[derive(Default, Clone)]
pub struct FixedSvf<F, M>
where
    F: Real,
    M: SvfMode<F>,
{
    mode: M,
    params: SvfParams<F>,
    coefs: SvfCoefs<F>,
    ic1eq: F,
    ic2eq: F,
}

impl<F, M> FixedSvf<F, M>
where
    F: Real,
    M: SvfMode<F>,
{
    pub fn new(mode: M, params: &SvfParams<F>) -> Self {
        let params = params.clone();
        let mut coefs = SvfCoefs::default();
        let mut mode = mode;
        mode.update(&params, &mut coefs);
        FixedSvf {
            mode,
            params,
            coefs,
            ic1eq: F::zero(),
            ic2eq: F::zero(),
        }
    }

    /// Sample rate in Hz.
    #[inline]
    pub fn sample_rate(&self) -> F {
        self.params.sample_rate
    }
    /// Filter cutoff in Hz. Synonymous with `center`.
    #[inline]
    pub fn cutoff(&self) -> F {
        self.params.cutoff
    }
    /// Filter center in Hz. Synonymous with `cutoff`.
    #[inline]
    pub fn center(&self) -> F {
        self.params.cutoff
    }
    /// Filter Q.
    #[inline]
    pub fn q(&self) -> F {
        self.params.q
    }
    /// Filter gain. Only equalization modes support gain; others ignore it.
    #[inline]
    pub fn gain(&self) -> F {
        self.params.gain
    }

    /// Set filter cutoff in Hz. Synonymous with `set_center`.
    #[inline]
    pub fn set_cutoff(&mut self, cutoff: F) {
        self.params.cutoff = cutoff;
        self.mode.update_frequency(&self.params, &mut self.coefs);
    }

    /// Set filter center in Hz. Synonymous with `set_cutoff`.
    #[inline]
    pub fn set_center(&mut self, center: F) {
        self.set_cutoff(center);
    }

    /// Set filter cutoff in Hz and Q. Synonymous with `set_center_q`.
    #[inline]
    pub fn set_cutoff_q(&mut self, cutoff: F, q: F) {
        self.params.cutoff = cutoff;
        self.params.q = q;
        self.mode.update_frequency(&self.params, &mut self.coefs);
    }

    /// Set filter center in Hz and Q. Synonymous with `set_cutoff_q`.
    #[inline]
    pub fn set_center_q(&mut self, center: F, q: F) {
        self.set_cutoff_q(center, q);
    }

    /// Set filter Q.
    #[inline]
    pub fn set_q(&mut self, q: F) {
        self.params.q = q;
        self.mode.update_q(&self.params, &mut self.coefs);
    }

    /// Set filter gain. Only equalizing modes support gain. Other modes ignore it.
    #[inline]
    pub fn set_gain(&mut self, gain: F) {
        self.params.gain = gain;
        self.mode.update_gain(&self.params, &mut self.coefs);
    }

    /// Set filter cutoff in Hz, Q and gain. Synonymous with `set_center_q_gain`.
    #[inline]
    pub fn set_cutoff_q_gain(&mut self, cutoff: F, q: F, gain: F) {
        self.params.cutoff = cutoff;
        self.params.q = q;
        self.params.gain = gain;
        self.mode.update(&self.params, &mut self.coefs);
    }

    /// Set filter center in Hz, Q and gain. Synonymous with `set_cutoff_q_gain`.
    #[inline]
    pub fn set_center_q_gain(&mut self, center: F, q: F, gain: F) {
        self.set_cutoff_q_gain(center, q, gain);
    }
}

impl<F, M> AudioNode for FixedSvf<F, M>
where
    F: Real,
    M: SvfMode<F>,
{
    const ID: u64 = 43;
    type Inputs = U1;
    type Outputs = U1;

    fn reset(&mut self) {
        self.ic1eq = F::zero();
        self.ic2eq = F::zero();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.params.sample_rate = convert(sample_rate);
        self.mode.update_frequency(&self.params, &mut self.coefs);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let v0 = convert(input[0]);
        let v3 = v0 - self.ic2eq;
        let v1 = self.coefs.a1 * self.ic1eq + self.coefs.a2 * v3;
        let v2 = self.ic2eq + self.coefs.a2 * self.ic1eq + self.coefs.a3 * v3;
        self.ic1eq = F::new(2) * v1 - self.ic1eq;
        self.ic2eq = F::new(2) * v2 - self.ic2eq;
        [convert(
            self.coefs.m0 * v0 + self.coefs.m1 * v1 + self.coefs.m2 * v2,
        )]
        .into()
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

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = SignalFrame::new(self.outputs());
        output.set(
            0,
            input
                .at(0)
                .filter(0.0, |r| r * self.mode.response(&self.params, frequency)),
        );
        output
    }
}

/// Morphing filter that morphs between lowpass, peak and highpass modes.
/// - Input 0: input signal
/// - Input 1: center frequency (Hz)
/// - Input 2: Q
/// - Input 3: morph in -1...1 (-1 = lowpass, 0 = peak, 1 = highpass)
/// - Output 0: filtered signal
#[derive(Clone)]
pub struct Morph<F: Real> {
    filter: Svf<F, PeakMode<F>>,
    morph: f32,
}

impl<F: Real> Morph<F> {
    pub fn new(cutoff: F, q: F, morph: F) -> Self {
        let params = SvfParams {
            sample_rate: F::from_f64(DEFAULT_SR),
            cutoff,
            q,
            gain: F::zero(),
        };
        let mut node = Self {
            filter: Svf::new(PeakMode::new(), &params),
            morph: convert(morph),
        };
        let hash = node.ping(true, AttoHash::new(Self::ID));
        node.ping(false, hash);
        node
    }
}

impl<F: Real> AudioNode for Morph<F> {
    const ID: u64 = 62;
    type Inputs = U4;
    type Outputs = U1;

    fn reset(&mut self) {
        self.filter.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.filter.set_sample_rate(sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        self.morph = input[3];
        let filter_out = self.filter.tick(Frame::from_slice(&input[0..3]));
        [(filter_out[0] + input[3] * input[0]) * 0.5].into()
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        self.filter.process(size, &input.subset(0, 3), output);
        for i in 0..simd_items(size) {
            output.set(
                0,
                i,
                (output.at(0, i) + input.at(0, i) * input.at(3, i)) * 0.5,
            );
        }
        if size > 0 {
            self.morph = input.at_f32(3, size - 1);
        }
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = self.filter.route(input, frequency);
        output.set(
            0,
            output.at(0).filter(0.0, |r| {
                (r + Complex64::new(self.morph.to_f64(), 0.0)) * 0.5
            }),
        );
        output
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.filter.ping(probe, hash).hash(Self::ID)
    }
}
