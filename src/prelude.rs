pub use super::*;
pub use super::math::*;
pub use super::audiocomponent::*;
pub use super::combinator::*;

use super::delay::*;
use super::filter::*;
use super::envelope::*;
use super::noise::*;
use super::oscillator::*;
use numeric_array::typenum::*;

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

/// Constant component. Synonymous with dc.
pub fn constant<X: ConstantFrame>(x: X) -> Ac<ConstantComponent<X::Size>> {
    Ac(ConstantComponent::new(x.convert()))
}

/// Constant component. Synonymous with constant.
/// DC stands for "direct current", which is an electrical engineering term used with signals.
pub fn dc<X: ConstantFrame>(x: X) -> Ac<ConstantComponent<X::Size>> {
    Ac(ConstantComponent::new(x.convert()))
}

/// Zero generator.
#[inline] pub fn zero() -> Ac<ConstantComponent<U1>> { constant(0.0) }

/// Mono pass-through component.
#[inline] pub fn pass() -> Ac<PassComponent<U1>> { Ac(PassComponent::new()) }

/// Mono sink.
#[inline] pub fn sink() -> Ac<SinkComponent<U1>> { Ac(SinkComponent::new()) }

/// Sine oscillator with frequency input.
#[inline] pub fn sine() -> Ac<SineComponent> { Ac(SineComponent::new()) }

/// Fixed sine oscillator at f Hz.
#[inline] pub fn sine_hz(f: f48) -> Ac<impl AudioComponent<Inputs = U0, Outputs = U1>> {
    constant(f) >> sine()
}

/// Adds a constant to the input.
#[inline] pub fn add<X: ConstantFrame>(x: X) -> Ac<BinopComponent<PassComponent::<X::Size>, ConstantComponent::<X::Size>, FrameAdd<X::Size>>> where
  X::Size: Size + Add<U0>,
  <X::Size as Add<U0>>::Output: Size
{
    Ac(PassComponent::<X::Size>::new()) + dc(x)
}

/// Subtracts a constant from the input.
#[inline] pub fn sub<X: ConstantFrame>(x: X) -> Ac<BinopComponent<PassComponent::<X::Size>, ConstantComponent::<X::Size>, FrameSub<X::Size>>> where
  X::Size: Size + Add<U0>,
  <X::Size as Add<U0>>::Output: Size
{
    Ac(PassComponent::<X::Size>::new()) - dc(x)
}

/// Multiplies the input with a constant.
#[inline] pub fn mul<X: ConstantFrame>(x: X) -> Ac<BinopComponent<PassComponent::<X::Size>, ConstantComponent::<X::Size>, FrameMul<X::Size>>> where
  X::Size: Size + Add<U0>,
  <X::Size as Add<U0>>::Output: Size
{
    Ac(PassComponent::<X::Size>::new()) * dc(x)
}

/// Butterworth lowpass filter. Inputs are signal and cutoff frequency (Hz).
#[inline] pub fn lowpass() -> Ac<ButterLowpass<f64>> {
    Ac(ButterLowpass::new(DEFAULT_SR))
}

/// Butterworth lowpass filter with fixed cutoff frequency.
#[inline] pub fn lowpass_hz(cutoff: f48) -> Ac<impl AudioComponent<Inputs = U1, Outputs = U1>> {
    (pass() | constant(cutoff)) >> lowpass()
}

/// One-pole lowpass filter. Inputs are signal and cutoff frequency (Hz).
#[inline] pub fn lowpole() -> Ac<OnePoleLowpass<f48>> {
    Ac(OnePoleLowpass::new(DEFAULT_SR))
}

/// One-pole lowpass filter with fixed cutoff frequency.
#[inline] pub fn lowpole_hz(cutoff: f48) -> Ac<impl AudioComponent<Inputs = U1, Outputs = U1>> {
    (pass() | constant(cutoff)) >> lowpole()
}

/// Constant-gain bandpass resonator. Inputs are signal, cutoff frequency (Hz) and bandwidth (Hz).
#[inline] pub fn resonator() -> Ac<Resonator<f64>> {
    Ac(Resonator::new(DEFAULT_SR))
}

/// Constant-gain bandpass resonator with a fixed cutoff frequency and bandwidth.
#[inline] pub fn resonator_hz(cutoff: f48, bandwidth: f48) -> Ac<impl AudioComponent<Inputs = U1, Outputs = U1>> {
    (pass() | constant((cutoff, bandwidth))) >> resonator()
}

/// Control envelope from a time-varying function.
/// The output is linearly interpolated from samples at 2 ms intervals.
/// Synonymous with lfo.
pub fn envelope(f: impl Fn(f48) -> f48 + Clone) -> Ac<impl AudioComponent<Inputs = U0, Outputs = U1>> {
    // Signals containing frequencies no greater than about 20 Hz would be considered control rate.
    // Therefore, sampling at 500 Hz means these signals are fairly well represented.
    // While we represent time in double precision internally, it is often okay to use single precision
    // in envelopes, as local component time typically does not get far from origin.
    Ac(EnvelopeComponent::new(0.002, DEFAULT_SR, f))
}

/// Control signal from a time-varying function.
/// The output is linearly interpolated from samples at 2 ms intervals.
/// Synonymous with envelope.
pub fn lfo(f: impl Fn(f48) -> f48 + Clone) -> Ac<impl AudioComponent<Inputs = U0, Outputs = U1>> {
    Ac(EnvelopeComponent::new(0.002, DEFAULT_SR, f))
}

/// MLS noise generator.
pub fn mls() -> Ac<MlsNoise> { Ac(MlsNoise::new_default()) }

/// MLS noise generator from an n-bit MLS sequence.
pub fn mls_bits(n: u32) -> Ac<MlsNoise> { Ac(MlsNoise::new(Mls::new(n))) }

/// White noise generator. Synonymous with white().
pub fn noise() -> Ac<NoiseComponent> { Ac(NoiseComponent::new()) }

/// White noise generator. Synonymous with noise().
pub fn white() -> Ac<NoiseComponent> { Ac(NoiseComponent::new()) }

/// Single sample delay.
pub fn tick() -> Ac<TickComponent<U1>> { Ac(TickComponent::new(DEFAULT_SR)) }

/// Fixed delay of t seconds.
pub fn delay(t: f48) -> Ac<DelayComponent> { Ac(DelayComponent::new(t, DEFAULT_SR)) }

/// Feedback component.
/// Enclosed feedback circuit x must have an equal number of inputs and outputs.
// TODO. Should we somehow take into account the extra sample of delay induced by the feedback component.
pub fn feedback<X, S>(x: Ac<X>) -> Ac<FeedbackComponent<X, S>> where
    X: AudioComponent<Inputs = S, Outputs = S>,
    X::Inputs: Size,
    X::Outputs: Size,
    S: Size,
    { Ac(FeedbackComponent::new(x.0)) }

/// Replicate the input signals of a filter on its outputs to allow similar filters to be chained
///
/// # Examples
/// ```
/// # use fundsp::prelude::*;
/// let _ = pipeline(lowpass()) >> lowpass();
/// ```
/// ```
/// # use fundsp::prelude::*;
/// let _ = pipeline(lowpass() | lowpass()) >> (lowpass() | lowpass());
/// ```
pub fn pipeline<T: Pipeline>(x: Ac<T>) -> Ac<PipelineComponent<T>>
    where <T as AudioComponent>::Outputs: Cmp<<T as AudioComponent>::Inputs>,
    <T as AudioComponent>::Outputs: IsLessOrEqual<<T as AudioComponent>::Inputs, Output = True>,
{
    Ac(PipelineComponent::new(x.0))
}
