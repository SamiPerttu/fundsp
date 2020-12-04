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
pub fn constant<X: ConstantFrame<Sample = f64>>(x: X) -> An<ConstantNode<f64, X::Size>>
where
    X::Size: Size<f64>,
{
    An(ConstantNode::new(x.convert()))
}

/// Constant node.
/// Synonymous with `constant`.
/// (DC stands for "direct current", which is an electrical engineering term used with signals.)
#[inline]
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
pub fn butterpass() -> An<ButterLowpass<f64, f64>> {
    An(ButterLowpass::new(DEFAULT_SR))
}

/// Butterworth lowpass filter (2nd order) with fixed `cutoff` frequency.
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn butterpass_hz(cutoff: f64) -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = U1>> {
    (pass() | constant(cutoff)) >> butterpass()
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
/// - Output(s): envelope linearly interpolated from samples at 2 ms intervals (average).
#[inline]
pub fn envelope<E, R>(f: E) -> An<impl AudioNode<Sample = f64, Inputs = U0, Outputs = R::Size>>
where
    E: Fn(f64) -> R + Clone,
    R: ConstantFrame<Sample = f64>,
    R::Size: Size<f64>,
{
    // Signals containing frequencies no greater than about 20 Hz would be considered control rate.
    // Therefore, sampling at 500 Hz means these signals are fairly well represented.
    // While we represent time in double precision internally, it is often okay to use single precision
    // in envelopes, as local component time typically does not get far from origin.
    An(EnvelopeNode::new(0.002, DEFAULT_SR, f))
}

/// Control envelope from time-varying function `f(t)` with `t` in seconds.
/// Spaces samples using pseudorandom jittering.
/// Synonymous with `envelope`.
/// - Output(s): envelope linearly interpolated from samples at 2 ms intervals (average).
#[inline]
pub fn lfo<E, R>(f: E) -> An<impl AudioNode<Sample = f64, Inputs = U0, Outputs = R::Size>>
where
    E: Fn(f64) -> R + Clone,
    R: ConstantFrame<Sample = f64>,
    R::Size: Size<f64>,
{
    An(EnvelopeNode::new(0.002, DEFAULT_SR, f))
}

/// Maximum Length Sequence noise generator from an `n`-bit sequence.
/// - Output 0: repeating white noise sequence of only -1 and 1 values.
#[inline]
pub fn mls_bits(n: i64) -> An<MlsNoise<f64>> {
    An(MlsNoise::new(Mls::new(n as u32)))
}

/// Default Maximum Length Sequence noise generator.
/// - Output 0: repeating white noise sequence of only -1 and 1 values.
#[inline]
pub fn mls() -> An<MlsNoise<f64>> {
    mls_bits(29)
}

/// White noise generator.
/// Synonymous with `white`.
/// - Output 0: white noise.
#[inline]
pub fn noise() -> An<NoiseNode<f64>> {
    An(NoiseNode::new())
}

/// White noise generator.
/// Synonymous with `noise`.
/// - Output 0: white noise.
#[inline]
pub fn white() -> An<NoiseNode<f64>> {
    An(NoiseNode::new())
}

/// Single sample delay.
/// - Input 0: signal.
/// - Output 0: delayed signal.
#[inline]
pub fn tick() -> An<TickNode<f64, U1>> {
    An(TickNode::new(DEFAULT_SR))
}

/// Fixed delay of `t` seconds.
/// - Input 0: signal.
/// - Output 0: delayed signal.
#[inline]
pub fn delay(t: f64) -> An<DelayNode<f64>> {
    An(DelayNode::new(t, DEFAULT_SR))
}

/// Mix output of enclosed circuit `x` back to its input.
/// Feedback circuit `x` must have an equal number of inputs and outputs.
/// - Inputs: input signal.
/// - Outputs: `x` output signal.
#[inline]
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
#[inline]
pub fn dcblock_hz(c: f64) -> An<DCBlocker<f64, f64>> {
    An(DCBlocker::new(DEFAULT_SR, c))
}

/// Keeps a signal zero centered.
#[inline]
pub fn dcblock() -> An<DCBlocker<f64, f64>> {
    dcblock_hz(10.0)
}

#[inline]
pub fn declick() -> An<Declicker<f64, f64>> {
    An(Declicker::new(DEFAULT_SR, 0.010))
}

/// Shape signal with a waveshaper.
#[inline]
pub fn shape<S: Fn(f64) -> f64 + Clone>(
    f: S,
) -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = U1>> {
    An(MapNode::new(move |input: &Frame<f64, U1>| {
        [f(input[0])].into()
    }))
}

/// Parameter follower filter with halfway response time `t` seconds.
#[inline]
pub fn follow<S: ScalarOrPair<Sample = f64>>(t: S) -> An<AFollower<f64, f64, S>> {
    An(AFollower::new(DEFAULT_SR, t))
}

/// Look-ahead limiter with `attack` and `release` time in seconds. Look-ahead is equal to the attack time.
#[inline]
pub fn limiter<S: ScalarOrPair<Sample = f64>>(time: S) -> An<Limiter<f64, U1, S>> {
    An(Limiter::new(DEFAULT_SR, time))
}

/// Look-ahead stereo limiter with `attack` and `release` time in seconds. Look-ahead is equal to the attack time.
#[inline]
pub fn stereo_limiter<S: ScalarOrPair<Sample = f64>>(time: S) -> An<Limiter<f64, U2, S>> {
    An(Limiter::new(DEFAULT_SR, time))
}

/// Pinking filter.
#[inline]
pub fn pinkpass() -> An<PinkFilter<f64, f64>> {
    An(PinkFilter::new())
}

/// Pink noise.
#[inline]
pub fn pink() -> An<impl AudioNode<Sample = f64, Inputs = U0, Outputs = U1>> {
    white() >> pinkpass()
}

/// Brown noise.
#[inline]
pub fn brown() -> An<impl AudioNode<Sample = f64, Inputs = U0, Outputs = U1>> {
    // Empirical normalization factor.
    white() >> lowpole_hz(10.0) * dc(13.7)
}

/// Frequency detector.
#[inline]
pub fn goertzel() -> An<GoertzelNode<f64, f64>> {
    An(GoertzelNode::new(DEFAULT_SR))
}

/// Frequency detector of frequency `f` Hz.
#[inline]
pub fn goertzel_hz(f: f64) -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = U1>> {
    (pass() | constant(f)) >> goertzel()
}

/// Feedback delay network.
/// Mix output of enclosed circuit `x` back to its input.
/// The output is diffused with a Hadamard matrix for feedback.
/// Feedback circuit `x` must have an equal number of inputs and outputs.
/// - Inputs: input signal.
/// - Outputs: `x` output signal.
#[inline]
pub fn fdn<X, N>(x: An<X>) -> An<FeedbackNode<f64, X, N, FrameHadamard<f64, N>>>
where
    X: AudioNode<Sample = f64, Inputs = N, Outputs = N>,
    X::Inputs: Size<f64>,
    X::Outputs: Size<f64>,
    N: Size<f64>,
{
    An(FeedbackNode::new(x.0, FrameHadamard::new()))
}

/// Create bus with `n` nodes from indexed generator `f`.
#[inline]
pub fn busi<N, X, F>(f: F) -> An<MultiBusNode<f64, N, X>>
where
    N: Size<f64>,
    N: Size<X>,
    X: AudioNode<Sample = f64>,
    X::Inputs: Size<f64>,
    X::Outputs: Size<f64>,
    F: Fn(i64) -> An<X>,
{
    let nodes = Frame::generate(|i| f(i as i64).0);
    An(MultiBusNode::<f64, N, X>::new(nodes))
}

/// Stacks `n` nodes from a fractional generator,
/// which is given uniformly divided values in 0...1.
#[inline]
pub fn stackf<N, X, F>(f: F) -> An<MultiStackNode<f64, N, X>>
where
    N: Size<f64>,
    N: Size<X>,
    X: AudioNode<Sample = f64>,
    X::Inputs: Size<f64> + Mul<N>,
    X::Outputs: Size<f64> + Mul<N>,
    <X::Inputs as Mul<N>>::Output: Size<f64>,
    <X::Outputs as Mul<N>>::Output: Size<f64>,
    F: Fn(f64) -> An<X>,
{
    let nodes = Frame::generate(|i| {
        f(lerp(
            i as f64 / N::USIZE as f64,
            (i + 1) as f64 / N::USIZE as f64,
            0.5,
        ))
        .0
    });
    An(MultiStackNode::new(nodes))
}

/// Branches into `n` nodes from a fractional generator,
/// which is given uniformly divided values in 0...1.
#[inline]
pub fn branchf<N, X, F>(f: F) -> An<MultiBranchNode<f64, N, X>>
where
    N: Size<f64>,
    N: Size<X>,
    X: AudioNode<Sample = f64>,
    X::Inputs: Size<f64>,
    X::Outputs: Size<f64> + Mul<N>,
    <X::Outputs as Mul<N>>::Output: Size<f64>,
    F: Fn(f64) -> An<X>,
{
    let nodes = Frame::generate(|i| {
        f(lerp(
            i as f64 / N::USIZE as f64,
            (i + 1) as f64 / N::USIZE as f64,
            0.5,
        ))
        .0
    });
    An(MultiBranchNode::new(nodes))
}

/// Mixes together a bunch of similar nodes sourcing from disjoint inputs.
#[inline]
pub fn sumf<N, X, F>(f: F) -> An<ReduceNode<f64, N, X, FrameAdd<f64, X::Outputs>>>
where
    N: Size<f64>,
    N: Size<X>,
    X: AudioNode<Sample = f64>,
    X::Inputs: Size<f64> + Mul<N>,
    X::Outputs: Size<f64>,
    <X::Inputs as Mul<N>>::Output: Size<f64>,
    F: Fn(f64) -> An<X>,
{
    let nodes = Frame::generate(|i| {
        f(lerp(
            i as f64 / N::USIZE as f64,
            (i + 1) as f64 / N::USIZE as f64,
            0.5,
        ))
        .0
    });
    An(ReduceNode::new(nodes, FrameAdd::new()))
}

/// Split signal into N channels.
#[inline]
pub fn split<N>() -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = N>>
where
    N: Size<f64>,
{
    An(MapNode::new(|x| Frame::splat(x[0])))
}

/// Splits M channels into N branches. The output has M * N channels.
#[inline]
pub fn multisplit<M, N>(
) -> An<impl AudioNode<Sample = f64, Inputs = M, Outputs = numeric_array::typenum::Prod<M, N>>>
where
    M: Size<f64> + Mul<N>,
    N: Size<f64>,
    <M as Mul<N>>::Output: Size<f64>,
{
    An(MapNode::new(|x| Frame::generate(|i| x[i % M::USIZE])))
}

/// Average N channels into one. Inverse of `split`.
#[inline]
pub fn join<N>() -> An<impl AudioNode<Sample = f64, Inputs = N, Outputs = U1>>
where
    N: Size<f64>,
    U1: Size<f64>,
{
    An(MapNode::new(|x| {
        [x.iter().fold(0.0, |acc, &x| acc + x) / N::USIZE as f64].into()
    }))
}

/// Average N branches of M channels into one branch with M channels. The input has M * N channels. Inverse of `multisplit`.
#[inline]
pub fn multijoin<M, N>(
) -> An<impl AudioNode<Sample = f64, Inputs = numeric_array::typenum::Prod<M, N>, Outputs = M>>
where
    M: Size<f64> + Mul<N>,
    N: Size<f64>,
    <M as Mul<N>>::Output: Size<f64>,
{
    An(MapNode::new(|x| {
        Frame::generate(|i| {
            let mut output = x[i];
            for j in 1..N::USIZE {
                output += x[i + j * M::USIZE];
            }
            output / N::USIZE as f64
        })
    }))
}

/// Stacks `n` nodes from an indexed generator.
#[inline]
pub fn stacki<N, X, F>(f: F) -> An<MultiStackNode<f64, N, X>>
where
    N: Size<f64>,
    N: Size<X>,
    X: AudioNode<Sample = f64>,
    X::Inputs: Size<f64> + Mul<N>,
    X::Outputs: Size<f64> + Mul<N>,
    <X::Inputs as Mul<N>>::Output: Size<f64>,
    <X::Outputs as Mul<N>>::Output: Size<f64>,
    F: Fn(i64) -> An<X>,
{
    let nodes = Frame::generate(|i| f(i as i64).0);
    An(MultiStackNode::new(nodes))
}

/// Stereo reverb.
/// `wet` in 0...1 is balance of reverb mixed in, for example, 0.1.
/// `time` is approximate reverberation time to -60 dB in seconds.
pub fn stereo_reverb(
    wet: f64,
    time: f64,
) -> An<impl AudioNode<Sample = f64, Inputs = U2, Outputs = U2>> {
    // TODO: This is the simplest possible structure, there's probably a lot of scope for improvement.

    // Optimized delay times for a 32-channel FDN from a legacy project.
    const DELAYS: [f64; 32] = [
        0.073904, 0.052918, 0.066238, 0.066387, 0.037783, 0.080073, 0.050961, 0.075900, 0.043646,
        0.072095, 0.056194, 0.045961, 0.058934, 0.068016, 0.047529, 0.058156, 0.072972, 0.036084,
        0.062715, 0.076377, 0.044339, 0.076725, 0.077884, 0.046126, 0.067741, 0.049800, 0.051709,
        0.082923, 0.070121, 0.079315, 0.055039, 0.081859,
    ];

    let a = pow(db_amp(-60.0), 0.03 / time);

    // The feedback structure.
    let reverb = fdn(stacki::<U32, _, _>(|i| {
        // Index is i64 because of hacker prelude rules.
        // In the standard prelude, the index type would be usize.
        delay(DELAYS[i as usize]) >> lowpole_hz(1600.0) >> dcblock_hz(5.0) * a
    }));

    // Multiplex stereo into 32 channels, reverberate, then average them back.
    // Bus the reverb with the dry signal. Operator precedences work perfectly for us here.
    multisplit::<U2, U16>() >> reverb >> multijoin::<U2, U16>() * wet & mul((1.0 - wet, 1.0 - wet))
}

/// Saw wave oscillator.
#[inline]
pub fn saw() -> An<WaveSynth<'static, f64, U1>> {
    An(WaveSynth::new(DEFAULT_SR, &SAW_TABLE))
}

/// Saw wave oscillator with phase input.
#[inline]
pub fn sawp() -> An<PhaseSynth<'static, f64, U1>> {
    An(PhaseSynth::new(DEFAULT_SR, &SAW_TABLE))
}

/// Saw wave oscillator with extra phase output.
#[inline]
pub fn sawx() -> An<WaveSynth<'static, f64, U2>> {
    An(WaveSynth::new(DEFAULT_SR, &SAW_TABLE))
}

/// Square wave oscillator.
#[inline]
pub fn square() -> An<WaveSynth<'static, f64, U1>> {
    An(WaveSynth::new(DEFAULT_SR, &SQUARE_TABLE))
}

/// Square wave oscillator with phase input.
#[inline]
pub fn squarep() -> An<PhaseSynth<'static, f64, U1>> {
    An(PhaseSynth::new(DEFAULT_SR, &SQUARE_TABLE))
}

/// Square wave oscillator with extra phase output.
#[inline]
pub fn squarex() -> An<WaveSynth<'static, f64, U2>> {
    An(WaveSynth::new(DEFAULT_SR, &SQUARE_TABLE))
}

/// Triangle wave oscillator.
#[inline]
pub fn triangle() -> An<WaveSynth<'static, f64, U1>> {
    An(WaveSynth::new(DEFAULT_SR, &TRIANGLE_TABLE))
}

/// Triangle wave oscillator with phase input.
#[inline]
pub fn trianglep() -> An<PhaseSynth<'static, f64, U1>> {
    An(PhaseSynth::new(DEFAULT_SR, &TRIANGLE_TABLE))
}

/// Triangle wave oscillator with extra phase output.
#[inline]
pub fn trianglex() -> An<WaveSynth<'static, f64, U2>> {
    An(WaveSynth::new(DEFAULT_SR, &TRIANGLE_TABLE))
}

/// Fixed saw wave oscillator at `f` Hz.
/// - Output 0: saw wave
#[inline]
pub fn saw_hz(f: f64) -> An<impl AudioNode<Sample = f64, Inputs = U0, Outputs = U1>> {
    constant(f) >> saw()
}

/// Fixed square wave oscillator at `f` Hz.
/// - Output 0: square wave
#[inline]
pub fn square_hz(f: f64) -> An<impl AudioNode<Sample = f64, Inputs = U0, Outputs = U1>> {
    constant(f) >> square()
}

/// Fixed triangle wave oscillator at `f` Hz.
/// - Output 0: triangle wave
#[inline]
pub fn triangle_hz(f: f64) -> An<impl AudioNode<Sample = f64, Inputs = U0, Outputs = U1>> {
    constant(f) >> triangle()
}

/// Lowpass filter.
/// - Input 0: audio
/// - Input 1: cutoff frequency (Hz)
/// - Input 2: Q
/// - Output 0: filtered audio
#[inline]
pub fn lowpass() -> An<Svf<f64, f64, LowpassMode<f64>>> {
    An(Svf::new(
        LowpassMode::default(),
        &SvfParams {
            sample_rate: DEFAULT_SR,
            cutoff: 440.0,
            q: 1.0,
            gain: 1.0,
        },
    ))
}

/// Lowpass filter with cutoff frequency `f` Hz with Q value `q`.
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn lowpass_hz(f: f64, q: f64) -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = U1>> {
    (pass() | dc((f, q))) >> lowpass()
}

/// Lowpass filter with Q value `q`.
/// - Input 0: audio
/// - Input 1: cutoff frequency (Hz)
/// - Output 0: filtered audio
#[inline]
pub fn lowpass_q(q: f64) -> An<impl AudioNode<Sample = f64, Inputs = U2, Outputs = U1>> {
    (pass() | pass() | dc(q)) >> lowpass()
}

/// Highpass filter.
/// - Input 0: audio
/// - Input 1: cutoff frequency (Hz)
/// - Input 2: Q
/// - Output 0: filtered audio
#[inline]
pub fn highpass() -> An<Svf<f64, f64, HighpassMode<f64>>> {
    An(Svf::new(
        HighpassMode::default(),
        &SvfParams {
            sample_rate: DEFAULT_SR,
            cutoff: 440.0,
            q: 1.0,
            gain: 1.0,
        },
    ))
}

/// Highpass filter with cutoff frequency `f` Hz with Q value `q`.
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn highpass_hz(f: f64, q: f64) -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = U1>> {
    (pass() | dc((f, q))) >> highpass()
}

/// Highpass filter with Q value `q`.
/// - Input 0: audio
/// - Input 1: cutoff frequency (Hz)
/// - Output 0: filtered audio
#[inline]
pub fn highpass_q(q: f64) -> An<impl AudioNode<Sample = f64, Inputs = U2, Outputs = U1>> {
    (pass() | pass() | dc(q)) >> highpass()
}

/// Bandpass filter.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Input 2: Q
/// - Output 0: filtered audio
#[inline]
pub fn bandpass() -> An<Svf<f64, f64, BandpassMode<f64>>> {
    An(Svf::new(
        BandpassMode::default(),
        &SvfParams {
            sample_rate: DEFAULT_SR,
            cutoff: 440.0,
            q: 1.0,
            gain: 1.0,
        },
    ))
}

/// Bandpass filter centered at `f` Hz with Q value `q`.
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn bandpass_hz(f: f64, q: f64) -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = U1>> {
    (pass() | dc((f, q))) >> bandpass()
}

/// Bandpass filter with Q value `q`.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Output 0: filtered audio
#[inline]
pub fn bandpass_q(q: f64) -> An<impl AudioNode<Sample = f64, Inputs = U2, Outputs = U1>> {
    (pass() | pass() | dc(q)) >> bandpass()
}

/// Notch filter.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Input 2: Q
/// - Output 0: filtered audio
#[inline]
pub fn notch() -> An<Svf<f64, f64, NotchMode<f64>>> {
    An(Svf::new(
        NotchMode::default(),
        &SvfParams {
            sample_rate: DEFAULT_SR,
            cutoff: 440.0,
            q: 1.0,
            gain: 1.0,
        },
    ))
}

/// Notch filter centered at `f` Hz with Q value `q`.
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn notch_hz(f: f64, q: f64) -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = U1>> {
    (pass() | dc((f, q))) >> notch()
}

/// Notch filter with Q value `q`.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Output 0: filtered audio
#[inline]
pub fn notch_q(q: f64) -> An<impl AudioNode<Sample = f64, Inputs = U2, Outputs = U1>> {
    (pass() | pass() | dc(q)) >> notch()
}

/// Peaking filter.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Input 2: Q
/// - Output 0: filtered audio
#[inline]
pub fn peak() -> An<Svf<f64, f64, PeakMode<f64>>> {
    An(Svf::new(
        PeakMode::default(),
        &SvfParams {
            sample_rate: DEFAULT_SR,
            cutoff: 440.0,
            q: 1.0,
            gain: 1.0,
        },
    ))
}

/// Peaking filter centered at `f` Hz with Q value `q`.
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn peak_hz(f: f64, q: f64) -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = U1>> {
    (pass() | dc((f, q))) >> peak()
}

/// Peaking filter with Q value `q`.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Output 0: filtered audio
#[inline]
pub fn peak_q(q: f64) -> An<impl AudioNode<Sample = f64, Inputs = U2, Outputs = U1>> {
    (pass() | pass() | dc(q)) >> peak()
}

/// Allpass filter.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Input 2: Q
/// - Output 0: filtered audio
#[inline]
pub fn allpass() -> An<Svf<f64, f64, AllpassMode<f64>>> {
    An(Svf::new(
        AllpassMode::default(),
        &SvfParams {
            sample_rate: DEFAULT_SR,
            cutoff: 440.0,
            q: 1.0,
            gain: 1.0,
        },
    ))
}

/// Allpass filter centered at `f` Hz with Q value `q`.
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn allpass_hz(f: f64, q: f64) -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = U1>> {
    (pass() | dc((f, q))) >> allpass()
}

/// Allpass filter with Q value `q`.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Output 0: filtered audio
#[inline]
pub fn allpass_q(q: f64) -> An<impl AudioNode<Sample = f64, Inputs = U2, Outputs = U1>> {
    (pass() | pass() | dc(q)) >> allpass()
}

/// Peaking bell filter with adjustable gain.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Input 2: Q
/// - Input 3: amplitude gain
/// - Output 0: filtered audio
#[inline]
pub fn bell() -> An<Svf<f64, f64, BellMode<f64>>> {
    An(Svf::new(
        BellMode::default(),
        &SvfParams {
            sample_rate: DEFAULT_SR,
            cutoff: 440.0,
            q: 1.0,
            gain: 1.0,
        },
    ))
}

/// Peaking bell filter centered at `f` Hz with Q value `q` and amplitude gain `gain`.
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn bell_hz(
    f: f64,
    q: f64,
    gain: f64,
) -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = U1>> {
    (pass() | dc((f, q, gain))) >> bell()
}

/// Peaking bell filter with adjustable gain centered at `f` Hz with Q value `q`.
/// - Input 0: audio
/// - Input 1: amplitude gain
/// - Output 0: filtered audio
#[inline]
pub fn bell_eq(f: f64, q: f64) -> An<impl AudioNode<Sample = f64, Inputs = U2, Outputs = U1>> {
    (pass() | dc((f, q)) | pass()) >> bell()
}

/// Low shelf filter with adjustable gain.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Input 2: Q
/// - Input 3: amplitude gain
/// - Output 0: filtered audio
#[inline]
pub fn lowshelf() -> An<Svf<f64, f64, LowshelfMode<f64>>> {
    An(Svf::new(
        LowshelfMode::default(),
        &SvfParams {
            sample_rate: DEFAULT_SR,
            cutoff: 440.0,
            q: 1.0,
            gain: 1.0,
        },
    ))
}

/// Low shelf filter centered at `f` Hz with Q value `q` and amplitude gain `gain`.
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn lowshelf_hz(
    f: f64,
    q: f64,
    gain: f64,
) -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = U1>> {
    (pass() | dc((f, q, gain))) >> lowshelf()
}

/// Low shelf filter with adjustable gain centered at `f` Hz with Q value `q`.
/// - Input 0: audio
/// - Input 1: amplitude gain
/// - Output 0: filtered audio
#[inline]
pub fn lowshelf_eq(f: f64, q: f64) -> An<impl AudioNode<Sample = f64, Inputs = U2, Outputs = U1>> {
    (pass() | dc((f, q)) | pass()) >> lowshelf()
}

/// High shelf filter with adjustable gain.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Input 2: Q
/// - Input 3: amplitude gain
/// - Output 0: filtered audio
#[inline]
pub fn highshelf() -> An<Svf<f64, f64, HighshelfMode<f64>>> {
    An(Svf::new(
        HighshelfMode::default(),
        &SvfParams {
            sample_rate: DEFAULT_SR,
            cutoff: 440.0,
            q: 1.0,
            gain: 1.0,
        },
    ))
}

/// High shelf filter centered at `f` Hz with Q value `q` and amplitude gain `gain`.
/// - Input 0: audio
/// - Output 0: filtered audio
#[inline]
pub fn highshelf_hz(
    f: f64,
    q: f64,
    gain: f64,
) -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = U1>> {
    (pass() | dc((f, q, gain))) >> highshelf()
}

/// High shelf filter with adjustable gain centered at `f` Hz with Q value `q`.
/// - Input 0: audio
/// - Input 1: amplitude gain
/// - Output 0: filtered audio
#[inline]
pub fn highshelf_eq(f: f64, q: f64) -> An<impl AudioNode<Sample = f64, Inputs = U2, Outputs = U1>> {
    (pass() | dc((f, q)) | pass()) >> highshelf()
}
