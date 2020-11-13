pub use super::math::*;
pub use super::*;

// Function combinator environment. We like to define all kinds of useful functions here.

/// Smooth cubic fade curve.
#[inline] pub fn smooth3<T: Num>(x: T) -> T {
    (T::new(3) - T::new(2) * x) * x * x
}

/// Smooth quintic fade curve suggested by Ken Perlin.
#[inline] pub fn smooth5<T: Num>(x: T) -> T {
    ((x * T::new(6) - T::new(15)) * x + T::new(10)) * x * x * x
}

/// Smooth septic fade curve.
#[inline] pub fn smooth7<T: Num>(x: T) -> T {
    let x2 = x * x;
    x2 * x2 * (T::new(35) - T::new(84) * x + (T::new(70) - T::new(20) * x) * x2)
}

/// Smooth nonic fade curve.
#[inline] pub fn smooth9<T: Num>(x: T) -> T {
    let x2 = x * x;
    ((((T::new(70) * x - T::new(315)) * x + T::new(540)) * x - T::new(420)) * x + T::new(125)) * x2 * x2 * x
}

/// Fade that starts and ends at a slope but levels in the middle.
#[inline] pub fn shelf<T: Num>(x: T) -> T {
    ((T::new(4) * x - T::new(6)) * x + T::new(3)) * x
}

/// A quarter circle fade that slopes upwards. Inverse function of Fade.downarc.
#[inline] pub fn uparc<T: Real + Num>(x: T) -> T {
    T::one() - sqrt(max(T::zero(), T::one() - x * x))
}

/// A quarter circle fade that slopes downwards. Inverse function of Fade.uparc.
#[inline] pub fn downarc<T: Real + Num>(x: T) -> T {
    sqrt(max(T::new(0), (T::new(2) - x) * x))
}

/// Wave function stitched together from two symmetric pieces peaking at origin.
#[inline] pub fn wave<T: Num, F: Fn(T) -> T>(f: F, x: T) -> T {
    let u = (x - T::one()) / T::new(4);
    let u = (u - u.floor()) * T::new(2);
    let w0 = u.min(T::one());
    let w1 = u - w0;
    T::one() - (f(w0) - f(w1)) * T::new(2)
}

/// Wave function with smooth3 interpolation.
#[inline] pub fn wave3<T: Num>(x: T) -> T { wave(smooth3, x) }

/// Wave function with smooth5 interpolation.
#[inline] pub fn wave5<T: Num>(x: T) -> T { wave(smooth5, x) }

/// Linear congruential generator proposed by Donald Knuth. Cycles through all u64 values.
#[inline] pub fn lcg64(x: u64) -> u64 { x * 6364136223846793005 + 1442695040888963407 }

/// Sine that oscillates at the specified beats per minute. Time is input in seconds.
#[inline] pub fn sin_bpm<T: Num + Real>(bpm: T, t: T) -> T { sin(t * bpm * T::from_f64(TAU / 60.0)) }

/// Cosine that oscillates at the specified beats per minute. Time is input in seconds.
#[inline] pub fn cos_bpm<T: Num + Real>(bpm: T, t: T) -> T { cos(t * bpm * T::from_f64(TAU / 60.0)) }

/// Sine that oscillates at the specified frequency (Hz). Time is input in seconds.
#[inline] pub fn sin_hz<T: Num + Real>(hz: T, t: T) -> T { sin(t * hz * T::from_f64(TAU)) }

/// Cosine that oscillates at the specified frequency (Hz). Time is input in seconds.
#[inline] pub fn cos_hz<T: Num + Real>(hz: T, t: T) -> T { cos(t * hz * T::from_f64(TAU)) }

/// Returns a gain factor from a decibel argument.
#[inline] pub fn db_gain<T: Num + Real>(db: T) -> T { exp(log(T::new(10)) * db / T::new(20)) }
