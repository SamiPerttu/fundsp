use std::ops::{Add, Sub, Mul, Div, Neg};
use std::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr};
use std::cmp::PartialEq;

/// Single precision floating point is used in audio buffers.
#[cfg(not(feature = "double_precision"))]
pub type f48 = f32;
/// Double precision floating point is used in audio buffers.
#[cfg(feature = "double_precision")]
pub type f48 = f64;

pub type Frame<Length> = numeric_array::NumericArray<f48, Length>;

/// Default sample rate is 44.1 khz.
#[cfg(not(any(feature = "forty_eight_khz", feature = "eighty_eight_point_two_khz", feature = "ninety_six_khz", feature = "one_hundred_seventy_six_point_four_khz", feature = "one_hundred_ninety_two_khz" )))]
pub const DEFAULT_SR: f64 = 44100.0;
/// Default sample rate is 48 khz.
#[cfg(feature = "forty_eight_khz")]
pub const DEFAULT_SR: f64 = 48000.0;
/// Default sample rate is 88.2 khz.
#[cfg(feature = "eighty_eight_point_two_khz")]
pub const DEFAULT_SR: f64 = 88200.0;
/// Default sample rate is 96 khz.
#[cfg(feature = "ninety_six_khz")]
pub const DEFAULT_SR: f64 = 96000.0;
/// Default sample rate is 176.4 khz.
#[cfg(feature = "one_hundred_seventy_six_point_four_khz")]
pub const DEFAULT_SR: f64 = 176400.0;
/// Default sample rate is 192 khz.
#[cfg(feature = "one_hundred_ninety_two_khz")]
pub const DEFAULT_SR: f64 = 19200.0;

pub trait Num: Copy
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
{
    fn zero() -> Self;
    fn one() -> Self;
    fn new(x: i64) -> Self;
    fn from_u64(x: u64) -> Self;
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

#[inline] pub fn abs<T: Num>(x: T) -> T { x.abs() }
#[inline] pub fn signum<T: Num>(x: T) -> T { x.signum() }
#[inline] pub fn min<T: Num>(x: T, y: T) -> T { x.min(y) }
#[inline] pub fn max<T: Num>(x: T, y: T) -> T { x.max(y) }
#[inline] pub fn pow<T: Num>(x: T, y: T) -> T { x.pow(y) }
#[inline] pub fn floor<T: Num>(x: T) -> T { x.floor() }
#[inline] pub fn ceil<T: Num>(x: T) -> T { x.ceil() }
#[inline] pub fn round<T: Num>(x: T) -> T { x.round() }

macro_rules! impl_signed_num {
    ( $($t:ty),* ) => {
    $( impl Num for $t {
        #[inline] fn zero() -> Self { 0 }
        #[inline] fn one() -> Self { 1 }
        #[inline] fn new(x: i64) -> Self { x as Self }
        #[inline] fn from_u64(x: u64) -> Self { x as Self }
        #[inline] fn from_f64(x: f64) -> Self { x as Self }
        #[inline] fn from_f32(x: f32) -> Self { x as Self }
        #[inline] fn abs(self) -> Self { <$t>::abs(self) }
        #[inline] fn signum(self) -> Self { <$t>::signum(self) }
        #[inline] fn min(self, other: Self) -> Self { std::cmp::min(self, other) }
        #[inline] fn max(self, other: Self) -> Self { std::cmp::max(self, other) }
        #[inline] fn pow(self, other: Self) -> Self { <$t>::pow(self, other as u32) }
        #[inline] fn floor(self) -> Self { self }
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
        #[inline] fn from_u64(x: u64) -> Self { x as Self }
        #[inline] fn from_f64(x: f64) -> Self { x as Self }
        #[inline] fn from_f32(x: f32) -> Self { x as Self }
        #[inline] fn abs(self) -> Self { self }
        #[inline] fn signum(self) -> Self { 1 }
        #[inline] fn min(self, other: Self) -> Self { std::cmp::min(self, other) }
        #[inline] fn max(self, other: Self) -> Self { std::cmp::max(self, other) }
        #[inline] fn pow(self, other: Self) -> Self { <$t>::pow(self, other as u32) }
        #[inline] fn floor(self) -> Self { self }
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
        #[inline] fn from_u64(x: u64) -> Self { x as Self }
        #[inline] fn from_f64(x: f64) -> Self { x as Self }
        #[inline] fn from_f32(x: f32) -> Self { x as Self }
        #[inline] fn abs(self) -> Self { <$t>::abs(self) }
        #[inline] fn signum(self) -> Self { <$t>::signum(self) }
        #[inline] fn min(self, other: Self) -> Self { <$t>::min(self, other) }
        #[inline] fn max(self, other: Self) -> Self { <$t>::max(self, other) }
        #[inline] fn pow(self, other: Self) -> Self { <$t>::powf(self, other) }
        #[inline] fn floor(self) -> Self { <$t>::floor(self) }
        #[inline] fn ceil(self) -> Self { <$t>::ceil(self) }
        #[inline] fn round(self) -> Self { <$t>::round(self) }
    }) *
    }
}
impl_float_num! { f32, f64 }

pub trait Int: Num
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

pub trait Real : Copy
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
    + PartialEq
{
    fn sqrt(self) -> Self;
    fn exp(self) -> Self;
    fn log(self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn tan(self) -> Self;
    fn tanh(self) -> Self;
}

#[inline] pub fn sqrt<T: Real>(x: T) -> T { x.sqrt() }
#[inline] pub fn exp<T: Real>(x: T) -> T { x.exp() }
#[inline] pub fn log<T: Real>(x: T) -> T { x.log() }
#[inline] pub fn sin<T: Real>(x: T) -> T { x.sin() }
#[inline] pub fn cos<T: Real>(x: T) -> T { x.cos() }
#[inline] pub fn tan<T: Real>(x: T) -> T { x.tan() }
#[inline] pub fn tanh<T: Real>(x: T) -> T { x.tanh() }

macro_rules! impl_real {
    ( $($t:ty),* ) => {
    $( impl Real for $t {
        #[inline] fn sqrt(self) -> Self { self.sqrt() }    
        #[inline] fn exp(self) -> Self { self.exp() }
        #[inline] fn log(self) -> Self { self.ln() }
        #[inline] fn sin(self) -> Self { self.sin() }
        #[inline] fn cos(self) -> Self { self.cos() }
        #[inline] fn tan(self) -> Self { <$t>::tan(self) }
        #[inline] fn tanh(self) -> Self { <$t>::tanh(self) }
    }) *
    }
}
impl_real! { f32, f64 }

pub trait AsPrimitive<T: Copy>: Copy
{
    /// Convert a value using the as operator.
    fn as_(self) -> T;
}

macro_rules! impl_as_primitive {
    (@ $T: ty => $(#[$cfg:meta])* impl $U: ty ) => {
        $(#[$cfg])*
        impl AsPrimitive<$U> for $T {
            #[inline] fn as_(self) -> $U { self as $U }
        }
    };
    (@ $T: ty => { $( $U: ty ),* } ) => {$(
        impl_as_primitive!(@ $T => impl $U);
    )*};
    ($T: ty => { $( $U: ty ),* } ) => {
        impl_as_primitive!(@ $T => { $( $U ),* });
        impl_as_primitive!(@ $T => { u8, u16, u32, u64, u128, usize });
        impl_as_primitive!(@ $T => { i8, i16, i32, i64, i128, isize });
    };
}

impl_as_primitive!(u8 => { char, f32, f64 });
impl_as_primitive!(i8 => { f32, f64 });
impl_as_primitive!(u16 => { f32, f64 });
impl_as_primitive!(i16 => { f32, f64 });
impl_as_primitive!(u32 => { f32, f64 });
impl_as_primitive!(i32 => { f32, f64 });
impl_as_primitive!(u64 => { f32, f64 });
impl_as_primitive!(i64 => { f32, f64 });
impl_as_primitive!(u128 => { f32, f64 });
impl_as_primitive!(i128 => { f32, f64 });
impl_as_primitive!(usize => { f32, f64 });
impl_as_primitive!(isize => { f32, f64 });
impl_as_primitive!(f32 => { f32, f64 });
impl_as_primitive!(f64 => { f32, f64 });
impl_as_primitive!(char => { char });
impl_as_primitive!(bool => {});

/// AudioFloat trait for audio processing.
pub trait AudioFloat: Real + Num + Default + AsPrimitive<f48> + Into<f64> {}

impl AudioFloat for f32 {}
impl AudioFloat for f64 {}

/// Converts an f48 into an AudioFloat.
#[cfg(not(feature = "double_precision"))]
pub fn afloat<F: AudioFloat>(x: f48) -> F { F::from_f32(x) }
/// Converts an f48 into an AudioFloat.
#[cfg(feature = "double_precision")]
pub fn afloat<F: AudioFloat>(x: f48) -> F { F::from_f64(x) }

pub mod audiocomponent;
pub mod audiounit;
pub mod envelope;
pub mod filter;
pub mod lti;
pub mod noise;
pub mod prelude;
pub mod sample;
