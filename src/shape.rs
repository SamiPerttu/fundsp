//! Waveshaping components.

use super::audionode::*;
use super::buffer::*;
use super::math::*;
use super::signal::*;
use super::*;
use numeric_array::typenum::*;

/// A waveshaper: some kind of nonlinearity. It may have a state.
pub trait Shape: Clone + Sync + Send {
    /// Process a single sample.
    fn shape(&mut self, input: f32) -> f32;
    /// Process multiple samples at once in a SIMD element.
    #[inline]
    fn simd(&mut self, input: F32x) -> F32x {
        F32x::new(core::array::from_fn(|i| self.shape(input.as_array()[i])))
        /*
        let mut output = [0.0; SIMD_N];
        for i in 0..SIMD_N {
            output[i] = self.shape(input.as_array_ref()[i]);
        }
        F32x::new(output)
        */
    }
    /// Set the sample rate. The default sample rate is 44.1 kHz.
    #[allow(unused_variables)]
    fn set_sample_rate(&mut self, sample_rate: f64) {}
    /// Reset state.
    fn reset(&mut self) {}
}

/// Memoryless waveshaper from a closure.
#[derive(Clone)]
pub struct ShapeFn<S: Fn(f32) -> f32 + Clone + Sync + Send>(pub S);

impl<S: Fn(f32) -> f32 + Clone + Sync + Send> Shape for ShapeFn<S> {
    #[inline]
    fn shape(&mut self, input: f32) -> f32 {
        self.0(input)
    }
}

/// Clamp signal multiplied by the hardness parameter to -1...1.
#[derive(Clone)]
pub struct Clip(pub f32);

impl Shape for Clip {
    #[inline]
    fn shape(&mut self, input: f32) -> f32 {
        (input * self.0).clamp(-1.0, 1.0)
    }
    #[inline]
    fn simd(&mut self, input: F32x) -> F32x {
        (input * self.0).fast_max(-F32x::ONE).fast_min(F32x::ONE)
    }
}

/// Clamp signal between the two arguments (minimum and maximum).
#[derive(Clone)]
pub struct ClipTo(pub f32, pub f32);

impl Shape for ClipTo {
    #[inline]
    fn shape(&mut self, input: f32) -> f32 {
        input.clamp(self.0, self.1)
    }
    #[inline]
    fn simd(&mut self, input: F32x) -> F32x {
        input
            .fast_max(F32x::splat(self.0))
            .fast_min(F32x::splat(self.1))
    }
}

/// Apply `tanh` distortion with configurable hardness.
/// Argument to `tanh` is multiplied by the hardness value.
#[derive(Clone)]
pub struct Tanh(pub f32);

impl Shape for Tanh {
    #[inline]
    fn shape(&mut self, input: f32) -> f32 {
        (input * self.0).tanh()
    }
}

/// Apply `atan` distortion with configurable hardness.
/// The function has been modified to saturate at unity
/// while maintaining a slope of 1 at origin.
/// Argument to `atan` is multiplied by the hardness value.
#[derive(Clone)]
pub struct Atan(pub f32);

impl Shape for Atan {
    #[inline]
    fn shape(&mut self, input: f32) -> f32 {
        (input * (self.0 * f32::PI * 0.5)).atan() * (2.0 / f32::PI)
    }
    #[inline]
    fn simd(&mut self, input: F32x) -> F32x {
        (input * (self.0 * f32::PI * 0.5)).atan() * (2.0 / f32::PI)
    }
}

/// Apply `softsign` distortion with configurable hardness.
/// Argument to `softsign` is multiplied by the hardness value.
#[derive(Clone)]
pub struct Softsign(pub f32);

impl Shape for Softsign {
    #[inline]
    fn shape(&mut self, input: f32) -> f32 {
        softsign(input * self.0)
    }
    #[inline]
    fn simd(&mut self, input: F32x) -> F32x {
        input * self.0 / (F32x::ONE + input.abs() * self.0)
    }
}

/// A staircase function with configurable number of levels per unit.
#[derive(Clone)]
pub struct Crush(pub f32);

impl Shape for Crush {
    #[inline]
    fn shape(&mut self, input: f32) -> f32 {
        (input * self.0).round() / self.0
    }
    #[inline]
    fn simd(&mut self, input: F32x) -> F32x {
        (input * self.0).round() / self.0
    }
}

/// A smooth staircase function with configurable number of levels per unit.
#[derive(Clone)]
pub struct SoftCrush(pub f32);

impl Shape for SoftCrush {
    #[inline]
    fn shape(&mut self, input: f32) -> f32 {
        let x = input * self.0;
        let y = floor(x);
        (y + smooth9(x - y)) / self.0
    }
    #[inline]
    fn simd(&mut self, input: F32x) -> F32x {
        let x = input * self.0;
        let y = floor(x);
        (y + smooth9(x - y)) / self.0
    }
}

/// Adaptive normalizing distortion with smoothing timescale and inner shape as parameters.
/// Smoothing timescale is specified in seconds.
/// It is the time it takes for level estimation to move halfway to a new level.
/// Minimum estimated signal level for adaptive distortion is -60 dB.
/// The argument to the inner shape is divided by the RMS level of the signal.
#[derive(Clone)]
pub struct Adaptive<S: Shape> {
    /// Inner shape.
    inner: S,
    /// Smoothing timescale in seconds.
    timescale: f32,
    /// Per-sample smoothing factor.
    smoothing: f32,
    /// Current level estimate.
    state: f32,
}

impl<S: Shape> Adaptive<S> {
    pub fn new(timescale: f32, inner: S) -> Self {
        let mut adaptive = Self {
            inner,
            timescale,
            smoothing: 0.0,
            state: 0.0,
        };
        adaptive.set_sample_rate(DEFAULT_SR);
        adaptive
    }
}

impl<S: Shape> Shape for Adaptive<S> {
    #[inline]
    fn shape(&mut self, input: f32) -> f32 {
        self.state =
            self.smoothing * self.state + (1.0 - self.smoothing) * (1.0e-6 + squared(input));
        self.inner.shape(input / sqrt(self.state))
    }
    fn reset(&mut self) {
        self.state = 1.0e-3;
        self.inner.reset();
    }
    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.smoothing = pow(0.5, 1.0 / (self.timescale.to_f64() * sample_rate)).to_f32();
        self.inner.set_sample_rate(sample_rate);
    }
}

/// Waveshaper.
#[derive(Clone)]
pub struct Shaper<S: Shape> {
    shape: S,
}

impl<S: Shape> Shaper<S> {
    pub fn new(shape: S) -> Self {
        let mut shaper = Self { shape };
        shaper.set_sample_rate(DEFAULT_SR);
        shaper
    }
}

impl<S: Shape> AudioNode for Shaper<S> {
    const ID: u64 = 42;
    type Inputs = U1;
    type Outputs = U1;

    fn reset(&mut self) {
        self.shape.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.shape.set_sample_rate(sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        Frame::from([self.shape.shape(input[0])])
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        for i in 0..full_simd_items(size) {
            output.set(0, i, self.shape.simd(input.at(0, i)));
        }
        self.process_remainder(size, input, output);
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = SignalFrame::new(self.outputs());
        output.set(0, input.at(0).distort(0.0));
        output
    }
}
