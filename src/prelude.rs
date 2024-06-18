//! Generic prelude.

extern crate alloc;
use alloc::boxed::Box;
use alloc::sync::Arc;

pub use super::audionode::*;
pub use super::audiounit::*;
pub use super::buffer::*;
pub use super::combinator::*;
pub use super::delay::*;
pub use super::dynamics::*;
pub use super::envelope::*;
pub use super::feedback::*;
pub use super::filter::*;
pub use super::fir::*;
pub use super::follow::*;
pub use super::granular::*;
pub use super::math::*;
pub use super::moog::*;
pub use super::net::*;
pub use super::noise::*;
pub use super::oscillator::*;
pub use super::oversample::*;
pub use super::pan::*;
pub use super::realnet::*;
pub use super::resample::*;
pub use super::resynth::*;
pub use super::reverb::*;
pub use super::rez::*;
pub use super::sequencer::*;
pub use super::setting::*;
pub use super::shape::*;
pub use super::shared::*;
pub use super::signal::*;
pub use super::slot::*;
pub use super::snoop::*;
pub use super::svf::*;
pub use super::system::*;
pub use super::wave::*;
pub use super::wavetable::*;
pub use super::*;

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
pub type U101 = numeric_array::typenum::U101;
pub type U102 = numeric_array::typenum::U102;
pub type U103 = numeric_array::typenum::U103;
pub type U104 = numeric_array::typenum::U104;
pub type U105 = numeric_array::typenum::U105;
pub type U106 = numeric_array::typenum::U106;
pub type U107 = numeric_array::typenum::U107;
pub type U108 = numeric_array::typenum::U108;
pub type U109 = numeric_array::typenum::U109;
pub type U110 = numeric_array::typenum::U110;
pub type U111 = numeric_array::typenum::U111;
pub type U112 = numeric_array::typenum::U112;
pub type U113 = numeric_array::typenum::U113;
pub type U114 = numeric_array::typenum::U114;
pub type U115 = numeric_array::typenum::U115;
pub type U116 = numeric_array::typenum::U116;
pub type U117 = numeric_array::typenum::U117;
pub type U118 = numeric_array::typenum::U118;
pub type U119 = numeric_array::typenum::U119;
pub type U120 = numeric_array::typenum::U120;
pub type U121 = numeric_array::typenum::U121;
pub type U122 = numeric_array::typenum::U122;
pub type U123 = numeric_array::typenum::U123;
pub type U124 = numeric_array::typenum::U124;
pub type U125 = numeric_array::typenum::U125;
pub type U126 = numeric_array::typenum::U126;
pub type U127 = numeric_array::typenum::U127;
pub type U128 = numeric_array::typenum::U128;

/// Constant node. The constant can be scalar, tuple, or a Frame.
/// Synonymous with [`dc`].
/// - Output(s): constant
///
/// ### Example: Sine Oscillator
/// ```
/// use fundsp::prelude::*;
/// constant(440.0) >> sine();
/// ```
pub fn constant<X: ConstantFrame<Sample = f32>>(x: X) -> An<Constant<X::Size>>
where
    X::Size: Size<f32>,
{
    An(Constant::new(x.frame()))
}

/// Constant node. The constant can be scalar, tuple, or a Frame.
/// Synonymous with [`constant`].
/// (DC stands for "direct current", which is an electrical engineering term used with signals.)
/// - Output(s): constant
///
/// ### Example: Dual Sine Oscillator
/// ```
/// use fundsp::prelude::*;
/// dc((220.0, 440.0)) >> (sine() + sine());
/// ```
pub fn dc<X: ConstantFrame<Sample = f32>>(x: X) -> An<Constant<X::Size>>
where
    X::Size: Size<f32>,
{
    An(Constant::new(x.frame()))
}

/// Zero generator.
/// - Output 0: zero
///
/// ### Example: Pluck Oscillator
/// ```
/// use fundsp::prelude::*;
/// zero() >> pluck(220.0, db_amp(-6.0), 0.5);
/// ```
pub fn zero() -> An<Constant<U1>> {
    dc(0.0)
}

/// Multichannel zero generator.
/// - Output(s): zero
///
/// ### Example: Stereo Pluck Oscillator
/// ```
/// use fundsp::prelude::*;
/// multizero::<U2>() >> (pluck(220.0, db_amp(-6.0), 0.5) | pluck(220.0, db_amp(-6.0), 0.5));
/// ```
pub fn multizero<N: Size<f32>>() -> An<Constant<N>> {
    An(Constant::new(Frame::splat(0.0)))
}

/// Update enclosed node `x` with approximately `dt` seconds between updates.
/// The update function is `f(t, dt, x)` where `t` is current time,
/// `dt` is time from previous update, and `x` is the enclosed node.
pub fn update<X: AudioNode, F: FnMut(f32, f32, &mut X) + Clone + Send + Sync>(
    x: An<X>,
    dt: f32,
    f: F,
) -> An<System<X, F>> {
    An(System::new(x, dt, f))
}

/// Mono pass-through.
/// - Input 0: signal
/// - Output 0: signal
///
/// ### Example: Add Feedback Delay
/// ```
/// use fundsp::prelude::*;
/// pass() & 0.2 * feedback(delay(1.0) * db_amp(-3.0));
/// ```
pub fn pass() -> An<Pass> {
    An(Pass::new())
}

/// Multichannel pass-through.
/// - Input(s): signal
/// - Output(s): signal
///
/// ### Example: Add Feedback Delay In Stereo
/// ```
/// use fundsp::prelude::*;
/// multipass::<U2>() & 0.2 * feedback((delay(1.0) | delay(1.0)) * db_amp(-3.0));
/// ```
pub fn multipass<N: Size<f32>>() -> An<MultiPass<N>> {
    An(MultiPass::new())
}

/// Monitor node. Passes through input. Communicates via the shared variable
/// an aspect of the input signal according to the chosen metering mode.
/// - Input 0: signal
/// - Output 0: signal
///
/// ### Example
/// ```
/// use fundsp::prelude::*;
/// let rms = shared(0.0);
/// monitor(&rms, Meter::Rms(0.1));
/// ```
pub fn monitor(shared: &Shared, meter: Meter) -> An<Monitor> {
    An(Monitor::new(shared, meter))
}

/// Meter node.
/// Outputs a summary of the input according to the chosen metering mode.
/// - Input 0: signal
/// - Output 0: summary
///
/// ### Example
/// ```
/// use fundsp::prelude::*;
/// meter(Meter::Rms(0.1));
/// ```
pub fn meter(meter: Meter) -> An<MeterNode> {
    An(MeterNode::new(meter))
}

/// Mono sink. Input is discarded.
/// -Input 0: signal
pub fn sink() -> An<Sink<U1>> {
    An(Sink::new())
}

/// Multichannel sink. Inputs are discarded.
/// -Input(s): signal
pub fn multisink<N: Size<f32>>() -> An<Sink<N>> {
    An(Sink::new())
}

/// Reverse channel order.
/// - Input(s): signal
/// - Output(s): signal in reverse channel order
///
/// ### Example: Ping-Pong Delay
/// ```
/// use fundsp::prelude::*;
/// feedback::<U2, _>((delay(1.0) | delay(1.0)) >> reverse() * db_amp(-3.0));
/// ```
pub fn reverse<N: Size<f32>>() -> An<Reverse<N>> {
    An(Reverse::new())
}

/// Sine oscillator.
/// - Input 0: frequency (Hz)
/// - Output 0: sine wave
///
/// ### Example: Vibrato
/// ```
/// use fundsp::prelude::*;
/// lfo(|t| 110.0 + lerp11(-2.0, 2.0, sin_hz(t, 5.0))) >> sine();
/// ```
pub fn sine() -> An<Sine> {
    An(Sine::new())
}

/// Fixed sine oscillator at `f` Hz.
/// - Output 0: sine wave
///
/// ### Example
/// ```
/// use fundsp::prelude::*;
/// sine_hz(440.0);
/// ```
pub fn sine_hz(f: f32) -> An<Pipe<Constant<U1>, Sine>> {
    constant(f) >> sine()
}

/// Rossler dynamical system oscillator.
/// - Input 0: frequency. The Rossler oscillator exhibits peaks at multiples of this frequency.
/// - Output 0: system output
///
/// ### Example
/// ```
/// use fundsp::prelude::*;
/// lfo(|t| 440.0 + 10.0 * sin_hz(6.0, t)) >> rossler();
/// ```
pub fn rossler() -> An<Rossler> {
    An(Rossler::new())
}

/// Lorenz dynamical system oscillator.
/// - Input 0: frequency. The Lorenz system exhibits slight frequency effects.
/// - Output 0: system output
///
/// ### Example
/// ```
/// use fundsp::prelude::*;
/// lfo(|t| 110.0 + 5.0 * sin_hz(5.0, t)) >> lorenz();
/// ```
pub fn lorenz() -> An<Lorenz> {
    An(Lorenz::new())
}

/// Add constant to signal.
/// - Input(s): signal
/// - Output(s): signal plus constant
pub fn add<X: ConstantFrame<Sample = f32>>(
    x: X,
) -> An<Binop<FrameAdd<X::Size>, MultiPass<X::Size>, Constant<X::Size>>>
where
    X::Size: Size<f32> + Add<U0>,
    <X::Size as Add<U0>>::Output: Size<f32>,
{
    An(MultiPass::new()) + dc(x)
}

/// Subtract constant from signal.
/// - Input(s): signal
/// - Output(s): signal minus constant
pub fn sub<X: ConstantFrame<Sample = f32>>(
    x: X,
) -> An<Binop<FrameSub<X::Size>, MultiPass<X::Size>, Constant<X::Size>>>
where
    X::Size: Size<f32> + Add<U0>,
    <X::Size as Add<U0>>::Output: Size<f32>,
{
    An(MultiPass::new()) - dc(x)
}

/// Multiply signal with constant.
/// - Input(s): signal
/// - Output(s): signal times constant
pub fn mul<X: ConstantFrame<Sample = f32>>(
    x: X,
) -> An<Binop<FrameMul<X::Size>, MultiPass<X::Size>, Constant<X::Size>>>
where
    X::Size: Size<f32> + Add<U0>,
    <X::Size as Add<U0>>::Output: Size<f32>,
{
    An(MultiPass::new()) * dc(x)
}

/// Butterworth lowpass filter (2nd order).
/// - Input 0: audio
/// - Input 1: cutoff frequency (Hz)
/// - Output 0: filtered audio
///
/// ### Example: Filtered Noise
/// ```
/// use fundsp::prelude::*;
/// (noise() | dc(1000.0)) >> butterpass::<f32>();
/// ```
pub fn butterpass<F: Real>() -> An<ButterLowpass<F, U2>> {
    An(ButterLowpass::new(F::new(440)))
}

/// Butterworth lowpass filter (2nd order) with fixed cutoff frequency `f` Hz.
/// - Input 0: audio
/// - Output 0: filtered audio
pub fn butterpass_hz<F: Real>(f: F) -> An<ButterLowpass<F, U1>> {
    An(ButterLowpass::new(f))
}

/// One-pole lowpass filter (1st order).
/// - Input 0: audio
/// - Input 1: cutoff frequency (Hz)
/// - Output 0: filtered audio
///
/// ### Example: Brown Noise
/// ```
/// use fundsp::prelude::*;
/// (noise() | dc(10.0)) >> lowpole::<f32>();
/// ```
pub fn lowpole<F: Real>() -> An<Lowpole<F, U2>> {
    An(Lowpole::new(F::new(440)))
}

/// One-pole lowpass filter (1st order) with fixed `cutoff` frequency in Hz.
/// - Input 0: audio
/// - Output 0: filtered audio
///
/// ### Example: Brown Noise
/// ```
/// use fundsp::prelude::*;
/// noise() >> lowpole_hz::<f32>(10.0);
/// ```
pub fn lowpole_hz<F: Real>(cutoff: F) -> An<Lowpole<F, U1>> {
    An(Lowpole::new(cutoff))
}

/// Allpass filter (1st order) with a configurable delay (delay > 0) in samples at DC.
/// - Input 0: audio
/// - Input 1: delay in samples
/// - Output 0: filtered audio
pub fn allpole<F: Float>() -> An<Allpole<F, U2>> {
    An(Allpole::new(F::new(1)))
}

/// Allpass filter (1st order) with `delay` (`delay` > 0) in samples at DC.
/// - Input 0: audio
/// - Output 0: filtered audio
pub fn allpole_delay<F: Float>(delay: F) -> An<Allpole<F, U1>> {
    An(Allpole::new(delay))
}

/// One-pole, one-zero highpass filter (1st order).
/// - Input 0: audio
/// - Input 1: cutoff frequency (Hz)
/// - Output 0: filtered audio
pub fn highpole<F: Real>() -> An<Highpole<F, U2>> {
    An(Highpole::new(F::new(440)))
}

/// One-pole, one-zero highpass filter (1st order) with fixed `cutoff` frequency.
/// - Input 0: audio
/// - Output 0: filtered audio
pub fn highpole_hz<F: Real>(cutoff: F) -> An<Highpole<F, U1>> {
    An(Highpole::new(cutoff))
}

/// Constant-gain bandpass resonator.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Input 2: bandwidth (Hz)
/// - Output 0: filtered audio
///
/// ### Example: Filtered Noise Tone
/// ```
/// use fundsp::prelude::*;
/// (noise() | dc((440.0, 5.0))) >> resonator::<f64>();
/// ```
pub fn resonator<F: Real>() -> An<Resonator<F, U3>> {
    An(Resonator::new(F::new(440), F::new(110)))
}

/// Constant-gain bandpass resonator with fixed `center` frequency (Hz) and `bandwidth` (Hz).
/// - Input 0: audio
/// - Output 0: filtered audio
///
/// ### Example: Filtered Noise Tone
/// ```
/// use fundsp::prelude::*;
/// noise() >> resonator_hz::<f64>(440.0, 5.0);
/// ```
pub fn resonator_hz<F: Real>(center: F, bandwidth: F) -> An<Resonator<F, U1>> {
    An(Resonator::new(center, bandwidth))
}

/// An arbitrary biquad filter with coefficients in normalized form.
/// - Input 0: signal
/// - Output 0: filtered signal
pub fn biquad<F: Real>(a1: F, a2: F, b0: F, b1: F, b2: F) -> An<Biquad<F>> {
    An(Biquad::with_coefs(BiquadCoefs::arbitrary(
        a1, a2, b0, b1, b2,
    )))
}

/// Moog resonant lowpass filter.
/// - Input 0: input signal
/// - Input 1: cutoff frequency (Hz)
/// - Input 2: Q
/// - Output 0: filtered signal
pub fn moog<F: Real>() -> An<Moog<F, U3>> {
    An(Moog::new(F::new(1000), F::from_f64(0.1)))
}

/// Moog resonant lowpass filter with fixed Q.
/// - Input 0: input signal
/// - Input 1: cutoff frequency (Hz)
/// - Output 0: filtered signal
pub fn moog_q<F: Real>(q: F) -> An<Pipe<Stack<MultiPass<U2>, Constant<U1>>, Moog<F, U3>>> {
    (multipass::<U2>() | dc(q.to_f32())) >> An(Moog::new(F::new(1000), q))
}

/// Moog resonant lowpass filter with fixed cutoff frequency and Q.
/// - Input 0: input signal
/// - Output 0: filtered signal
pub fn moog_hz<F: Real>(frequency: F, q: F) -> An<Moog<F, U1>> {
    An(Moog::new(frequency, q))
}

/// Control envelope from time-varying function `f(t)` with `t` in seconds.
/// Spaces samples using pseudorandom jittering.
/// Synonymous with [`fn@lfo`].
/// - Output(s): envelope linearly interpolated from samples at 2 ms intervals (average).
///
/// ### Example: Mixing Pink And Brown Noise
/// ```
/// use fundsp::prelude::*;
/// envelope(|t| (sin_hz(1.0, t), cos_hz(1.0, t))) * (pink::<f32>() | brown::<f32>()) >> join();
/// ```
pub fn envelope<F, E, R>(f: E) -> An<Envelope<F, E, R>>
where
    F: Float,
    E: FnMut(F) -> R + Clone + Send + Sync,
    R: ConstantFrame<Sample = F>,
    R::Size: Size<F> + Size<f32>,
{
    // Signals containing frequencies no greater than about 20 Hz would be considered control rate.
    // Therefore, sampling at 500 Hz means these signals are fairly well represented.
    // While we represent time in double precision internally, it is often okay to use single precision
    // in envelopes, as local component time typically does not get far from origin.
    An(Envelope::new(F::from_f64(0.002), f))
}

/// Control envelope from time-varying function `f(t)` with `t` in seconds.
/// Spaces samples using pseudorandom jittering.
/// Synonymous with [`fn@envelope`].
/// - Output(s): envelope linearly interpolated from samples at 2 ms intervals (average).
///
/// ### Example: Exponentially Decaying White Noise
/// ```
/// use fundsp::prelude::*;
/// lfo(|t: f32| exp(-t)) * white();
/// ```
pub fn lfo<F, E, R>(f: E) -> An<Envelope<F, E, R>>
where
    F: Float,
    E: FnMut(F) -> R + Clone + Send + Sync,
    R: ConstantFrame<Sample = F>,
    R::Size: Size<F> + Size<f32>,
{
    An(Envelope::new(F::from_f64(0.002), f))
}

/// Control envelope from time-varying, input dependent function `f(t, x)` with `t` in seconds.
/// Spaces samples using pseudorandom jittering.
/// Synonymous with `lfo2`.
/// - Input 0: x
/// - Output(s): envelope linearly interpolated from samples at 2 ms intervals (average).
///
/// ### Example (LFO Speed Control)
/// ```
/// use fundsp::prelude::*;
/// let speed = shared(1.0);
/// var(&speed) >> envelope2(|t: f32, speed: f32| exp(-t * speed));
/// ```
pub fn envelope2<F, E, R>(
    mut f: E,
) -> An<EnvelopeIn<F, impl FnMut(F, &Frame<f32, U1>) -> R + Clone, U1, R>>
where
    F: Float,
    E: FnMut(F, F) -> R + Clone + Send + Sync,
    R: ConstantFrame<Sample = F>,
    R::Size: Size<F> + Size<f32>,
{
    An(EnvelopeIn::new(
        F::from_f64(0.002),
        move |t, i: &Frame<f32, U1>| f(t, convert(i[0])),
    ))
}

/// Control envelope from time-varying, input dependent function `f(t, x)` with `t` in seconds.
/// Spaces samples using pseudorandom jittering.
/// Synonymous with `envelope2`.
/// - Input 0: x
/// - Output(s): envelope linearly interpolated from samples at 2 ms intervals (average).
///
/// ### Example (Amplitude Control)
/// ```
/// use fundsp::prelude::*;
/// let amp = shared(1.0);
/// var(&amp) >> lfo2(|t: f32, amp: f32| amp * exp(-t));
/// ```
pub fn lfo2<F, E, R>(
    mut f: E,
) -> An<EnvelopeIn<F, impl FnMut(F, &Frame<f32, U1>) -> R + Clone, U1, R>>
where
    F: Float,
    E: FnMut(F, F) -> R + Clone + Send + Sync,
    R: ConstantFrame<Sample = F>,
    R::Size: Size<F> + Size<f32>,
{
    An(EnvelopeIn::new(
        F::from_f64(0.002),
        move |t, i: &Frame<f32, U1>| f(t, convert(i[0])),
    ))
}

/// Control envelope from time-varying, input dependent function `f(t, x, y)` with `t` in seconds.
/// Spaces samples using pseudorandom jittering.
/// Synonymous with `lfo3`.
/// - Input 0: x
/// - Input 1: y
/// - Output(s): envelope linearly interpolated from samples at 2 ms intervals (average).
pub fn envelope3<F, E, R>(
    mut f: E,
) -> An<EnvelopeIn<F, impl FnMut(F, &Frame<f32, U2>) -> R + Clone, U2, R>>
where
    F: Float,
    E: FnMut(F, F, F) -> R + Clone + Send + Sync,
    R: ConstantFrame<Sample = F>,
    R::Size: Size<F> + Size<f32>,
{
    An(EnvelopeIn::new(
        F::from_f64(0.002),
        move |t, i: &Frame<f32, U2>| f(t, convert(i[0]), convert(i[1])),
    ))
}

/// Control envelope from time-varying, input dependent function `f(t, x, y)` with `t` in seconds.
/// Spaces samples using pseudorandom jittering.
/// Synonymous with `envelope3`.
/// - Input 0: x
/// - Input 1: y
/// - Output(s): envelope linearly interpolated from samples at 2 ms intervals (average).
///
/// ### Example (Clamped Sine)
/// ```
/// use fundsp::prelude::*;
/// let min = shared(-1.0);
/// let max = shared(1.0);
/// (var(&min) | var(&max)) >> lfo3(|t, min, max| clamp(min, max, sin_hz(110.0, t)));
/// max.set(0.5);
/// ```
pub fn lfo3<F, E, R>(
    mut f: E,
) -> An<EnvelopeIn<F, impl FnMut(F, &Frame<f32, U2>) -> R + Clone, U2, R>>
where
    F: Float,
    E: FnMut(F, F, F) -> R + Clone + Send + Sync,
    R: ConstantFrame<Sample = F>,
    R::Size: Size<F> + Size<f32>,
{
    An(EnvelopeIn::new(
        F::from_f64(0.002),
        move |t, i: &Frame<f32, U2>| f(t, convert(i[0]), convert(i[1])),
    ))
}

/// Control envelope from time-varying, input dependent function `f(t, i)` with `t` in seconds
/// and `i` of type `&Frame<T, I>` where `I` is the number of input channels.
/// Spaces samples using pseudorandom jittering.
/// Synonymous with `lfo_in`.
/// - Inputs: i
/// - Output(s): envelope linearly interpolated from samples at 2 ms intervals (average).
pub fn envelope_in<F, E, I, R>(f: E) -> An<EnvelopeIn<F, E, I, R>>
where
    F: Float,
    E: FnMut(F, &Frame<f32, I>) -> R + Clone + Send + Sync,
    I: Size<f32>,
    R: ConstantFrame<Sample = F>,
    R::Size: Size<F> + Size<f32>,
{
    An(EnvelopeIn::new(F::from_f64(0.002), f))
}

/// Control envelope from time-varying, input dependent function `f(t, i)` with `t` in seconds
/// and `i` of type `&Frame<T, I>` where `I` is the number of input channels.
/// Spaces samples using pseudorandom jittering.
/// Synonymous with `envelope_in`.
/// - Inputs: i
/// - Output(s): envelope linearly interpolated from samples at 2 ms intervals (average).
pub fn lfo_in<F, E, I, R>(f: E) -> An<EnvelopeIn<F, E, I, R>>
where
    F: Float,
    E: FnMut(F, &Frame<f32, I>) -> R + Clone + Send + Sync,
    I: Size<f32>,
    R: ConstantFrame<Sample = F>,
    R::Size: Size<F> + Size<f32>,
{
    An(EnvelopeIn::new(F::from_f64(0.002), f))
}

/// ADSR envelope.
///
/// When a positive value is given by the input, its output increases from 0.0 to 1.0 in the time
/// interval denoted by `attack`. It then decreases from 1.0 to the `sustain` level in the time
/// interval denoted by `decay`. It remains at the `sustain` level until a zero or negative value
/// is given by the input, after which it decreases from the `sustain` level to 0.0 in a time
/// interval denoted by `release`.
///
/// - Input 0: control start of attack and release
/// - Output 0: scaled ADSR value from 0.0 to 1.0
///
/// See [live_adsr.rs](https://github.com/SamiPerttu/fundsp/blob/master/examples/live_adsr.rs) for
/// a program that uses this function to control the volume of live notes from a MIDI instrument.
pub fn adsr_live(
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
) -> An<EnvelopeIn<f32, impl FnMut(f32, &Frame<f32, U1>) -> f32 + Clone, U1, f32>> {
    super::adsr::adsr_live(attack, decay, sustain, release)
}

/// Maximum Length Sequence noise generator from an `n`-bit sequence (1 <= `n` <= 31).
/// - Output 0: repeating white noise sequence of only -1 and 1 values.
///
/// ### Example
/// ```
/// use fundsp::prelude::*;
/// mls_bits(31);
/// ```
pub fn mls_bits(n: u64) -> An<Mls> {
    An(Mls::new(MlsState::new(n as u32)))
}

/// Default Maximum Length Sequence noise generator.
/// - Output 0: repeating white noise sequence of only -1 and 1 values.
///
/// ### Example
/// ```
/// use fundsp::prelude::*;
/// mls();
/// ```
pub fn mls() -> An<Mls> {
    mls_bits(29)
}

/// White noise generator.
/// Synonymous with [`fn@white`].
/// - Output 0: white noise.
///
/// ### Example
/// ```
/// use fundsp::prelude::*;
/// noise();
/// ```
pub fn noise() -> An<Noise> {
    An(Noise::new())
}

/// White noise generator.
/// Synonymous with [`fn@noise`].
/// - Output 0: white noise.
///
/// ### Example
/// ```
/// use fundsp::prelude::*;
/// white();
/// ```
pub fn white() -> An<Noise> {
    An(Noise::new())
}

/// Sample-and-hold component. Sampling frequency `variability` is in 0...1.
/// - Input 0: signal.
/// - Input 1: sampling frequency (Hz).
/// - Output 0: sampled signal.
///
/// ### Example (Sampled And Held Noise)
/// ```
/// use fundsp::prelude::*;
/// (pink::<f64>() | dc(440.0)) >> hold(0.5);
/// ```
pub fn hold(variability: f32) -> An<Hold> {
    An(Hold::new(variability))
}

/// Sample-and-hold component. Sampling frequency `variability` is in 0...1.
/// - Input 0: signal.
/// - Output 0: sampled signal.
pub fn hold_hz(f: f32, variability: f32) -> An<Pipe<Stack<Pass, Constant<U1>>, Hold>> {
    (pass() | dc(f)) >> hold(variability)
}

/// FIR filter.
/// - Input 0: signal.
/// - Output 0: filtered signal.
///
/// ### Example: 3-Point Lowpass Filter
/// ```
/// use fundsp::prelude::*;
/// fir(Frame::<f32, _>::from([0.5, 1.0, 0.5]));
/// ```
pub fn fir<X: ConstantFrame<Sample = f32>>(weights: X) -> An<Fir<X::Size>> {
    An(Fir::new(weights))
}

/// Create a 3-point symmetric FIR from desired `gain` (`gain` >= 0) at the Nyquist frequency.
/// Results in a monotonic low-pass filter when `gain` < 1.
/// - Input 0: signal.
/// - Output 0: filtered signal.
pub fn fir3(gain: f32) -> An<Fir<U3>> {
    let alpha = (gain + 1.0) / 2.0;
    let beta = (1.0 - alpha) / 2.0;
    An(Fir::new((beta, alpha, beta)))
}

/// Single sample delay.
/// - Input 0: signal.
/// - Output 0: delayed signal.
///
/// ### Example: 2-Point Sum Filter
/// ```
/// use fundsp::prelude::*;
/// tick() & pass();
/// ```
pub fn tick() -> An<Tick<U1>> {
    An(Tick::new())
}

/// Multichannel single sample delay.
/// - Inputs: signal.
/// - Outputs: delayed signal.
///
/// ### Example
/// ```
/// use fundsp::prelude::*;
/// multitick::<U2>();
/// ```
pub fn multitick<N: Size<f32>>() -> An<Tick<N>> {
    An(Tick::new())
}

/// Fixed delay of `t` seconds.
/// Delay time is rounded to the nearest sample. The minimum delay is one sample.
/// - Allocates: the delay line.
/// - Input 0: signal.
/// - Output 0: delayed signal.
///
/// ### Example
/// ```
/// use fundsp::prelude::*;
/// delay(1.0);
/// ```
pub fn delay(t: f64) -> An<Delay> {
    An(Delay::new(t))
}

/// Tapped delay line with cubic interpolation.
/// Minimum and maximum delay times are in seconds.
/// - Allocates: the delay line.
/// - Input 0: signal.
/// - Input 1: delay time in seconds.
/// - Output 0: delayed signal.
///
/// ### Example: Variable Delay
/// ```
/// use fundsp::prelude::*;
/// pass() & (pass() | lfo(|t| lerp11(0.01, 0.1, spline_noise(0, t)))) >> tap(0.01, 0.1);
/// ```
pub fn tap(min_delay: f32, max_delay: f32) -> An<Tap<U1>> {
    An(Tap::new(min_delay, max_delay))
}

/// Tapped delay line with cubic interpolation.
/// The number of taps is `N`.
/// Minimum and maximum delay times are in seconds.
/// - Allocates: the delay line.
/// - Input 0: signal.
/// - Inputs 1...N: delay time in seconds.
/// - Output 0: delayed signal.
///
/// ### Example: Dual Variable Delay
/// ```
/// use fundsp::prelude::*;
/// (pass() | lfo(|t| (lerp11(0.01, 0.1, spline_noise(0, t)), lerp11(0.1, 0.2, spline_noise(1, t))))) >> multitap::<U2>(0.01, 0.2);
/// ```
pub fn multitap<N>(min_delay: f32, max_delay: f32) -> An<Tap<N>>
where
    N: Size<f32> + Add<U1>,
    <N as Add<U1>>::Output: Size<f32>,
{
    An(Tap::new(min_delay, max_delay))
}

/// Tapped delay line with linear interpolation.
/// Minimum and maximum delay times are in seconds.
/// - Allocates: the delay line.
/// - Input 0: signal.
/// - Input 1: delay time in seconds.
/// - Output 0: delayed signal.
///
/// ### Example: Variable Delay
/// ```
/// use fundsp::prelude::*;
/// pass() & (pass() | lfo(|t| lerp11(0.01, 0.1, spline_noise(0, t)))) >> tap_linear(0.01, 0.1);
/// ```
pub fn tap_linear(min_delay: f32, max_delay: f32) -> An<TapLinear<U1>> {
    An(TapLinear::new(min_delay, max_delay))
}

/// Tapped delay line with linear interpolation.
/// The number of taps is `N`.
/// Minimum and maximum delay times are in seconds.
/// - Allocates: the delay line.
/// - Input 0: signal.
/// - Inputs 1...N: delay time in seconds.
/// - Output 0: delayed signal.
///
/// ### Example: Dual Variable Delay
/// ```
/// use fundsp::prelude::*;
/// (pass() | lfo(|t| (lerp11(0.01, 0.1, spline_noise(0, t)), lerp11(0.1, 0.2, spline_noise(1, t))))) >> multitap_linear::<U2>(0.01, 0.2);
/// ```
pub fn multitap_linear<N>(min_delay: f32, max_delay: f32) -> An<TapLinear<N>>
where
    N: Size<f32> + Add<U1>,
    <N as Add<U1>>::Output: Size<f32>,
{
    An(TapLinear::new(min_delay, max_delay))
}

/// 2x oversample enclosed `node`.
/// - Inputs and outputs: from `node`.
///
/// ### Example: Oversampled FM Oscillator
/// ```
/// use fundsp::prelude::*;
/// let f: f32 = 440.0;
/// let m: f32 = 1.0;
/// oversample(sine_hz(f) * f * m + f >> sine());
/// ```
pub fn oversample<X>(node: An<X>) -> An<Oversampler<X>>
where
    X: AudioNode,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
    X::Inputs: Size<Frame<f32, U128>>,
    X::Outputs: Size<Frame<f32, U128>>,
{
    An(Oversampler::new(DEFAULT_SR, node.0))
}

/// Resample enclosed generator `node` using cubic interpolation
/// at speed obtained from input 0, where 1 is the original speed.
/// Input 0: Sampling speed.
/// Output(s): Resampled outputs of contained generator.
///
/// ### Example: Resampled Pink Noise
/// ```
/// use fundsp::prelude::*;
/// lfo(|t: f64| xerp11(0.5, 2.0, spline_noise(1, t))) >> resample(pink::<f64>());
/// ```
pub fn resample<X>(node: An<X>) -> An<Resampler<X>>
where
    X: AudioNode<Inputs = U0>,
    X::Outputs: Size<f32>,
    X::Outputs: Size<Frame<f32, U128>>,
{
    An(Resampler::new(DEFAULT_SR, node.0))
}

/// Mix output of enclosed circuit `node` back to its input.
/// Feedback circuit `node` must have an equal number of inputs and outputs.
/// - Input(s): signal.
/// - Output(s): signal with feedback.
///
/// ### Example: Feedback Delay With Lowpass
/// ```
/// use fundsp::prelude::*;
/// pass() & feedback(delay(1.0) >> lowpass_hz::<f64>(1000.0, 1.0));
/// ```
pub fn feedback<N, X>(node: An<X>) -> An<Feedback<N, X, FrameId<N>>>
where
    N: Size<f32>,
    X: AudioNode<Inputs = N, Outputs = N>,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
{
    An(Feedback::new(node.0, FrameId::new()))
}

/// Mix output of enclosed circuit `node` back to its input
/// with extra `loopback` feedback loop processing.
/// Feedback circuits `node` and `loopback` must have an equal number of inputs and outputs.
/// - Input(s): signal.
/// - Output(s): signal with feedback.
///
/// ### Example: Feedback Delay With Lowpass
/// ```
/// use fundsp::prelude::*;
/// pass() & feedback2(delay(1.0), lowpass_hz::<f32>(1000.0, 1.0));
/// ```
pub fn feedback2<N, X, Y>(node: An<X>, loopback: An<Y>) -> An<Feedback2<N, X, Y, FrameId<N>>>
where
    N: Size<f32>,
    X: AudioNode<Inputs = N, Outputs = N>,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
    Y: AudioNode<Inputs = N, Outputs = N>,
    Y::Inputs: Size<f32>,
    Y::Outputs: Size<f32>,
{
    An(Feedback2::new(node.0, loopback.0, FrameId::new()))
}

/// A nested allpass. The feedforward coefficient of the outer allpass
/// is set from `coefficient`, which should have an absolute value smaller than one to prevent a blowup.
/// The delay element of the outer allpass is replaced with `x`.
/// The result is an allpass filter if `x` is allpass.
/// If `x` is `pass()` then the result is a 1st order allpass.
/// If `x` is a delay element then the result is a Schroeder allpass.
/// If `x` is a 1st order allpass (`allpole`) then the result is a 2nd order nested allpass.
/// - Input 0: input signal
/// - Output 0: filtered signal
///
/// ### Example: Schroeder Allpass
/// ```
/// use fundsp::prelude::*;
/// allnest_c(0.5, delay(0.01));
/// ```
pub fn allnest_c<X>(coefficient: f32, x: An<X>) -> An<AllNest<U1, X>>
where
    X: AudioNode<Inputs = U1, Outputs = U1>,
{
    An(AllNest::new(coefficient, x.0))
}

/// A nested allpass. The feedforward coefficient of the outer allpass
/// is set from the second input, which should have an absolute value smaller than one to prevent a blowup.
/// The delay element of the outer allpass is replaced with `x`.
/// The result is an allpass filter if `x` is allpass.
/// If `x` is `pass()` then the result is a 1st order allpass.
/// If `x` is a delay element then the result is a Schroeder allpass.
/// If `x` is a 1st order allpass (`allpole`) then the result is a 2nd order nested allpass.
/// - Input 0: input signal
/// - Input 1: feedforward coefficient
/// - Output 0: filtered signal
///
/// ### Example: Schroeder Allpass
/// ```
/// use fundsp::prelude::*;
/// allnest(delay(0.01));
/// ```
pub fn allnest<X>(x: An<X>) -> An<AllNest<U2, X>>
where
    X: AudioNode<Inputs = U1, Outputs = U1>,
{
    An(AllNest::new(0.0, x.0))
}

/// Transform channels freely. Accounted as non-linear processing for signal flow.
///
/// ### Example: Max Operator
/// ```
/// use fundsp::prelude::*;
/// map(|i: &Frame<f32, U2>| max(i[0], i[1]));
/// ```
pub fn map<M, I, O>(f: M) -> An<Map<M, I, O>>
where
    M: Fn(&Frame<f32, I>) -> O + Clone + Send + Sync,
    I: Size<f32>,
    O: ConstantFrame<Sample = f32>,
    O::Size: Size<f32>,
{
    An(Map::new(f, Routing::Arbitrary(0.0)))
}

/// Keeps a signal zero centered.
/// Filter `cutoff` (in Hz) is usually somewhere below the audible range.
/// The default blocker cutoff is 10 Hz.
/// - Input 0: signal
/// - Output 0: filtered signal
///
/// ### Example
/// ```
/// use fundsp::prelude::*;
/// dcblock_hz::<f64>(8.0);
/// ```
pub fn dcblock_hz<F: Real>(cutoff: F) -> An<DCBlock<F>> {
    An(DCBlock::new(cutoff))
}

/// Keeps a signal zero centered. The cutoff of the filter is 10 Hz.
/// - Input 0: signal
/// - Output 0: filtered signal
///
/// ### Example: Stereo DC Blocker
/// ```
/// use fundsp::prelude::*;
/// dcblock::<f32>() | dcblock::<f32>();
/// ```
pub fn dcblock<F: Real>() -> An<DCBlock<F>> {
    An(DCBlock::new(F::new(10)))
}

/// Apply 10 ms of fade-in to signal at time zero.
/// - Input 0: input signal
/// - Output 0: signal with fade-in
pub fn declick<F: Real>() -> An<Declick<F>> {
    An(Declick::new(F::from_f64(0.010)))
}

/// Apply `t` seconds of fade-in to signal at time zero.
/// - Input 0: input signal
/// - Output 0: signal with fade-in
pub fn declick_s<F: Real>(t: F) -> An<Declick<F>> {
    An(Declick::new(t))
}

/// Shape signal with a waveshaper function.
/// - Input 0: input signal
/// - Output 0: shaped signal
pub fn shape_fn<S: Fn(f32) -> f32 + Clone + Send + Sync>(f: S) -> An<Shaper<ShapeFn<S>>> {
    An(Shaper::new(ShapeFn(f)))
}

/// Shape signal.
/// - Input 0: input signal
/// - Output 0: shaped signal
///
/// ### Example: Tanh Distortion
/// ```
/// use fundsp::prelude::*;
/// shape(Tanh(1.0));
/// ```
pub fn shape<S: Shape>(mode: S) -> An<Shaper<S>> {
    An(Shaper::new(mode))
}

/// Clip signal to -1...1.
/// - Input 0: input signal
/// - Output 0: clipped signal
pub fn clip() -> An<Shaper<Clip>> {
    An(Shaper::new(Clip))
}

/// Clip signal to `minimum`...`maximum`.
/// - Input 0: input signal
/// - Output 0: clipped signal
pub fn clip_to(minimum: f32, maximum: f32) -> An<Shaper<ClipTo>> {
    An(Shaper::new(ClipTo(minimum, maximum)))
}

/// Equal power mono-to-stereo panner.
/// - Input 0: input signal
/// - Input 1: pan in -1...1 (left to right).
/// - Output 0: left channel
/// - Output 1: right channel
///
/// ### Example: Panning Noise
/// ```
/// use fundsp::prelude::*;
/// (noise() | sine_hz(0.5)) >> panner();
/// ```
pub fn panner() -> An<Panner<U2>> {
    An(Panner::new(0.0))
}

/// Fixed equal power mono-to-stereo panner with `pan` value in -1...1 (left to right).
/// - Input 0: input signal
/// - Output 0: left channel
/// - Output 1: right channel
///
/// ### Example (Center Panned Saw Wave)
/// ```
/// use fundsp::prelude::*;
/// saw_hz(440.0) >> pan(0.0);
/// ```
pub fn pan(pan: f32) -> An<Panner<U1>> {
    An(Panner::new(pan))
}

/// Parameter follower filter with halfway response time in seconds.
/// - Input 0: input signal
/// - Output 0: smoothed signal
///
/// ### Example (Smoothed Atomic Parameter)
/// ```
/// use fundsp::prelude::*;
/// let parameter = shared(1.0);
/// var(&parameter) >> follow(0.01);
/// ```
pub fn follow<F: Real>(response_time: F) -> An<Follow<F>> {
    An(Follow::new(response_time))
}

/// Parameter follower filter with halfway response times in seconds.
/// The attack time is used for rising segments while the release time is used for falling segments.
/// - Input 0: input signal
/// - Output 0: smoothed signal
///
/// ### Example (Smoothed Atomic Parameter)
/// ```
/// use fundsp::prelude::*;
/// let parameter = shared(1.0);
/// var(&parameter) >> afollow(0.01, 0.02);
/// ```
pub fn afollow<F: Real>(attack_time: F, release_time: F) -> An<AFollow<F>> {
    An(AFollow::new(attack_time, release_time))
}

/// Look-ahead limiter with attack and release times in seconds.
/// Look-ahead is equal to the attack time.
/// - Allocates: look-ahead buffers.
/// - Input 0: signal
/// - Output 0: signal limited to -1...1
pub fn limiter(attack_time: f32, release_time: f32) -> An<Limiter<U1>> {
    An(Limiter::new(DEFAULT_SR, attack_time, release_time))
}

/// Stereo look-ahead limiter with attack and release times in seconds.
/// Look-ahead is equal to the attack time.
/// - Allocates: look-ahead buffers.
/// - Input 0: left signal
/// - Input 1: right signal
/// - Output 0: left signal limited to -1...1
/// - Output 1: right signal limited to -1...1
pub fn limiter_stereo(attack_time: f32, release_time: f32) -> An<Limiter<U2>> {
    An(Limiter::new(DEFAULT_SR, attack_time, release_time))
}

/// Pinking filter.
/// - Input 0: input signal
/// - Output 0: filtered signal
pub fn pinkpass<F: Float>() -> An<Pinkpass<F>> {
    An(Pinkpass::new())
}

/// Pink noise.
/// - Output 0: pink noise
pub fn pink<F: Float>() -> An<Pipe<Noise, Pinkpass<F>>> {
    white() >> pinkpass::<F>()
}

/// Brown noise.
/// - Output 0: brown noise
pub fn brown<F: Real>() -> An<Pipe<Noise, Binop<FrameMul<U1>, Lowpole<F, U1>, Constant<U1>>>> {
    // Empirical normalization factor.
    white() >> lowpole_hz::<F>(F::from_f64(10.0)) * dc(13.7)
}

/// Feedback delay network.
/// Mix output of enclosed circuit `x` back to its input.
/// The output is diffused with a Hadamard matrix for feedback.
/// Feedback circuit `x` must have an equal number of inputs and outputs.
/// The number of inputs and outputs must be a power of two.
/// - Inputs: input signal.
/// - Outputs: `x` output signal.
///
/// *** Example: Mono Reverb
/// ```
/// use fundsp::prelude::*;
/// split() >> fdn::<U16, _>(stacki::<U16, _, _>(|i| { delay(lerp(0.01, 0.03, rnd1(i))) >> fir((0.2, 0.4, 0.2)) })) >> join();
/// ```
pub fn fdn<N, X>(x: An<X>) -> An<Feedback<N, X, FrameHadamard<N>>>
where
    N: Size<f32>,
    X: AudioNode<Inputs = N, Outputs = N>,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
{
    An(Feedback::new(x.0, FrameHadamard::new()))
}

/// Feedback delay network.
/// Mix output of enclosed circuit `x` back to its input,
/// using `y` for extra feedback processing. The feedforward path does not include `y`.
/// After `y`, the feedback signal is diffused with a Hadamard matrix.
/// Feedback circuits `x` and `y` must have an equal number of inputs and outputs.
/// - Input(s): signal.
/// - Output(s): signal with feedback.
pub fn fdn2<N, X, Y>(x: An<X>, y: An<Y>) -> An<Feedback2<N, X, Y, FrameHadamard<N>>>
where
    N: Size<f32>,
    X: AudioNode<Inputs = N, Outputs = N>,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
    Y: AudioNode<Inputs = N, Outputs = N>,
    Y::Inputs: Size<f32>,
    Y::Outputs: Size<f32>,
{
    An(Feedback2::new(x.0, y.0, FrameHadamard::new()))
}

/// Bus `x` and `y` together: same as `x & y`.
///
/// - Input(s): from `x` and `y`.
/// - Output(s): from `x` and `y`.
pub fn bus<X, Y>(x: An<X>, y: An<Y>) -> An<Bus<X, Y>>
where
    X: AudioNode,
    Y: AudioNode<Inputs = X::Inputs, Outputs = X::Outputs>,
{
    x & y
}

/// Bus `N` similar nodes from indexed generator `f`.
/// - Input(s): from `f`.
/// - Output(s): from `f`.
///
/// ### Example (Sine Bundle)
/// ```
/// use fundsp::prelude::*;
/// busi::<U20, _, _>(|i| sine_hz(110.0 * exp(lerp(-0.2, 0.2, rnd2(i) as f32))));
/// ```
pub fn busi<N, X, F>(f: F) -> An<MultiBus<N, X>>
where
    N: Size<f32> + Size<X>,
    X: AudioNode,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
    F: Fn(u64) -> An<X>,
{
    assert!(N::USIZE > 0);
    let nodes = Frame::generate(|i| f(i as u64).0);
    An(MultiBus::new(nodes))
}

/// Bus `N` similar nodes from fractional generator `f`.
/// The fractional generator is given values in the range 0...1.
/// - Input(s): from `f`.
/// - Output(s): from `f`.
///
/// ### Example (Noise Bundle)
/// ```
/// use fundsp::prelude::*;
/// busf::<U20, _, _, f32>(|t| (noise() | dc((xerp(100.0, 1000.0, t), 20.0))) >> !resonator::<f32>() >> resonator::<f32>());
/// ```
pub fn busf<N, X, Y, F>(f: Y) -> An<MultiBus<N, X>>
where
    N: Size<f32> + Size<X>,
    F: Float,
    X: AudioNode,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
    Y: Fn(F) -> An<X>,
{
    assert!(N::USIZE > 0);
    let nodes = Frame::generate(|i| {
        f(if N::USIZE > 1 {
            F::from_f64(i as f64 / (N::USIZE - 1) as f64)
        } else {
            F::from_f64(0.5)
        })
        .0
    });
    An(MultiBus::new(nodes))
}

/// Stack `x` and `y`. Identical with `x | y`.
///
/// - Input(s): Inputs of `x` followed with inputs of `y`.
/// - Output(s): Outputs of `x` followed with outputs of `y`.
pub fn stack<X, Y>(x: An<X>, y: An<Y>) -> An<Stack<X, Y>>
where
    X: AudioNode,
    Y: AudioNode,
    X::Inputs: Add<Y::Inputs>,
    X::Outputs: Add<Y::Outputs>,
    <X::Inputs as Add<Y::Inputs>>::Output: Size<f32>,
    <X::Outputs as Add<Y::Outputs>>::Output: Size<f32>,
{
    x | y
}

/// Stack `N` similar nodes from indexed generator `f`.
/// - Input(s): `N` times `f`.
/// - Output(s): `N` times `f`.
pub fn stacki<N, X, F>(f: F) -> An<MultiStack<N, X>>
where
    N: Size<f32> + Size<X>,
    X: AudioNode,
    X::Inputs: Size<f32> + Mul<N>,
    X::Outputs: Size<f32> + Mul<N>,
    <X::Inputs as Mul<N>>::Output: Size<f32>,
    <X::Outputs as Mul<N>>::Output: Size<f32>,
    F: Fn(u64) -> An<X>,
{
    assert!(N::USIZE > 0);
    let nodes = Frame::generate(|i| f(i as u64).0);
    An(MultiStack::new(nodes))
}

/// Stack `N` similar nodes from fractional generator `f`.
/// The fractional generator is given values in the range 0...1.
/// - Input(s): `N` times `f`.
/// - Output(s): `N` times `f`.
pub fn stackf<N, X, Y, F>(f: Y) -> An<MultiStack<N, X>>
where
    N: Size<f32> + Size<X>,
    F: Float,
    X: AudioNode,
    X::Inputs: Size<f32> + Mul<N>,
    X::Outputs: Size<f32> + Mul<N>,
    <X::Inputs as Mul<N>>::Output: Size<f32>,
    <X::Outputs as Mul<N>>::Output: Size<f32>,
    Y: Fn(F) -> An<X>,
{
    assert!(N::USIZE > 0);
    let nodes = Frame::generate(|i| {
        f(if N::USIZE > 1 {
            F::from_f64(i as f64 / (N::USIZE - 1) as f64)
        } else {
            F::from_f64(0.5)
        })
        .0
    });
    An(MultiStack::new(nodes))
}

/// Branch `x` and `y`. Identical with `x ^ y`.
///
/// - Input(s): From `x` and `y`.
/// - Output(s): Outputs of `x` followed with outputs of `y`.
pub fn branch<X, Y>(x: An<X>, y: An<Y>) -> An<Branch<X, Y>>
where
    X: AudioNode,
    Y: AudioNode<Inputs = X::Inputs>,
    X::Outputs: Add<Y::Outputs>,
    <X::Outputs as Add<Y::Outputs>>::Output: Size<f32>,
{
    x ^ y
}

/// Branch into `N` similar nodes from indexed generator `f`.
/// - Input(s): from `f`.
/// - Output(s): `N` times `f`.
pub fn branchi<N, X, F>(f: F) -> An<MultiBranch<N, X>>
where
    N: Size<f32> + Size<X>,
    X: AudioNode,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32> + Mul<N>,
    <X::Outputs as Mul<N>>::Output: Size<f32>,
    F: Fn(u64) -> An<X>,
{
    let nodes = Frame::generate(|i| f(i as u64).0);
    An(MultiBranch::new(nodes))
}

/// Branch into `N` similar nodes from fractional generator `f`.
/// The fractional generator is given values in the range 0...1.
/// - Input(s): from `f`.
/// - Output(s): `N` times `f`.
pub fn branchf<N, X, Y, F>(f: Y) -> An<MultiBranch<N, X>>
where
    N: Size<f32> + Size<X>,
    F: Float,
    X: AudioNode,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32> + Mul<N>,
    <X::Outputs as Mul<N>>::Output: Size<f32>,
    Y: Fn(F) -> An<X>,
{
    let nodes = Frame::generate(|i| {
        f(if N::USIZE > 1 {
            F::from_f64(i as f64 / (N::USIZE - 1) as f64)
        } else {
            F::from_f64(0.5)
        })
        .0
    });
    An(MultiBranch::new(nodes))
}

/// Pass through inputs that are missing from outputs. Identical with `!x`.
/// - Input(s): from `x`.
/// - Output(s): from `x`, followed with any extra passed through inputs.
pub fn thru<X>(x: An<X>) -> An<Thru<X>>
where
    X: AudioNode,
{
    !x
}

/// Multiply outputs of `x` and `y` channelwise. Identical with `x * y`.
/// - Input(s): Inputs of `x` followed with inputs of `y`.
/// - Output(s): Product of `x` and `y`.
pub fn product<X, Y>(x: An<X>, y: An<Y>) -> An<Binop<FrameMul<X::Outputs>, X, Y>>
where
    X: AudioNode,
    Y: AudioNode<Outputs = X::Outputs>,
    X::Inputs: Add<Y::Inputs>,
    <X::Inputs as Add<Y::Inputs>>::Output: Size<f32>,
{
    x * y
}

/// Add outputs of `x` and `y` channelwise. Identical with `x + y`.
/// - Input(s): Inputs of `x` followed with inputs of `y`.
/// - Output(s): Sum of `x` and `y`.
pub fn sum<X, Y>(x: An<X>, y: An<Y>) -> An<Binop<FrameAdd<X::Outputs>, X, Y>>
where
    X: AudioNode,
    Y: AudioNode<Outputs = X::Outputs>,
    X::Inputs: Add<Y::Inputs>,
    <X::Inputs as Add<Y::Inputs>>::Output: Size<f32>,
{
    x + y
}

/// Mix together `N` similar nodes from indexed generator `f`.
/// - Input(s): `N` times `f`.
/// - Output(s): from `f`.
pub fn sumi<N, X, F>(f: F) -> An<Reduce<N, X, FrameAdd<X::Outputs>>>
where
    N: Size<f32> + Size<X>,
    X: AudioNode,
    X::Inputs: Size<f32> + Mul<N>,
    X::Outputs: Size<f32>,
    <X::Inputs as Mul<N>>::Output: Size<f32>,
    F: Fn(u64) -> An<X>,
{
    let nodes = Frame::generate(|i| f(i as u64).0);
    An(Reduce::new(nodes, FrameAdd::new()))
}

/// Mix together `N` similar nodes from fractional generator `f`.
/// The fractional generator is given values in the range 0...1.
/// - Input(s): `N` times `f`.
/// - Output(s): from `f`.
pub fn sumf<N, X, Y, F>(f: Y) -> An<Reduce<N, X, FrameAdd<X::Outputs>>>
where
    N: Size<f32> + Size<X>,
    F: Float,
    X: AudioNode,
    X::Inputs: Size<f32> + Mul<N>,
    X::Outputs: Size<f32>,
    <X::Inputs as Mul<N>>::Output: Size<f32>,
    Y: Fn(F) -> An<X>,
{
    let nodes = Frame::generate(|i| {
        f(if N::USIZE > 1 {
            F::from_f64(i as f64 / (N::USIZE - 1) as f64)
        } else {
            F::from_f64(0.5)
        })
        .0
    });
    An(Reduce::new(nodes, FrameAdd::new()))
}

/// Pipe `x` to `y`. Identical with `x >> y`.
/// - Input(s): Inputs from `x`.
/// - Output(s): Outputs from `y`.
pub fn pipe<X, Y>(x: An<X>, y: An<Y>) -> An<Pipe<X, Y>>
where
    X: AudioNode,
    Y: AudioNode<Inputs = X::Outputs>,
{
    x >> y
}

/// Chain together `N` similar nodes from indexed generator `f`.
/// - Input(s): from `f`.
/// - Output(s): from `f`.
pub fn pipei<N, X, F>(f: F) -> An<Chain<N, X>>
where
    N: Size<f32> + Size<X>,
    X: AudioNode,
    F: Fn(u64) -> An<X>,
{
    let nodes = Frame::generate(|i| f(i as u64).0);
    An(Chain::new(nodes))
}

/// Chain together `N` similar nodes from fractional generator `f`.
/// The fractional generator is given values in the range 0...1.
/// - Input(s): from `f`.
/// - Output(s): from `f`.
pub fn pipef<N, X, Y, F>(f: Y) -> An<Chain<N, X>>
where
    N: Size<f32> + Size<X>,
    F: Float,
    X: AudioNode,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
    Y: Fn(F) -> An<X>,
{
    let nodes = Frame::generate(|i| {
        f(if N::USIZE > 1 {
            F::from_f64(i as f64 / (N::USIZE - 1) as f64)
        } else {
            F::from_f64(0.5)
        })
        .0
    });
    An(Chain::new(nodes))
}

/// Split signal into N channels.
/// - Input 0: signal.
/// - Output(s): `N` copies of signal.
pub fn split<N>() -> An<Split<N>>
where
    N: Size<f32>,
{
    An(Split::new())
}

/// Split `M` channels into `N` branches. The output has `N` * `M` channels.
/// - Input(s): `M`.
/// - Output(s): `N` * `M`. Each branch contains a copy of the input(s).
pub fn multisplit<M, N>() -> An<MultiSplit<M, N>>
where
    M: Size<f32> + Mul<N>,
    N: Size<f32>,
    <M as Mul<N>>::Output: Size<f32>,
{
    An(MultiSplit::new())
}

/// Average `N` channels into one. Inverse of `split`.
/// - Input(s): `N`.
/// - Output 0: average.
pub fn join<N>() -> An<Join<N>>
where
    N: Size<f32>,
{
    An(Join::new())
}

/// Average `N` branches of `M` channels into one branch with `M` channels.
/// The input has `N` * `M` channels. Inverse of `multisplit::<M, N>`.
/// - Input(s): `N` * `M`.
/// - Output(s): `M`.
pub fn multijoin<M, N>() -> An<MultiJoin<M, N>>
where
    N: Size<f32>,
    M: Size<f32> + Mul<N>,
    <M as Mul<N>>::Output: Size<f32>,
{
    An(MultiJoin::new())
}

/// Stereo reverb (32-channel FDN).
/// `room_size` is in meters. An average room size is 10 meters.
/// `time` is approximate reverberation time to -60 dB in seconds.
/// `damping` is high frequency damping in 0...1.
/// - Allocates: delay lines
/// - Input 0: left signal
/// - Input 1: right signal
/// - Output 0: reverberated left signal
/// - Output 1: reverberated right signal
///
/// ### Example: Add 20% Reverb
/// ```
/// use fundsp::prelude::*;
/// multipass() & 0.2 * reverb_stereo(10.0, 5.0, 0.5);
/// ```
pub fn reverb_stereo(
    room_size: f64,
    time: f64,
    damping: f64,
) -> An<impl AudioNode<Inputs = U2, Outputs = U2>> {
    // Optimized delay times for a 32-channel FDN from a legacy project.
    // These are applied unchanged for a 10 meter room.
    const DELAYS: [f64; 32] = [
        0.073904, 0.052918, 0.066238, 0.066387, 0.037783, 0.080073, 0.050961, 0.075900, 0.043646,
        0.072095, 0.056194, 0.045961, 0.058934, 0.068016, 0.047529, 0.058156, 0.072972, 0.036084,
        0.062715, 0.076377, 0.044339, 0.076725, 0.077884, 0.046126, 0.067741, 0.049800, 0.051709,
        0.082923, 0.070121, 0.079315, 0.055039, 0.081859,
    ];

    // Damping filter weights.
    let a = pow(db_amp(-60.0), 0.03 * room_size / 10.0 / time) as f32;
    let weights = fir3(1.0 - damping as f32).weights() * Frame::splat(a);

    // Delay lines.
    let line =
        stacki::<U32, _, _>(|i| delay(DELAYS[i as usize] * room_size / 10.0) >> fir(weights));

    // The feedback structure.
    let reverb = fdn::<U32, _>(line);

    // Pan the channels with an S shape.
    multisplit::<U2, U16>()
        >> reverb
        >> sumf::<U32, _, _, f32>(|x| pan(lerp(-1.0, 1.0, smooth9(x))))
            * dc((1.0 / 16.0, 1.0 / 16.0))
}

/// Create a stereo reverb unit (32-channel hybrid FDN).
/// Parameters are room size (in meters, between 10 and 30 meters),
/// reverberation `time` (in seconds, to -60 dB), diffusion amount (in 0...1),
/// modulation speed (nominal range from 0 to 1, values beyond 1 are permitted
/// and will start to create audible Doppler effects), and a user configurable loop filter.
/// The loop filter is applied repeatedly to the reverb tail and can be used to implement
/// frequency dependent filtering and other effects.
/// More sophisticated (and expensive) than `reverb_stereo`.
/// - Allocates: delay lines
/// - Input 0: left signal
/// - Input 1: right signal
/// - Output 0: reverberated left signal
/// - Output 1: reverberated right signal
///
/// ### Example: Add 20% Reverb
/// ```
/// use fundsp::prelude::*;
/// multipass() & 0.2 * reverb2_stereo(10.0, 1.0, 0.5, 1.0, lowpole_hz::<f32>(8000.0));
/// ```
pub fn reverb2_stereo(
    room_size: f64,
    time: f64,
    diffusion: f64,
    modulation_speed: f64,
    filter: An<impl AudioNode<Inputs = U1, Outputs = U1>>,
) -> An<impl AudioNode<Inputs = U2, Outputs = U2>> {
    let room_size = clamp(10.0, 30.0, room_size);

    // Schroeder allpass delays.
    let delays = [
        11, 13, 17, 23, 31, 41, 53, 71, 97, 113, 131, 163, 193, 223, 241, 269, 293, 311, 337, 359,
        397, 421, 449, 479, 503, 523, 557, 571, 601, 631, 653, 677, 709, 733, 757, 787, 809, 827,
        857, 877, 911, 929, 953, 977, 1009,
    ];

    let delay_min = 0.010 + room_size * 0.003;
    let delay_max = max(delay_min * 2.0, delay_min + 0.002 * 31.0);
    let delay_d = (delay_max - delay_min) / 32.0;

    // Damping filter weights.
    let a = pow(db_amp(-60.0), 0.5 * delay_min / time) as f32;

    // Schroeder allpass coefficient.
    let coeff = lerp(0.5, 0.9, diffusion as f32);

    // The feedback structure.
    let line = stacki::<U32, _, _>(|i| {
        let j = if i < 16 { i * 2 } else { (31 - i) * 2 + 1 };
        let allpass_delay = delays[j as usize] as f64 / DEFAULT_SR;
        let d = delay_min as f32 + j as f32 * delay_d as f32 - allpass_delay as f32;
        let dv = 0.001;
        let min_d = d - dv;
        let max_d = d + dv;
        (filter.clone() * dc(a)
            | An(Envelope::<f32, _, _>::new(0.01, move |t| {
                lerp11(
                    min_d,
                    max_d,
                    spline_noise(i, t * modulation_speed as f32 * 0.5),
                )
            })))
            >> tap_linear(min_d, max_d)
            >> allnest_c(coeff, delay(allpass_delay - 1.0 / DEFAULT_SR))
    });

    // Pre-diffusers.
    let lpass =
        allnest_c(0.618, delay(224.0 / DEFAULT_SR)) >> allnest_c(0.618, delay(340.0 / DEFAULT_SR));

    let rpass =
        allnest_c(0.618, delay(247.0 / DEFAULT_SR)) >> allnest_c(0.618, delay(366.0 / DEFAULT_SR));

    // Pan the channels with an S shape.
    (lpass | rpass)
        >> multisplit::<U2, U16>()
        >> fdn(line)
        >> sumf::<U32, _, _, f32>(|x| pan(lerp(-1.0, 1.0, smooth9(x)))) * (1.0 / 8.0)
}

/// Allpass loop based stereo reverb. Parameters are reverberation `time` (in seconds to -60 dB),
/// diffusion amount (in 0...1), and a user configurable loop filter.
/// The loop filter is applied repeatedly to the reverb tail and can be used to implement
/// frequency dependent filtering and other effects.
/// - Allocates: delay lines
/// - Input 0: left signal
/// - Input 1: right signal
/// - Output 0: reverberated left signal
/// - Output 1: reverberated right signal
///
/// ### Example: Add 25% Reverb
/// ```
/// use fundsp::prelude::*;
/// multipass() & 0.25 * reverb3_stereo(2.0, 0.5, lowpole_hz::<f32>(8000.0));
/// ```
pub fn reverb3_stereo(
    time: f64,
    diffusion: f64,
    filter: An<impl AudioNode<Inputs = U1, Outputs = U1>>,
) -> An<impl AudioNode<Inputs = U2, Outputs = U2>> {
    An(super::reverb::Reverb::new(time, diffusion, filter.0))
}

/// Stereo reverb with a slow fade-in envelope.
/// `room_size` is in meters (at least 15 meters).
/// `time` is approximate reverberation time to -60 dB in seconds.
/// - Input 0: left signal
/// - Input 1: right signal
/// - Output 0: reverberated left signal
/// - Output 1: reverberated right signal
pub fn reverb4_stereo(room_size: f64, time: f64) -> An<impl AudioNode<Inputs = U2, Outputs = U2>> {
    // Optimized delay times from `optimize.rs` example. Fitness -4546.
    let mut delays = [
        0.059326634,
        0.04778291,
        0.06995449,
        0.0393001,
        0.041604012,
        0.06215825,
        0.052269846,
        0.043227978,
        0.06966107,
        0.031615064,
        0.068442,
        0.037332155,
        0.032944717,
        0.034493037,
        0.06787566,
        0.038824916,
        0.068260126,
        0.068044715,
        0.0688076,
        0.066724524,
        0.051293883,
        0.06023173,
        0.040897705,
        0.031507637,
        0.060309593,
        0.049584292,
        0.04532072,
        0.056379095,
        0.035180368,
        0.041291796,
        0.046129026,
        0.05504605,
    ];
    for delay in delays.iter_mut() {
        // The delays sound like garbage below 15 meters.
        *delay *= (max(room_size as f32, 15.0)) / 10.0;
    }
    reverb4_stereo_delays(&delays, time)
}

/// Create a stereo reverb unit, given delay times (in seconds) for the 32 delay lines
/// and reverberation `time` (in seconds). WIP.
/// - Input 0: left signal
/// - Input 1: right signal
/// - Output 0: reverberated left signal
/// - Output 1: reverberated right signal
pub fn reverb4_stereo_delays(
    delays: &[f32],
    time: f64,
) -> An<impl AudioNode<Inputs = U2, Outputs = U2>> {
    assert!(delays.len() == 32);
    let room_size = 10.0; // delays.iter().sum::<f64>() / 32.0 / 0.03 * 10.0;
    let a = pow(db_amp(-60.0), 0.03 * room_size / 10.0 / time) as f32;

    let line1 = stacki::<U16, _, _>(|i| {
        delay(delays[i as usize] as f64) >> fir((-a / 4.0, -a / 2.0, -a / 4.0))
    });

    let line2 = stacki::<U16, _, _>(|i| {
        delay(delays[16 + i as usize] as f64) >> fir((-a / 4.0, -a / 2.0, -a / 4.0))
    });

    let fdn1 = fdn(line1);
    let fdn2 = fdn(line2);

    multisplit::<U2, U8>()
        >> fdn1
        >> multijoin::<U2, U8>()
        >> multisplit::<U2, U8>()
        >> fdn2
        >> sumf::<U16, _, _, f32>(|x| pan(lerp(-1.0, 1.0, smooth9(x)))) * dc((1.0 / 4.0, 1.0 / 4.0))
}

/// Saw-like discrete summation formula oscillator.
/// - Input 0: frequency in Hz
/// - Input 1: roughness in 0...1 is the attenuation of successive partials.
/// - Output 0: DSF wave
pub fn dsf_saw() -> An<Dsf<U2>> {
    An(Dsf::new(1.0, 0.5))
}

/// Saw-like discrete summation formula oscillator.
/// Roughness in 0...1 is the attenuation of successive partials.
/// - Input 0: frequency in Hz
/// - Output 0: DSF wave
pub fn dsf_saw_r(roughness: f32) -> An<Dsf<U1>> {
    An(Dsf::new(1.0, roughness))
}

/// Square-like discrete summation formula oscillator.
/// - Input 0: frequency in Hz
/// - Input 1: roughness in 0...1 is the attenuation of successive partials.
/// - Output 0: DSF wave
pub fn dsf_square() -> An<Dsf<U2>> {
    An(Dsf::new(2.0, 0.5))
}

/// Square-like discrete summation formula oscillator.
/// Roughness in 0...1 is the attenuation of successive partials.
/// - Input 0: frequency in Hz
/// - Output 0: DSF wave
pub fn dsf_square_r(roughness: f32) -> An<Dsf<U1>> {
    An(Dsf::new(2.0, roughness))
}

/// Karplus-Strong plucked string oscillator with `frequency` in Hz.
/// High frequency damping is in 0...1.
/// - Allocates: pluck buffer.
/// - Input 0: string excitation
/// - Output 0: oscillator output
///
/// ### Example
/// ```
/// use fundsp::prelude::*;
/// let node = zero() >> pluck(440.0, 0.5, 1.0);
/// ```
pub fn pluck(frequency: f32, gain_per_second: f32, high_frequency_damping: f32) -> An<Pluck> {
    An(Pluck::new(
        frequency,
        gain_per_second,
        high_frequency_damping,
    ))
}

/// Saw wavetable oscillator.
/// - Allocates: global saw wavetable.
/// - Input 0: frequency in Hz
/// - Output 0: saw wave
pub fn saw() -> An<WaveSynth<U1>> {
    An(WaveSynth::new(saw_table()))
}

/// Square wavetable oscillator.
/// - Allocates: global square wavetable.
/// - Input 0: frequency in Hz
/// - Output 0: square wave
pub fn square() -> An<WaveSynth<U1>> {
    An(WaveSynth::new(square_table()))
}

/// Triangle wavetable oscillator.
/// - Allocates: global triangle wavetable.
/// - Input 0: frequency in Hz
/// - Output 0: triangle wave
pub fn triangle() -> An<WaveSynth<U1>> {
    An(WaveSynth::new(triangle_table()))
}

/// Organ wavetable oscillator. Emphasizes octave partials.
/// - Allocates: global organ wavetable.
/// - Input 0: frequency in Hz
/// - Output 0: organ wave
pub fn organ() -> An<WaveSynth<U1>> {
    An(WaveSynth::new(organ_table()))
}

/// Soft saw wavetable oscillator.
/// Contains all partials, falls off like a triangle wave.
/// - Allocates: global soft saw wavetable.
/// - Input 0: frequency in Hz
/// - Output 0: soft saw wave
pub fn soft_saw() -> An<WaveSynth<U1>> {
    An(WaveSynth::new(soft_saw_table()))
}

/// Hammond wavetable oscillator. Emphasizes first three partials.
/// - Allocates: global Hammond wavetable.
/// - Input 0: frequency in Hz
/// - Output 0: Hammond wave
pub fn hammond() -> An<WaveSynth<U1>> {
    An(WaveSynth::new(hammond_table()))
}

/// Fixed saw wavetable oscillator at `f` Hz.
/// - Allocates: global saw wavetable.
/// - Output 0: saw wave
pub fn saw_hz(f: f32) -> An<Pipe<Constant<U1>, WaveSynth<U1>>> {
    constant(f) >> saw()
}

/// Fixed square wavetable oscillator at `f` Hz.
/// - Allocates: global square wavetable.
/// - Output 0: square wave
pub fn square_hz(f: f32) -> An<Pipe<Constant<U1>, WaveSynth<U1>>> {
    constant(f) >> square()
}

/// Fixed triangle wavetable oscillator at `f` Hz.
/// - Allocates: global triangle wavetable.
/// - Output 0: triangle wave
pub fn triangle_hz(f: f32) -> An<Pipe<Constant<U1>, WaveSynth<U1>>> {
    constant(f) >> triangle()
}

/// Fixed organ wavetable oscillator at `f` Hz. Emphasizes octave partials.
/// - Allocates: global organ wavetable.
/// - Output 0: organ wave
pub fn organ_hz(f: f32) -> An<Pipe<Constant<U1>, WaveSynth<U1>>> {
    constant(f) >> organ()
}

/// Fixed soft saw wavetable oscillator at `f` Hz.
/// Contains all partials, falls off like a triangle wave.
/// - Allocates: global soft saw wavetable.
/// - Output 0: soft saw wave
pub fn soft_saw_hz(f: f32) -> An<Pipe<Constant<U1>, WaveSynth<U1>>> {
    constant(f) >> soft_saw()
}

/// Fixed Hammond wavetable oscillator at `f` Hz. Emphasizes first three partials.
/// - Allocates: global Hammond wavetable.
/// - Output 0: Hammond wave
pub fn hammond_hz(f: f32) -> An<Pipe<Constant<U1>, WaveSynth<U1>>> {
    constant(f) >> hammond()
}

/// Lowpass filter.
/// - Input 0: audio
/// - Input 1: cutoff frequency (Hz)
/// - Input 2: Q
/// - Output 0: filtered audio
pub fn lowpass<F: Real>() -> An<Svf<F, LowpassMode<F>>> {
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

/// Lowpass filter with cutoff frequency `f` Hz and Q value `q`.
/// - Input 0: audio
/// - Output 0: filtered audio
pub fn lowpass_hz<F: Real>(f: F, q: F) -> An<FixedSvf<F, LowpassMode<F>>> {
    An(FixedSvf::new(
        LowpassMode::default(),
        &SvfParams {
            sample_rate: convert(DEFAULT_SR),
            cutoff: f,
            q,
            gain: F::one(),
        },
    ))
}

/// Lowpass filter with Q value `q`.
/// - Input 0: audio
/// - Input 1: cutoff frequency (Hz)
/// - Output 0: filtered audio
pub fn lowpass_q<F: Real>(
    q: F,
) -> An<Pipe<Stack<MultiPass<U2>, Constant<U1>>, Svf<F, LowpassMode<F>>>> {
    (multipass::<U2>() | dc(q.to_f32()))
        >> An(Svf::new(
            LowpassMode::default(),
            &SvfParams {
                sample_rate: convert(DEFAULT_SR),
                cutoff: F::new(440),
                q,
                gain: F::one(),
            },
        ))
}

/// Highpass filter.
/// - Input 0: audio
/// - Input 1: cutoff frequency (Hz)
/// - Input 2: Q
/// - Output 0: filtered audio
pub fn highpass<F: Real>() -> An<Svf<F, HighpassMode<F>>> {
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
pub fn highpass_hz<F: Real>(f: F, q: F) -> An<FixedSvf<F, HighpassMode<F>>> {
    An(FixedSvf::new(
        HighpassMode::default(),
        &SvfParams {
            sample_rate: convert(DEFAULT_SR),
            cutoff: f,
            q,
            gain: F::one(),
        },
    ))
}

/// Highpass filter with Q value `q`.
/// - Input 0: audio
/// - Input 1: cutoff frequency (Hz)
/// - Output 0: filtered audio
pub fn highpass_q<F: Real>(
    q: F,
) -> An<Pipe<Stack<MultiPass<U2>, Constant<U1>>, Svf<F, HighpassMode<F>>>> {
    (multipass::<U2>() | dc(q.to_f32()))
        >> An(Svf::new(
            HighpassMode::default(),
            &SvfParams {
                sample_rate: convert(DEFAULT_SR),
                cutoff: F::new(440),
                q: convert(q),
                gain: F::one(),
            },
        ))
}

/// Bandpass filter.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Input 2: Q
/// - Output 0: filtered audio
pub fn bandpass<F: Real>() -> An<Svf<F, BandpassMode<F>>> {
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
pub fn bandpass_hz<F: Real>(f: F, q: F) -> An<FixedSvf<F, BandpassMode<F>>> {
    An(FixedSvf::new(
        BandpassMode::default(),
        &SvfParams {
            sample_rate: convert(DEFAULT_SR),
            cutoff: f,
            q,
            gain: F::one(),
        },
    ))
}

/// Bandpass filter with Q value `q`.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Output 0: filtered audio
pub fn bandpass_q<F: Real>(
    q: F,
) -> An<Pipe<Stack<MultiPass<U2>, Constant<U1>>, Svf<F, BandpassMode<F>>>> {
    (multipass::<U2>() | dc(q.to_f32()))
        >> An(Svf::new(
            BandpassMode::default(),
            &SvfParams {
                sample_rate: convert(DEFAULT_SR),
                cutoff: F::new(440),
                q: convert(q),
                gain: F::one(),
            },
        ))
}

/// Notch filter.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Input 2: Q
/// - Output 0: filtered audio
pub fn notch<F: Real>() -> An<Svf<F, NotchMode<F>>> {
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
pub fn notch_hz<F: Real>(f: F, q: F) -> An<FixedSvf<F, NotchMode<F>>> {
    An(FixedSvf::new(
        NotchMode::default(),
        &SvfParams {
            sample_rate: convert(DEFAULT_SR),
            cutoff: f,
            q,
            gain: F::one(),
        },
    ))
}

/// Notch filter with Q value `q`.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Output 0: filtered audio
pub fn notch_q<F: Real>(
    q: F,
) -> An<Pipe<Stack<MultiPass<U2>, Constant<U1>>, Svf<F, NotchMode<F>>>> {
    (multipass::<U2>() | dc(q.to_f32()))
        >> An(Svf::new(
            NotchMode::default(),
            &SvfParams {
                sample_rate: convert(DEFAULT_SR),
                cutoff: F::new(440),
                q: convert(q),
                gain: F::one(),
            },
        ))
}

/// Peaking filter.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Input 2: Q
/// - Output 0: filtered audio
pub fn peak<F: Real>() -> An<Svf<F, PeakMode<F>>> {
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
pub fn peak_hz<F: Real>(f: F, q: F) -> An<FixedSvf<F, PeakMode<F>>> {
    An(FixedSvf::new(
        PeakMode::default(),
        &SvfParams {
            sample_rate: convert(DEFAULT_SR),
            cutoff: f,
            q,
            gain: F::one(),
        },
    ))
}

/// Peaking filter with Q value `q`.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Output 0: filtered audio
pub fn peak_q<F: Real>(q: F) -> An<Pipe<Stack<MultiPass<U2>, Constant<U1>>, Svf<F, PeakMode<F>>>> {
    (multipass::<U2>() | dc(q.to_f32()))
        >> An(Svf::new(
            PeakMode::default(),
            &SvfParams {
                sample_rate: convert(DEFAULT_SR),
                cutoff: F::new(440),
                q: convert(q),
                gain: F::one(),
            },
        ))
}

/// Allpass filter.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Input 2: Q
/// - Output 0: filtered audio
pub fn allpass<F: Real>() -> An<Svf<F, AllpassMode<F>>> {
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
pub fn allpass_hz<F: Real>(f: F, q: F) -> An<FixedSvf<F, AllpassMode<F>>> {
    An(FixedSvf::new(
        AllpassMode::default(),
        &SvfParams {
            sample_rate: convert(DEFAULT_SR),
            cutoff: f,
            q,
            gain: F::one(),
        },
    ))
}

/// Allpass filter with Q value `q`.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Output 0: filtered audio
pub fn allpass_q<F: Real>(
    q: F,
) -> An<Pipe<Stack<MultiPass<U2>, Constant<U1>>, Svf<F, AllpassMode<F>>>> {
    (multipass::<U2>() | dc(q.to_f32()))
        >> An(Svf::new(
            AllpassMode::default(),
            &SvfParams {
                sample_rate: convert(DEFAULT_SR),
                cutoff: F::new(440),
                q: convert(q),
                gain: F::one(),
            },
        ))
}

/// Bell filter with adjustable gain.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Input 2: Q
/// - Input 3: amplitude gain
/// - Output 0: filtered audio
pub fn bell<F: Real>() -> An<Svf<F, BellMode<F>>> {
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

/// Bell filter centered at `f` Hz with Q value `q` and amplitude gain `gain`.
/// - Input 0: audio
/// - Output 0: filtered audio
pub fn bell_hz<F: Real>(f: F, q: F, gain: F) -> An<FixedSvf<F, BellMode<F>>> {
    An(FixedSvf::new(
        BellMode::default(),
        &SvfParams::<F> {
            sample_rate: convert(DEFAULT_SR),
            cutoff: f,
            q,
            gain,
        },
    ))
}

/// Bell filter with with Q value `q` and amplitude gain `gain`.
/// - Input 0: audio
/// - Input 1: center frequency
/// - Output 0: filtered audio
pub fn bell_q<F: Real>(
    q: F,
    gain: F,
) -> An<Pipe<Stack<MultiPass<U2>, Constant<U2>>, Svf<F, BellMode<F>>>> {
    (multipass::<U2>() | dc((q.to_f32(), gain.to_f32())))
        >> An(Svf::new(
            BellMode::default(),
            &SvfParams {
                sample_rate: convert(DEFAULT_SR),
                cutoff: F::new(440),
                q: convert(q),
                gain: convert(gain),
            },
        ))
}

/// Low shelf filter with adjustable gain.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Input 2: Q
/// - Input 3: amplitude gain
/// - Output 0: filtered audio
pub fn lowshelf<F: Real>() -> An<Svf<F, LowshelfMode<F>>> {
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
pub fn lowshelf_hz<F: Real>(f: F, q: F, gain: F) -> An<FixedSvf<F, LowshelfMode<F>>> {
    An(FixedSvf::new(
        LowshelfMode::default(),
        &SvfParams::<F> {
            sample_rate: convert(DEFAULT_SR),
            cutoff: f,
            q,
            gain,
        },
    ))
}

/// Low shelf filter with with Q value `q` and amplitude gain `gain`.
/// - Input 0: audio
/// - Input 1: cutoff frequency
/// - Output 0: filtered audio
pub fn lowshelf_q<F: Real>(
    q: F,
    gain: F,
) -> An<Pipe<Stack<MultiPass<U2>, Constant<U2>>, Svf<F, LowshelfMode<F>>>> {
    (multipass::<U2>() | dc((q.to_f32(), gain.to_f32())))
        >> An(Svf::new(
            LowshelfMode::default(),
            &SvfParams {
                sample_rate: convert(DEFAULT_SR),
                cutoff: convert(440.0),
                q: convert(q),
                gain: F::one(),
            },
        ))
}

/// High shelf filter with adjustable gain.
/// - Input 0: audio
/// - Input 1: center frequency (Hz)
/// - Input 2: Q
/// - Input 3: amplitude gain
/// - Output 0: filtered audio
pub fn highshelf<F: Real>() -> An<Svf<F, HighshelfMode<F>>> {
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
pub fn highshelf_hz<F: Real>(f: F, q: F, gain: F) -> An<FixedSvf<F, HighshelfMode<F>>> {
    An(FixedSvf::new(
        HighshelfMode::default(),
        &SvfParams::<F> {
            sample_rate: convert(DEFAULT_SR),
            cutoff: f,
            q,
            gain,
        },
    ))
}

/// High shelf filter with with Q value `q` and amplitude gain `gain`.
/// - Input 0: audio
/// - Input 1: cutoff frequency
/// - Output 0: filtered audio
pub fn highshelf_q<F: Real>(
    q: F,
    gain: F,
) -> An<Pipe<Stack<MultiPass<U2>, Constant<U2>>, Svf<F, HighshelfMode<F>>>> {
    (multipass::<U2>() | dc((q.to_f32(), gain.to_f32())))
        >> An(Svf::new(
            HighshelfMode::default(),
            &SvfParams {
                sample_rate: convert(DEFAULT_SR),
                cutoff: convert(440.0),
                q: convert(q),
                gain: F::one(),
            },
        ))
}

/// Resonant two-pole lowpass filter.
/// - Input 0: audio
/// - Input 1: cutoff frequency
/// - Input 2: Q
/// - Output 0: filtered audio
pub fn lowrez<F: Real>() -> An<Rez<F, U3>> {
    An(Rez::new(F::zero(), F::new(440), F::one()))
}

/// Resonant two-pole lowpass filter with fixed cutoff frequency and Q.
/// - Input 0: audio
/// - Output 0: filtered audio
pub fn lowrez_hz<F: Real>(cutoff: F, q: F) -> An<Rez<F, U1>> {
    An(Rez::new(F::zero(), cutoff, q))
}

/// Resonant two-pole lowpass filter with fixed Q.
/// - Input 0: audio
/// - Input 1: cutoff frequency
/// - Output 0: filtered audio
pub fn lowrez_q<F: Real>(q: F) -> An<Pipe<Stack<MultiPass<U2>, Constant<U1>>, Rez<F, U3>>> {
    (multipass::<U2>() | dc(q.to_f32())) >> lowrez()
}

/// Resonant two-pole bandpass filter.
/// - Input 0: audio
/// - Input 1: center frequency
/// - Input 2: Q
/// - Output 0: filtered audio
pub fn bandrez<F: Real>() -> An<Rez<F, U3>> {
    An(Rez::new(F::one(), F::new(440), F::one()))
}

/// Resonant two-pole bandpass filter with fixed center frequency and Q.
/// - Input 0: audio
/// - Output 0: filtered audio
pub fn bandrez_hz<F: Real>(center: F, q: F) -> An<Rez<F, U1>> {
    An(Rez::new(F::one(), center, q))
}

/// Resonant two-pole bandpass filter with fixed Q.
/// - Input 0: audio
/// - Input 1: cutoff frequency
/// - Output 0: filtered audio
pub fn bandrez_q<F: Real>(q: F) -> An<Pipe<Stack<MultiPass<U2>, Constant<U1>>, Rez<F, U3>>> {
    (multipass::<U2>() | dc(q.to_f32())) >> bandrez()
}

/// Pulse wave oscillator.
/// - Input 0: frequency in Hz
/// - Input 1: pulse duty cycle in 0...1
/// - Output 0: pulse wave
pub fn pulse() -> An<PulseWave> {
    An(PulseWave::new())
}

/// Morphing filter that morphs between lowpass, peak and highpass modes.
/// - Input 0: input signal
/// - Input 1: center frequency (Hz)
/// - Input 2: Q
/// - Input 3: morph in -1...1 (-1 = lowpass, 0 = peak, 1 = highpass)
/// - Output 0: filtered signal
pub fn morph<F: Real>() -> An<Morph<F>> {
    An(Morph::new(F::new(440), F::one(), F::zero()))
}

/// Morphing filter with center frequency `f`, Q value `q`, and morph `morph`
/// (-1 = lowpass, 0 = peaking, 1 = highpass).
/// - Input 0: input signal
/// - Output 0: filtered signal
pub fn morph_hz<F: Real>(f: F, q: F, morph: F) -> An<Pipe<Stack<Pass, Constant<U3>>, Morph<F>>> {
    (pass() | dc((f.to_f32(), q.to_f32(), morph.to_f32()))) >> An(Morph::new(f, q, morph))
}

/// Play back a channel of a `Wave`.
/// Optional loop point is the index to jump to at the end of the wave.
/// - Output 0: wave
pub fn wavech(wave: &Arc<Wave>, channel: usize, loop_point: Option<usize>) -> An<WavePlayer> {
    An(WavePlayer::new(wave, channel, 0, wave.length(), loop_point))
}

/// Play back a channel of a `Wave` starting from sample `start_point`, inclusive,
/// and ending at sample `end_point`, exclusive.
/// Optional loop point is the index to jump to at the end.
/// - Output 0: wave
pub fn wavech_at(
    wave: &Arc<Wave>,
    channel: usize,
    start_point: usize,
    end_point: usize,
    loop_point: Option<usize>,
) -> An<WavePlayer> {
    An(WavePlayer::new(
        wave,
        channel,
        start_point,
        end_point,
        loop_point,
    ))
}

/// Mono chorus, 5 voices. For stereo, stack two of these using different seed values.
/// `seed`: LFO seed.
/// `separation`: base voice separation in seconds (for example, 0.015).
/// `variation`: delay variation in seconds (for example, 0.005).
/// `mod_frequency`: delay modulation frequency (for example, 0.2).
/// - Input 0: audio.
/// - Output 0: chorused audio, including original signal.
///
/// ### Example: Chorused Saw Wave
/// ```
/// use fundsp::prelude::*;
/// saw_hz(110.0) >> chorus(0, 0.015, 0.005, 0.5);
/// ```
pub fn chorus(
    seed: u64,
    separation: f32,
    variation: f32,
    mod_frequency: f32,
) -> An<impl AudioNode<Inputs = U1, Outputs = U1>> {
    (pass()
        & (pass()
            | lfo(move |t| {
                (
                    lerp11(
                        separation,
                        separation + variation,
                        spline_noise(seed, t * mod_frequency),
                    ),
                    lerp11(
                        separation * 2.0,
                        separation * 2.0 + variation,
                        spline_noise(hash1(seed), t * (mod_frequency + 0.02)),
                    ),
                    lerp11(
                        separation * 3.0,
                        separation * 3.0 + variation,
                        spline_noise(hash2(seed), t * (mod_frequency + 0.04)),
                    ),
                    lerp11(
                        separation * 4.0,
                        separation * 4.0 + variation,
                        spline_noise(hash1(seed ^ 0xfedcba), t * (mod_frequency + 0.06)),
                    ),
                )
            }))
            >> multitap::<U4>(separation, separation * 4.0 + variation))
        * dc(0.2)
}

/// Mono flanger.
/// `feedback_amount`: amount of feedback (for example, 0.9 or -0.9). Negative feedback inverts feedback phase.
/// `minimum_delay`: minimum delay in seconds (for example, 0.005).
/// `maximum_delay`: maximum delay in seconds (for example, 0.010).
/// ´delay_f´: Delay in `minimum_delay`...`maximum_delay` as a function of time. For example, `|t| lerp11(0.005, 0.010, sin_hz(0.1, t))`.
/// - Input 0: audio
/// - Output 0: flanged audio, including original signal
///
/// ### Example: Flanged Saw Wave
/// ```
/// use fundsp::prelude::*;
/// saw_hz(110.0) >> flanger(0.5, 0.005, 0.010, |t| lerp11(0.005, 0.010, sin_hz(0.1, t)));
/// ```
pub fn flanger<X: Fn(f32) -> f32 + Clone + Send + Sync>(
    feedback_amount: f32,
    minimum_delay: f32,
    maximum_delay: f32,
    delay_f: X,
) -> An<impl AudioNode<Inputs = U1, Outputs = U1>> {
    pass()
        & feedback2(
            (pass() | lfo(delay_f)) >> tap(minimum_delay, maximum_delay),
            shape(Tanh(feedback_amount.to_f32())),
        )
}

/// Mono phaser.
/// `feedback_amount`: amount of feedback (for example, 0.5). Negative feedback inverts feedback phase.
/// `phase_f`: allpass modulation value in 0...1 as function of time, for example `|t| sin_hz(0.1, t) * 0.5 + 0.5`.
/// - Input 0: audio
/// - Output 0: phased audio
///
/// ### Example: Phased Saw Wave
/// ```
/// use fundsp::prelude::*;
/// saw_hz(110.0) >> phaser(0.5, |t| sin_hz(0.1, t) * 0.5 + 0.5);
/// ```
pub fn phaser<X: Fn(f32) -> f32 + Clone + Send + Sync>(
    feedback_amount: f32,
    phase_f: X,
) -> An<impl AudioNode<Inputs = U1, Outputs = U1>> {
    pass()
        & feedback(
            (pass() | lfo(move |t| lerp(1.0, 10.0, phase_f(t))))
                >> pipei::<U10, _, _>(|_i| (pass() | add(0.05)) >> !allpole::<f32>())
                >> (mul(feedback_amount) | sink()),
        )
}

/// Shared float variable. Can be read from and written to from multiple threads.
///
/// ### Example: Add Chorus With Wetness Control
/// ```
/// use fundsp::prelude::*;
/// let wet = shared(0.2);
/// pass() & var(&wet) * chorus(0, 0.015, 0.005, 0.5);
/// ```
pub fn shared(value: f32) -> Shared {
    Shared::new(value)
}

/// Outputs the value of the shared variable.
///
/// - Output 0: value
///
/// ### Example: Add Chorus With Wetness Control
/// ```
/// use fundsp::prelude::*;
/// let wet = shared(0.2);
/// pass() & var(&wet) * chorus(0, 0.015, 0.005, 0.5);
/// ```
pub fn var(shared: &Shared) -> An<Var> {
    An(Var::new(shared))
}

/// Shared variable mapped through a function.
/// Outputs the value of the function, which may be scalar or tuple.
///
/// - Outputs: value
///
/// ### Example: Control Pitch In MIDI Semitones With Smoothing
/// ```
/// use fundsp::prelude::*;
/// let pitch = shared(69.0);
/// var_fn(&pitch, |x| midi_hz(x)) >> follow(0.01) >> saw();
/// ```
pub fn var_fn<F, R>(shared: &Shared, f: F) -> An<VarFn<F, R>>
where
    F: Clone + Fn(f32) -> R + Send + Sync,
    R: ConstantFrame<Sample = f32>,
    R::Size: Size<f32>,
{
    An(VarFn::new(shared, f))
}

/// Timer node. A node with no inputs or outputs that maintains
/// current stream time in a shared variable.
/// It can be added to any node by stacking.
///
/// ### Example: Timer And LFO
/// ```
/// use fundsp::prelude::*;
/// let time = shared(0.0);
/// timer(&time) | lfo(|t: f32| 1.0 / (1.0 + t));
/// ```
pub fn timer(shared: &Shared) -> An<Timer> {
    An(Timer::new(shared))
}

/// Snoop node for sharing audio data with a frontend thread.
/// The latest samples buffer has room for at least `capacity` samples.
/// Returns (frontend, backend).
/// - Input 0: signal to snoop.
/// - Output 0: signal passed through.
pub fn snoop(capacity: usize) -> (Snoop, An<SnoopBackend>) {
    let (snoop, backend) = Snoop::new(capacity);
    (snoop, An(backend))
}

/// Frequency domain resynthesizer.
/// The number of inputs is `I` and the number of outputs is `O`.
/// The window length (in samples) must be a power of two and at least four.
/// The resynthesizer processes windows of input samples transformed into the frequency domain.
/// The user supplied processing function processes frequency domain inputs into frequency domain outputs.
/// The outputs are inverse transformed and overlap-added.
/// The latency in samples is equal to window length.
/// If any output is a copy of an input, then the input will be reconstructed exactly
/// once all windows are overlapping, which takes `window_length` extra samples.
/// - Allocates: all needed buffers when created.
/// - Input(s): `I` input signals.
/// - Output(s): `O` processed signals.
///
/// ### Example: FFT Brickwall Lowpass Filter
/// ```
/// use fundsp::prelude::*;
/// let cutoff = 1000.0;
/// let synth = resynth::<U1, U1, _>(1024, |fft|
///     for i in 0..fft.bins() {
///         if fft.frequency(i) <= cutoff {
///             fft.set(0, i, fft.at(0, i));
///         }
///     });
/// ```
pub fn resynth<I, O, F>(window_length: usize, processing: F) -> An<Resynth<I, O, F>>
where
    I: Size<f32>,
    O: Size<f32>,
    F: FnMut(&mut FftWindow) + Clone + Send + Sync,
{
    An(Resynth::new(window_length, processing))
}

/// `N`-channel impulse. The first sample on each channel is one and the rest are zero.
/// - Output(s): impulse.
pub fn impulse<N: Size<f32>>() -> An<Impulse<N>> {
    An(Impulse::new())
}

/// Rotate stereo signal `angle` radians and apply amplitude `gain`.
/// Rotations can be useful for mixing because they maintain the L2 norm of the signal.
/// - Input 0: left input
/// - Input 1: right input
/// - Output 0: rotated left output
/// - Output 1: rotated right output
///
/// ### Example (45 Degree Rotation)
/// ```
/// use fundsp::prelude::*;
/// rotate(f32::PI / 4.0, 1.0);
/// ```
pub fn rotate(angle: f32, gain: f32) -> An<Mixer<U2, U2>> {
    An(Mixer::new(
        [
            [cos(angle) * gain, -sin(angle) * gain].into(),
            [sin(angle) * gain, cos(angle) * gain].into(),
        ]
        .into(),
    ))
}

/// Convert `AudioUnit` `unit` to an `AudioNode` with 32-bit sample type `f32`.
/// The number of inputs and outputs is chosen statically and must match
/// the `AudioUnit`.
/// - Input(s): from `unit`.
/// - Output(s): from `unit`.
pub fn unit<I: Size<f32>, O: Size<f32>>(unit: Box<dyn AudioUnit>) -> An<Unit<I, O>> {
    An(Unit::new(unit))
}
