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
    (x * T::from_f64(LN_2)).exp()
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
    x.log() / T::from_f64(LN_2)
}
#[inline]
pub fn log10<T: Real>(x: T) -> T {
    x.log() / T::from_f64(LN_10)
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
/// e (Euler's constant)
pub const E: f64 = std::f64::consts::E;
/// pi
pub const PI: f64 = std::f64::consts::PI;
/// tau = 2 * pi
pub const TAU: f64 = std::f64::consts::TAU;
/// log(2)
pub const LN_2: f64 = std::f64::consts::LN_2;
/// log(10)
pub const LN_10: f64 = std::f64::consts::LN_10;

/// Clamps x between x0 and x1.
#[inline]
pub fn clamp<T: Num>(x0: T, x1: T, x: T) -> T {
    x.max(x0).min(x1)
}

/// Clamps x between 0 and 1.
#[inline]
pub fn clamp01<T: Num>(x: T) -> T {
    x.max(T::zero()).min(T::one())
}

/// Clamps x between -1 and 1.
#[inline]
pub fn clamp11<T: Num>(x: T) -> T {
    x.max(T::new(-1)).min(T::one())
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

/// Linear de-interpolation. Recovers t from interpolated x.
#[inline]
pub fn delerp<T: Num>(a: T, b: T, x: T) -> T {
    (x - a) / (b - a)
}

/// Exponential interpolation. a, b > 0.
#[inline]
pub fn xerp<U: Lerp<T> + Real, T>(a: U, b: U, t: T) -> U {
    exp(lerp(log(a), log(b), t))
}

/// Returns a dissonance amount between pure tones at f0 and f1 Hz.
/// Dissonance amounts range between 0 and 1.
#[inline]
pub fn dissonance<T: Num + Real>(f0: T, f1: T) -> T {
    let q = abs(f0 - f1) / (T::from_f64(0.021) * min(f0, f1) + T::from_f64(19.0));
    T::from_f64(5.531753) * (exp(T::from_f64(-0.84) * q) - exp(T::from_f64(-1.38) * q))
}

/// Returns maximally dissonant pure frequency above f Hz.
#[inline]
pub fn dissonance_max<T: Num>(f: T) -> T {
    T::from_f64(1.0193) * f + T::from_f64(17.4672)
}

/// Exponential de-interpolation. a, b, x > 0. Recovers t from interpolated x.
#[inline]
pub fn dexerp<T: Num + Real>(a: T, b: T, x: T) -> T {
    log(x / a) / log(b / a)
}

/// Returns a gain amount from a decibel argument.
#[inline]
pub fn db_gain<T: Num + Real>(db: T) -> T {
    exp10(db / T::new(20))
}

/// M-weighted noise response function. Returns human ear amplitude response at f Hz.
#[inline]
pub fn m_weight<T: Float>(f: T) -> T {
    let i0: f64 = log(max(f.to_f64(), 1.0));
    let r2 = softsign(pow(i0, i0) * -6.08068842478902e-05 - 255817.484465234);
    let r4 = softsign(i0 * -0.0120284517694679 + 0.0137065071001576);
    let db = pow(r2, r4) * -201414774.297872 - 201414825.531915;
    convert(db_gain(db))
}

/// Catmull-Rom cubic spline interpolation, which is a form of cubic Hermite spline. Interpolates between
/// y1 (returns y1 when x = 0) and y2 (returns y2 when x = 1) while using the previous (y0) and next (y3)
/// points to define slopes at the endpoints. The maximum overshoot is 1/8th of the range of the arguments.
#[inline]
pub fn spline<T: Num>(y0: T, y1: T, y2: T, y3: T, x: T) -> T {
    y1 + x / T::new(2)
        * (y2 - y0
            + x * (T::new(2) * y0 - T::new(5) * y1 + T::new(4) * y2 - y3
                + x * (T::new(3) * (y1 - y2) + y3 - y0)))
}

/// Monotonic cubic interpolation via Steffen's method. The result never overshoots.
/// It is first order continuous. Interpolates between y1 (at x = 0) and y2 (at x = 1)
/// while using the previous (y0) and next (y3) values to influence slopes.
pub fn splinem<T: Num>(y0: T, y1: T, y2: T, y3: T, x: T) -> T {
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

/// Logistic sigmoid.
#[inline]
pub fn logistic<T: Num + Real>(x: T) -> T {
    T::one() / (T::one() + exp(T::zero() - x))
}

/// Softsign function.
#[inline]
pub fn softsign<T: Num>(x: T) -> T {
    x / (T::one() + x.abs())
}

/// This exp-like response function is second order continuous.
/// It has asymmetrical magnitude curves: (inverse) linear when x < 0 and quadratic when x > 0.
/// f(x) >= 0 for all x. Like the exponential function, f(0) = f'(0) = 1.
#[inline]
pub fn exq<T: Num>(x: T) -> T {
    // With a branch:
    // if x > 0 { x * x + x + 1 } else { 1 / (1 - x) }
    let p = max(x, T::zero());
    p * p + p + T::one() / (T::one() + p - x)
}

// Softmin function when bias < 0, softmax when bias > 0, and average when bias = 0.
#[inline]
pub fn softmix<T: Num>(x: T, y: T, bias: T) -> T {
    let xw = exq(x * bias);
    let yw = exq(y * bias);
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
    ((((T::new(70) * x - T::new(315)) * x + T::new(540)) * x - T::new(420)) * x + T::new(125))
        * x2
        * x2
        * x
}

/// A quarter circle fade that slopes upwards. Inverse function of Fade.downarc.
#[inline]
pub fn arcup<T: Real + Num>(x: T) -> T {
    T::one() - sqrt(max(T::zero(), T::one() - x * x))
}

/// A quarter circle fade that slopes downwards. Inverse function of Fade.uparc.
#[inline]
pub fn arcdown<T: Real + Num>(x: T) -> T {
    sqrt(max(T::new(0), (T::new(2) - x) * x))
}

/// Wave function stitched together from two symmetric pieces peaking at origin.
#[inline]
pub fn easewave<T: Num, F: Fn(T) -> T>(f: F, x: T) -> T {
    let u = (x - T::from_f64(PI)) / T::from_f64(4.0 * PI);
    let u = (u - u.floor()) * T::new(2);
    let w0 = u.min(T::one());
    let w1 = u - w0;
    T::one() - (f(w0) - f(w1)) * T::new(2)
}

/// Sine that oscillates at the specified beats per minute. Time is input in seconds.
#[inline]
pub fn sin_bpm<T: Num + Real>(bpm: T, t: T) -> T {
    sin(t * bpm * T::from_f64(TAU / 60.0))
}

/// Cosine that oscillates at the specified beats per minute. Time is input in seconds.
#[inline]
pub fn cos_bpm<T: Num + Real>(bpm: T, t: T) -> T {
    cos(t * bpm * T::from_f64(TAU / 60.0))
}

/// Sine that oscillates at the specified frequency (Hz). Time is input in seconds.
#[inline]
pub fn sin_hz<T: Num + Real>(hz: T, t: T) -> T {
    sin(t * hz * T::from_f64(TAU))
}

/// Cosine that oscillates at the specified frequency (Hz). Time is input in seconds.
#[inline]
pub fn cos_hz<T: Num + Real>(hz: T, t: T) -> T {
    cos(t * hz * T::from_f64(TAU))
}

/// Converts from semitone interval to frequency ratio.
#[inline]
pub fn semitone<T: Num + Real>(x: T) -> T {
    exp2(x / T::from_f64(12.0))
}

/// 32-bit hash by Chris Wellon, used for pinging.
pub const fn hashw(x: u32) -> u32 {
    let x = (x ^ (x >> 16)).wrapping_mul(0x7feb352d);
    let x = (x ^ (x >> 15)).wrapping_mul(0x846ca68b);
    x ^ (x >> 16)
}

/// SplitMix hash as an indexed RNG.
/// Returns pseudorandom f64 in range [0, 1[.
pub fn rnd(x: u64) -> f64 {
    let x = x.wrapping_mul(0x9e3779b97f4a7c15);
    let x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    let x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    let x = x ^ (x >> 31);
    (x >> 11) as f64 / (1u64 << 53) as f64
}
