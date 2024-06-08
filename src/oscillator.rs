//! Oscillator components.

use super::audionode::*;
use super::buffer::*;
use super::filter::*;
use super::fir::*;
use super::math::*;
use super::setting::*;
use super::signal::*;
use super::*;
use core::marker::PhantomData;
use funutd::Rnd;
use numeric_array::*;
extern crate alloc;
use alloc::vec::Vec;

/// Sine oscillator.
/// - Input 0: frequency in Hz.
/// - Output 0: sine wave.
#[derive(Default, Clone)]
pub struct Sine {
    phase: f32,
    sample_duration: f32,
    hash: u64,
    initial_phase: Option<f32>,
}

impl Sine {
    /// Create sine oscillator.
    pub fn new() -> Self {
        let mut sine = Sine::default();
        sine.reset();
        sine.set_sample_rate(DEFAULT_SR);
        sine
    }
    /// Create sine oscillator with initial phase in 0...1.
    pub fn with_phase(initial_phase: f32) -> Self {
        let mut sine = Self {
            phase: 0.0,
            sample_duration: 0.0,
            hash: 0,
            initial_phase: Some(initial_phase),
        };
        sine.reset();
        sine.set_sample_rate(DEFAULT_SR);
        sine
    }
}

impl AudioNode for Sine {
    const ID: u64 = 21;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;

    fn reset(&mut self) {
        self.phase = match self.initial_phase {
            Some(phase) => phase,
            None => rnd1(self.hash) as f32,
        };
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_duration = convert(1.0 / sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        self.phase += input[0] * self.sample_duration;
        self.phase -= self.phase.floor();
        [sin(self.phase * f32::TAU)].into()
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        let mut phase = self.phase;
        for i in 0..size >> SIMD_S {
            let element: [f32; SIMD_N] = core::array::from_fn(|j| {
                phase += input.at_f32(0, (i << SIMD_S) + j) * self.sample_duration;
                phase
            });
            output.set(0, i, (F32x::new(element) * f32::TAU).sin());
        }
        self.phase = phase;
        self.phase -= self.phase.floor();
        self.process_remainder(size, input, output);
    }

    /*
    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        for i in 0..size {
            self.phase += T::from_f32(input.at_f32(0, i)) * self.sample_duration;
            output.set_f32(0, i, (self.phase * T::TAU).sin().to_f32());
        }
        self.phase -= self.phase.floor();
    }
    */

    fn set_hash(&mut self, hash: u64) {
        self.hash = hash;
        self.reset();
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        super::signal::Routing::Generator(0.0).route(input, self.outputs())
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
pub struct Dsf<N: Size<f32>> {
    phase: f32,
    roughness: f32,
    harmonic_spacing: f32,
    sample_duration: f32,
    hash: u64,
    _marker: PhantomData<N>,
}

impl<N: Size<f32>> Dsf<N> {
    pub fn new(harmonic_spacing: f32, roughness: f32) -> Self {
        let mut node = Self {
            phase: 0.0,
            roughness,
            harmonic_spacing,
            sample_duration: 0.0,
            hash: 0,
            _marker: PhantomData,
        };
        node.reset();
        node.set_sample_rate(DEFAULT_SR);
        node.set_roughness(roughness);
        node
    }

    /// Roughness accessor.
    #[inline]
    pub fn roughness(&self) -> f32 {
        self.roughness
    }

    /// Set roughness. Roughness in 0...1 is the relative amplitude of successive partials.
    #[inline]
    pub fn set_roughness(&mut self, roughness: f32) {
        self.roughness = clamp(0.0001, 0.9999, roughness);
    }
}

impl<N: Size<f32>> AudioNode for Dsf<N> {
    const ID: u64 = 55;
    type Inputs = N;
    type Outputs = typenum::U1;

    fn reset(&mut self) {
        self.phase = rnd1(self.hash) as f32;
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_duration = convert(1.0 / sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        if N::USIZE > 1 {
            self.set_roughness(input[1]);
        }
        self.phase += input[0] * self.sample_duration;
        self.phase -= self.phase.floor();
        let n = floor(22_050.0 / input[0] / self.harmonic_spacing);
        Frame::from([dsf(
            self.phase * f32::TAU,
            self.phase * f32::TAU * self.harmonic_spacing,
            self.roughness,
            n,
        )])
    }

    fn set(&mut self, setting: Setting) {
        if let Parameter::Roughness(roughness) = setting.parameter() {
            self.set_roughness(*roughness);
        }
    }

    fn set_hash(&mut self, hash: u64) {
        self.hash = hash;
        self.reset();
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        super::signal::Routing::Generator(0.0).route(input, self.outputs())
    }
}

/// Karplus-Strong oscillator.
/// - Allocates: pluck buffer.
/// - Input 0: extra string excitation.
/// - Output 0: plucked string.
#[derive(Clone)]
pub struct Pluck {
    damping: Fir<typenum::U3>,
    tuning: Allpole<f32, typenum::U1>,
    line: Vec<f32>,
    gain: f32,
    pos: usize,
    hash: u64,
    frequency: f32,
    sample_rate: f64,
    initialized: bool,
}

impl Pluck {
    // Create new Karplus-Strong oscillator. High frequency damping is in 0...1.
    pub fn new(frequency: f32, gain_per_second: f32, high_frequency_damping: f32) -> Self {
        Self {
            damping: super::prelude::fir3(1.0 - high_frequency_damping).0,
            tuning: Allpole::new(1.0),
            line: Vec::new(),
            gain: pow(gain_per_second.to_f64(), 1.0 / frequency.to_f64()) as f32,
            pos: 0,
            hash: 0,
            frequency,
            sample_rate: DEFAULT_SR,
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
        self.tuning.reset();
        self.tuning.set_sample_rate(self.sample_rate);
        self.tuning.set_delay(allpass_delay as f32);
        self.line.resize(loop_delay as usize, 0.0);
        let mut rnd = Rnd::from_u64(self.hash);
        let mut mean = 0.0;
        for i in 0..self.line.len() {
            self.line[i] = rnd.f32_in(-1.0, 1.0);
            mean += self.line[i].to_f64();
        }
        mean /= self.line.len() as f64;
        for x in self.line.iter_mut() {
            *x -= mean as f32;
        }
        self.pos = 0;
        self.initialized = true;
    }
}

impl AudioNode for Pluck {
    const ID: u64 = 58;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;

    fn reset(&mut self) {
        self.damping.reset();
        self.initialized = false;
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        if self.sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            self.damping.set_sample_rate(sample_rate);
            self.initialized = false;
        }
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
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

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        super::signal::Routing::Generator(0.0).route(input, self.outputs())
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
pub struct Rossler {
    x: f32,
    y: f32,
    z: f32,
    sr: f32,
    hash: u64,
}

impl Rossler {
    /// Create new Rossler oscillator.
    pub fn new() -> Self {
        let mut rossler = Self::default();
        rossler.reset();
        rossler.set_sample_rate(DEFAULT_SR);
        rossler
    }
}

impl AudioNode for Rossler {
    const ID: u64 = 73;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;

    fn reset(&mut self) {
        self.x = lerp(0.0, 1.0, rnd1(self.hash) as f32);
        self.y = 1.0;
        self.z = 1.0;
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sr = convert(sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let dx = -self.y - self.z;
        let dy = self.x + 0.15 * self.y;
        let dz = 0.2 + self.z * (self.x - 10.0);
        let dt = 2.91 * input[0] / self.sr;
        self.x += dx * dt;
        self.y += dy * dt;
        self.z += dz * dt;
        [self.x * 0.05757].into()
    }

    fn set_hash(&mut self, hash: u64) {
        self.hash = hash;
        self.reset();
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        super::signal::Routing::Generator(0.0).route(input, self.outputs())
    }
}

/// Lorenz dynamical system oscillator.
/// - Input 0: frequency. The Lorenz system exhibits slight frequency effects.
/// - Output 0: system output
#[derive(Clone, Default)]
pub struct Lorenz {
    x: f32,
    y: f32,
    z: f32,
    sr: f32,
    hash: u64,
}

impl Lorenz {
    /// Create new Lorenz oscillator.
    pub fn new() -> Self {
        let mut lorenz = Self::default();
        lorenz.reset();
        lorenz.set_sample_rate(DEFAULT_SR);
        lorenz
    }
}

impl AudioNode for Lorenz {
    const ID: u64 = 74;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;

    fn reset(&mut self) {
        self.x = lerp(0.0, 1.0, rnd1(self.hash) as f32);
        self.y = 1.0;
        self.z = 1.0;
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sr = convert(sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let dx = 10.0 * (self.y - self.x);
        let dy = self.x * (28.0 - self.z) - self.y;
        let dz = self.x * self.y - (8.0 / 3.0) * self.z;
        let dt = input[0] / self.sr;
        self.x += dx * dt;
        self.y += dy * dt;
        self.z += dz * dt;
        [self.x * 0.05107].into()
    }

    fn set_hash(&mut self, hash: u64) {
        self.hash = hash;
        self.reset();
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        super::signal::Routing::Generator(0.0).route(input, self.outputs())
    }
}
