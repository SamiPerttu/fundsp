use utd::math::*;

// TODO: Are we going to use this for anything?

/// Sample depicts a storage format for sample values based on interchange with floating point type F.
pub trait Sample<F: Real> : Copy {
    /// Converts from sample into Float.
    fn get(self) -> F;
    /// Converts from Float into sample. The range of supported F values should cover at least the canonical range [-1, 1].
    fn put(x : F) -> Self;
}

impl Sample<f64> for f64 {
    fn get(self) -> f64 { self }
    fn put(x : f64) -> f64 { x }
}

impl Sample<f64> for f32 {
    fn get(self) -> f64 { self as f64 }
    fn put(x : f64) -> f32 { x as f32 }
}

impl Sample<f64> for i16 {
    fn get(self) -> f64 { self as f64 / 32767.0 }
    fn put(x : f64) -> i16 { (x * 32767.0) as i16 }
}
