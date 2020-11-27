pub use super::audionode::*;
pub use super::combinator::*;
pub use super::math::*;
pub use super::*;

use super::delay::*;
use super::dynamics::*;
use super::envelope::*;
use super::feedback::*;
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
pub fn constant<X: ConstantFrame<Sample = f64>>(x: X) -> An<ConstantNode<f64, X::Size>>
where
    X::Size: Size<f64>,
{
    An(ConstantNode::new(x.convert()))
}

/// Constant node.
/// Synonymous with `constant`.
/// (DC stands for "direct current", which is an electrical engineering term used with signals.)
pub fn dc<X: ConstantFrame<Sample = f64>>(x: X) -> An<ConstantNode<f64, X::Size>>
where
    X::Size: Size<f64>,
{
    An(ConstantNode::new(x.convert()))
}

/// Zero generator.
/// - Output 0: zero
#[inline]
pub fn zero() -> An<ConstantNode<f64, U1>> {
    constant(0.0)
}

/// Mono pass-through.
#[inline]
pub fn pass() -> An<PassNode<f64, U1>> {
    An(PassNode::new())
}

/// Mono sink.
#[inline]
pub fn sink() -> An<SinkNode<f64, U1>> {
    An(SinkNode::new())
}

/// Sine oscillator.
/// - Input 0: frequency (Hz)
/// - Output 0: sine wave
#[inline]
pub fn sine() -> An<SineNode<f64>> {
    An(SineNode::new(DEFAULT_SR))
}

/// Fixed sine oscillator at `f` Hz.
/// - Output 0: sine wave
#[inline]
pub fn sine_hz(f: f64) -> An<impl AudioNode<Sample = f64, Inputs = U0, Outputs = U1>> {
    constant(f) >> sine()
}

/// Add constant to signal.
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

/// Subtract constant from signal.
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

/// Multiply signal with constant.
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

/// Butterworth lowpass filter (2nd order).
/// - Input 0: audio
/// - Input 1: cutoff frequency (Hz)
/// - Output 0: filtered audio
#[inline]
pub fn lowpass() -> An<ButterLowpass<f64, f64>> {
    An(ButterLowpass::new(DEFAULT_SR))
}

/// Butterworth lowpass filter (2nd order) with fixed `cutoff` frequency.
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn lowpass_hz(cutoff: f64) -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = U1>> {
    (pass() | constant(cutoff)) >> lowpass()
}

/// One-pole lowpass filter (1st order).
/// - Input 0: audio
/// - Input 1: cutoff frequency (Hz)
/// - Output 0: filtered audio
#[inline]
pub fn lowpole() -> An<OnePoleLowpass<f64, f64>> {
    An(OnePoleLowpass::new(DEFAULT_SR))
}

/// One-pole lowpass filter (1st order) with fixed `cutoff` frequency.
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn lowpole_hz(cutoff: f64) -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = U1>> {
    (pass() | constant(cutoff)) >> lowpole()
}

/// Constant-gain bandpass resonator.
/// - Input 0: audio
/// - Input 1: cutoff frequency (Hz)
/// - Input 2: bandwidth (Hz)
/// - Output 0: filtered audio
#[inline]
pub fn resonator() -> An<Resonator<f64, f64>> {
    An(Resonator::new(DEFAULT_SR))
}

/// Constant-gain bandpass resonator with fixed `cutoff` frequency (Hz) and `bandwidth` (Hz).
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn resonator_hz(
    cutoff: f64,
    bandwidth: f64,
) -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = U1>> {
    (pass() | constant((cutoff, bandwidth))) >> resonator()
}

/// Control envelope from time-varying function `f(t)` with `t` in seconds.
/// Spaces samples using pseudorandom jittering.
/// Synonymous with `lfo`.
/// - Output 0: envelope linearly interpolated from samples at 2 ms intervals (average).
pub fn envelope(
    f: impl Fn(f64) -> f64 + Clone,
) -> An<impl AudioNode<Sample = f64, Inputs = U0, Outputs = U1>> {
    // Signals containing frequencies no greater than about 20 Hz would be considered control rate.
    // Therefore, sampling at 500 Hz means these signals are fairly well represented.
    // While we represent time in double precision internally, it is often okay to use single precision
    // in envelopes, as local component time typically does not get far from origin.
    An(EnvelopeNode::new(0.002, DEFAULT_SR, f))
}

/// Control envelope from time-varying function `f(t)` with `t` in seconds.
/// Spaces samples using pseudorandom jittering.
/// Synonymous with `envelope`.
/// - Output 0: envelope linearly interpolated from samples at 2 ms intervals (average).
pub fn lfo(
    f: impl Fn(f64) -> f64 + Clone,
) -> An<impl AudioNode<Sample = f64, Inputs = U0, Outputs = U1>> {
    An(EnvelopeNode::new(0.002, DEFAULT_SR, f))
}

/// Maximum Length Sequence noise generator from an `n`-bit sequence.
/// - Output 0: repeating white noise sequence of only -1 and 1 values.
pub fn mls_bits(n: i64) -> An<MlsNoise<f64>> {
    An(MlsNoise::new(Mls::new(n as u32)))
}

/// Default Maximum Length Sequence noise generator.
/// - Output 0: repeating white noise sequence of only -1 and 1 values.
pub fn mls() -> An<MlsNoise<f64>> {
    mls_bits(29)
}

/// White noise generator.
/// Synonymous with `white`.
/// - Output 0: white noise.
pub fn noise() -> An<NoiseNode<f64>> {
    An(NoiseNode::new())
}

/// White noise generator.
/// Synonymous with `noise`.
/// - Output 0: white noise.
pub fn white() -> An<NoiseNode<f64>> {
    An(NoiseNode::new())
}

/// Single sample delay.
/// - Input 0: signal.
/// - Output 0: delayed signal.
pub fn tick() -> An<TickNode<f64, U1>> {
    An(TickNode::new(DEFAULT_SR))
}

/// Fixed delay of `t` seconds.
/// - Input 0: signal.
/// - Output 0: delayed signal.
pub fn delay(t: f64) -> An<DelayNode<f64>> {
    An(DelayNode::new(t, DEFAULT_SR))
}

/// Mix output of enclosed circuit `x` back to its input.
/// Feedback circuit `x` must have an equal number of inputs and outputs.
/// - Inputs: input signal.
/// - Outputs: `x` output signal.
pub fn feedback<X, N>(x: An<X>) -> An<FeedbackNode<f64, X, N, FrameId<f64, N>>>
where
    X: AudioNode<Sample = f64, Inputs = N, Outputs = N>,
    X::Inputs: Size<f64>,
    X::Outputs: Size<f64>,
    N: Size<f64>,
{
    An(FeedbackNode::new(x.0, FrameId::new()))
}

/// Keeps a signal zero centered.
/// Filter cutoff `c` is usually somewhere below the audible range.
/// The default blocker cutoff is 10 Hz.
pub fn dcblock_hz(c: f64) -> An<DCBlocker<f64, f64>> {
    An(DCBlocker::new(DEFAULT_SR, c))
}

/// Keeps a signal zero centered.
pub fn dcblock() -> An<DCBlocker<f64, f64>> {
    dcblock_hz(10.0)
}

pub fn declick() -> An<Declicker<f64, f64>> {
    An(Declicker::new(DEFAULT_SR, 0.010))
}

/// Shape signal with a waveshaper.
pub fn shape<S: Fn(f64) -> f64 + Clone>(
    f: S,
) -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = U1>> {
    An(MapNode::new(move |input: &Frame<f64, U1>| {
        [f(input[0])].into()
    }))
}

/// Parameter follower filter with fixed halfway response time `t` seconds.
pub fn follow(t: f64) -> An<Follower<f64, f64>> {
    An(Follower::new(DEFAULT_SR, t))
}

/// Look-ahead limiter with `lookahead` in seconds.
pub fn limiter(lookahead: f64) -> An<Limiter<f64, U1>> {
    An(Limiter::new(DEFAULT_SR, lookahead))
}

/// Pinking filter.
pub fn pinkpass() -> An<PinkFilter<f64, f64>> {
    An(PinkFilter::new())
}

/// Pink noise.
pub fn pink() -> An<impl AudioNode<Sample = f64, Inputs = U0, Outputs = U1>> {
    white() >> pinkpass()
}

/// Brown noise.
pub fn brown() -> An<impl AudioNode<Sample = f64, Inputs = U0, Outputs = U1>> {
    // Empirical normalization factor.
    white() >> lowpole_hz(10.0) * dc(13.7)
}

/// Frequency detector.
pub fn goertzel() -> An<GoertzelNode<f64, f64>> {
    An(GoertzelNode::new(DEFAULT_SR))
}

/// Frequency detector of frequency `f` Hz.
pub fn goertzel_hz(f: f64) -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = U1>> {
    (pass() | constant(f)) >> goertzel()
}

/// Feedback delay network.
/// Mix output of enclosed circuit `x` back to its input.
/// The output is diffused with a Hadamard matrix for feedback.
/// Feedback circuit `x` must have an equal number of inputs and outputs.
/// - Inputs: input signal.
/// - Outputs: `x` output signal.
pub fn fdn<X, N>(x: An<X>) -> An<FeedbackNode<f64, X, N, FrameHadamard<f64, N>>>
where
    X: AudioNode<Sample = f64, Inputs = N, Outputs = N>,
    X::Inputs: Size<f64>,
    X::Outputs: Size<f64>,
    N: Size<f64>,
{
    An(FeedbackNode::new(x.0, FrameHadamard::new()))
}
