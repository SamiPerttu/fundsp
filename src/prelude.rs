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
use super::svf::*;
use super::wavetable::*;

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
pub type U21 = numeric_array::typenum::U21;
pub type U22 = numeric_array::typenum::U22;
pub type U23 = numeric_array::typenum::U23;
pub type U24 = numeric_array::typenum::U24;
pub type U25 = numeric_array::typenum::U25;
pub type U26 = numeric_array::typenum::U26;
pub type U27 = numeric_array::typenum::U27;
pub type U28 = numeric_array::typenum::U28;
pub type U29 = numeric_array::typenum::U29;
pub type U30 = numeric_array::typenum::U30;
pub type U31 = numeric_array::typenum::U31;
pub type U32 = numeric_array::typenum::U32;
pub type U33 = numeric_array::typenum::U33;
pub type U34 = numeric_array::typenum::U34;
pub type U35 = numeric_array::typenum::U35;
pub type U36 = numeric_array::typenum::U36;
pub type U37 = numeric_array::typenum::U37;
pub type U38 = numeric_array::typenum::U38;
pub type U39 = numeric_array::typenum::U39;
pub type U40 = numeric_array::typenum::U40;
pub type U41 = numeric_array::typenum::U41;
pub type U42 = numeric_array::typenum::U42;
pub type U43 = numeric_array::typenum::U43;
pub type U44 = numeric_array::typenum::U44;
pub type U45 = numeric_array::typenum::U45;
pub type U46 = numeric_array::typenum::U46;
pub type U47 = numeric_array::typenum::U47;
pub type U48 = numeric_array::typenum::U48;
pub type U49 = numeric_array::typenum::U49;
pub type U50 = numeric_array::typenum::U50;
pub type U51 = numeric_array::typenum::U51;
pub type U52 = numeric_array::typenum::U52;
pub type U53 = numeric_array::typenum::U53;
pub type U54 = numeric_array::typenum::U54;
pub type U55 = numeric_array::typenum::U55;
pub type U56 = numeric_array::typenum::U56;
pub type U57 = numeric_array::typenum::U57;
pub type U58 = numeric_array::typenum::U58;
pub type U59 = numeric_array::typenum::U59;
pub type U60 = numeric_array::typenum::U60;
pub type U61 = numeric_array::typenum::U61;
pub type U62 = numeric_array::typenum::U62;
pub type U63 = numeric_array::typenum::U63;
pub type U64 = numeric_array::typenum::U64;
pub type U65 = numeric_array::typenum::U65;
pub type U66 = numeric_array::typenum::U66;
pub type U67 = numeric_array::typenum::U67;
pub type U68 = numeric_array::typenum::U68;
pub type U69 = numeric_array::typenum::U69;
pub type U70 = numeric_array::typenum::U70;
pub type U71 = numeric_array::typenum::U71;
pub type U72 = numeric_array::typenum::U72;
pub type U73 = numeric_array::typenum::U73;
pub type U74 = numeric_array::typenum::U74;
pub type U75 = numeric_array::typenum::U75;
pub type U76 = numeric_array::typenum::U76;
pub type U77 = numeric_array::typenum::U77;
pub type U78 = numeric_array::typenum::U78;
pub type U79 = numeric_array::typenum::U79;
pub type U80 = numeric_array::typenum::U80;
pub type U81 = numeric_array::typenum::U81;
pub type U82 = numeric_array::typenum::U82;
pub type U83 = numeric_array::typenum::U83;
pub type U84 = numeric_array::typenum::U84;
pub type U85 = numeric_array::typenum::U85;
pub type U86 = numeric_array::typenum::U86;
pub type U87 = numeric_array::typenum::U87;
pub type U88 = numeric_array::typenum::U88;
pub type U89 = numeric_array::typenum::U89;
pub type U90 = numeric_array::typenum::U80;
pub type U91 = numeric_array::typenum::U91;
pub type U92 = numeric_array::typenum::U92;
pub type U93 = numeric_array::typenum::U93;
pub type U94 = numeric_array::typenum::U94;
pub type U95 = numeric_array::typenum::U95;
pub type U96 = numeric_array::typenum::U96;
pub type U97 = numeric_array::typenum::U97;
pub type U98 = numeric_array::typenum::U98;
pub type U99 = numeric_array::typenum::U99;
pub type U100 = numeric_array::typenum::U100;

/// Constant node.
/// Synonymous with `[dc]`.
#[inline]
pub fn constant<T: Float, X: ConstantFrame<Sample = T>>(x: X) -> An<ConstantNode<T, X::Size>>
where
    X::Size: Size<T>,
{
    An(ConstantNode::new(x.convert()))
}

/// Constant node.
/// Synonymous with `constant`.
/// (DC stands for "direct current", which is an electrical engineering term used with signals.)
#[inline]
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
pub fn sine<T: Float>() -> An<SineNode<T>> {
    An(SineNode::new(DEFAULT_SR))
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
    // TODO: Should we use MapNode in these?
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
pub fn butterpass<T: Float, F: Real>() -> An<ButterLowpass<T, F>> {
    An(ButterLowpass::new(convert(DEFAULT_SR)))
}

/// Butterworth lowpass filter (2nd order) with fixed `cutoff` frequency.
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn butterpass_hz<T: Float, F: Real>(
    cutoff: T,
) -> An<impl AudioNode<Sample = T, Inputs = U1, Outputs = U1>> {
    (pass() | constant(cutoff)) >> butterpass::<T, F>()
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
/// - Output(s): envelope linearly interpolated from samples at 2 ms intervals (average).
#[inline]
pub fn envelope<T, F, E, R>(f: E) -> An<impl AudioNode<Sample = T, Inputs = U0, Outputs = R::Size>>
where
    T: Float,
    F: Float,
    E: Fn(F) -> R + Clone,
    R: ConstantFrame<Sample = F>,
    R::Size: Size<F>,
    R::Size: Size<T>,
{
    // Signals containing frequencies no greater than about 20 Hz would be considered control rate.
    // Therefore, sampling at 500 Hz means these signals are fairly well represented.
    // While we represent time in double precision internally, it is often okay to use single precision
    // in envelopes, as local component time typically does not get far from origin.
    An(EnvelopeNode::new(F::from_f64(0.002), DEFAULT_SR, f))
}

/// Control envelope from time-varying function `f(t)` with `t` in seconds.
/// Spaces samples using pseudorandom jittering.
/// Synonymous with `envelope`.
/// - Output(s): envelope linearly interpolated from samples at 2 ms intervals (average).
#[inline]
pub fn lfo<T, F, E, R>(f: E) -> An<impl AudioNode<Sample = T, Inputs = U0, Outputs = R::Size>>
where
    T: Float,
    F: Float,
    E: Fn(F) -> R + Clone,
    R: ConstantFrame<Sample = F>,
    R::Size: Size<F>,
    R::Size: Size<T>,
{
    // Signals containing frequencies no greater than about 20 Hz would be considered control rate.
    // Therefore, sampling at 500 Hz means these signals are fairly well represented.
    // While we represent time in double precision internally, it is often okay to use single precision
    // in envelopes, as local component time typically does not get far from origin.
    An(EnvelopeNode::new(F::from_f64(0.002), DEFAULT_SR, f))
}

/// Maximum Length Sequence noise generator from an `n`-bit sequence.
/// - Output 0: repeating white noise sequence of only -1 and 1 values.
#[inline]
pub fn mls_bits<T: Float>(n: u32) -> An<MlsNoise<T>> {
    An(MlsNoise::new(Mls::new(n)))
}

/// Default Maximum Length Sequence noise generator.
/// - Output 0: repeating white noise sequence of only -1 and 1 values.
#[inline]
pub fn mls<T: Float>() -> An<MlsNoise<T>> {
    mls_bits(29)
}

/// White noise generator.
/// Synonymous with `white`.
/// - Output 0: white noise.
#[inline]
pub fn noise<T: Float>() -> An<NoiseNode<T>> {
    An(NoiseNode::new())
}

/// White noise generator.
/// Synonymous with `noise`.
/// - Output 0: white noise.
#[inline]
pub fn white<T: Float>() -> An<NoiseNode<T>> {
    An(NoiseNode::new())
}

/// Single sample delay.
/// - Input 0: signal.
/// - Output 0: delayed signal.
#[inline]
pub fn tick<T: Float>() -> An<TickNode<T, U1>> {
    An(TickNode::new(convert(DEFAULT_SR)))
}

/// Fixed delay of `t` seconds.
/// - Input 0: signal.
/// - Output 0: delayed signal.
#[inline]
pub fn delay<T: Float>(t: f64) -> An<DelayNode<T>> {
    An(DelayNode::new(t, DEFAULT_SR))
}

/// Mix output of enclosed circuit `x` back to its input.
/// Feedback circuit `x` must have an equal number of inputs and outputs.
/// - Inputs: input signal.
/// - Outputs: `x` output signal.
#[inline]
pub fn feedback<T, X, N>(x: An<X>) -> An<FeedbackNode<T, X, N, FrameId<T, N>>>
where
    T: Float,
    X: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    N: Size<T>,
{
    An(FeedbackNode::new(x.0, FrameId::new()))
}

/// Transform channels freely.
///
/// # Example
/// ```
/// # use fundsp::prelude::*;
/// let my_sum = map(|i: &Frame<f64, U2>| Frame::<f64, U1>::splat(i[0] + i[1]));
/// ```
// TODO: ConstantFrame (?) based version for prelude.
#[inline]
pub fn map<T, M, I, O>(f: M) -> MapNode<T, M, I, O>
where
    T: Float,
    M: Clone + Fn(&Frame<T, I>) -> Frame<T, O>,
    I: Size<T>,
    O: Size<T>,
{
    MapNode::new(f)
}

/// Keeps a signal zero centered.
/// Filter cutoff `c` is usually somewhere below the audible range.
/// The default blocker cutoff is 10 Hz.
#[inline]
pub fn dcblock_hz<T: Float, F: Real>(c: F) -> An<DCBlocker<T, F>> {
    An(DCBlocker::new(DEFAULT_SR, c))
}

/// Keeps a signal zero centered.
#[inline]
pub fn dcblock<T: Float, F: Real>() -> An<DCBlocker<T, F>> {
    An(DCBlocker::new(DEFAULT_SR, F::new(10)))
}

#[inline]
pub fn declick<T: Float, F: Real>() -> An<Declicker<T, F>> {
    An(Declicker::new(DEFAULT_SR, F::from_f64(0.010)))
}

// This alternative version includes a DC blocker.
//pub fn declick<T: Float, F: Real>() -> An<PipeNode<T, Declicker<T, F>, DCBlocker<T, F>>> {
//    An(Declicker::new(DEFAULT_SR, F::from_f64(0.010))) >> dcblock()
//}

/// Shape signal with a waveshaper.
#[inline]
pub fn shape<T: Float, S: Fn(T) -> T + Clone>(
    f: S,
) -> An<impl AudioNode<Sample = T, Inputs = U1, Outputs = U1>> {
    An(MapNode::new(move |input: &Frame<T, U1>| {
        [f(input[0])].into()
    }))
}

/// Parameter follower filter with halfway response time `t` seconds.
#[inline]
pub fn follow<T: Float, F: Real, S: ScalarOrPair<Sample = F>>(t: S) -> An<AFollower<T, F, S>> {
    An(AFollower::new(DEFAULT_SR, t))
}

/// Look-ahead limiter with response time in seconds. Look-ahead is equal to the attack time.
#[inline]
pub fn limiter<T: Float, S: ScalarOrPair<Sample = f64>>(time: S) -> An<Limiter<T, U1, S>> {
    An(Limiter::new(DEFAULT_SR, time))
}

/// Stereo look-ahead limiter with response time in seconds. Look-ahead is equal to the attack time.
#[inline]
pub fn stereo_limiter<T: Float, S: ScalarOrPair<Sample = f64>>(time: S) -> An<Limiter<T, U2, S>> {
    An(Limiter::new(DEFAULT_SR, time))
}

/// Pinking filter.
#[inline]
pub fn pinkpass<T: Float, F: Float>() -> An<PinkFilter<T, F>> {
    An(PinkFilter::new())
}

/// Pink noise.
#[inline]
pub fn pink<T: Float, F: Float>() -> An<impl AudioNode<Sample = T, Inputs = U0, Outputs = U1>> {
    white() >> pinkpass::<T, F>()
}

/// Brown noise.
#[inline]
pub fn brown<T: Float, F: Real>() -> An<impl AudioNode<Sample = T, Inputs = U0, Outputs = U1>> {
    // Empirical normalization factor.
    white() >> lowpole_hz::<T, F>(T::from_f64(10.0)) * dc(T::from_f64(13.7))
}

/// Frequency detector.
#[inline]
pub fn goertzel<T: Float, F: Real>() -> An<GoertzelNode<T, F>> {
    An(GoertzelNode::new(DEFAULT_SR))
}

/// Frequency detector of frequency `f` Hz.
#[inline]
pub fn goertzel_hz<T: Float, F: Real>(
    f: T,
) -> An<impl AudioNode<Sample = T, Inputs = U1, Outputs = U1>> {
    (pass() | constant(f)) >> goertzel::<T, F>()
}

/// Feedback delay network.
/// Mix output of enclosed circuit `x` back to its input.
/// The output is diffused with a Hadamard matrix for feedback.
/// Feedback circuit `x` must have an equal number of inputs and outputs.
/// - Inputs: input signal.
/// - Outputs: `x` output signal.
#[inline]
pub fn fdn<T, X, N>(x: An<X>) -> An<FeedbackNode<T, X, N, FrameHadamard<T, N>>>
where
    T: Float,
    X: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    N: Size<T>,
{
    An(FeedbackNode::new(x.0, FrameHadamard::new()))
}

/// Buses a bunch of similar nodes.
#[inline]
pub fn bus<T, N, X>(x: Frame<X, N>) -> An<MultiBusNode<T, N, X>>
where
    T: Float,
    N: Size<T>,
    N: Size<X>,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
{
    An(MultiBusNode::new(x))
}

/// Stacks a bunch of similar nodes.
#[inline]
pub fn stack<T, N, X>(x: Frame<X, N>) -> An<MultiStackNode<T, N, X>>
where
    T: Float,
    N: Size<T>,
    N: Size<X>,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T> + Mul<N>,
    X::Outputs: Size<T> + Mul<N>,
    <X::Inputs as Mul<N>>::Output: Size<T>,
    <X::Outputs as Mul<N>>::Output: Size<T>,
{
    An(MultiStackNode::new(x))
}

/// Branches into a bunch of similar nodes.
#[inline]
pub fn branch<T, N, X>(x: Frame<X, N>) -> An<MultiBranchNode<T, N, X>>
where
    T: Float,
    N: Size<T>,
    N: Size<X>,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T> + Mul<N>,
    <X::Outputs as Mul<N>>::Output: Size<T>,
{
    An(MultiBranchNode::new(x))
}

/// Mixes together a bunch of similar nodes sourcing from disjoint inputs.
#[inline]
pub fn sum<T, N, X>(x: Frame<X, N>) -> An<ReduceNode<T, N, X, FrameAdd<T, X::Outputs>>>
where
    T: Float,
    N: Size<T>,
    N: Size<X>,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T> + Mul<N>,
    X::Outputs: Size<T>,
    <X::Inputs as Mul<N>>::Output: Size<T>,
{
    An(ReduceNode::new(x, FrameAdd::new()))
}

/// Split signal into N channels.
#[inline]
pub fn split<T, N>() -> An<impl AudioNode<Sample = T, Inputs = U1, Outputs = N>>
where
    T: Float,
    N: Size<T>,
    U1: Size<T>,
{
    An(MapNode::new(|x| Frame::splat(x[0])))
}

/// Average N channels into one. Inverse of `split`.
#[inline]
pub fn join<T, N>() -> An<impl AudioNode<Sample = T, Inputs = N, Outputs = U1>>
where
    T: Float,
    N: Size<T>,
    U1: Size<T>,
{
    An(MapNode::new(|x| {
        [x.iter().fold(T::zero(), |acc, &x| acc + x) / T::new(N::I64)].into()
    }))
}

/// Saw wave oscillator.
#[inline]
pub fn saw<T: Float>() -> An<WaveSynth<'static, T, U1>> {
    An(WaveSynth::new(DEFAULT_SR, &SAW_TABLE))
}

/// Saw wave oscillator with phase input.
#[inline]
pub fn sawp<T: Float>() -> An<PhaseSynth<'static, T, U1>> {
    An(PhaseSynth::new(DEFAULT_SR, &SAW_TABLE))
}

/// Saw wave oscillator with extra phase output.
#[inline]
pub fn sawx<T: Float>() -> An<WaveSynth<'static, T, U2>> {
    An(WaveSynth::new(DEFAULT_SR, &SAW_TABLE))
}

/// Square wave oscillator.
#[inline]
pub fn square<T: Float>() -> An<WaveSynth<'static, T, U1>> {
    An(WaveSynth::new(DEFAULT_SR, &SQUARE_TABLE))
}

/// Square wave oscillator with phase input.
#[inline]
pub fn squarep<T: Float>() -> An<PhaseSynth<'static, T, U1>> {
    An(PhaseSynth::new(DEFAULT_SR, &SQUARE_TABLE))
}

/// Square wave oscillator with extra phase output.
#[inline]
pub fn squarex<T: Float>() -> An<WaveSynth<'static, T, U2>> {
    An(WaveSynth::new(DEFAULT_SR, &SQUARE_TABLE))
}

/// Triangle wave oscillator.
#[inline]
pub fn triangle<T: Float>() -> An<WaveSynth<'static, T, U1>> {
    An(WaveSynth::new(DEFAULT_SR, &TRIANGLE_TABLE))
}

/// Triangle wave oscillator with phase input.
#[inline]
pub fn trianglep<T: Float>() -> An<PhaseSynth<'static, T, U1>> {
    An(PhaseSynth::new(DEFAULT_SR, &TRIANGLE_TABLE))
}

/// Triangle wave oscillator with extra phase output.
#[inline]
pub fn trianglex<T: Float>() -> An<WaveSynth<'static, T, U2>> {
    An(WaveSynth::new(DEFAULT_SR, &TRIANGLE_TABLE))
}

/// Fixed saw wave oscillator at `f` Hz.
/// - Output 0: saw wave
#[inline]
pub fn saw_hz<T: Float>(f: T) -> An<impl AudioNode<Sample = T, Inputs = U0, Outputs = U1>> {
    constant(f) >> saw()
}

/// Fixed square wave oscillator at `f` Hz.
/// - Output 0: square wave
#[inline]
pub fn square_hz<T: Float>(f: T) -> An<impl AudioNode<Sample = T, Inputs = U0, Outputs = U1>> {
    constant(f) >> square()
}

/// Fixed triangle wave oscillator at `f` Hz.
/// - Output 0: triangle wave
#[inline]
pub fn triangle_hz<T: Float>(f: T) -> An<impl AudioNode<Sample = T, Inputs = U0, Outputs = U1>> {
    constant(f) >> triangle()
}

/// Lowpass filter.
/// - Input 0: audio
/// - Input 1: cutoff frequency (Hz)
/// - Input 2: Q
/// - Output 0: filtered audio
#[inline]
pub fn lowpass<T: Float, F: Real>() -> An<Svf<T, F, LowpassMode<F>>> {
    An(Svf::new(
        LowpassMode::default(),
        &SvfParams {
            sample_rate: convert(DEFAULT_SR),
            cutoff: F::new(440),
            q: F::one(),
            gain: F::one(),
        },
    ))
}

/// Lowpass filter with cutoff frequency `f` Hz with Q value `q`.
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn lowpass_hz<T: Float, F: Real>(
    f: T,
    q: T,
) -> An<impl AudioNode<Sample = T, Inputs = U1, Outputs = U1>> {
    (pass() | dc((f, q))) >> lowpass::<T, F>()
}

/// Lowpass filter with Q value `q`.
/// - Input 0: audio
/// - Input 1: cutoff frequency (Hz)
/// - Output 0: filtered audio
#[inline]
pub fn lowpass_q<T: Float, F: Real>(
    q: T,
) -> An<impl AudioNode<Sample = T, Inputs = U2, Outputs = U1>> {
    (pass() | pass() | dc(q)) >> lowpass::<T, F>()
}

/// Highpass filter.
/// - Input 0: audio
/// - Input 1: cutoff frequency (Hz)
/// - Input 2: Q
/// - Output 0: filtered audio
#[inline]
pub fn highpass<T: Float, F: Real>() -> An<Svf<T, F, HighpassMode<F>>> {
    An(Svf::new(
        HighpassMode::default(),
        &SvfParams {
            sample_rate: convert(DEFAULT_SR),
            cutoff: F::new(440),
            q: F::one(),
            gain: F::one(),
        },
    ))
}

/// Highpass filter with cutoff frequency `f` Hz with Q value `q`.
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn highpass_hz<T: Float, F: Real>(
    f: T,
    q: T,
) -> An<impl AudioNode<Sample = T, Inputs = U1, Outputs = U1>> {
    (pass() | dc((f, q))) >> highpass::<T, F>()
}

/// Highpass filter with Q value `q`.
/// - Input 0: audio
/// - Input 1: cutoff frequency (Hz)
/// - Output 0: filtered audio
#[inline]
pub fn highpass_q<T: Float, F: Real>(
    q: T,
) -> An<impl AudioNode<Sample = T, Inputs = U2, Outputs = U1>> {
    (pass() | pass() | dc(q)) >> highpass::<T, F>()
}

/// Bandpass filter.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Input 2: Q
/// - Output 0: filtered audio
#[inline]
pub fn bandpass<T: Float, F: Real>() -> An<Svf<T, F, BandpassMode<F>>> {
    An(Svf::new(
        BandpassMode::default(),
        &SvfParams {
            sample_rate: convert(DEFAULT_SR),
            cutoff: F::new(440),
            q: F::one(),
            gain: F::one(),
        },
    ))
}

/// Bandpass filter centered at `f` Hz with Q value `q`.
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn bandpass_hz<T: Float, F: Real>(
    f: T,
    q: T,
) -> An<impl AudioNode<Sample = T, Inputs = U1, Outputs = U1>> {
    (pass() | dc((f, q))) >> bandpass::<T, F>()
}

/// Bandpass filter with Q value `q`.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Output 0: filtered audio
#[inline]
pub fn bandpass_q<T: Float, F: Real>(
    q: T,
) -> An<impl AudioNode<Sample = T, Inputs = U2, Outputs = U1>> {
    (pass() | pass() | dc(q)) >> bandpass::<T, F>()
}

/// Notch filter.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Input 2: Q
/// - Output 0: filtered audio
#[inline]
pub fn notch<T: Float, F: Real>() -> An<Svf<T, F, NotchMode<F>>> {
    An(Svf::new(
        NotchMode::default(),
        &SvfParams {
            sample_rate: convert(DEFAULT_SR),
            cutoff: F::new(440),
            q: F::one(),
            gain: F::one(),
        },
    ))
}

/// Notch filter centered at `f` Hz with Q value `q`.
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn notch_hz<T: Float, F: Real>(
    f: T,
    q: T,
) -> An<impl AudioNode<Sample = T, Inputs = U1, Outputs = U1>> {
    (pass() | dc((f, q))) >> notch::<T, F>()
}

/// Notch filter with Q value `q`.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Output 0: filtered audio
#[inline]
pub fn notch_q<T: Float, F: Real>(
    q: T,
) -> An<impl AudioNode<Sample = T, Inputs = U2, Outputs = U1>> {
    (pass() | pass() | dc(q)) >> notch::<T, F>()
}

/// Peaking filter.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Input 2: Q
/// - Output 0: filtered audio
#[inline]
pub fn peak<T: Float, F: Real>() -> An<Svf<T, F, PeakMode<F>>> {
    An(Svf::new(
        PeakMode::default(),
        &SvfParams {
            sample_rate: convert(DEFAULT_SR),
            cutoff: F::new(440),
            q: F::one(),
            gain: F::one(),
        },
    ))
}

/// Peaking filter centered at `f` Hz with Q value `q`.
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn peak_hz<T: Float, F: Real>(
    f: T,
    q: T,
) -> An<impl AudioNode<Sample = T, Inputs = U1, Outputs = U1>> {
    (pass() | dc((f, q))) >> peak::<T, F>()
}

/// Peaking filter with Q value `q`.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Output 0: filtered audio
#[inline]
pub fn peak_q<T: Float, F: Real>(
    q: T,
) -> An<impl AudioNode<Sample = T, Inputs = U2, Outputs = U1>> {
    (pass() | pass() | dc(q)) >> peak::<T, F>()
}

/// Allpass filter.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Input 2: Q
/// - Output 0: filtered audio
#[inline]
pub fn allpass<T: Float, F: Real>() -> An<Svf<T, F, AllpassMode<F>>> {
    An(Svf::new(
        AllpassMode::default(),
        &SvfParams {
            sample_rate: convert(DEFAULT_SR),
            cutoff: F::new(440),
            q: F::one(),
            gain: F::one(),
        },
    ))
}

/// Allpass filter centered at `f` Hz with Q value `q`.
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn allpass_hz<T: Float, F: Real>(
    f: T,
    q: T,
) -> An<impl AudioNode<Sample = T, Inputs = U1, Outputs = U1>> {
    (pass() | dc((f, q))) >> allpass::<T, F>()
}

/// Allpass filter with Q value `q`.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Output 0: filtered audio
#[inline]
pub fn allpass_q<T: Float, F: Real>(
    q: T,
) -> An<impl AudioNode<Sample = T, Inputs = U2, Outputs = U1>> {
    (pass() | pass() | dc(q)) >> allpass::<T, F>()
}

/// Peaking bell filter with adjustable gain.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Input 2: Q
/// - Input 3: amplitude gain
/// - Output 0: filtered audio
#[inline]
pub fn bell<T: Float, F: Real>() -> An<Svf<T, F, BellMode<F>>> {
    An(Svf::new(
        BellMode::default(),
        &SvfParams {
            sample_rate: convert(DEFAULT_SR),
            cutoff: F::new(440),
            q: F::one(),
            gain: F::one(),
        },
    ))
}

/// Peaking bell filter centered at `f` Hz with Q value `q` and amplitude gain `gain`.
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn bell_hz<T: Float, F: Real>(
    f: T,
    q: T,
    gain: T,
) -> An<impl AudioNode<Sample = T, Inputs = U1, Outputs = U1>> {
    (pass() | dc((f, q, gain))) >> bell::<T, F>()
}

/// Peaking bell filter with adjustable gain centered at `f` Hz with Q value `q`.
/// - Input 0: audio
/// - Input 1: amplitude gain
/// - Output 0: filtered audio
#[inline]
pub fn bell_eq<T: Float, F: Real>(
    f: T,
    q: T,
) -> An<impl AudioNode<Sample = T, Inputs = U2, Outputs = U1>> {
    (pass() | dc((f, q)) | pass()) >> bell::<T, F>()
}

/// Low shelf filter with adjustable gain.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Input 2: Q
/// - Input 3: amplitude gain
/// - Output 0: filtered audio
#[inline]
pub fn lowshelf<T: Float, F: Real>() -> An<Svf<T, F, LowshelfMode<F>>> {
    An(Svf::new(
        LowshelfMode::default(),
        &SvfParams {
            sample_rate: convert(DEFAULT_SR),
            cutoff: F::new(440),
            q: F::one(),
            gain: F::one(),
        },
    ))
}

/// Low shelf filter centered at `f` Hz with Q value `q` and amplitude gain `gain`.
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn lowshelf_hz<T: Float, F: Real>(
    f: T,
    q: T,
    gain: T,
) -> An<impl AudioNode<Sample = T, Inputs = U1, Outputs = U1>> {
    (pass() | dc((f, q, gain))) >> lowshelf::<T, F>()
}

/// Low shelf filter with adjustable gain centered at `f` Hz with Q value `q`.
/// - Input 0: audio
/// - Input 1: amplitude gain
/// - Output 0: filtered audio
#[inline]
pub fn lowshelf_eq<T: Float, F: Real>(
    f: T,
    q: T,
) -> An<impl AudioNode<Sample = T, Inputs = U2, Outputs = U1>> {
    (pass() | dc((f, q)) | pass()) >> lowshelf::<T, F>()
}

/// High shelf filter with adjustable gain.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Input 2: Q
/// - Input 3: amplitude gain
/// - Output 0: filtered audio
#[inline]
pub fn highshelf<T: Float, F: Real>() -> An<Svf<T, F, HighshelfMode<F>>> {
    An(Svf::new(
        HighshelfMode::default(),
        &SvfParams {
            sample_rate: convert(DEFAULT_SR),
            cutoff: F::new(440),
            q: F::one(),
            gain: F::one(),
        },
    ))
}

/// High shelf filter centered at `f` Hz with Q value `q` and amplitude gain `gain`.
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn highshelf_hz<T: Float, F: Real>(
    f: T,
    q: T,
    gain: T,
) -> An<impl AudioNode<Sample = T, Inputs = U1, Outputs = U1>> {
    (pass() | dc((f, q, gain))) >> highshelf::<T, F>()
}

/// High shelf filter with adjustable gain centered at `f` Hz with Q value `q`.
/// - Input 0: audio
/// - Input 1: amplitude gain
/// - Output 0: filtered audio
#[inline]
pub fn highshelf_eq<T: Float, F: Real>(
    f: T,
    q: T,
) -> An<impl AudioNode<Sample = T, Inputs = U2, Outputs = U1>> {
    (pass() | dc((f, q)) | pass()) >> highshelf::<T, F>()
}
