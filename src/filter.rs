//! Various filters.

use super::audionode::*;
use super::math::*;
use super::setting::*;
use super::signal::*;
use super::*;
use core::marker::PhantomData;
use numeric_array::typenum::*;
use numeric_array::*;

/// One-pole lowpass filter.
/// Setting: cutoff.
/// The number of inputs is `N`, either `U1` or `U2`.
/// - Input 0: input signal
/// - Input 1 (optional): cutoff frequency (Hz)
/// - Output 0: filtered signal
#[derive(Default, Clone)]
pub struct Lowpole<F: Real, N: Size<f32>> {
    _marker: PhantomData<N>,
    value: F,
    coeff: F,
    cutoff: F,
    sample_rate: F,
}

impl<F: Real, N: Size<f32>> Lowpole<F, N> {
    /// Create new lowpass filter. Cutoff frequency is specified in Hz.
    pub fn new(cutoff: F) -> Self {
        let mut node = Lowpole {
            _marker: PhantomData,
            value: F::zero(),
            coeff: F::zero(),
            cutoff,
            sample_rate: convert(DEFAULT_SR),
        };
        node.set_cutoff(cutoff);
        node
    }

    /// Set the cutoff frequency (in Hz).
    /// This has no effect if the filter has a cutoff frequency input.
    pub fn set_cutoff(&mut self, cutoff: F) {
        self.cutoff = cutoff;
        self.coeff = exp(-F::TAU * cutoff / self.sample_rate);
    }
}

impl<F: Real, N: Size<f32>> AudioNode for Lowpole<F, N> {
    const ID: u64 = 18;
    type Inputs = N;
    type Outputs = typenum::U1;

    fn reset(&mut self) {
        self.value = F::zero();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = convert(sample_rate);
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
        let x = convert(input[0]);
        self.value = (F::one() - self.coeff) * x + self.coeff * self.value;
        [convert(self.value)].into()
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
                let c = self.coeff.to_f64();
                let f = frequency * f64::TAU / self.sample_rate.to_f64();
                let z1 = Complex64::from_polar(1.0, -f);
                r * ((1.0 - c) / (1.0 - c * z1))
            }),
        );
        output
    }
}

/// DC blocking filter with cutoff frequency in Hz.
/// Setting: cutoff.
/// - Input 0: signal
/// - Output 0: zero centered signal
#[derive(Default, Clone)]
pub struct DCBlock<F: Real> {
    x1: F,
    y1: F,
    cutoff: F,
    coeff: F,
    sample_rate: F,
}

impl<F: Real> DCBlock<F> {
    /// Create new DC blocking filter with `cutoff` frequency specified in Hz.
    pub fn new(cutoff: F) -> Self {
        let mut node = Self {
            cutoff,
            ..Default::default()
        };
        node.reset();
        node.set_sample_rate(DEFAULT_SR);
        node
    }

    /// Set the cutoff frequency (in Hz).
    pub fn set_cutoff(&mut self, cutoff: F) {
        self.cutoff = cutoff;
        self.coeff = F::one() - F::TAU / self.sample_rate * cutoff;
    }
}

impl<F: Real> AudioNode for DCBlock<F> {
    const ID: u64 = 22;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;

    fn reset(&mut self) {
        self.x1 = F::zero();
        self.y1 = F::zero();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = convert(sample_rate);
        self.set_cutoff(self.cutoff);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let x = convert(input[0]);
        let y0 = x - self.x1 + self.coeff * self.y1;
        self.x1 = x;
        self.y1 = y0;
        [convert(y0)].into()
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
                let c = self.coeff.to_f64();
                let f = frequency * f64::TAU / self.sample_rate.to_f64();
                let z1 = Complex64::from_polar(1.0, -f);
                r * ((1.0 - z1) / (1.0 - c * z1))
            }),
        );
        output
    }
}

/// Pinking filter (3 dB/octave lowpass).
/// - Input 0: input signal
/// - Output 0: filtered signal
#[derive(Default, Clone)]
pub struct Pinkpass<F: Float> {
    // Algorithm by Paul Kellett. +-0.05 dB accuracy above 9.2 Hz @ 44.1 kHz.
    b0: F,
    b1: F,
    b2: F,
    b3: F,
    b4: F,
    b5: F,
    b6: F,
    sample_rate: F,
}

impl<F: Float> Pinkpass<F> {
    /// Create pinking filter.
    pub fn new() -> Self {
        Self {
            sample_rate: convert(DEFAULT_SR),
            ..Pinkpass::default()
        }
    }
}

impl<F: Float> AudioNode for Pinkpass<F> {
    const ID: u64 = 26;
    type Inputs = U1;
    type Outputs = U1;

    fn reset(&mut self) {
        self.b0 = F::zero();
        self.b1 = F::zero();
        self.b2 = F::zero();
        self.b3 = F::zero();
        self.b4 = F::zero();
        self.b5 = F::zero();
        self.b6 = F::zero();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = convert(sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
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

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = SignalFrame::new(self.outputs());
        output.set(
            0,
            input.at(0).filter(0.0, |r| {
                let f = frequency * f64::TAU / self.sample_rate.to_f64();
                let z1 = Complex64::from_polar(1.0, -f);
                let pole0 = 0.0555179 / (1.0 - 0.99886 * z1);
                let pole1 = 0.0750759 / (1.0 - 0.99332 * z1);
                let pole2 = 0.1538520 / (1.0 - 0.96900 * z1);
                let pole3 = 0.3104856 / (1.0 - 0.86650 * z1);
                let pole4 = 0.5329522 / (1.0 - 0.55000 * z1);
                let pole5 = -0.016898 / (1.0 + 0.7616 * z1);
                r * (pole0 + pole1 + pole2 + pole3 + pole4 + pole5 + 0.115926 * z1 + 0.5362)
                    * 0.115830421
            }),
        );
        output
    }
}

/// 1st order allpass filter.
/// Setting: delay.
/// The number of inputs is `N`, either `U1` or `U2`.
/// - Input 0: input signal
/// - Input 1 (optional): delay in samples at DC (delay > 0)
/// - Output 0: filtered signal
#[derive(Default, Clone)]
pub struct Allpole<F: Float, N: Size<f32>> {
    _marker: PhantomData<N>,
    eta: F,
    x1: F,
    y1: F,
    sample_rate: F,
}

impl<F: Float, N: Size<f32>> Allpole<F, N> {
    /// Create new allpass filter. Initial `delay` is specified in samples.
    pub fn new(delay: F) -> Self {
        assert!(delay > F::zero());
        let mut node = Allpole {
            _marker: PhantomData,
            eta: F::zero(),
            x1: F::zero(),
            y1: F::zero(),
            sample_rate: convert(DEFAULT_SR),
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

impl<F: Float, N: Size<f32>> AudioNode for Allpole<F, N> {
    const ID: u64 = 46;
    type Inputs = N;
    type Outputs = typenum::U1;

    fn reset(&mut self) {
        self.x1 = F::zero();
        self.y1 = F::zero();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = convert(sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        if N::USIZE > 1 {
            self.set_delay(convert(input[1]));
        }
        let x0 = convert(input[0]);
        let y0 = self.eta * (x0 - self.y1) + self.x1;
        self.x1 = x0;
        self.y1 = y0;
        [convert(y0)].into()
    }

    fn set(&mut self, setting: Setting) {
        if let Parameter::Delay(delay) = setting.parameter() {
            self.set_delay(F::from_f32(*delay));
        }
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = SignalFrame::new(self.outputs());
        output.set(
            0,
            input.at(0).filter(0.0, |r| {
                let eta = self.eta.to_f64();
                let z1 =
                    Complex64::from_polar(1.0, -frequency * f64::TAU / self.sample_rate.to_f64());
                r * (eta + z1) / (1.0 + eta * z1)
            }),
        );
        output
    }
}

/// One-pole, one-zero highpass filter.
/// Setting: cutoff.
/// The number of inputs is `N`, either `U1` or `U2`.
/// - Input 0: input signal
/// - Input 1 (optional): cutoff frequency (Hz)
/// - Output 0: filtered signal
#[derive(Default, Clone)]
pub struct Highpole<F: Real, N: Size<f32>> {
    _marker: PhantomData<N>,
    x1: F,
    y1: F,
    coeff: F,
    cutoff: F,
    sample_rate: F,
}

impl<F: Real, N: Size<f32>> Highpole<F, N> {
    /// Create new highpass filter. Initial `cutoff` frequency is specified in Hz.
    pub fn new(cutoff: F) -> Self {
        let mut node = Highpole {
            _marker: PhantomData,
            x1: F::zero(),
            y1: F::zero(),
            coeff: F::zero(),
            cutoff,
            sample_rate: convert(DEFAULT_SR),
        };
        node.set_cutoff(cutoff);
        node
    }
    pub fn set_cutoff(&mut self, cutoff: F) {
        self.cutoff = cutoff;
        self.coeff = exp(-F::TAU * cutoff / self.sample_rate);
    }
}

impl<F: Real, N: Size<f32>> AudioNode for Highpole<F, N> {
    const ID: u64 = 47;
    type Inputs = N;
    type Outputs = typenum::U1;

    fn reset(&mut self) {
        self.x1 = F::zero();
        self.y1 = F::zero();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = convert(sample_rate);
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
        let x0 = convert(input[0]);
        let y0 = self.coeff * (self.y1 + x0 - self.x1);
        self.x1 = x0;
        self.y1 = y0;
        [convert(y0)].into()
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
                let c = self.coeff.to_f64();
                let f = frequency * f64::TAU / self.sample_rate.to_f64();
                let z1 = Complex64::from_polar(1.0, -f);
                r * (c * (1.0 - z1) / (1.0 - c * z1))
            }),
        );
        output
    }
}
