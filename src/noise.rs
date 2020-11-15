use super::*;
use super::audiocomponent::*;
use super::math::*;
use numeric_array::*;

/// Maximum length sequences (MLS) are pseudorandom, spectrally flat,
/// binary white noise sequences with interesting properties.
/// We have pre-baked sequences with state space sizes from 1 to 31 bits.
#[derive(Copy, Clone)]
pub struct Mls {
    /// State space size in bits.
    n: u32,
    /// Current state.
    s: u32,
}

// Feedback table for MLS sequence generation.
static MLS_POLY : [u32; 31] =
    [
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
        0b1001000000000000000000000000000
    ];

impl Mls {

    /// Creates a MLS.
    /// Number of bits in the state space is n (1 <= n <= 31).
    pub fn new(n: u32) -> Mls {
        assert!(n >= 1 && n <= 31);
        Mls { n, s: (1 << n) - 1 }
    }

    /// Creates a default MLS with 29-bit state space
    /// (which repeats @ ~12174 seconds @ 44.1 kHz).
    pub fn new_default() -> Mls {
        Mls::new(29)
    }

    /// Creates a MLS from seed.
    /// Number of bits in the state space is n (1 <= n <= 31).
    pub fn new_with_seed(n: u32, seed: u32) -> Mls {
        assert!(n >= 1 && n <= 31);
        Mls { n, s: 1 + seed % ((1 << n) - 1) }
    }

    /// Sequence length. The sequence repeats after 2**n - 1 steps.
    pub fn length(self) -> u32 {
        (1 << self.n) - 1
    }

    /// Returns the next state in the sequence.
    pub fn next(self) -> Mls {
        let feedback = MLS_POLY[(self.n - 1) as usize] & self.s;
        let parity = feedback.count_ones() & 1;
        Mls { n: self.n, s: ((self.s << 1) | parity) & self.length() }
    }

    /// Returns the current value in the sequence, either 0 or 1.
    pub fn value(self) -> u32 {
        (self.s >> (self.n - 1)) & 1
    }
}

/// MLS noise component.
#[derive(Clone)]
pub struct MlsNoise {
    mls : Mls,
}

impl MlsNoise {
    pub fn new(mls : Mls) -> MlsNoise { MlsNoise { mls } }
    pub fn new_default() -> MlsNoise { MlsNoise::new(Mls::new_default()) }
}

impl AudioComponent for MlsNoise {
    type Inputs = typenum::U0;
    type Outputs = typenum::U1;

    fn reset(&mut self, _sample_rate: Option<f64>)
    {
        self.mls = Mls::new(self.mls.n);
    }

    #[inline] fn tick(&mut self, _input: &Frame<Self::Inputs>) -> Frame<Self::Outputs>
    {
        let value = self.mls.value() as f48;
        self.mls = self.mls.next();
        [value * 2.0 - 1.0].into()
    }
}

/// White noise component.
#[derive(Clone)]
pub struct NoiseComponent {
    x: u64,
}

impl NoiseComponent {
    pub fn new() -> NoiseComponent { NoiseComponent { x: 0 } }
}

impl AudioComponent for NoiseComponent {
    type Inputs = typenum::U0;
    type Outputs = typenum::U1;

    fn reset(&mut self, _sample_rate: Option<f64>)
    {
        self.x = 0;
    }

    #[inline] fn tick(&mut self, _input: &Frame<Self::Inputs>) -> Frame<Self::Outputs>
    {
        self.x = self.x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        // Pick some number of most significant bits from the linear congruential generator.
        let use_bits = 16;
        let value = (self.x >> (64 - use_bits)) as f48 / pow(2.0, (use_bits - 1) as f48) - 1.0;
        [value].into()
    }
}
