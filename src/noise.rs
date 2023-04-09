//! Noise components.

use super::audionode::*;
use super::math::*;
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
    pub fn value(self) -> u32 {
        (self.s >> (self.n - 1)) & 1
    }
}

/// MLS noise component.
/// - Output 0: noise.
#[derive(Clone)]
pub struct Mls<T> {
    _marker: std::marker::PhantomData<T>,
    mls: MlsState,
    hash: u64,
}

impl<T: Float> Mls<T> {
    pub fn new(mls: MlsState) -> Self {
        Self {
            _marker: std::marker::PhantomData,
            mls,
            hash: 0,
        }
    }
}

impl<T: Float> AudioNode for Mls<T> {
    const ID: u64 = 19;
    type Sample = T;
    type Inputs = typenum::U0;
    type Outputs = typenum::U1;
    type Setting = ();

    fn reset(&mut self, _sample_rate: Option<f64>) {
        self.mls = MlsState::new_with_seed(self.mls.n, (self.hash >> 32) as u32);
    }

    #[inline]
    fn tick(
        &mut self,
        _input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let value = T::new(self.mls.value() as i64);
        self.mls = self.mls.next();
        [value * T::new(2) - T::new(1)].into()
    }

    #[inline]
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

/// White noise component.
/// - Output 0: noise.
#[derive(Default, Clone)]
pub struct Noise<T> {
    _marker: std::marker::PhantomData<T>,
    rnd: Rnd,
    hash: u64,
}

impl<T: Float> Noise<T> {
    pub fn new() -> Self {
        Noise::default()
    }
}

impl<T: Float> AudioNode for Noise<T> {
    const ID: u64 = 20;
    type Sample = T;
    type Inputs = typenum::U0;
    type Outputs = typenum::U1;
    type Setting = ();

    fn reset(&mut self, _sample_rate: Option<f64>) {
        self.rnd = Rnd::from_u64(self.hash);
    }

    #[inline]
    fn tick(
        &mut self,
        _input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let value = T::from_f32(self.rnd.f32() * 2.0 - 1.0);
        [value].into()
    }

    #[inline]
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

/// Sample-and-hold component.
/// Setting: variability in 0...1 is the randomness in individual hold times.
/// - Input 0: signal.
/// - Input 1: sampling frequency (Hz).
/// - Output 0: sampled signal.
#[derive(Default, Clone)]
pub struct Hold<T> {
    rnd: Rnd,
    hash: u64,
    sample_duration: f64,
    variability: T,
    t: f64,
    next_t: f64,
    hold: T,
}

impl<T: Float> Hold<T> {
    /// Create new sample-and-hold component.
    /// Variability is the randomness in individual hold times in 0...1.
    pub fn new(variability: T) -> Self {
        let mut node = Self {
            variability,
            ..Self::default()
        };
        node.reset(Some(DEFAULT_SR));
        node
    }
    /// Variability is the randomness in individual hold times in 0...1.
    pub fn variability(&self) -> T {
        self.variability
    }
    /// Set variability. Variability is the randomness in individual hold times in 0...1.
    pub fn set_variability(&mut self, variability: T) {
        self.variability = variability;
    }
}

impl<T: Float> AudioNode for Hold<T> {
    const ID: u64 = 76;
    type Sample = T;
    type Inputs = typenum::U2;
    type Outputs = typenum::U1;
    type Setting = T;

    fn set(&mut self, setting: T) {
        self.set_variability(setting);
    }

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.rnd = Rnd::from_u64(self.hash);
        self.t = 0.0;
        self.next_t = 0.0;
        if let Some(sr) = sample_rate {
            self.sample_duration = 1.0 / sr;
        }
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        if self.t >= self.next_t {
            self.hold = input[0];
            self.next_t = self.t
                + lerp(
                    1.0 - self.variability.to_f64(),
                    1.0 + self.variability.to_f64(),
                    self.rnd.f64(),
                ) / input[1].to_f64();
        }
        self.t += self.sample_duration;
        [self.hold].into()
    }

    #[inline]
    fn set_hash(&mut self, hash: u64) {
        self.hash = hash;
        self.reset(None);
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].distort(0.0);
        output
    }
}
