#![cfg_attr(docsrs, feature(doc_cfg))]
//! FunDSP is an audio processing and synthesis library.
//!
//! See `README.md` in crate root folder for an overview.

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

#[macro_use]
pub extern crate lazy_static;

use std::cmp::PartialEq;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use std::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr};

/// Default sample rate is 44.1 khz.
pub const DEFAULT_SR: f64 = 44_100.0;

/// Maximum buffer size for block processing is 64 samples.
pub const MAX_BUFFER_SIZE: usize = 64;

/// Number abstraction.
pub trait Num:
    Copy
    + Default
    + std::fmt::Display
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + AddAssign
    + MulAssign
    + SubAssign
    + DivAssign
    + PartialEq
    + PartialOrd
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
    fn fract(self) -> Self;
    fn ceil(self) -> Self;
    fn round(self) -> Self;
}

macro_rules! impl_signed_num {
    ( $($t:ty),* ) => {
    $( impl Num for $t {
        #[inline] fn zero() -> Self { 0 }
        #[inline] fn one() -> Self { 1 }
        #[inline] fn new(x: i64) -> Self { x as Self }
        #[inline] fn from_f64(x: f64) -> Self { x as Self }
        #[inline] fn from_f32(x: f32) -> Self { x as Self }
        #[inline] fn abs(self) -> Self { <$t>::abs(self) }
        #[inline] fn signum(self) -> Self { <$t>::signum(self) }
        #[inline] fn min(self, other: Self) -> Self { std::cmp::min(self, other) }
        #[inline] fn max(self, other: Self) -> Self { std::cmp::max(self, other) }
        #[inline] fn pow(self, other: Self) -> Self { <$t>::pow(self, other as u32) }
        #[inline] fn floor(self) -> Self { self }
        #[inline] fn fract(self) -> Self { self }
        #[inline] fn ceil(self) -> Self { self }
        #[inline] fn round(self) -> Self { self }
    }) *
    }
}
impl_signed_num! { i8, i16, i32, i64, i128, isize }

macro_rules! impl_unsigned_num {
    ( $($t:ty),* ) => {
    $( impl Num for $t {
        #[inline] fn zero() -> Self { 0 }
        #[inline] fn one() -> Self { 1 }
        #[inline] fn new(x: i64) -> Self { x as Self }
        #[inline] fn from_f64(x: f64) -> Self { x as Self }
        #[inline] fn from_f32(x: f32) -> Self { x as Self }
        #[inline] fn abs(self) -> Self { self }
        #[inline] fn signum(self) -> Self { 1 }
        #[inline] fn min(self, other: Self) -> Self { std::cmp::min(self, other) }
        #[inline] fn max(self, other: Self) -> Self { std::cmp::max(self, other) }
        #[inline] fn pow(self, other: Self) -> Self { <$t>::pow(self, other as u32) }
        #[inline] fn floor(self) -> Self { self }
        #[inline] fn fract(self) -> Self { self }
        #[inline] fn ceil(self) -> Self { self }
        #[inline] fn round(self) -> Self { self }
    }) *
    }
}
impl_unsigned_num! { u8, u16, u32, u64, u128, usize }

macro_rules! impl_float_num {
    ( $($t:ty),* ) => {
    $( impl Num for $t {
        #[inline] fn zero() -> Self { 0.0 }
        #[inline] fn one() -> Self { 1.0 }
        #[inline] fn new(x: i64) -> Self { x as Self }
        #[inline] fn from_f64(x: f64) -> Self { x as Self }
        #[inline] fn from_f32(x: f32) -> Self { x as Self }
        #[inline] fn abs(self) -> Self { <$t>::abs(self) }
        #[inline] fn signum(self) -> Self { <$t>::signum(self) }
        #[inline] fn min(self, other: Self) -> Self { <$t>::min(self, other) }
        #[inline] fn max(self, other: Self) -> Self { <$t>::max(self, other) }
        #[inline] fn pow(self, other: Self) -> Self { <$t>::powf(self, other) }
        #[inline] fn floor(self) -> Self { <$t>::floor(self) }
        #[inline] fn fract(self) -> Self { <$t>::fract(self) }
        #[inline] fn ceil(self) -> Self { <$t>::ceil(self) }
        #[inline] fn round(self) -> Self { <$t>::round(self) }
    }) *
    }
}
impl_float_num! { f32, f64 }

/// Integer abstraction.
pub trait Int:
    Num
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
        #[inline] fn wrapping_add(self, other: Self) -> Self { <$t>::wrapping_add(self, other) }
        #[inline] fn wrapping_sub(self, other: Self) -> Self { <$t>::wrapping_sub(self, other) }
        #[inline] fn wrapping_mul(self, other: Self) -> Self { <$t>::wrapping_mul(self, other) }
    }) *
    }
}
impl_int! { i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize }

/// Float abstraction.
pub trait Float: Num + Neg<Output = Self> {
    fn from_float<T: Float>(x: T) -> Self;
    fn to_f64(self) -> f64;
    fn to_f32(self) -> f32;
    fn to_i64(self) -> i64;
}

impl Float for f32 {
    fn from_float<T: Float>(x: T) -> Self {
        x.to_f32()
    }

    fn to_f64(self) -> f64 {
        self as f64
    }

    fn to_f32(self) -> f32 {
        self
    }

    fn to_i64(self) -> i64 {
        self as i64
    }
}

impl Float for f64 {
    fn from_float<T: Float>(x: T) -> Self {
        x.to_f64()
    }

    fn to_f64(self) -> f64 {
        self
    }

    fn to_f32(self) -> f32 {
        self as f32
    }

    fn to_i64(self) -> i64 {
        self as i64
    }
}

/// Generic floating point conversion function.
pub fn convert<T: Float, U: Float>(x: T) -> U {
    U::from_float(x)
}

/// Refined float abstraction.
pub trait Real: Num + Float {
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
}

macro_rules! impl_real {
    ( $($t:ty),* ) => {
    $( impl Real for $t {
        #[inline] fn sqrt(self) -> Self { self.sqrt() }
        #[inline] fn exp(self) -> Self { self.exp() }
        #[inline] fn exp2(self) -> Self { self.exp2() }
        #[inline] fn log(self) -> Self { self.ln() }
        #[inline] fn log2(self) -> Self { self.log2() }
        #[inline] fn log10(self) -> Self { self.log10() }
        #[inline] fn sin(self) -> Self { self.sin() }
        #[inline] fn cos(self) -> Self { self.cos() }
        #[inline] fn tan(self) -> Self { <$t>::tan(self) }
        #[inline] fn tanh(self) -> Self { <$t>::tanh(self) }
    }) *
    }
}
impl_real! { f32, f64 }

pub mod audionode;
pub mod audiounit;
pub mod buffer;
pub mod combinator;
pub mod delay;
pub mod dynamics;
pub mod effect;
pub mod envelope;
pub mod feedback;
pub mod filter;
pub mod fir;
pub mod hacker;
pub mod hacker32;
pub mod math;
pub mod moog;
pub mod net;
pub mod noise;
pub mod oscillator;
pub mod oversample;
pub mod pan;
pub mod pattern;
pub mod prelude;
pub mod scale;
pub mod sequencer;
pub mod shape;
pub mod signal;
pub mod sound;
pub mod svf;
pub mod wave;
pub mod wavetable;

// For Frame::generate.
pub use generic_array::sequence::GenericSequence;
