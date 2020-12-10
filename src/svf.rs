use super::audionode::*;
use super::math::*;
use super::signal::*;
use super::*;
use num_complex::Complex64;
use numeric_array::typenum::*;
use std::marker::PhantomData;

// Andrew Simper's state variable filters from
// https://cytomic.com/files/dsp/SvfLinearTrapOptimised2.pdf

/// State variable filter coefficients, generic formulation.
#[derive(Clone, Default)]
pub struct SvfCoeffs<F: Real> {
    pub a1: F,
    pub a2: F,
    pub a3: F,
    pub m0: F,
    pub m1: F,
    pub m2: F,
}

impl<F: Real> SvfCoeffs<F> {
    /// Calculate coefficients for a lowpass filter.
    pub fn lowpass(sample_rate: F, cutoff: F, q: F) -> Self {
        let g = tan(F::from_f64(PI) * cutoff / sample_rate);
        let k = F::one() / q;
        let a1 = F::one() / (F::one() + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        let m0 = F::zero();
        let m1 = F::zero();
        let m2 = F::one();

        SvfCoeffs {
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
        let g = tan(F::from_f64(PI) * cutoff / sample_rate);
        let k = F::one() / q;
        let a1 = F::one() / (F::one() + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        let m0 = F::one();
        let m1 = -k;
        let m2 = -F::one();

        SvfCoeffs {
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
        let g = tan(F::from_f64(PI) * cutoff / sample_rate);
        let k = F::one() / q;
        let a1 = F::one() / (F::one() + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        let m0 = F::zero();
        let m1 = F::one();
        let m2 = F::zero();

        SvfCoeffs {
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
        let g = tan(F::from_f64(PI) * cutoff / sample_rate);
        let k = F::one() / q;
        let a1 = F::one() / (F::one() + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        let m0 = F::one();
        let m1 = -k;
        let m2 = F::zero();

        SvfCoeffs {
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
        let g = tan(F::from_f64(PI) * cutoff / sample_rate);
        let k = F::one() / q;
        let a1 = F::one() / (F::one() + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        let m0 = F::one();
        let m1 = -k;
        let m2 = F::new(-2);

        SvfCoeffs {
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
        let g = tan(F::from_f64(PI) * cutoff / sample_rate);
        let k = F::one() / q;
        let a1 = F::one() / (F::one() + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        let m0 = F::one();
        let m1 = F::new(-2) * k;
        let m2 = F::zero();

        SvfCoeffs {
            a1,
            a2,
            a3,
            m0,
            m1,
            m2,
        }
    }

    /// Calculate coefficients for a bell filter.
    /// Gain is amplitude gain.
    pub fn bell(sample_rate: F, cutoff: F, q: F, gain: F) -> Self {
        let a = sqrt(gain);
        let g = tan(F::from_f64(PI) * cutoff / sample_rate);
        let k = F::one() / (q * a);
        let a1 = F::one() / (F::one() + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        let m0 = F::one();
        let m1 = k * (a * a - F::one());
        let m2 = F::zero();

        SvfCoeffs {
            a1,
            a2,
            a3,
            m0,
            m1,
            m2,
        }
    }

    /// Calculate coefficients for a low shelf filter.
    /// Gain is amplitude gain.
    pub fn lowshelf(sample_rate: F, cutoff: F, q: F, gain: F) -> Self {
        let a = sqrt(gain);
        let g = tan(F::from_f64(PI) * cutoff / sample_rate) / sqrt(a);
        let k = F::one() / q;
        let a1 = F::one() / (F::one() + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        let m0 = F::one();
        let m1 = k * (a - F::one());
        let m2 = a * a - F::one();

        SvfCoeffs {
            a1,
            a2,
            a3,
            m0,
            m1,
            m2,
        }
    }

    /// Calculate coefficients for a high shelf filter.
    /// Gain is amplitude gain.
    pub fn highshelf(sample_rate: F, cutoff: F, q: F, gain: F) -> Self {
        let a = sqrt(gain);
        let g = tan(F::from_f64(PI) * cutoff / sample_rate) * sqrt(a);
        let k = F::one() / q;
        let a1 = F::one() / (F::one() + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        let m0 = a * a;
        let m1 = k * (F::one() - a) * a;
        let m2 = F::one() - a * a;

        SvfCoeffs {
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
pub trait SvfMode<F: Real>: Clone + Default {
    /// Number of inputs, which includes the audio input. Equal to the number of continuous parameters plus one.
    type Inputs: Size<F>;

    /// Update coefficients and state from the full set of parameters.
    fn update(&mut self, params: &SvfParams<F>, coeffs: &mut SvfCoeffs<F>);

    /// Update coefficients and state from sample rate and/or cutoff.
    /// Other parameters are untouched.
    fn update_frequency(&mut self, params: &SvfParams<F>, coeffs: &mut SvfCoeffs<F>) {
        // Do a bulk update by default.
        self.update(params, coeffs);
    }

    /// Update coefficients and state from Q.
    /// Other parameters are untouched.
    fn update_q(&mut self, params: &SvfParams<F>, coeffs: &mut SvfCoeffs<F>) {
        // Do a bulk update by default.
        self.update(params, coeffs);
    }

    /// Update coefficients and state from gain. Gain is given as amplitude.
    /// Other parameters are untouched.
    /// Only equalizing modes (bell and shelf) support gain. It is ignored by default.
    fn update_gain(&mut self, _params: &SvfParams<F>, _coeffs: &mut SvfCoeffs<F>) {}

    /// Update coefficients, parameters and state from input.
    fn update_inputs(
        &mut self,
        _input: &Frame<F, Self::Inputs>,
        _params: &mut SvfParams<F>,
        _coeffs: &mut SvfCoeffs<F>,
    ) {
    }

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

#[derive(Clone, Default)]
pub struct Svf<T, F, M>
where
    T: Float,
    F: Real,
    M: SvfMode<F>,
    M::Inputs: Size<T>,
{
    mode: M,
    params: SvfParams<F>,
    coeffs: SvfCoeffs<F>,
    ic1eq: F,
    ic2eq: F,
    _marker: PhantomData<T>,
}

impl<T, F, M> Svf<T, F, M>
where
    T: Float,
    F: Real,
    M: SvfMode<F>,
    M::Inputs: Size<T>,
{
    pub fn new(mode: M, params: &SvfParams<F>) -> Self {
        let params = params.clone();
        let mut coeffs = SvfCoeffs::default();
        let mut mode = mode;
        mode.update(&params, &mut coeffs);
        Svf {
            mode,
            params,
            coeffs,
            ic1eq: F::zero(),
            ic2eq: F::zero(),
            _marker: PhantomData::default(),
        }
    }

    /// Sample rate in Hz.
    pub fn sample_rate(&self) -> F {
        self.params.sample_rate
    }
    /// Filter cutoff in Hz. Synonymous with `center`.
    pub fn cutoff(&self) -> F {
        self.params.cutoff
    }
    /// Filter center in Hz. Synonymous with `cutoff`.
    pub fn center(&self) -> F {
        self.params.cutoff
    }
    /// Filter Q.
    pub fn q(&self) -> F {
        self.params.q
    }
    /// Filter gain. Only equalization modes support gain; others ignore it.
    pub fn gain(&self) -> F {
        self.params.gain
    }

    /// Set filter cutoff in Hz. Synonymous with `set_center`.
    pub fn set_cutoff(&mut self, cutoff: F) {
        self.params.cutoff = cutoff;
        self.mode.update_frequency(&self.params, &mut self.coeffs);
    }

    /// Set filter center in Hz. Synonymous with `set_cutoff`.
    pub fn set_center(&mut self, center: F) {
        self.set_cutoff(center);
    }

    /// Set filter cutoff in Hz and Q. Synonymous with `set_center_q`.
    pub fn set_cutoff_q(&mut self, cutoff: F, q: F) {
        self.params.cutoff = cutoff;
        self.params.q = q;
        self.mode.update_frequency(&self.params, &mut self.coeffs);
    }

    /// Set filter center in Hz and Q. Synonymous with `set_cutoff_q`.
    pub fn set_center_q(&mut self, center: F, q: F) {
        self.set_cutoff_q(center, q);
    }

    /// Set filter Q.
    pub fn set_q(&mut self, q: F) {
        self.params.q = q;
        self.mode.update_q(&self.params, &mut self.coeffs);
    }

    /// Set filter gain. Only equalizing modes support gain. Other modes ignore it.
    pub fn set_gain(&mut self, gain: F) {
        self.params.gain = gain;
        self.mode.update_gain(&self.params, &mut self.coeffs);
    }
}

impl<T, F, M> AudioNode for Svf<T, F, M>
where
    T: Float,
    F: Real,
    M: SvfMode<F>,
    M::Inputs: Size<T>,
{
    const ID: u64 = 36;
    type Sample = T;
    type Inputs = M::Inputs;
    type Outputs = U1;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.ic1eq = F::zero();
        self.ic2eq = F::zero();
        if let Some(sample_rate) = sample_rate {
            self.params.sample_rate = convert(sample_rate);
            self.mode.update_frequency(&self.params, &mut self.coeffs);
        }
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        // Update parameters from input.
        let input = Frame::generate(|i| convert(input[i]));
        self.mode
            .update_inputs(&input, &mut self.params, &mut self.coeffs);
        let v0 = input[0];
        let v3 = v0 - self.ic2eq;
        let v1 = self.coeffs.a1 * self.ic1eq + self.coeffs.a2 * v3;
        let v2 = self.ic2eq + self.coeffs.a2 * self.ic1eq + self.coeffs.a3 * v3;
        self.ic1eq = F::new(2) * v1 - self.ic1eq;
        self.ic2eq = F::new(2) * v2 - self.ic2eq;
        [convert(
            self.coeffs.m0 * v0 + self.coeffs.m1 * v1 + self.coeffs.m2 * v2,
        )]
        .into()
    }

    fn propagate(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame();
        output[0] = filter_signal(input[0], 0.0, |r| {
            r * self.mode.response(&self.params, frequency)
        });
        output
    }
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
    fn update(&mut self, params: &SvfParams<F>, coeffs: &mut SvfCoeffs<F>) {
        *coeffs = SvfCoeffs::lowpass(params.sample_rate, params.cutoff, params.q);
    }
    fn update_inputs(
        &mut self,
        input: &Frame<F, Self::Inputs>,
        params: &mut SvfParams<F>,
        coeffs: &mut SvfCoeffs<F>,
    ) {
        let cutoff = input[1];
        let q = input[2];
        if cutoff != params.cutoff || q != params.q {
            params.cutoff = cutoff;
            params.q = q;
            self.update(&params, coeffs);
        }
    }

    fn response(&self, params: &SvfParams<F>, frequency: f64) -> Complex64 {
        let g = tan(PI * params.cutoff.to_f64() / params.sample_rate.to_f64());
        let k = 1.0 / params.q.to_f64();
        let f = frequency * TAU / params.sample_rate.to_f64();
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
    fn update(&mut self, params: &SvfParams<F>, coeffs: &mut SvfCoeffs<F>) {
        *coeffs = SvfCoeffs::highpass(params.sample_rate, params.cutoff, params.q);
    }
    fn update_inputs(
        &mut self,
        input: &Frame<F, Self::Inputs>,
        params: &mut SvfParams<F>,
        coeffs: &mut SvfCoeffs<F>,
    ) {
        let cutoff = input[1];
        let q = input[2];
        if cutoff != params.cutoff || q != params.q {
            params.cutoff = cutoff;
            params.q = q;
            self.update(&params, coeffs);
        }
    }

    fn response(&self, params: &SvfParams<F>, frequency: f64) -> Complex64 {
        let g = tan(PI * params.cutoff.to_f64() / params.sample_rate.to_f64());
        let k = 1.0 / params.q.to_f64();
        let f = frequency * TAU / params.sample_rate.to_f64();
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
    fn update(&mut self, params: &SvfParams<F>, coeffs: &mut SvfCoeffs<F>) {
        *coeffs = SvfCoeffs::bandpass(params.sample_rate, params.cutoff, params.q);
    }
    fn update_inputs(
        &mut self,
        input: &Frame<F, Self::Inputs>,
        params: &mut SvfParams<F>,
        coeffs: &mut SvfCoeffs<F>,
    ) {
        let cutoff = input[1];
        let q = input[2];
        if cutoff != params.cutoff || q != params.q {
            params.cutoff = cutoff;
            params.q = q;
            self.update(&params, coeffs);
        }
    }

    fn response(&self, params: &SvfParams<F>, frequency: f64) -> Complex64 {
        let g = tan(PI * params.cutoff.to_f64() / params.sample_rate.to_f64());
        let k = 1.0 / params.q.to_f64();
        let f = frequency * TAU / params.sample_rate.to_f64();
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
    fn update(&mut self, params: &SvfParams<F>, coeffs: &mut SvfCoeffs<F>) {
        *coeffs = SvfCoeffs::notch(params.sample_rate, params.cutoff, params.q);
    }
    fn update_inputs(
        &mut self,
        input: &Frame<F, Self::Inputs>,
        params: &mut SvfParams<F>,
        coeffs: &mut SvfCoeffs<F>,
    ) {
        let cutoff = input[1];
        let q = input[2];
        if cutoff != params.cutoff || q != params.q {
            params.cutoff = cutoff;
            params.q = q;
            self.update(&params, coeffs);
        }
    }

    fn response(&self, params: &SvfParams<F>, frequency: f64) -> Complex64 {
        let g = tan(PI * params.cutoff.to_f64() / params.sample_rate.to_f64());
        let k = 1.0 / params.q.to_f64();
        let f = frequency * TAU / params.sample_rate.to_f64();
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
    fn update(&mut self, params: &SvfParams<F>, coeffs: &mut SvfCoeffs<F>) {
        *coeffs = SvfCoeffs::peak(params.sample_rate, params.cutoff, params.q);
    }
    fn update_inputs(
        &mut self,
        input: &Frame<F, Self::Inputs>,
        params: &mut SvfParams<F>,
        coeffs: &mut SvfCoeffs<F>,
    ) {
        let cutoff = input[1];
        let q = input[2];
        if cutoff != params.cutoff || q != params.q {
            params.cutoff = cutoff;
            params.q = q;
            self.update(&params, coeffs);
        }
    }

    fn response(&self, params: &SvfParams<F>, frequency: f64) -> Complex64 {
        let g = tan(PI * params.cutoff.to_f64() / params.sample_rate.to_f64());
        let k = 1.0 / params.q.to_f64();
        let f = frequency * TAU / params.sample_rate.to_f64();
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
    fn update(&mut self, params: &SvfParams<F>, coeffs: &mut SvfCoeffs<F>) {
        *coeffs = SvfCoeffs::allpass(params.sample_rate, params.cutoff, params.q);
    }
    fn update_inputs(
        &mut self,
        input: &Frame<F, Self::Inputs>,
        params: &mut SvfParams<F>,
        coeffs: &mut SvfCoeffs<F>,
    ) {
        let cutoff = input[1];
        let q = input[2];
        if cutoff != params.cutoff || q != params.q {
            params.cutoff = cutoff;
            params.q = q;
            self.update(&params, coeffs);
        }
    }

    fn response(&self, params: &SvfParams<F>, frequency: f64) -> Complex64 {
        let g = tan(PI * params.cutoff.to_f64() / params.sample_rate.to_f64());
        let k = 1.0 / params.q.to_f64();
        let f = frequency * TAU / params.sample_rate.to_f64();
        let z = Complex64::from_polar(1.0, f);
        ((z - 1.0) * (z - 1.0) + g * g * (1.0 + z) * (1.0 + z) + g * (k - k * z * z))
            / ((z - 1.0) * (z - 1.0) + g * g * (1.0 + z) * (1.0 + z) + g * k * (z * z - 1.0))
    }
}

/// Bell filter with center, Q and gain inputs.
/// - Input 0: audio
/// - Input 1: center in Hz
/// - Input 2: Q
/// - Input 3: amplitude gain
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
    fn update(&mut self, params: &SvfParams<F>, coeffs: &mut SvfCoeffs<F>) {
        *coeffs = SvfCoeffs::bell(params.sample_rate, params.cutoff, params.q, params.gain);
    }
    fn update_inputs(
        &mut self,
        input: &Frame<F, Self::Inputs>,
        params: &mut SvfParams<F>,
        coeffs: &mut SvfCoeffs<F>,
    ) {
        let cutoff = input[1];
        let q = input[2];
        let gain = input[3];
        if cutoff != params.cutoff || q != params.q || gain != params.gain {
            params.cutoff = cutoff;
            params.q = q;
            params.gain = gain;
            self.update(&params, coeffs);
        }
    }

    fn response(&self, params: &SvfParams<F>, frequency: f64) -> Complex64 {
        let a = sqrt(params.gain.to_f64());
        let g = tan(PI * params.cutoff.to_f64() / params.sample_rate.to_f64());
        let k = 1.0 / params.q.to_f64();
        let z = Complex64::from_polar(1.0, frequency * TAU / params.sample_rate.to_f64());
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
/// - Input 3: amplitude gain
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
    fn update(&mut self, params: &SvfParams<F>, coeffs: &mut SvfCoeffs<F>) {
        *coeffs = SvfCoeffs::lowshelf(params.sample_rate, params.cutoff, params.q, params.gain);
    }
    fn update_inputs(
        &mut self,
        input: &Frame<F, Self::Inputs>,
        params: &mut SvfParams<F>,
        coeffs: &mut SvfCoeffs<F>,
    ) {
        let cutoff = input[1];
        let q = input[2];
        let gain = input[3];
        if cutoff != params.cutoff || q != params.q || gain != params.gain {
            params.cutoff = cutoff;
            params.q = q;
            params.gain = gain;
            self.update(&params, coeffs);
        }
    }

    fn response(&self, params: &SvfParams<F>, frequency: f64) -> Complex64 {
        let a = sqrt(params.gain.to_f64());
        let g = tan(PI * params.cutoff.to_f64() / params.sample_rate.to_f64());
        let k = 1.0 / params.q.to_f64();
        let z = Complex64::from_polar(1.0, frequency * TAU / params.sample_rate.to_f64());
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
/// - Input 3: amplitude gain
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
    fn update(&mut self, params: &SvfParams<F>, coeffs: &mut SvfCoeffs<F>) {
        *coeffs = SvfCoeffs::highshelf(params.sample_rate, params.cutoff, params.q, params.gain);
    }
    fn update_inputs(
        &mut self,
        input: &Frame<F, Self::Inputs>,
        params: &mut SvfParams<F>,
        coeffs: &mut SvfCoeffs<F>,
    ) {
        let cutoff = input[1];
        let q = input[2];
        let gain = input[3];
        if cutoff != params.cutoff || q != params.q || gain != params.gain {
            params.cutoff = cutoff;
            params.q = q;
            params.gain = gain;
            self.update(&params, coeffs);
        }
    }

    fn response(&self, params: &SvfParams<F>, frequency: f64) -> Complex64 {
        let a = sqrt(params.gain.to_f64());
        let g = tan(PI * params.cutoff.to_f64() / params.sample_rate.to_f64());
        let k = 1.0 / params.q.to_f64();
        let z = Complex64::from_polar(1.0, frequency * TAU / params.sample_rate.to_f64());
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
