pub use super::audionode::*;
pub use super::combinator::*;
pub use super::math::*;
pub use super::*;

use super::delay::*;
use super::envelope::*;
use super::filter::*;
use super::noise::*;
use super::oscillator::*;

// Combinator environment.
// We like to define all kinds of useful functions here.

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

/// Constant node.
/// Synonymous with `[dc]`.
pub fn constant<T: Float, X: ConstantFrame<Sample = T>>(x: X) -> An<ConstantNode<T, X::Size>>
where
    X::Size: Size<T>,
{
    An(ConstantNode::new(x.convert()))
}

/// Constant node.
/// Synonymous with `constant`.
/// (DC stands for "direct current", which is an electrical engineering term used with signals.)
pub fn dc<T: Float, X: ConstantFrame<Sample = T>>(x: X) -> An<ConstantNode<T, X::Size>>
where
    X::Size: Size<T>,
{
    An(ConstantNode::new(x.convert()))
}

/// Zero generator.
/// - Output 0: zero
#[inline]
pub fn zero<T: Float>() -> An<ConstantNode<T, U1>> {
    dc(T::new(0))
}

/// Mono pass-through.
#[inline]
pub fn pass<T: Float>() -> An<PassNode<T, U1>> {
    An(PassNode::new())
}

/// Mono sink.
#[inline]
pub fn sink<T: Float>() -> An<SinkNode<T, U1>> {
    An(SinkNode::new())
}

/// Sine oscillator.
/// - Input 0: frequency (Hz)
/// - Output 0: sine wave
#[inline]
pub fn sine<T: Float>() -> An<SineComponent<T>> {
    An(SineComponent::new(DEFAULT_SR))
}

/// Fixed sine oscillator at `f` Hz.
/// - Output 0: sine wave
#[inline]
pub fn sine_hz<T: Float>(f: T) -> An<impl AudioNode<Sample = T, Inputs = U0, Outputs = U1>> {
    constant(f) >> sine()
}

/// Add constant to signal.
#[inline]
pub fn add<X: ConstantFrame>(
    x: X,
) -> An<
    BinopNode<
        X::Sample,
        PassNode<X::Sample, X::Size>,
        ConstantNode<X::Sample, X::Size>,
        FrameAdd<X::Sample, X::Size>,
    >,
>
where
    X::Size: Size<X::Sample> + Add<U0>,
    <X::Size as Add<U0>>::Output: Size<X::Sample>,
{
    An(PassNode::<X::Sample, X::Size>::new()) + dc(x)
}

/// Subtract constant from signal.
#[inline]
pub fn sub<X: ConstantFrame>(
    x: X,
) -> An<
    BinopNode<
        X::Sample,
        PassNode<X::Sample, X::Size>,
        ConstantNode<X::Sample, X::Size>,
        FrameSub<X::Sample, X::Size>,
    >,
>
where
    X::Size: Size<X::Sample> + Add<U0>,
    <X::Size as Add<U0>>::Output: Size<X::Sample>,
{
    An(PassNode::<X::Sample, X::Size>::new()) - dc(x)
}

/// Multiply signal with constant.
#[inline]
pub fn mul<X: ConstantFrame>(
    x: X,
) -> An<
    BinopNode<
        X::Sample,
        PassNode<X::Sample, X::Size>,
        ConstantNode<X::Sample, X::Size>,
        FrameMul<X::Sample, X::Size>,
    >,
>
where
    X::Size: Size<X::Sample> + Add<U0>,
    <X::Size as Add<U0>>::Output: Size<X::Sample>,
{
    An(PassNode::<X::Sample, X::Size>::new()) * dc(x)
}

/// Butterworth lowpass filter (2nd order).
/// - Input 0: audio
/// - Input 1: cutoff frequency (Hz)
/// - Output 0: filtered audio
#[inline]
pub fn lowpass<T: Float, F: Real>() -> An<ButterLowpass<T, F>> {
    An(ButterLowpass::new(convert(DEFAULT_SR)))
}

/// Butterworth lowpass filter (2nd order) with fixed `cutoff` frequency.
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn lowpass_hz<T: Float, F: Real>(
    cutoff: T,
) -> An<impl AudioNode<Sample = T, Inputs = U1, Outputs = U1>> {
    (pass() | constant(cutoff)) >> lowpass::<T, F>()
}

/// One-pole lowpass filter (1st order).
/// - Input 0: audio
/// - Input 1: cutoff frequency (Hz)
/// - Output 0: filtered audio
#[inline]
pub fn lowpole<T: Float, F: Real>() -> An<OnePoleLowpass<T, F>> {
    An(OnePoleLowpass::new(convert(DEFAULT_SR)))
}

/// One-pole lowpass filter (1st order) with fixed `cutoff` frequency.
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn lowpole_hz<T: Float, F: Real>(
    cutoff: T,
) -> An<impl AudioNode<Sample = T, Inputs = U1, Outputs = U1>> {
    (pass::<T>() | constant(cutoff)) >> lowpole::<T, F>()
}

/// Constant-gain bandpass resonator.
/// - Input 0: audio
/// - Input 1: cutoff frequency (Hz)
/// - Input 2: bandwidth (Hz)
/// - Output 0: filtered audio
#[inline]
pub fn resonator<T: Float, F: Real>() -> An<Resonator<T, F>> {
    An(Resonator::new(convert(DEFAULT_SR)))
}

/// Constant-gain bandpass resonator with fixed `cutoff` frequency (Hz) and `bandwidth` (Hz).
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn resonator_hz<T: Float, F: Real>(
    cutoff: T,
    bandwidth: T,
) -> An<impl AudioNode<Sample = T, Inputs = U1, Outputs = U1>> {
    (pass::<T>() | constant((cutoff, bandwidth))) >> resonator::<T, F>()
}

/// Control envelope from time-varying function `f(t)` with `t` in seconds.
/// Spaces samples using pseudorandom jittering.
/// Synonymous with `lfo`.
/// - Output 0: envelope linearly interpolated from samples at 2 ms intervals (average).
pub fn envelope<T: Float, F: Real>(
    f: impl Fn(F) -> F + Clone,
) -> An<impl AudioNode<Sample = T, Inputs = U0, Outputs = U1>> {
    // Signals containing frequencies no greater than about 20 Hz would be considered control rate.
    // Therefore, sampling at 500 Hz means these signals are fairly well represented.
    // While we represent time in double precision internally, it is often okay to use single precision
    // in envelopes, as local component time typically does not get far from origin.
    An(EnvelopeNode::new(F::from_f64(0.002), DEFAULT_SR, f))
}

/// Control envelope from time-varying function `f(t)` with `t` in seconds.
/// Spaces samples using pseudorandom jittering.
/// Synonymous with `envelope`.
/// - Output 0: envelope linearly interpolated from samples at 2 ms intervals (average).
pub fn lfo<T: Float, F: Real>(
    f: impl Fn(F) -> F + Clone,
) -> An<impl AudioNode<Sample = T, Inputs = U0, Outputs = U1>> {
    An(EnvelopeNode::new(F::from_f64(0.002), DEFAULT_SR, f))
}

/// Maximum Length Sequence noise generator from an `n`-bit sequence.
/// - Output 0: repeating white noise sequence of only -1 and 1 values.
pub fn mls_bits<T: Float>(n: u32) -> An<MlsNoise<T>> {
    An(MlsNoise::new(Mls::new(n)))
}

/// Default Maximum Length Sequence noise generator.
/// - Output 0: repeating white noise sequence of only -1 and 1 values.
pub fn mls<T: Float>() -> An<MlsNoise<T>> {
    mls_bits(29)
}

/// White noise generator.
/// Synonymous with `white`.
/// - Output 0: white noise.
pub fn noise<T: Float>() -> An<NoiseNode<T>> {
    An(NoiseNode::new())
}

/// White noise generator.
/// Synonymous with `noise`.
/// - Output 0: white noise.
pub fn white<T: Float>() -> An<NoiseNode<T>> {
    An(NoiseNode::new())
}

/// Single sample delay.
/// - Input 0: signal.
/// - Output 0: delayed signal.
pub fn tick<T: Float>() -> An<TickNode<T, U1>> {
    An(TickNode::new(convert(DEFAULT_SR)))
}

/// Fixed delay of `t` seconds.
/// - Input 0: signal.
/// - Output 0: delayed signal.
pub fn delay<T: Float>(t: f64) -> An<DelayNode<T>> {
    An(DelayNode::new(t, DEFAULT_SR))
}

/// Mix output of enclosed circuit `x` back to its input.
/// Feedback circuit `x` must have an equal number of inputs and outputs.
/// - Inputs: input signal.
/// - Outputs: `x` output signal.
pub fn feedback<T, X, N>(x: An<X>) -> An<FeedbackNode<T, X, N>>
where
    T: Float,
    X: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    N: Size<T>,
{
    An(FeedbackNode::new(x.0))
}

/// Transform channels freely.
///
/// # Example
/// ```
/// # use fundsp::prelude::*;
/// let my_sum = map(|i: &Frame<f64, U2>| Frame::<f64, U1>::splat(i[0] + i[1]));
/// ```
// TODO: ConstantFrame (?) based version for prelude.
pub fn map<F, I, O>(f: F) -> Map<f64, F, I, O>
where
    F: Clone + FnMut(&Frame<f64, I>) -> Frame<f64, O>,
    I: Size<f64>,
    O: Size<f64>,
{
    Map::new(f)
}
