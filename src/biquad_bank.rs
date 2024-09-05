use core::marker::PhantomData;
use core::ops::Neg;
use hacker::Parameter;
use wide::{f32x8, f64x4};

use super::audionode::*;
use super::prelude::{U4, U8};
use super::*;
use crate::setting::Setting;
use numeric_array::ArrayLength;

pub trait Realx<Size: ArrayLength>: Num + Sized + Neg<Output = Self> {
    const PI: Self;
    const TAU: Self;
    fn exp(self) -> Self;
    fn cos(self) -> Self;
    fn sqrt(self) -> Self;
    fn reduce_add(self) -> f32;
    fn to_frame(self) -> Frame<f32, Size>;
    fn set(&mut self, index: usize, value: f32);
}

impl Realx<U8> for f32x8 {
    const PI: Self = f32x8::PI;
    const TAU: Self = f32x8::TAU;

    #[inline(always)]
    fn exp(self) -> Self {
        f32x8::exp(self)
    }
    #[inline(always)]
    fn cos(self) -> Self {
        f32x8::cos(self)
    }
    #[inline(always)]
    fn sqrt(self) -> Self {
        f32x8::sqrt(self)
    }
    #[inline(always)]
    fn reduce_add(self) -> f32 {
        f32x8::reduce_add(self)
    }
    #[inline(always)]
    fn to_frame(self) -> Frame<f32, U8> {
        f32x8::to_array(self).into()
    }
    #[inline(always)]
    fn set(&mut self, index: usize, value: f32) {
        self.as_array_mut()[index] = value;
    }
}

impl Realx<U4> for f64x4 {
    const PI: Self = f64x4::PI;
    const TAU: Self = f64x4::TAU;

    #[inline(always)]
    fn exp(self) -> Self {
        f64x4::exp(self)
    }
    #[inline(always)]
    fn cos(self) -> Self {
        f64x4::cos(self)
    }
    #[inline(always)]
    fn sqrt(self) -> Self {
        f64x4::sqrt(self)
    }
    #[inline(always)]
    fn reduce_add(self) -> f32 {
        f64x4::reduce_add(self) as f32
    }
    #[inline(always)]
    fn to_frame(self) -> Frame<f32, U4> {
        let array_f64: [f64; 4] = f64x4::to_array(self);
        let array_f32: [f32; 4] = array_f64.map(|x| x as f32);
        array_f32.into()
    }
    #[inline(always)]
    fn set(&mut self, index: usize, value: f32) {
        self.as_array_mut()[index] = value as f64;
    }
}

/// BiquadBank coefficients in normalized form using SIMD.
#[derive(Copy, Clone, Debug, Default)]
pub struct BiquadCoefsBank<F, Size>
where
    F: Realx<Size>,
    Size: ArrayLength,
{
    pub a1: F,
    pub a2: F,
    pub b0: F,
    pub b1: F,
    pub b2: F,
    _marker: PhantomData<Size>,
}

impl<F, Size> BiquadCoefsBank<F, Size>
where
    F: Realx<Size>,
    Size: ArrayLength,
{
    /// Return settings for a constant-gain bandpass resonator-bank.
    /// Sample rate and center frequency are in Hz.
    /// The overall gain of the filter is independent of bandwidth.
    #[inline]
    pub fn resonator(sample_rate: f32, center: F, q: F) -> Self {
        let c = F::from_f64;
        let sr = F::from_f32(sample_rate);
        let r: F = (-F::PI * center / (q * sr)).exp();
        let a1: F = c(-2.0) * r * (F::TAU * center / sr).cos();
        let a2: F = r * r;
        let b0: F = (c(1.0) - r * r).sqrt() * c(0.5);
        let b1: F = c(0.0);
        let b2: F = -b0;
        Self {
            a1,
            a2,
            b0,
            b1,
            b2,
            _marker: PhantomData,
        }
    }

    /// Arbitrary biquad.
    #[inline]
    pub fn arbitrary(a1: F, a2: F, b0: F, b1: F, b2: F) -> Self {
        Self {
            a1,
            a2,
            b0,
            b1,
            b2,
            _marker: PhantomData,
        }
    }

    ///// Frequency response at frequency `omega` expressed as fraction of sampling rate.
    //pub fn response(&self, omega: f64) -> Complex64 {
    //    let z1 = Complex64::from_polar(1.0, -f64::TAU * omega);
    //    let z2 = z1 * z1;
    //    /// Complex64 with real component `x` and imaginary component zero.
    //    fn re<T: Float>(x: T) -> Complex64 {
    //        Complex64::new(x.to_f64(), 0.0)
    //    }
    //    (re(self.b0) + re(self.b1) * z1 + re(self.b2) * z2)
    //        / (re(1.0) + re(self.a1) * z1 + re(self.a2) * z2)
    //}
}

/// 2nd order IIR filter-bank implemented in normalized Direct Form I and SIMD.
/// - Setting: coefficients as tuple Parameter::BiquadBank(a1, a2, b0, b1, b2).
/// - Input 0: input signal.
/// - Output 0: filtered signal.
#[derive(Default, Clone)]
pub struct BiquadBank<F, Size>
where
    F: Realx<Size>,
    Size: ArrayLength + Sync + Send,
{
    coefs: BiquadCoefsBank<F, Size>,
    x1: F,
    x2: F,
    y1: F,
    y2: F,
    sample_rate: f64,
}

impl<F, Size> BiquadBank<F, Size>
where
    F: Realx<Size>,
    Size: ArrayLength + Sync + Send,
{
    pub fn new() -> Self {
        Self {
            sample_rate: DEFAULT_SR,
            ..Default::default()
        }
    }

    pub fn with_coefs(coefs: BiquadCoefsBank<F, Size>) -> Self {
        Self {
            coefs,
            sample_rate: DEFAULT_SR,
            ..Default::default()
        }
    }

    pub fn coefs(&self) -> &BiquadCoefsBank<F, Size> {
        &self.coefs
    }

    pub fn set_coefs(&mut self, coefs: BiquadCoefsBank<F, Size>) {
        self.coefs = coefs;
    }
}

impl<F, Size> AudioNode for BiquadBank<F, Size>
where
    F: Realx<Size>,
    Size: ArrayLength + Sync + Send,
{
    const ID: u64 = 15;
    type Inputs = typenum::U1;
    type Outputs = Size;

    fn reset(&mut self) {
        self.x1 = F::zero();
        self.x2 = F::zero();
        self.y1 = F::zero();
        self.y2 = F::zero();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let x0 = F::from_f32(input[0]);
        let y0 = self.coefs.b0 * x0 + self.coefs.b1 * self.x1 + self.coefs.b2 * self.x2
            - self.coefs.a1 * self.y1
            - self.coefs.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x0;
        self.y2 = self.y1;
        self.y1 = y0;
        y0.to_frame()
    }

    fn set(&mut self, setting: Setting) {
        if let Parameter::BiquadBank(index, a1, a2, b0, b1, b2) = setting.parameter() {
            let mut coefs = self.coefs;
            coefs.a1.set(*index, *a1);
            coefs.a2.set(*index, *a2);
            coefs.b0.set(*index, *b0);
            coefs.b1.set(*index, *b1);
            coefs.b2.set(*index, *b2);
        }
    }

    //fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
    //    let mut output = SignalFrame::new(self.outputs());
    //    output.set(
    //        0,
    //        input.at(0).filter(0.0, |r| {
    //            r * self.coefs().response(frequency / self.sample_rate)
    //        }),
    //    );
    //    output
    //}
}
