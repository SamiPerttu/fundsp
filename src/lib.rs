#![cfg_attr(docsrs, feature(doc_cfg))]
//! FunDSP is an audio processing and synthesis library.
//!
//! See `README.md` in crate root folder for an overview.
//! For a list of changes, see `CHANGES.md` in the same folder.
//!
//! The central abstractions are located in the `audionode` and `audiounit` modules.
//! The `combinator` module defines the graph operators.
#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![allow(
    clippy::precedence,
    clippy::type_complexity,
    clippy::float_cmp,
    clippy::len_zero,
    clippy::double_neg,
    clippy::needless_range_loop,
    clippy::manual_range_contains,
    clippy::too_many_arguments,
    clippy::comparison_chain
)]

use core::cmp::PartialEq;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use core::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr};

/// Default sample rate is 44.1 kHz.
pub const DEFAULT_SR: f64 = 44_100.0;

/// Maximum buffer size for block processing is 64 samples.
pub const MAX_BUFFER_SIZE: usize = 64;

/// Blocks are explicitly SIMD accelerated. This is the type of a SIMD element
/// containing successive `f32` samples.
pub type F32x = wide::f32x8;

/// The 32-bit unsigned integer SIMD element corresponding to `F32x`.
pub type U32x = wide::u32x8;

/// The 32-bit signed integer SIMD element corresponding to `F32x`.
pub type I32x = wide::i32x8;

/// Right shift for converting from samples to SIMD elements.
pub const SIMD_S: usize = 3;

/// Blocks are explicitly SIMD accelerated. This is the length of a SIMD element in `f32` samples.
pub const SIMD_N: usize = 1 << SIMD_S;

/// "SIMD mask", `SIMD_N` minus one.
pub const SIMD_M: usize = SIMD_N - 1;

/// Left shift for converting from channel number to SIMD index.
// Linked to MAX_BUFFER_SIZE = 1 << 6.
pub const SIMD_C: usize = 6 - SIMD_S;

/// The length of a buffer in SIMD elements.
pub const SIMD_LEN: usize = MAX_BUFFER_SIZE / SIMD_N;

/// Convert amount from samples to (full or partial) SIMD elements.
#[inline(always)]
pub fn simd_items(samples: usize) -> usize {
    (samples + SIMD_M) >> SIMD_S
}

/// Convert amount from samples to full SIMD elements.
#[inline(always)]
pub fn full_simd_items(samples: usize) -> usize {
    samples >> SIMD_S
}

/// Number abstraction.
pub trait Num:
    Copy
    + Default
    + Send
    + Sync
    + core::fmt::Display
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + AddAssign
    + MulAssign
    + SubAssign
    + DivAssign
    + PartialEq
{
    fn zero() -> Self;
    fn one() -> Self;
    fn new(x: i64) -> Self;
    fn from_f64(x: f64) -> Self;
    fn from_f32(x: f32) -> Self;
    fn abs(self) -> Self;
    fn signum(self) -> Self;
    // Note that in numerical code we do not want to define min() and max() in terms of comparisons.
    // It is inadvisable in general to link traits like this; Min and Max traits would be preferable.
    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
    fn pow(self, other: Self) -> Self;
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn round(self) -> Self;
}

macro_rules! impl_signed_num {
    ( $($t:ty),* ) => {
    $( impl Num for $t {
        #[inline(always)] fn zero() -> Self { 0 }
        #[inline(always)] fn one() -> Self { 1 }
        #[inline(always)] fn new(x: i64) -> Self { x as Self }
        #[inline(always)] fn from_f64(x: f64) -> Self { x as Self }
        #[inline(always)] fn from_f32(x: f32) -> Self { x as Self }
        #[inline(always)] fn abs(self) -> Self { <$t>::abs(self) }
        #[inline(always)] fn signum(self) -> Self { <$t>::signum(self) }
        #[inline(always)] fn min(self, other: Self) -> Self { core::cmp::min(self, other) }
        #[inline(always)] fn max(self, other: Self) -> Self { core::cmp::max(self, other) }
        #[inline(always)] fn pow(self, other: Self) -> Self { <$t>::pow(self, other as u32) }
        #[inline(always)] fn floor(self) -> Self { self }
        #[inline(always)] fn ceil(self) -> Self { self }
        #[inline(always)] fn round(self) -> Self { self }
    }) *
    }
}
impl_signed_num! { i8, i16, i32, i64, i128, isize }

macro_rules! impl_unsigned_num {
    ( $($t:ty),* ) => {
    $( impl Num for $t {
        #[inline(always)] fn zero() -> Self { 0 }
        #[inline(always)] fn one() -> Self { 1 }
        #[inline(always)] fn new(x: i64) -> Self { x as Self }
        #[inline(always)] fn from_f64(x: f64) -> Self { x as Self }
        #[inline(always)] fn from_f32(x: f32) -> Self { x as Self }
        #[inline(always)] fn abs(self) -> Self { self }
        #[inline(always)] fn signum(self) -> Self { 1 }
        #[inline(always)] fn min(self, other: Self) -> Self { core::cmp::min(self, other) }
        #[inline(always)] fn max(self, other: Self) -> Self { core::cmp::max(self, other) }
        #[inline(always)] fn pow(self, other: Self) -> Self { <$t>::pow(self, other as u32) }
        #[inline(always)] fn floor(self) -> Self { self }
        #[inline(always)] fn ceil(self) -> Self { self }
        #[inline(always)] fn round(self) -> Self { self }
    }) *
    }
}
impl_unsigned_num! { u8, u16, u32, u64, u128, usize }

impl Num for f32 {
    #[inline(always)]
    fn zero() -> Self {
        0.0
    }
    #[inline(always)]
    fn one() -> Self {
        1.0
    }
    #[inline(always)]
    fn new(x: i64) -> Self {
        x as Self
    }
    #[inline(always)]
    fn from_f64(x: f64) -> Self {
        x as Self
    }
    #[inline(always)]
    fn from_f32(x: f32) -> Self {
        x as Self
    }
    #[inline(always)]
    fn abs(self) -> Self {
        libm::fabsf(self)
    }
    #[inline(always)]
    fn signum(self) -> Self {
        libm::copysignf(1.0, self)
    }
    #[inline(always)]
    fn min(self, other: Self) -> Self {
        self.min(other)
    }
    #[inline(always)]
    fn max(self, other: Self) -> Self {
        self.max(other)
    }
    #[inline(always)]
    fn pow(self, other: Self) -> Self {
        libm::powf(self, other)
    }
    #[inline(always)]
    fn floor(self) -> Self {
        libm::floorf(self)
    }
    #[inline(always)]
    fn ceil(self) -> Self {
        libm::ceilf(self)
    }
    #[inline(always)]
    fn round(self) -> Self {
        libm::roundf(self)
    }
}

impl Num for f64 {
    #[inline(always)]
    fn zero() -> Self {
        0.0
    }
    #[inline(always)]
    fn one() -> Self {
        1.0
    }
    #[inline(always)]
    fn new(x: i64) -> Self {
        x as Self
    }
    #[inline(always)]
    fn from_f64(x: f64) -> Self {
        x as Self
    }
    #[inline(always)]
    fn from_f32(x: f32) -> Self {
        x as Self
    }
    #[inline(always)]
    fn abs(self) -> Self {
        libm::fabs(self)
    }
    #[inline(always)]
    fn signum(self) -> Self {
        libm::copysign(1.0, self)
    }
    #[inline(always)]
    fn min(self, other: Self) -> Self {
        self.min(other)
    }
    #[inline(always)]
    fn max(self, other: Self) -> Self {
        self.max(other)
    }
    #[inline(always)]
    fn pow(self, other: Self) -> Self {
        libm::pow(self, other)
    }
    #[inline(always)]
    fn floor(self) -> Self {
        libm::floor(self)
    }
    #[inline(always)]
    fn ceil(self) -> Self {
        libm::ceil(self)
    }
    #[inline(always)]
    fn round(self) -> Self {
        libm::round(self)
    }
}

impl Num for F32x {
    #[inline(always)]
    fn zero() -> Self {
        F32x::ZERO
    }
    #[inline(always)]
    fn one() -> Self {
        F32x::ONE
    }
    #[inline(always)]
    fn new(x: i64) -> Self {
        F32x::splat(x as f32)
    }
    #[inline(always)]
    fn from_f64(x: f64) -> Self {
        F32x::splat(x as f32)
    }
    #[inline(always)]
    fn from_f32(x: f32) -> Self {
        F32x::splat(x)
    }
    #[inline(always)]
    fn abs(self) -> Self {
        self.abs()
    }
    #[inline(always)]
    fn signum(self) -> Self {
        F32x::ONE.copysign(self)
    }
    #[inline(always)]
    fn min(self, other: Self) -> Self {
        self.fast_min(other)
    }
    #[inline(always)]
    fn max(self, other: Self) -> Self {
        self.fast_max(other)
    }
    #[inline(always)]
    fn pow(self, other: Self) -> Self {
        self.pow_f32x8(other)
    }
    #[inline(always)]
    fn floor(self) -> Self {
        (self - 0.4999999).round()
    }
    #[inline(always)]
    fn ceil(self) -> Self {
        (self + 0.4999999).round()
    }
    #[inline(always)]
    fn round(self) -> Self {
        self.round()
    }
}

/// Integer abstraction.
pub trait Int:
    Num
    + PartialOrd
    + Not<Output = Self>
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitXor<Output = Self>
    + Shl<usize, Output = Self>
    + Shr<usize, Output = Self>
{
    fn wrapping_add(self, other: Self) -> Self;
    fn wrapping_sub(self, other: Self) -> Self;
    fn wrapping_mul(self, other: Self) -> Self;
}

macro_rules! impl_int {
    ( $($t:ty),* ) => {
    $( impl Int for $t {
        #[inline(always)] fn wrapping_add(self, other: Self) -> Self { <$t>::wrapping_add(self, other) }
        #[inline(always)] fn wrapping_sub(self, other: Self) -> Self { <$t>::wrapping_sub(self, other) }
        #[inline(always)] fn wrapping_mul(self, other: Self) -> Self { <$t>::wrapping_mul(self, other) }
    }) *
    }
}
impl_int! { i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize }

/// Float abstraction.
pub trait Float: Num + PartialOrd + Neg<Output = Self> {
    const PI: Self;
    const TAU: Self;
    const SQRT_2: Self;
    fn from_float<T: Float>(x: T) -> Self;
    fn to_f64(self) -> f64;
    fn to_f32(self) -> f32;
    fn to_i64(self) -> i64;
}

impl Float for f32 {
    const PI: Self = core::f32::consts::PI;
    const TAU: Self = core::f32::consts::TAU;
    const SQRT_2: Self = core::f32::consts::SQRT_2;

    #[inline(always)]
    fn from_float<T: Float>(x: T) -> Self {
        x.to_f32()
    }

    #[inline(always)]
    fn to_f64(self) -> f64 {
        self as f64
    }

    #[inline(always)]
    fn to_f32(self) -> f32 {
        self
    }

    #[inline(always)]
    fn to_i64(self) -> i64 {
        self as i64
    }
}

impl Float for f64 {
    const PI: Self = core::f64::consts::PI;
    const TAU: Self = core::f64::consts::TAU;
    const SQRT_2: Self = core::f64::consts::SQRT_2;

    #[inline(always)]
    fn from_float<T: Float>(x: T) -> Self {
        x.to_f64()
    }

    #[inline(always)]
    fn to_f64(self) -> f64 {
        self
    }

    #[inline(always)]
    fn to_f32(self) -> f32 {
        self as f32
    }

    #[inline(always)]
    fn to_i64(self) -> i64 {
        self as i64
    }
}

/// Generic floating point conversion function.
#[inline(always)]
pub fn convert<T: Float, U: Float>(x: T) -> U {
    U::from_float(x)
}

/// Refined float abstraction.
pub trait Real: Float {
    fn sqrt(self) -> Self;
    fn exp(self) -> Self;
    fn exp2(self) -> Self;
    fn log(self) -> Self;
    fn log2(self) -> Self;
    fn log10(self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn tan(self) -> Self;
    fn tanh(self) -> Self;
    fn atan(self) -> Self;
}

impl Real for f32 {
    #[inline(always)]
    fn sqrt(self) -> Self {
        libm::sqrtf(self)
    }
    #[inline(always)]
    fn exp(self) -> Self {
        libm::expf(self)
    }
    #[inline(always)]
    fn exp2(self) -> Self {
        libm::exp2f(self)
    }
    #[inline(always)]
    fn log(self) -> Self {
        libm::logf(self)
    }
    #[inline(always)]
    fn log2(self) -> Self {
        libm::log2f(self)
    }
    #[inline(always)]
    fn log10(self) -> Self {
        libm::log10f(self)
    }
    #[inline(always)]
    fn sin(self) -> Self {
        libm::sinf(self)
    }
    #[inline(always)]
    fn cos(self) -> Self {
        libm::cosf(self)
    }
    #[inline(always)]
    fn tan(self) -> Self {
        libm::tanf(self)
    }
    #[inline(always)]
    fn tanh(self) -> Self {
        libm::tanhf(self)
    }
    #[inline(always)]
    fn atan(self) -> Self {
        libm::atanf(self)
    }
}

impl Real for f64 {
    #[inline(always)]
    fn sqrt(self) -> Self {
        libm::sqrt(self)
    }
    #[inline(always)]
    fn exp(self) -> Self {
        libm::exp(self)
    }
    #[inline(always)]
    fn exp2(self) -> Self {
        libm::exp2(self)
    }
    #[inline(always)]
    fn log(self) -> Self {
        libm::log(self)
    }
    #[inline(always)]
    fn log2(self) -> Self {
        libm::log2(self)
    }
    #[inline(always)]
    fn log10(self) -> Self {
        libm::log10(self)
    }
    #[inline(always)]
    fn sin(self) -> Self {
        libm::sin(self)
    }
    #[inline(always)]
    fn cos(self) -> Self {
        libm::cos(self)
    }
    #[inline(always)]
    fn tan(self) -> Self {
        libm::tan(self)
    }
    #[inline(always)]
    fn tanh(self) -> Self {
        libm::tanh(self)
    }
    #[inline(always)]
    fn atan(self) -> Self {
        libm::atan(self)
    }
}

pub mod adsr;
pub mod audionode;
pub mod audiounit;
pub mod buffer;
pub mod combinator;
pub mod delay;
pub mod dynamics;
pub mod envelope;
pub mod feedback;
pub mod fft;
pub mod filter;
pub mod fir;
pub mod follow;
pub mod gen;
pub mod granular;
pub mod hacker;
pub mod hacker32;
pub mod math;
pub mod moog;
pub mod net;
pub mod noise;
pub mod oscillator;
pub mod oversample;
pub mod pan;
pub mod prelude;
pub mod realnet;
pub mod realseq;
pub mod resample;
pub mod resynth;
pub mod reverb;
pub mod rez;
pub mod sequencer;
pub mod setting;
pub mod shape;
pub mod shared;
pub mod signal;
pub mod slot;
pub mod snoop;
pub mod sound;
pub mod svf;
pub mod system;
pub mod vertex;
pub mod wave;
pub mod wavetable;

// GenericSequence is for Frame::generate.
pub use numeric_array::{self, generic_array::sequence::GenericSequence, typenum};

pub use funutd;
pub use thingbuf;
pub use wide;

#[cfg(feature = "std")]
pub mod write;

#[cfg(all(feature = "std", feature = "files"))]
pub mod read;
