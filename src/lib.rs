#![allow(incomplete_features)]
#![feature(trait_alias)]

use num_complex::Complex64;
use utd::math::*;

/// Single precision floating point is used in audio buffers.
#[cfg(not(feature = "double_precision"))]
pub type F32 = f32;
/// Double precision floating point is used in audio buffers.
#[cfg(feature = "double_precision")]
pub type F32 = f64;

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

/// AudioFloat trait for audio processing.
pub trait AudioFloat = Real + Num + Default + AsPrimitive<F32> + Into<f64>;

/// Returns a Complex64 with real component x and imaginary component zero.
pub fn re<T: Into<f64>>(x: T) -> Complex64 { Complex64::new(x.into(), 0.0) }

/// Makes an AudioFloat.
#[cfg(not(feature = "double_precision"))]
pub fn afloat<F: AudioFloat>(x: F32) -> F { F::new_f32(x) }
/// Makes an AudioFloat.
#[cfg(feature = "double_precision")]
pub fn afloat<F: AudioFloat>(x: F32) -> F { F::new_f64(x) }

pub mod audiocomponent;
pub mod audiounit;
pub mod filter;
pub mod frame;
pub mod lti;
pub mod noise;
pub mod sample;
