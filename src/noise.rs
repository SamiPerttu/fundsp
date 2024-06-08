//! Noise components.

use super::audionode::*;
use super::buffer::*;
use super::setting::*;
use super::signal::*;
use super::*;
use funutd::Rnd;
use numeric_array::*;

/// Maximum length sequences (MLS) are pseudorandom, spectrally flat,
/// binary white noise sequences with interesting properties.
/// We have pre-baked sequences with state space sizes from 1 to 31 bits.
#[derive(Copy, Clone)]
pub struct MlsState {
    /// State space size in bits.
    n: u32,
    /// Current state.
    s: u32,
}

// Feedback table for MLS sequence generation.
static MLS_POLY: [u32; 31] = [
    0b1,
    0b11,
    0b110,
    0b1100,
    0b10100,
    0b110000,
    0b1001000,
    0b10111000,
    0b100010000,
    0b1001000000,
    0b10100000000,
    0b110010100000,
    0b1101100000000,
    0b11000010001000,
    0b110000000000000,
    0b1101000000001000,
    0b10010000000000000,
    0b100000010000000000,
    0b1100011000000000000,
    0b10010000000000000000,
    0b101000000000000000000,
    0b1100000000000000000000,
    0b10000100000000000000000,
    0b111000010000000000000000,
    0b1001000000000000000000000,
    0b10000000000000000000100011,
    0b100000000000000000000010011,
    0b1001000000000000000000000000,
    0b10100000000000000000000000000,
    0b100000000000000000000000101001,
    0b1001000000000000000000000000000,
];

impl MlsState {
    /// Creates a MLS.
    /// Number of bits in the state space is n (1 <= n <= 31).
    pub fn new(n: u32) -> MlsState {
        assert!(n >= 1 && n <= 31);
        MlsState { n, s: (1 << n) - 1 }
    }

    /// Creates a MLS from seed.
    /// Number of bits in the state space is n (1 <= n <= 31).
    pub fn new_with_seed(n: u32, seed: u32) -> MlsState {
        assert!(n >= 1 && n <= 31);
        MlsState {
            n,
            s: 1 + seed % ((1 << n) - 1),
        }
    }

    /// Sequence length. The sequence repeats after 2**n - 1 steps.
    #[inline]
    pub fn length(self) -> u32 {
        (1 << self.n) - 1
    }

    /// Returns the next state in the sequence.
    pub fn next(self) -> MlsState {
        let feedback = MLS_POLY[(self.n - 1) as usize] & self.s;
        let parity = feedback.count_ones() & 1;
        MlsState {
            n: self.n,
            s: ((self.s << 1) | parity) & self.length(),
        }
    }

    /// The current value in the sequence, either 0 or 1.
    #[inline]
    pub fn value(self) -> u32 {
        (self.s >> (self.n - 1)) & 1
    }
}

/// MLS noise component.
/// - Output 0: noise.
#[derive(Clone)]
pub struct Mls {
    mls: MlsState,
    hash: u64,
}

impl Mls {
    pub fn new(mls: MlsState) -> Self {
        Self { mls, hash: 0 }
    }
}

impl AudioNode for Mls {
    const ID: u64 = 19;
    type Inputs = typenum::U0;
    type Outputs = typenum::U1;

    fn reset(&mut self) {
        self.mls = MlsState::new_with_seed(self.mls.n, (self.hash >> 32) as u32);
    }

    #[inline]
    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let value = self.mls.value() as f32;
        self.mls = self.mls.next();
        [value * 2.0 - 1.0].into()
    }

    fn set_hash(&mut self, hash: u64) {
        self.hash = hash;
        self.reset();
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        super::signal::Routing::Generator(0.0).route(input, self.outputs())
    }
}

/// 32-bit hash modified from a hash by degski. Extra high quality.
#[inline]
fn hash32f(x: u32) -> u32 {
    let x = (x ^ (x >> 16)).wrapping_mul(0x45d9f3b);
    let x = (x ^ (x >> 16)).wrapping_mul(0x45d9f3b);
    let x = (x ^ (x >> 16)).wrapping_mul(0x45d9f3b);
    x ^ (x >> 16)
}

/// 32-bit hash modified from a hash by degski.
/// Hashes many values at once. Extra high quality.
#[inline]
fn hash32f_simd(x: U32x) -> U32x {
    let m = U32x::splat(0x45d9f3b);
    let x = (x ^ (x >> 16)) * m;
    let x = (x ^ (x >> 16)) * m;
    let x = (x ^ (x >> 16)) * m;
    x ^ (x >> 16)
}

/// White noise component.
/// - Output 0: noise.
#[derive(Default, Clone)]
pub struct Noise {
    state: u32,
    hash: u64,
}

impl Noise {
    pub fn new() -> Self {
        Noise::default()
    }
}

impl AudioNode for Noise {
    const ID: u64 = 20;
    type Inputs = typenum::U0;
    type Outputs = typenum::U1;

    fn reset(&mut self) {
        self.state = self.hash as u32;
    }

    #[inline]
    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        self.state = self.state.wrapping_add(1);
        let value = (hash32f(self.state) >> 8) as f32 * (1.0 / (1 << 23) as f32) - 1.0;
        [value].into()
    }

    fn process(&mut self, size: usize, _input: &BufferRef, output: &mut BufferMut) {
        let mut state = U32x::new(core::array::from_fn(|i| {
            self.state.wrapping_add(i as u32 + 1)
        }));
        let output = output.channel_f32_mut(0);
        let m = 1.0 / (1 << 23) as f32;
        for i in 0..simd_items(size) {
            let value: U32x = hash32f_simd(state) >> 8;
            let value_ref = value.as_array_ref();
            for j in 0..SIMD_N {
                output[(i << SIMD_S) + j] = value_ref[j] as f32 * m - 1.0;
            }
            state += U32x::splat(SIMD_N as u32);
        }
        self.state = self.state.wrapping_add(size as u32);
    }

    fn set_hash(&mut self, hash: u64) {
        self.hash = hash;
        self.reset();
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Generator(0.0).route(input, self.outputs())
    }
}

/// Sample-and-hold component.
/// Setting: variability in 0...1 is the randomness in individual hold times.
/// - Input 0: signal.
/// - Input 1: sampling frequency (Hz).
/// - Output 0: sampled signal.
#[derive(Default, Clone)]
pub struct Hold {
    rnd: Rnd,
    hash: u64,
    sample_duration: f64,
    variability: f32,
    t: f64,
    next_t: f64,
    hold: f32,
}

impl Hold {
    /// Create new sample-and-hold component.
    /// Variability is the randomness in individual hold times in 0...1.
    pub fn new(variability: f32) -> Self {
        let mut node = Self {
            variability,
            ..Self::default()
        };
        node.reset();
        node.set_sample_rate(DEFAULT_SR);
        node
    }
    /// Variability is the randomness in individual hold times in 0...1.
    #[inline]
    pub fn variability(&self) -> f32 {
        self.variability
    }
    /// Set variability. Variability is the randomness in individual hold times in 0...1.
    #[inline]
    pub fn set_variability(&mut self, variability: f32) {
        self.variability = variability;
    }
}

impl AudioNode for Hold {
    const ID: u64 = 76;
    type Inputs = typenum::U2;
    type Outputs = typenum::U1;

    fn reset(&mut self) {
        self.rnd = Rnd::from_u64(self.hash);
        self.t = 0.0;
        self.next_t = 0.0;
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_duration = 1.0 / sample_rate;
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        if self.t >= self.next_t {
            self.hold = input[0];
            self.next_t = self.t
                + super::math::lerp(
                    1.0 - self.variability.to_f64(),
                    1.0 + self.variability.to_f64(),
                    self.rnd.f64(),
                ) / input[1].to_f64();
        }
        self.t += self.sample_duration;
        [self.hold].into()
    }

    fn set(&mut self, setting: Setting) {
        if let Parameter::Variability(variability) = setting.parameter() {
            self.set_variability(*variability);
        }
    }

    fn set_hash(&mut self, hash: u64) {
        self.hash = hash;
        self.reset();
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = SignalFrame::new(self.outputs());
        output.set(0, input.at(0).distort(0.0));
        output
    }
}
