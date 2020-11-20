pub use super::audionode::*;
pub use super::combinator::*;
pub use super::math::*;
pub use super::*;

use super::delay::*;
use super::envelope::*;
use super::filter::*;
use super::noise::*;
use super::oscillator::*;

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
pub fn constant<X: ConstantFrame<Sample = f64>>(x: X) -> An<ConstantNode<f64, X::Size>>
where
    X::Size: Size<f64>,
{
    An(ConstantNode::new(x.convert()))
}

/// Constant component. Synonymous with constant.
/// DC stands for "direct current", which is an electrical engineering term used with signals.
pub fn dc<X: ConstantFrame<Sample = f64>>(x: X) -> An<ConstantNode<f64, X::Size>>
where
    X::Size: Size<f64>,
{
    An(ConstantNode::new(x.convert()))
}

/// Zero generator.
#[inline]
pub fn zero() -> An<ConstantNode<f64, U1>> {
    constant(0.0)
}

/// Mono pass-through component.
#[inline]
pub fn pass() -> An<PassNode<f64, U1>> {
    An(PassNode::new())
}

/// Mono sink.
#[inline]
pub fn sink() -> An<SinkNode<f64, U1>> {
    An(SinkNode::new())
}

/// Sine oscillator with frequency input.
#[inline]
pub fn sine() -> An<SineComponent<f64>> {
    An(SineComponent::new())
}

/// Fixed sine oscillator at f Hz.
#[inline]
pub fn sine_hz(f: f64) -> An<impl AudioNode<Sample = f64, Inputs = U0, Outputs = U1>> {
    constant(f) >> sine()
}

/// Adds a constant to the input.
#[inline]
pub fn add<X: ConstantFrame<Sample = f64>>(
    x: X,
) -> An<BinopNode<f64, PassNode<f64, X::Size>, ConstantNode<f64, X::Size>, FrameAdd<f64, X::Size>>>
where
    X::Size: Size<f64> + Add<U0>,
    <X::Size as Add<U0>>::Output: Size<f64>,
{
    An(PassNode::<f64, X::Size>::new()) + dc(x)
}

/// Subtracts a constant from the input.
#[inline]
pub fn sub<X: ConstantFrame<Sample = f64>>(
    x: X,
) -> An<BinopNode<f64, PassNode<f64, X::Size>, ConstantNode<f64, X::Size>, FrameSub<f64, X::Size>>>
where
    X::Size: Size<f64> + Add<U0>,
    <X::Size as Add<U0>>::Output: Size<f64>,
{
    An(PassNode::<f64, X::Size>::new()) - dc(x)
}

/// Multiplies the input with a constant.
#[inline]
pub fn mul<X: ConstantFrame<Sample = f64>>(
    x: X,
) -> An<BinopNode<f64, PassNode<f64, X::Size>, ConstantNode<f64, X::Size>, FrameMul<f64, X::Size>>>
where
    X::Size: Size<f64> + Add<U0>,
    <X::Size as Add<U0>>::Output: Size<f64>,
{
    An(PassNode::<f64, X::Size>::new()) * dc(x)
}

/// Butterworth lowpass filter. Inputs are signal and cutoff frequency (Hz).
#[inline]
pub fn lowpass() -> An<ButterLowpass<f64, f64>> {
    An(ButterLowpass::new(DEFAULT_SR))
}

/// Butterworth lowpass filter with fixed cutoff frequency.
#[inline]
pub fn lowpass_hz(cutoff: f64) -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = U1>> {
    (pass() | constant(cutoff)) >> lowpass()
}

/// One-pole lowpass filter. Inputs are signal and cutoff frequency (Hz).
#[inline]
pub fn lowpole() -> An<OnePoleLowpass<f64, f64>> {
    An(OnePoleLowpass::new(DEFAULT_SR))
}

/// One-pole lowpass filter with fixed cutoff frequency.
#[inline]
pub fn lowpole_hz(cutoff: f64) -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = U1>> {
    (pass() | constant(cutoff)) >> lowpole()
}

/// Constant-gain bandpass resonator. Inputs are signal, cutoff frequency (Hz) and bandwidth (Hz).
#[inline]
pub fn resonator() -> An<Resonator<f64, f64>> {
    An(Resonator::new(DEFAULT_SR))
}

/// Constant-gain bandpass resonator with a fixed cutoff frequency and bandwidth.
#[inline]
pub fn resonator_hz(
    cutoff: f64,
    bandwidth: f64,
) -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = U1>> {
    (pass() | constant((cutoff, bandwidth))) >> resonator()
}

/// Control envelope from a time-varying function.
/// The output is linearly interpolated from samples at 2 ms intervals.
/// Synonymous with lfo.
pub fn envelope(
    f: impl Fn(f64) -> f64 + Clone,
) -> An<impl AudioNode<Sample = f64, Inputs = U0, Outputs = U1>> {
    // Signals containing frequencies no greater than about 20 Hz would be considered control rate.
    // Therefore, sampling at 500 Hz means these signals are fairly well represented.
    // While we represent time in double precision internally, it is often okay to use single precision
    // in envelopes, as local component time typically does not get far from origin.
    An(EnvelopeNode::new(0.002, DEFAULT_SR, f))
}

/// Control signal from a time-varying function.
/// The output is linearly interpolated from samples at 2 ms intervals.
/// Synonymous with envelope.
pub fn lfo(
    f: impl Fn(f64) -> f64 + Clone,
) -> An<impl AudioNode<Sample = f64, Inputs = U0, Outputs = U1>> {
    An(EnvelopeNode::new(0.002, DEFAULT_SR, f))
}

/// MLS noise generator.
pub fn mls() -> An<MlsNoise<f64>> {
    An(MlsNoise::new_default())
}

/// MLS noise generator from an n-bit MLS sequence.
pub fn mls_bits(n: u32) -> An<MlsNoise<f64>> {
    An(MlsNoise::new(Mls::new(n)))
}

/// White noise generator. Synonymous with white().
pub fn noise() -> An<NoiseNode<f64>> {
    An(NoiseNode::new())
}

/// White noise generator. Synonymous with noise().
pub fn white() -> An<NoiseNode<f64>> {
    An(NoiseNode::new())
}

/// Single sample delay.
pub fn tick() -> An<TickNode<f64, U1>> {
    An(TickNode::new(DEFAULT_SR))
}

/// Fixed delay of t seconds.
pub fn delay(t: f64) -> An<DelayNode<f64>> {
    An(DelayNode::new(t, DEFAULT_SR))
}

/// Feedback component.
/// Enclosed feedback circuit x must have an equal number of inputs and outputs.
// TODO. Should we somehow take into account the extra sample of delay induced by the feedback component.
pub fn feedback<X, N>(x: An<X>) -> An<FeedbackNode<f64, X, N>>
where
    X: AudioNode<Sample = f64, Inputs = N, Outputs = N>,
    X::Inputs: Size<f64>,
    X::Outputs: Size<f64>,
    N: Size<f64>,
{
    An(FeedbackNode::new(x.0))
}

/// Transform channels freely
///
/// # Example
/// ```
/// # use fundsp::prelude::*;
/// let my_sum = map(|i: &Frame<f64, U2>| Frame::<f64, U1>::splat(i[0] + i[1]));
/// ```
pub fn map<F, I, O>(f: F) -> Map<f64, F, I, O>
where
    F: Clone + FnMut(&Frame<f64, I>) -> Frame<f64, O>,
    I: Size<f64>,
    O: Size<f64>,
{
    Map::new(f)
}
