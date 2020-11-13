use super::*;

/// sqrt(2)
pub const SQRT_2: f64 = std::f64::consts::SQRT_2;
/// e (Euler's constant)
pub const E: f64 = std::f64::consts::E;
/// pi
pub const PI: f64 = std::f64::consts::PI;
/// tau = 2 * pi
pub const TAU: f64 = std::f64::consts::TAU;

/// Clamps x between x0 and x1.
#[inline] pub fn clamp<T: Num>(x0: T, x1: T, x: T) -> T { x.max(x0).min(x1) }

/// Clamps x between 0 and 1.
#[inline] pub fn clamp01<T: Num>(x: T) -> T { x.max(T::zero()).min(T::one()) }

/// Clamps x between -1 and 1.
#[inline] pub fn clamp11<T: Num>(x: T) -> T { x.max(T::new(-1)).min(T::one()) }

/// Generic linear interpolation trait.
pub trait Lerp<T> {
    fn lerp(self, other: Self, t: T) -> Self;
}

impl<U, T> Lerp<T> for U where U: Add<Output = U> + Mul<T, Output = U>, T: Num {
    #[inline] fn lerp(self, other: U, t: T) -> U {
        self * (T::one() - t) + other * t
    }
}

/// Linear interpolation.
#[inline] pub fn lerp<U: Lerp<T>, T>(a: U, b: U, t: T) -> U {
    a.lerp(b, t)
}

/// Linear de-interpolation. Recovers t from interpolated x.
#[inline] pub fn delerp<T: Num>(a: T, b: T, x: T) -> T {
    (x - a) / (b - a)
}

/// Exponential interpolation. a, b > 0.
#[inline] pub fn xerp<U: Lerp<T> + Real, T>(a: U, b: U, t: T) -> U {
    exp(lerp(log(a), log(b), t))
}

/// Exponential de-interpolation. a, b, x > 0. Recovers t from interpolated x.
#[inline] pub fn dexerp<T: Num + Real>(a: T, b: T, x: T) -> T {
    log(x / a) / log(b / a)
}

/// Rounds to a multiple of step.
#[inline] pub fn discretize<T: Num>(x: T, step: T) -> T {
    (x / step).round() * step
}

/// Square of x.
#[inline] pub fn squared<T: Mul<Output = T> + Copy>(x: T) -> T {
    x * x
}

/// Cube of x.
#[inline] pub fn cubed<T: Mul<Output = T> + Copy>(x: T) -> T {
    x * x * x
}

/// Catmull-Rom cubic spline interpolation, which is a form of cubic Hermite spline. Interpolates between
/// y1 (returns y1 when x = 0) and y2 (returns y2 when x = 1) while using the previous (y0) and next (y3)
/// points to define slopes at the endpoints. The maximum overshoot is 1/8th of the range of the arguments.
#[inline] pub fn spline<T: Num>(y0: T, y1: T, y2: T, y3: T, x: T) -> T {
    y1 + x / T::new(2) * (y2 - y0 + x * (T::new(2) * y0 - T::new(5) * y1 + T::new(4) * y2 - y3 + x * (T::new(3) * (y1 - y2) + y3 - y0)))
}

/// Monotonic cubic interpolation via Steffen's method. The result never overshoots.
/// It is first order continuous. Interpolates between y1 (at x = 0) and y2 (at x = 1)
/// while using the previous (y0) and next (y3) values to influence slopes.
pub fn cerp<T: Num>(y0: T, y1: T, y2: T, y3: T, x: T) -> T {
  let d0 = y1 - y0;
  let d1 = y2 - y1;
  let d2 = y3 - y2;
  let d1d = (signum(d0) + signum(d1)) * min(d0 + d1, min(abs(d0), abs(d1)));
  let d2d = (signum(d1) + signum(d2)) * min(d1 + d2, min(abs(d1), abs(d2)));
  cubed(x) * (T::new(2) * y1 - T::new(2) * y2 + d1d + d2d) + squared(x) * (T::new(-3) * y1 + T::new(3) * y2 - T::new(2) * d1d - d2d) + x * d1d + y1
}

/// Logistic sigmoid.
#[inline] pub fn logistic<T: Num + Real>(x: T) -> T {
    T::one() / (T::one() + exp(T::zero() - x))
}

/// Softsign function.
#[inline] pub fn softsign<T: Num>(x: T) -> T {
    x / (T::one() + x.abs())
}

/// This exp-like response function is second order continuous.
/// It has asymmetrical magnitude curves: (inverse) linear when x < 0 and quadratic when x > 0.
/// f(x) >= 0 for all x. Like the exponential function, f(0) = f'(0) = 1.
#[inline] pub fn exq<T: Num>(x: T) -> T {
    // With a branch:
    // if x > 0 { x * x + x + 1 } else { 1 / (1 - x) }
    let p = max(x, T::zero());
    p * p + p + T::one() / (T::one() + p - x)
}

// Softmin function when amount < 0, softmax when amount > 0, and average when amount = 0.
#[inline] pub fn softmix<T: Num>(amount: T, x: T, y: T) -> T {
    let xw = exq(x * amount);
    let yw = exq(y * amount);
    let epsilon = T::from_f32(1.0e-10);
    (x * xw + y * yw) / (xw + yw + epsilon)
}

/// Sum of an arithmetic series with n terms: sum over i in [0, n[ of (a0 + step * i).
#[inline] pub fn arithmetic_sum<T: Num>(n: T, a0: T, step: T) -> T {
    n * (T::new(2) * a0 + step * (n - T::one())) / T::new(2)
}

/// Sum of a geometric series with n terms: sum over i in [0, n[ of (a0 * ratio ** i).
#[inline] pub fn geometric_sum<T: Num + PartialOrd>(n: T, a0: T, ratio: T) -> T {
    let denom = T::one() - ratio;
    if denom != T::zero() {
        a0 * (T::one() - ratio.pow(n)) / denom
    } else {
        a0 * n
    }
}
