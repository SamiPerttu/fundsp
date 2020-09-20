use num_complex::Complex64;
use num_traits::Float;
use num_traits::AsPrimitive;

/// Single precision floating point is used in audio buffers and as glue between audio components.
#[cfg(feature = "double_precision")]
pub type F32 = f64;
/// Double precision floating point is used in audio buffers and as glue between audio components.
#[cfg(not(feature = "double_precision"))]
pub type F32 = f32;

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
pub trait AudioFloat = Float + Default + Into<f64> + AsPrimitive<F32> where f64: AsPrimitive<Self>, f32: AsPrimitive<Self>;

/// Returns a Complex64 with real component x and imaginary component zero.
pub fn re<T: Into<f64>>(x: T) -> Complex64 { Complex64::new(x.into(), 0.0) }
pub fn tan<F: Float>(x : F) -> F { x.tan() }
pub fn exp<F: Float>(x : F) -> F { x.exp() }
pub fn sin<F: Float>(x : F) -> F { x.sin() }
pub fn cos<F: Float>(x : F) -> F { x.cos() }
pub fn sqrt<F: Float>(x : F) -> F { x.sqrt() }

pub const PI: f64 = std::f64::consts::PI;
pub const TAU: f64 = std::f64::consts::TAU;
pub const SQRT_2: f64 = std::f64::consts::SQRT_2;

/// Lossy conversion trait for converting between floating point types.
pub trait LossyUnto<F> {
    fn unto(self) -> F;
}
impl LossyUnto<f64> for f32 {
    fn unto(self) -> f64 { self as f64 }
}
impl LossyUnto<f32> for f32 {
    fn unto(self) -> f32 { self }
}
impl LossyUnto<f64> for f64 {
    fn unto(self) -> f64 { self }
}
impl LossyUnto<f32> for f64 {
    fn unto(self) -> f32 { self as f32 }
}
