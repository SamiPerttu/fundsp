//! Math functions and utilities and procedural generation tools.

use super::*;

#[inline]
pub fn abs<T: Num>(x: T) -> T {
    x.abs()
}
#[inline]
pub fn signum<T: Num>(x: T) -> T {
    x.signum()
}
#[inline]
pub fn min<T: Num>(x: T, y: T) -> T {
    x.min(y)
}
#[inline]
pub fn max<T: Num>(x: T, y: T) -> T {
    x.max(y)
}
#[inline]
pub fn pow<T: Num>(x: T, y: T) -> T {
    x.pow(y)
}
#[inline]
pub fn floor<T: Num>(x: T) -> T {
    x.floor()
}
#[inline]
pub fn fract<T: Num>(x: T) -> T {
    x.fract()
}
#[inline]
pub fn ceil<T: Num>(x: T) -> T {
    x.ceil()
}
#[inline]
pub fn round<T: Num>(x: T) -> T {
    x.round()
}

#[inline]
pub fn sqrt<T: Real>(x: T) -> T {
    x.sqrt()
}
#[inline]
pub fn exp<T: Real>(x: T) -> T {
    x.exp()
}
#[inline]
pub fn exp2<T: Real>(x: T) -> T {
    x.exp2()
}
#[inline]
pub fn exp10<T: Real>(x: T) -> T {
    (x * T::from_f64(LN_10)).exp()
}
#[inline]
pub fn log<T: Real>(x: T) -> T {
    x.log()
}
#[inline]
pub fn log2<T: Real>(x: T) -> T {
    x.log2()
}
#[inline]
pub fn log10<T: Real>(x: T) -> T {
    x.log10()
}
#[inline]
pub fn sin<T: Real>(x: T) -> T {
    x.sin()
}
#[inline]
pub fn cos<T: Real>(x: T) -> T {
    x.cos()
}
#[inline]
pub fn tan<T: Real>(x: T) -> T {
    x.tan()
}
#[inline]
pub fn tanh<T: Real>(x: T) -> T {
    x.tanh()
}

/// sqrt(2)
pub const SQRT_2: f64 = std::f64::consts::SQRT_2;
/// pi
pub const PI: f64 = std::f64::consts::PI;
/// tau = 2 * pi
pub const TAU: f64 = std::f64::consts::TAU;
/// log(2)
pub const LN_2: f64 = std::f64::consts::LN_2;
/// log(10)
pub const LN_10: f64 = std::f64::consts::LN_10;

/// Clamps `x` between `x0` and `x1`.
#[inline]
pub fn clamp<T: Num>(x0: T, x1: T, x: T) -> T {
    x.max(x0).min(x1)
}

/// Clamps `x` between 0 and 1.
#[inline]
pub fn clamp01<T: Num>(x: T) -> T {
    x.max(T::zero()).min(T::one())
}

/// Clamps `x` between -1 and 1.
#[inline]
pub fn clamp11<T: Num>(x: T) -> T {
    x.max(T::new(-1)).min(T::one())
}

/// Identity function.
#[inline]
pub fn id<T>(x: T) -> T {
    x
}

/// Square function.
#[inline]
pub fn squared<T: Num>(x: T) -> T {
    x * x
}

/// Cube function.
#[inline]
pub fn cubed<T: Num>(x: T) -> T {
    x * x * x
}

/// Generic linear interpolation trait.
pub trait Lerp<T> {
    fn lerp(self, other: Self, t: T) -> Self;
}

impl<U, T> Lerp<T> for U
where
    U: Add<Output = U> + Mul<T, Output = U>,
    T: Num,
{
    #[inline]
    fn lerp(self, other: U, t: T) -> U {
        self * (T::one() - t) + other * t
    }
}

/// Linear interpolation.
#[inline]
pub fn lerp<U: Lerp<T>, T>(a: U, b: U, t: T) -> U {
    a.lerp(b, t)
}

/// Linear interpolation with `t` in -1...1.
#[inline]
pub fn lerp11<U: Lerp<T>, T: Num>(a: U, b: U, t: T) -> U {
    a.lerp(b, t * T::from_f32(0.5) + T::from_f32(0.5))
}

/// Linear de-interpolation. Recovers `t` from interpolated `x`.
#[inline]
pub fn delerp<T: Num>(a: T, b: T, x: T) -> T {
    (x - a) / (b - a)
}

/// Linear de-interpolation. Recovers `t` in -1...1 from interpolated `x`.
#[inline]
pub fn delerp11<T: Num>(a: T, b: T, x: T) -> T {
    (x - a) / (b - a) * T::new(2) - T::new(1)
}

/// Exponential interpolation. `a`, `b` > 0.
#[inline]
pub fn xerp<U: Lerp<T> + Real, T>(a: U, b: U, t: T) -> U {
    exp(lerp(log(a), log(b), t))
}

/// Exponential interpolation with `t` in -1...1. `a`, `b` > 0.
#[inline]
pub fn xerp11<U: Lerp<T> + Real, T: Num>(a: U, b: U, t: T) -> U {
    exp(lerp(
        log(a),
        log(b),
        t * T::from_f32(0.5) + T::from_f32(0.5),
    ))
}

/// Exponential de-interpolation. `a`, `b`, `x` > 0.
/// Recovers `t` from interpolated `x`.
#[inline]
pub fn dexerp<T: Real>(a: T, b: T, x: T) -> T {
    log(x / a) / log(b / a)
}

/// Exponential de-interpolation. `a`, `b`, `x` > 0.
/// Recovers `t` in -1...1 from interpolated `x`.
#[inline]
pub fn dexerp11<T: Real>(a: T, b: T, x: T) -> T {
    log(x / a) / log(b / a) * T::new(2) - T::new(1)
}

/// Return a dissonance amount between pure tones at `f0` and `f1` Hz.
/// Dissonance amounts range between 0 and 1.
#[inline]
pub fn dissonance<T: Real>(f0: T, f1: T) -> T {
    let q = abs(f0 - f1) / (T::from_f64(0.021) * min(f0, f1) + T::from_f64(19.0));
    T::from_f64(5.531753) * (exp(T::from_f64(-0.84) * q) - exp(T::from_f64(-1.38) * q))
}

/// Return the maximally dissonant pure frequency above `f` Hz.
#[inline]
pub fn dissonance_max<T: Num>(f: T) -> T {
    T::from_f64(1.0193) * f + T::from_f64(17.4672)
}

/// Convert decibels to gain. 0 dB = 1.0.
#[inline]
pub fn db_amp<T: Real>(db: T) -> T {
    exp10(db / T::new(20))
}

/// Convert amplitude `gain` (`gain` > 0) to decibels. Unity gain = 0 dB.
#[inline]
pub fn amp_db<T: Real>(gain: T) -> T {
    log10(gain) * T::new(20)
}

/// A-weighted response function.
/// Returns equal loudness amplitude response at `f` Hz.
/// Normalized to 1.0 at 1 kHz.
#[inline]
pub fn a_weight<T: Real>(f: T) -> T {
    let f2 = squared(f);
    let c0 = squared(T::from_f64(12194.0));
    let c1 = squared(T::from_f64(20.6));
    let c2 = squared(T::from_f64(107.7));
    let c3 = squared(T::from_f64(737.9));
    let c4 = T::from_f64(1.2589048990582914);
    c4 * c0 * f2 * f2 / ((f2 + c1) * sqrt((f2 + c2) * (f2 + c3)) * (f2 + c0))
}

/// M-weighted response function normalized to 1 kHz.
/// M-weighting is an unofficial name for
/// the frequency response curve of the ITU-R 468 noise weighting standard.
/// Returns equal loudness amplitude response at f Hz.
/// Normalized to 1.0 at 1 kHz.
#[inline]
pub fn m_weight<T: Real>(f: T) -> T {
    let c0 = T::from_f64(1.246332637532143 * 1.0e-4);
    let c1 = T::from_f64(-4.737338981378384 * 1.0e-24);
    let c2 = T::from_f64(2.04382833606125 * 1.0e-15);
    let c3 = T::from_f64(-1.363894795463638 * 1.0e-7);
    let c4 = T::from_f64(1.306612257412824 * 1.0e-19);
    let c5 = T::from_f64(-2.118150887518656 * 1.0e-11);
    let c6 = T::from_f64(5.559488023498642 * 1.0e-4);
    let c7 = T::from_f64(8.164578311186197);
    let f2 = f * f;
    let f4 = f2 * f2;
    c7 * c0 * f
        / sqrt(
            squared(c1 * f4 * f2 + c2 * f4 + c3 * f2 + T::one())
                + squared(c4 * f4 * f + c5 * f2 * f + c6 * f),
        )
}

/// Catmull-Rom cubic spline interpolation, which is a form of cubic Hermite spline.
/// Interpolates between `y1` (returns `y1` when `x` = 0) and `y2` (returns `y2` when `x` = 1)
/// while using the previous (`y0`) and next (`y3`)
/// points to define slopes at the endpoints.
/// The maximum overshoot is 1/8th of the range of the arguments.
#[inline]
pub fn spline<T: Num>(y0: T, y1: T, y2: T, y3: T, x: T) -> T {
    y1 + x / T::new(2)
        * (y2 - y0
            + x * (T::new(2) * y0 - T::new(5) * y1 + T::new(4) * y2 - y3
                + x * (T::new(3) * (y1 - y2) + y3 - y0)))
}

/// Monotonic cubic interpolation via Steffen's method. The result never overshoots.
/// It is first order continuous. Interpolates between `y1` (at `x` = 0) and `y2` (at `x` = 1)
/// while using the previous (`y0`) and next (`y3`) values to influence slopes.
pub fn spline_mono<T: Num>(y0: T, y1: T, y2: T, y3: T, x: T) -> T {
    let d0 = y1 - y0;
    let d1 = y2 - y1;
    let d2 = y3 - y2;
    let d1d = (signum(d0) + signum(d1)) * min(d0 + d1, min(abs(d0), abs(d1)));
    let d2d = (signum(d1) + signum(d2)) * min(d1 + d2, min(abs(d1), abs(d2)));
    x * x * x * (T::new(2) * y1 - T::new(2) * y2 + d1d + d2d)
        + x * x * (T::new(-3) * y1 + T::new(3) * y2 - T::new(2) * d1d - d2d)
        + x * d1d
        + y1
}

/// Softsign function.
#[inline]
pub fn softsign<T: Num>(x: T) -> T {
    x / (T::one() + x.abs())
}

/// This exp-like response function is second order continuous.
/// It has asymmetrical magnitude curves: (inverse) linear when `x` < 0 and quadratic when `x` > 0.
/// `softexp(x)` >= 0 for all `x`. Like the exponential function, `softexp(0)` = `softexp'(0)` = 1.
#[inline]
pub fn softexp<T: Num>(x: T) -> T {
    // With a branch:
    // if x > 0 { x * x + x + 1 } else { 1 / (1 - x) }
    let p = max(x, T::zero());
    p * p + p + T::one() / (T::one() + p - x)
}

/// Softmin function when `bias` < 0, softmax when `bias` > 0, and average when `bias` = 0.
#[inline]
pub fn softmix<T: Num>(x: T, y: T, bias: T) -> T {
    let xw = softexp(x * bias);
    let yw = softexp(y * bias);
    let epsilon = T::from_f32(1.0e-10);
    (x * xw + y * yw) / (xw + yw + epsilon)
}

/// Smooth 3rd degree easing polynomial.
#[inline]
pub fn smooth3<T: Num>(x: T) -> T {
    (T::new(3) - T::new(2) * x) * x * x
}

/// Smooth 5th degree easing polynomial.
#[inline]
pub fn smooth5<T: Num>(x: T) -> T {
    ((x * T::new(6) - T::new(15)) * x + T::new(10)) * x * x * x
}

/// Smooth 7th degree easing polynomial.
#[inline]
pub fn smooth7<T: Num>(x: T) -> T {
    let x2 = x * x;
    x2 * x2 * (T::new(35) - T::new(84) * x + (T::new(70) - T::new(20) * x) * x2)
}

/// Smooth 9th degree easing polynomial.
#[inline]
pub fn smooth9<T: Num>(x: T) -> T {
    let x2 = x * x;
    ((((T::new(70) * x - T::new(315)) * x + T::new(540)) * x - T::new(420)) * x + T::new(126))
        * x2
        * x2
        * x
}

/// A quarter circle fade that slopes upwards. Inverse function of `downarc`.
#[inline]
pub fn uparc<T: Real>(x: T) -> T {
    T::one() - sqrt(max(T::zero(), T::one() - x * x))
}

/// A quarter circle fade that slopes downwards. Inverse function of `uparc`.
#[inline]
pub fn downarc<T: Real>(x: T) -> T {
    sqrt(max(T::new(0), (T::new(2) - x) * x))
}

/// Sine that oscillates at the specified frequency (Hz). Time is input in seconds.
#[inline]
pub fn sin_hz<T: Real>(hz: T, t: T) -> T {
    sin(t * hz * T::from_f64(TAU))
}

/// Cosine that oscillates at the specified frequency (Hz). Time is input in seconds.
#[inline]
pub fn cos_hz<T: Real>(hz: T, t: T) -> T {
    cos(t * hz * T::from_f64(TAU))
}

/// Converts from semitone interval to frequency ratio.
#[inline]
pub fn semitone<T: Real>(x: T) -> T {
    exp2(x / T::from_f64(12.0))
}

/// SplitMix hash as an indexed RNG.
/// Returns pseudorandom f64 in 0...1.
#[inline]
pub fn rnd(x: i64) -> f64 {
    let x = x as u64 ^ 0x5555555555555555;
    let x = x.wrapping_mul(0x9e3779b97f4a7c15);
    let x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    let x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    let x = x ^ (x >> 31);
    (x >> 11) as f64 / (1u64 << 53) as f64
}

/// 64-bit hash function.
/// This hash is a pseudorandom permutation.
#[inline]
pub fn hash(x: i64) -> i64 {
    let x = x as u64 ^ 0x5555555555555555;
    let x = x.wrapping_mul(0x517cc1b727220a95);
    // Following hash is by degsky.
    let x = (x ^ (x >> 32)).wrapping_mul(0xd6e8feb86659fd93);
    let x = (x ^ (x >> 32)).wrapping_mul(0xd6e8feb86659fd93);
    (x ^ (x >> 32)) as i64
}

/// Convert MIDI note number to frequency in Hz. Returns 440 Hz for A_4 (note number 69).
/// The lowest key on a grand piano is A_0 at 27.5 Hz (note number 21).
/// Note number 0 is C_-1.
#[inline]
pub fn midi_hz<T: Real>(x: T) -> T {
    T::new(440) * exp2((x - T::new(69)) / T::new(12))
}

/// Convert BPM to Hz.
#[inline]
pub fn bpm_hz<T: Real>(bpm: T) -> T {
    bpm / T::new(60)
}

/// Pico sized RNG.
#[derive(Default, Clone)]
pub struct AttoRand {
    state: u64,
}

impl AttoRand {
    /// Create new `AttoRand` from seed.
    #[inline]
    pub fn new(seed: u64) -> AttoRand {
        AttoRand {
            state: seed ^ 0x5555555555555555,
        }
    }
    /// Hashes `data`. Consumes self and returns a new `AttoRand`.
    #[inline]
    pub fn hash(self, data: u64) -> Self {
        // Hash taken from FxHasher.
        AttoRand {
            state: self
                .state
                .rotate_left(5)
                .bitxor(data)
                .wrapping_mul(0x517cc1b727220a95),
        }
    }
    /// Current 64-bit value.
    #[inline]
    pub fn value(&self) -> u64 {
        // Use SplitMix hash.
        let x = self.state.wrapping_mul(0x9e3779b97f4a7c15);
        let x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        let x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
        x ^ (x >> 31)
    }
    /// Get next 64-bit value.
    #[inline]
    pub fn get(&mut self) -> u64 {
        // Multiplier taken from https://arxiv.org/abs/2001.05304.
        self.state = self.state.wrapping_mul(0xd1342543de82ef95).wrapping_add(1);
        self.value()
    }
    /// Get next floating point value in 0...1.
    #[inline]
    pub fn get01<T: Float>(&mut self) -> T {
        let x = self.get();
        T::from_f64((x >> 11) as f64 / (1u64 << 53) as f64)
    }
    /// Get next floating point value in -1...1.
    #[inline]
    pub fn get11<T: Float>(&mut self) -> T {
        let x = self.get();
        T::from_f64((x >> 10) as f64 / (1u64 << 53) as f64 - 1.0)
    }
}

/// Trait for symmetric/asymmetric interpolation in `ease_noise`.
pub trait SegmentInterpolator<T: Float>: Clone {
    /// Interpolate between `y1` and `y2` at relative position `t` in 0...1.
    /// `x1` and `x2` are additional information.
    fn interpolate(&self, x1: T, y1: T, x2: T, y2: T, t: T) -> T;
}

impl<T: Float, X> SegmentInterpolator<T> for X
where
    X: Fn(T) -> T + Clone,
{
    #[inline]
    fn interpolate(&self, _x1: T, y1: T, _x2: T, y2: T, t: T) -> T {
        lerp(y1, y2, (*self)(t))
    }
}

impl<T: Float, X, Y> SegmentInterpolator<T> for (X, Y)
where
    X: SegmentInterpolator<T>,
    Y: SegmentInterpolator<T>,
{
    #[inline]
    fn interpolate(&self, x1: T, y1: T, x2: T, y2: T, t: T) -> T {
        if y2 >= y1 {
            self.0.interpolate(x1, y1, x2, y2, t)
        } else {
            self.1.interpolate(x1, y1, x2, y2, t)
        }
    }
}

/// 1-D easing noise in -1...1 with frequency of 1.
/// Value noise interpolated with an easing function.
/// When interpolated linearly, the noise follows
/// a roughly triangular distribution in -1...1.
/// Each integer cell is an interpolation segment.
/// Easing function `ease` (for example, `smooth3`) can be asymmetric:
/// `(r, f)` employs `r` for rising and `f` for falling segments.
pub fn ease_noise<T: Float>(ease: impl SegmentInterpolator<T>, seed: i64, x: T) -> T {
    let fx = floor(x);
    let dx = x - fx;
    let ix = fx.to_i64();

    fn get_point<T: Float>(seed: i64, i: i64) -> T {
        AttoRand::new((seed ^ i) as u64).get11()
    }

    let y1 = get_point(seed, ix);
    let y2 = get_point(seed, ix.wrapping_add(1));

    ease.interpolate(fx, y1, fx + T::one(), y2, dx)
}

/// 1-D spline noise in -1...1 with frequency of 1.
/// Value noise interpolated with a cubic spline.
/// The noise follows a roughly triangular distribution in -1...1.
/// Each integer cell is an interpolation segment.
pub fn spline_noise<T: Float>(seed: i64, x: T) -> T {
    let fx = floor(x);
    let dx = x - fx;
    let ix = fx.to_i64();

    fn get_point<T: Float>(seed: i64, i: i64) -> T {
        AttoRand::new((seed ^ i) as u64).get11()
    }

    let y0 = get_point(seed, ix.wrapping_sub(1));
    let y1 = get_point(seed, ix);
    let y2 = get_point(seed, ix.wrapping_add(1));
    let y3 = get_point(seed, ix.wrapping_add(2));

    // The divisor brings the final result into the range -1...1.
    // Maximum overshoot occurs with spline(-1.0, 1.0, 1.0, -1.0, 0.5).
    spline(y0, y1, y2, y3, dx) / T::from_f32(1.25)
}
