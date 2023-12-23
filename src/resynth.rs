//! Frequency domain resynthesis. WIP.

use super::audionode::*;
use super::*;
use num_complex::Complex32;

pub struct FftWindow<T, I, O>
where
    T: Float,
    I: Size<T>,
    O: Size<T>,
{
    /// Window length. Equals the length of each input and output channel vector.
    length: usize,
    /// Input samples for each input channel.
    input: Vec<Vec<f32>>,
    /// Input samples for each input channel in frequency domain.
    input_fft: Vec<Vec<Complex32>>,
    /// Output samples for each output channel in frequency domain.
    output_fft: Vec<Vec<Complex32>>,
    /// Output samples for each output channel.
    output: Vec<Vec<f32>>,
    /// Sample rate for convenience.
    sample_rate: f32,
    _marker: std::marker::PhantomData<(T, I, O)>,
}

impl<T, I, O> FftWindow<T, I, O>
where
    T: Float,
    I: Size<T>,
    O: Size<T>,
{
    /// Number of input channels.
    #[inline]
    pub fn inputs(&self) -> usize {
        self.input.len()
    }

    /// Number of output channels.
    #[inline]
    pub fn outputs(&self) -> usize {
        self.output.len()
    }

    /// FFT window length. Must be divisible by four.
    #[inline]
    pub fn length(&self) -> usize {
        self.length
    }

    /// Number of FFT bins. Equals the length of each frequency domain vector.
    #[inline]
    pub fn bins(&self) -> usize {
        (self.length() >> 1) + 1
    }

    /// Get input value at bin `i` of `channel`.
    #[inline]
    pub fn at(&self, channel: usize, i: usize) -> Complex32 {
        self.input_fft[channel][i]
    }

    /// Set output value for bin `i` of `channel`.
    #[inline]
    pub fn set(&mut self, channel: usize, i: usize, value: Complex32) {
        self.output_fft[channel][i] = value;
    }

    /// Return frequency associated with bin `i`.
    #[inline]
    pub fn frequency(&self, i: usize) -> f32 {
        self.sample_rate / self.length() as f32 * i as f32
    }
}
