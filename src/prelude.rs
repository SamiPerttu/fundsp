pub use super::math::*;
pub use super::*;

// Function combinator environment. We like to define all kinds of useful functions here.

// Import some typenum integers for reporting arities.
pub type U0 = numeric_array::typenum::U0;
pub type U1 = numeric_array::typenum::U1;
pub type U2 = numeric_array::typenum::U2;
pub type U3 = numeric_array::typenum::U3;
pub type U4 = numeric_array::typenum::U4;
pub type U5 = numeric_array::typenum::U5;
pub type U6 = numeric_array::typenum::U6;
pub type U7 = numeric_array::typenum::U7;
pub type U8 = numeric_array::typenum::U8;
pub type U9 = numeric_array::typenum::U9;
pub type U10 = numeric_array::typenum::U10;
pub type U11 = numeric_array::typenum::U11;
pub type U12 = numeric_array::typenum::U12;
pub type U13 = numeric_array::typenum::U13;
pub type U14 = numeric_array::typenum::U14;
pub type U15 = numeric_array::typenum::U15;
pub type U16 = numeric_array::typenum::U16;
pub type U17 = numeric_array::typenum::U17;
pub type U18 = numeric_array::typenum::U18;
pub type U19 = numeric_array::typenum::U19;
pub type U20 = numeric_array::typenum::U20;

/// Smooth cubic fade curve.
#[inline] pub fn smooth3<T: Num>(x: T) -> T {
    (T::new(3) - T::new(2) * x) * x * x
}

/// Smooth quintic fade curve suggested by Ken Perlin.
#[inline] pub fn smooth5<T: Num>(x: T) -> T {
    ((x * T::new(6) - T::new(15)) * x + T::new(10)) * x * x * x
}

/// Smooth septic fade curve.
#[inline] pub fn smooth7<T: Num>(x: T) -> T {
    let x2 = x * x;
    x2 * x2 * (T::new(35) - T::new(84) * x + (T::new(70) - T::new(20) * x) * x2)
}

/// Smooth nonic fade curve.
#[inline] pub fn smooth9<T: Num>(x: T) -> T {
    let x2 = x * x;
    ((((T::new(70) * x - T::new(315)) * x + T::new(540)) * x - T::new(420)) * x + T::new(125)) * x2 * x2 * x
}

/// Fade that starts and ends at a slope but levels in the middle.
#[inline] pub fn shelf<T: Num>(x: T) -> T {
    ((T::new(4) * x - T::new(6)) * x + T::new(3)) * x
}

/// A quarter circle fade that slopes upwards. Inverse function of Fade.downarc.
#[inline] pub fn uparc<T: Real + Num>(x: T) -> T {
    T::one() - sqrt(max(T::zero(), T::one() - x * x))
}

/// A quarter circle fade that slopes downwards. Inverse function of Fade.uparc.
#[inline] pub fn downarc<T: Real + Num>(x: T) -> T {
    sqrt(max(T::new(0), (T::new(2) - x) * x))
}

/// Wave function stitched together from two symmetric pieces peaking at origin.
#[inline] pub fn wave<T: Num, F: Fn(T) -> T>(f: F, x: T) -> T {
    let u = (x - T::one()) / T::new(4);
    let u = (u - u.floor()) * T::new(2);
    let w0 = u.min(T::one());
    let w1 = u - w0;
    T::one() - (f(w0) - f(w1)) * T::new(2)
}

/// Wave function with smooth3 interpolation.
#[inline] pub fn wave3<T: Num>(x: T) -> T { wave(smooth3, x) }

/// Wave function with smooth5 interpolation.
#[inline] pub fn wave5<T: Num>(x: T) -> T { wave(smooth5, x) }

/// Linear congruential generator proposed by Donald Knuth. Cycles through all u64 values.
#[inline] pub fn lcg64(x: u64) -> u64 { x * 6364136223846793005 + 1442695040888963407 }

/// Sine that oscillates at the specified beats per minute. Time is input in seconds.
#[inline] pub fn sin_bpm<T: Num + Real>(bpm: T, t: T) -> T { sin(t * bpm * T::from_f64(TAU / 60.0)) }

/// Cosine that oscillates at the specified beats per minute. Time is input in seconds.
#[inline] pub fn cos_bpm<T: Num + Real>(bpm: T, t: T) -> T { cos(t * bpm * T::from_f64(TAU / 60.0)) }

/// Sine that oscillates at the specified frequency (Hz). Time is input in seconds.
#[inline] pub fn sin_hz<T: Num + Real>(hz: T, t: T) -> T { sin(t * hz * T::from_f64(TAU)) }

/// Cosine that oscillates at the specified frequency (Hz). Time is input in seconds.
#[inline] pub fn cos_hz<T: Num + Real>(hz: T, t: T) -> T { cos(t * hz * T::from_f64(TAU)) }

/// Returns a gain factor from a decibel argument.
#[inline] pub fn db_gain<T: Num + Real>(db: T) -> T { exp(log(T::new(10)) * db / T::new(20)) }

use super::audiocomponent::*;
use super::filter::*;
use super::envelope::*;

/// Makes a zero generator.
#[inline] pub fn zero() -> Ac<ConstantComponent<U1>> { dc(0.0) }

/// Makes a mono pass-through component.
#[inline] pub fn pass() -> Ac<PassComponent<U1>> { Ac(PassComponent::new()) }

/// Makes a mono sink.
#[inline] pub fn sink() -> Ac<SinkComponent<U1>> { Ac(SinkComponent::new()) }

/// Component that adds a constant to its input.
#[inline] pub fn add<X: ConstantFrame>(x: X) -> Ac<BinaryComponent<PassComponent::<X::Size>, ConstantComponent::<X::Size>>> where
  X::Size: Size + Add<U0>,
  <X::Size as Add<U0>>::Output: Size
{
    Ac(PassComponent::<X::Size>::new()) + dc(x)
}

/// Component that multiplies its input with a constant.
#[inline] pub fn mul<X: ConstantFrame>(x: X) -> Ac<BinopComponent<PassComponent::<X::Size>, ConstantComponent::<X::Size>, FrameMul<X::Size>>> where
  X::Size: Size + Add<U0>,
  <X::Size as Add<U0>>::Output: Size
{
    Ac(PassComponent::<X::Size>::new()) * dc(x)
}

/// Makes a Butterworth lowpass filter. Inputs are signal and cutoff frequency (Hz).
#[inline] pub fn lowpass() -> Ac<ButterLowpass<f64>> {
    Ac(ButterLowpass::new(DEFAULT_SR))
}

/// Makes a Butterworth lowpass filter with a fixed cutoff frequency.
#[inline] pub fn lowpass_hz(cutoff: f48) -> Ac<impl AudioComponent<Inputs = U1, Outputs = U1>> {
    (pass() | constant(cutoff)) >> lowpass()
}

/// Makes a constant-gain resonator. Inputs are signal, cutoff frequency (Hz) and bandwidth (Hz).
#[inline] pub fn resonator() -> Ac<Resonator<f64>> {
    Ac(Resonator::new(DEFAULT_SR))
}

/// Makes a control envelope from a time-varying function.
/// The output is linearly interpolated from samples at 2 ms intervals.
/// Synonymous with lfo.
pub fn envelope(f: impl Fn(f48) -> f48) -> Ac<impl AudioComponent<Inputs = U0, Outputs = U1>> {
    // Signals containing frequencies no greater than about 20 Hz would be considered control rate.
    // Therefore, sampling at 500 Hz means these signals are fairly well represented.
    // While we represent time in double precision internally, it is often okay to use single precision
    // in envelopes, as local component time typically does not get far from origin.
    Ac(EnvelopeComponent::new(0.002, DEFAULT_SR, f))
}

/// Makes a control signal from a time-varying function.
/// The output is linearly interpolated from samples at 2 ms intervals.
/// Synonymous with envelope.
pub fn lfo(f: impl Fn(f48) -> f48) -> Ac<impl AudioComponent<Inputs = U0, Outputs = U1>> {
    Ac(EnvelopeComponent::new(0.002, DEFAULT_SR, f))
}
