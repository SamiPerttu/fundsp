//! Oscillator components.

use super::audionode::*;
use super::filter::*;
use super::fir::*;
use super::math::*;
use super::signal::*;
use super::*;
use funutd::Rnd;
use numeric_array::*;
use std::marker::PhantomData;

/// Sine oscillator.
/// - Input 0: frequency in Hz.
/// - Output 0: sine wave.
#[derive(Default, Clone)]
pub struct Sine<T: Real> {
    phase: T,
    sample_duration: T,
    hash: u64,
    initial_phase: Option<T>,
}

impl<T: Real> Sine<T> {
    /// Create sine oscillator.
    pub fn new(sample_rate: f64) -> Self {
        let mut sine = Sine::default();
        sine.reset(Some(sample_rate));
        sine
    }
    /// Create sine oscillator with optional initial phase in 0...1.
    pub fn with_phase(sample_rate: f64, initial_phase: Option<T>) -> Self {
        let mut sine = Self {
            phase: T::zero(),
            sample_duration: T::zero(),
            hash: 0,
            initial_phase,
        };
        sine.reset(Some(sample_rate));
        sine
    }
}

impl<T: Real> AudioNode for Sine<T> {
    const ID: u64 = 21;
    type Sample = T;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;
    type Setting = ();

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.phase = match self.initial_phase {
            Some(phase) => phase,
            None => T::from_f64(rnd(self.hash as i64)),
        };
        if let Some(sr) = sample_rate {
            self.sample_duration = T::from_f64(1.0 / sr);
        }
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        self.phase += input[0] * self.sample_duration;
        // This is supposedly faster than self.phase -= self.phase.floor();
        while self.phase > T::one() {
            self.phase -= T::one();
        }
        [sin(self.phase * T::from_f64(TAU))].into()
    }

    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        for i in 0..size {
            self.phase += input[0][i] * self.sample_duration;
            output[0][i] = sin(self.phase * T::from_f64(TAU));
        }
        self.phase -= self.phase.floor();
    }

    fn set_hash(&mut self, hash: u64) {
        self.hash = hash;
        self.reset(None);
    }

    fn route(&mut self, _input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = Signal::Latency(0.0);
        output
    }
}

/// Discrete summation formula. Returns sum, of `i` in `0..n`, of `r ** i * sin(f + i * d)`.
fn dsf<T: Real>(f: T, d: T, r: T, n: T) -> T {
    // Note: beware of division by zero, which results when `r` = 1 and `d` = 0.
    // Formula is from Moorer, J. A., The Synthesis of Complex Audio Spectra by Means of Discrete Summation Formulae, 1976.
    (sin(f)
        - r * sin(f - d)
        - pow(r, n + T::one()) * (sin(f + (n + T::one()) * d) - r * sin(f + n * d)))
        / (T::one() + r * r - T::new(2) * r * cos(d))
}

/// DSF oscillator. Number of inputs is `N`, either 1 or 2.
/// Setting: roughness.
/// - Input 0: frequency in Hz.
/// - Input 1 (optional): roughness in 0...1 is the relative amplitude of successive partials.
/// - Output 0: DSF wave.
#[derive(Clone)]
pub struct Dsf<T: Real, N: Size<T>> {
    phase: T,
    roughness: T,
    harmonic_spacing: T,
    sample_duration: T,
    hash: u64,
    _marker: PhantomData<N>,
}

impl<T: Real, N: Size<T>> Dsf<T, N> {
    pub fn new(sample_rate: f64, harmonic_spacing: T, roughness: T) -> Self {
        let mut node = Dsf {
            phase: T::zero(),
            roughness,
            harmonic_spacing,
            sample_duration: T::zero(),
            hash: 0,
            _marker: PhantomData::default(),
        };
        node.reset(Some(sample_rate));
        node.set_roughness(roughness);
        node
    }

    /// Roughness accessor.
    #[inline]
    pub fn roughness(&self) -> T {
        self.roughness
    }

    /// Set roughness. Roughness in 0...1 is the relative amplitude of successive partials.
    #[inline]
    pub fn set_roughness(&mut self, roughness: T) {
        self.roughness = clamp(T::from_f64(0.0001), T::from_f64(0.9999), roughness);
    }
}

impl<T: Real, N: Size<T>> AudioNode for Dsf<T, N> {
    const ID: u64 = 55;
    type Sample = T;
    type Inputs = N;
    type Outputs = typenum::U1;
    type Setting = T;

    fn set(&mut self, setting: Self::Setting) {
        self.set_roughness(setting);
    }

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.phase = T::from_f64(rnd(self.hash as i64));
        if let Some(sr) = sample_rate {
            self.sample_duration = T::from_f64(1.0 / sr);
        }
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        if N::USIZE > 1 {
            self.set_roughness(input[1]);
        }
        self.phase = (self.phase + input[0] * self.sample_duration).fract();
        let n = floor(T::new(22_050) / input[0] / self.harmonic_spacing);
        Frame::from([dsf(
            self.phase * T::from_f64(TAU),
            self.phase * T::from_f64(TAU) * self.harmonic_spacing,
            self.roughness,
            n,
        )])
    }

    fn set_hash(&mut self, hash: u64) {
        self.hash = hash;
        self.reset(None);
    }

    fn route(&mut self, _input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = Signal::Latency(0.0);
        output
    }
}

/// Karplus-Strong oscillator.
/// Allocates: pluck buffer.
/// - Input 0: extra string excitation.
/// - Output 0: plucked string.
#[derive(Clone)]
pub struct Pluck<T: Float> {
    damping: Fir<T, typenum::U3>,
    tuning: Allpole<T, T, typenum::U1>,
    line: Vec<T>,
    gain: T,
    pos: usize,
    hash: u64,
    frequency: T,
    sample_rate: f64,
    initialized: bool,
}

impl<T: Float> Pluck<T> {
    // Create new Karplus-Strong oscillator. High frequency damping is in 0...1.
    pub fn new(
        sample_rate: f64,
        frequency: T,
        gain_per_second: T,
        high_frequency_damping: T,
    ) -> Self {
        Self {
            damping: fir3(T::one() - high_frequency_damping),
            tuning: Allpole::new(sample_rate, T::one()),
            line: Vec::new(),
            gain: T::from_f64(pow(gain_per_second.to_f64(), 1.0 / frequency.to_f64())),
            pos: 0,
            hash: 0,
            frequency,
            sample_rate,
            initialized: false,
        }
    }

    fn initialize_line(&mut self) {
        // Allpass filter delay is in epsilon ... epsilon + 1.
        let epsilon = 0.2;
        // Damping filter delay is 1 sample.
        let total_delay = self.sample_rate / self.frequency.to_f64() - 1.0;
        let loop_delay = floor(total_delay - epsilon);
        let allpass_delay = total_delay - loop_delay;
        self.tuning = Allpole::new(self.sample_rate, T::from_f64(allpass_delay));
        self.line.resize(loop_delay as usize, T::zero());
        let mut rnd = Rnd::from_u64(self.hash);
        for i in 0..self.line.len() {
            self.line[i] = T::from_f32(rnd.f32() * 2.0 - 1.0);
        }
        self.pos = 0;
        self.initialized = true;
    }
}

impl<T: Float> AudioNode for Pluck<T> {
    const ID: u64 = 58;
    type Sample = T;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;
    type Setting = ();

    fn reset(&mut self, sample_rate: Option<f64>) {
        if let Some(sr) = sample_rate {
            self.sample_rate = sr;
        }
        self.damping.reset(sample_rate);
        self.initialized = false;
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        if !self.initialized {
            self.initialize_line();
        }
        let output = self.line[self.pos] * self.gain + input[0];
        let output = self.damping.filter_mono(output);
        let output = self.tuning.filter_mono(output);
        self.line[self.pos] = output;
        self.pos += 1;
        if self.pos == self.line.len() {
            self.pos = 0;
        }
        [output].into()
    }

    fn set_hash(&mut self, hash: u64) {
        self.hash = hash;
        self.initialized = false;
    }

    fn route(&mut self, _input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = Signal::Latency(0.0);
        output
    }

    fn allocate(&mut self) {
        if !self.initialized {
            self.initialize_line();
        }
    }
}

/// Rossler dynamical system oscillator.
/// - Input 0: frequency. The Rossler oscillator exhibits peaks at multiples of this frequency.
/// - Output 0: system output
#[derive(Clone, Default)]
pub struct Rossler<T: Float> {
    x: T,
    y: T,
    z: T,
    sr: T,
    hash: u64,
}

impl<T: Float> Rossler<T> {
    /// Create new Rossler oscillator.
    pub fn new() -> Self {
        let mut rossler = Self::default();
        rossler.reset(Some(DEFAULT_SR));
        rossler
    }
}

impl<T: Float> AudioNode for Rossler<T> {
    const ID: u64 = 73;
    type Sample = T;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;
    type Setting = ();

    fn reset(&mut self, sample_rate: Option<f64>) {
        if let Some(sr) = sample_rate {
            self.sr = T::from_f64(sr);
        }
        self.x = lerp(T::zero(), T::one(), convert(rnd(self.hash as i64)));
        self.y = T::one();
        self.z = T::one();
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let dx = -self.y - self.z;
        let dy = self.x + T::from_f64(0.15) * self.y;
        let dz = T::from_f64(0.2) + self.z * (self.x - T::from_f64(10.0));
        let dt = T::from_f64(2.91) * input[0] / self.sr;
        self.x += dx * dt;
        self.y += dy * dt;
        self.z += dz * dt;
        [self.x * T::from_f64(0.05757)].into()
    }

    fn set_hash(&mut self, hash: u64) {
        self.hash = hash;
        self.reset(None);
    }

    fn route(&mut self, _input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = Signal::Latency(0.0);
        output
    }
}

/// Lorenz dynamical system oscillator.
/// - Input 0: frequency. The Lorenz system exhibits slight frequency effects.
/// - Output 0: system output
#[derive(Clone, Default)]
pub struct Lorenz<T: Float> {
    x: T,
    y: T,
    z: T,
    sr: T,
    hash: u64,
}

impl<T: Float> Lorenz<T> {
    /// Create new Lorenz oscillator.
    pub fn new() -> Self {
        let mut lorenz = Self::default();
        lorenz.reset(Some(DEFAULT_SR));
        lorenz
    }
}

impl<T: Float> AudioNode for Lorenz<T> {
    const ID: u64 = 74;
    type Sample = T;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;
    type Setting = ();

    fn reset(&mut self, sample_rate: Option<f64>) {
        if let Some(sr) = sample_rate {
            self.sr = T::from_f64(sr);
        }
        self.x = lerp(T::zero(), T::one(), convert(rnd(self.hash as i64)));
        self.y = T::one();
        self.z = T::one();
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let dx = T::from_f64(10.0) * (self.y - self.x);
        let dy = self.x * (T::from_f64(28.0) - self.z) - self.y;
        let dz = self.x * self.y - T::from_f64(2.666) * self.z;
        let dt = input[0] / self.sr;
        self.x += dx * dt;
        self.y += dy * dt;
        self.z += dz * dt;
        [self.x * T::from_f64(0.05107)].into()
    }

    fn set_hash(&mut self, hash: u64) {
        self.hash = hash;
        self.reset(None);
    }

    fn route(&mut self, _input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = Signal::Latency(0.0);
        output
    }
}
