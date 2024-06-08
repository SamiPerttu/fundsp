//! Frequency domain resynthesis.

// For more information on this technique, see
// "Fourier analysis and reconstruction of audio signals" at
// http://msp.ucsd.edu/techniques/v0.11/book-html/node172.html

use super::audionode::*;
use super::math::*;
use super::signal::*;
use super::*;
use num_complex::Complex32;
extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

/// Number of overlapping FFT windows.
const WINDOWS: usize = 4;

/// A single FFT window. Contains input and output
/// values in the frequency domain.
#[derive(Clone)]
pub struct FftWindow {
    /// Window length. Must be a power of two and at least four.
    /// Equals the length of each input and output channel vector.
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
    /// Current index into input and output vectors.
    index: usize,
    /// Total number of processed samples.
    samples: u64,
}

impl FftWindow {
    /// Number of input channels.
    #[inline]
    pub fn inputs(&self) -> usize {
        self.input.len()
    }

    /// Extend positive frequency values to negative frequencies to keep the inverse FFT result
    /// real only.
    pub(crate) fn extend_values(&mut self) {
        for channel in 0..self.outputs() {
            for i in self.length() / 2 + 1..self.length() {
                self.output_fft[channel][i] = self.output_fft[channel][self.length() - i].conj();
            }
        }
    }

    /// Number of output channels.
    #[inline]
    pub fn outputs(&self) -> usize {
        self.output.len()
    }

    /// Sample rate in Hz.
    #[inline]
    pub fn sample_rate(&self) -> f64 {
        self.sample_rate as f64
    }

    /// Processing latency of the resynthesizer in seconds.
    /// Equal to one window length.
    #[inline]
    pub fn latency(&self) -> f64 {
        self.length as f64 / self.sample_rate as f64
    }

    /// Time in seconds at the center (peak) of the window.
    /// For time varying effects.
    /// The window is Hann squared shaped.
    /// Latency is subtracted from stream time.
    /// Add `latency()` to this if you need stream time.
    #[inline]
    pub fn time(&self) -> f64 {
        (self.samples - (self.length as u64 >> 1)) as f64 / self.sample_rate as f64
    }

    /// Time in seconds at sample `i` of the window.
    /// For time varying effects.
    /// There are `length()` samples in total.
    /// Latency is subtracted from stream time.
    /// Add `latency()` to this if you need stream time.
    #[inline]
    pub fn time_at(&self, i: usize) -> f64 {
        (self.samples - self.length as u64 + i as u64) as f64 / self.sample_rate as f64
    }

    /// Get forward vectors for forward FFT.
    #[inline]
    pub(crate) fn forward_vectors(&mut self, channel: usize) -> (&Vec<f32>, &mut Vec<Complex32>) {
        (&self.input[channel], &mut self.input_fft[channel])
    }

    /// Get inverse vectors for inverse FFT.
    #[inline]
    pub(crate) fn inverse_vectors(&mut self, channel: usize) -> (&Vec<Complex32>, &mut Vec<f32>) {
        (&self.output_fft[channel], &mut self.output[channel])
    }

    /// FFT window length. This is a power of two and at least four.
    #[inline]
    pub fn length(&self) -> usize {
        self.length
    }

    /// Number of FFT bins.
    /// Equals the length of each frequency domain vector.
    /// The lowest bin is zero and the highest bin (corresponding to the Nyquist frequency) is `bins() - 1`.
    #[inline]
    pub fn bins(&self) -> usize {
        (self.length() >> 1) + 1
    }

    /// Return frequency (in Hz) associated with bin `i`.
    #[inline]
    pub fn frequency(&self, i: usize) -> f32 {
        self.sample_rate / self.length() as f32 * i as f32
    }

    /// Get input value at bin `i` of `channel`.
    #[inline]
    pub fn at(&self, channel: usize, i: usize) -> Complex32 {
        self.input_fft[channel][i]
    }

    /// Return output value for bin `i` of `channel`.
    #[inline]
    pub fn at_output(&self, channel: usize, i: usize) -> Complex32 {
        self.output_fft[channel][i]
    }

    /// Set output value for bin `i` of `channel`.
    #[inline]
    pub fn set(&mut self, channel: usize, i: usize, value: Complex32) {
        self.output_fft[channel][i] = value;
    }

    /// Create new window.
    pub fn new(length: usize, index: usize, inputs: usize, outputs: usize) -> Self {
        let mut window = Self {
            length,
            input: vec![vec!(0.0; length); inputs],
            input_fft: Vec::new(),
            output_fft: Vec::new(),
            output: vec![vec!(0.0; length); outputs],
            sample_rate: DEFAULT_SR as f32,
            index,
            samples: 0,
        };
        window
            .input_fft
            .resize(inputs, vec![Complex32::default(); window.bins()]);
        window
            .output_fft
            .resize(outputs, vec![Complex32::default(); length]);
        window
    }

    /// Set the sample rate.
    pub(crate) fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    /// Write input for current index.
    #[inline]
    pub(crate) fn write<T: Float, N: Size<T>>(&mut self, input: &Frame<T, N>, window_value: f32) {
        for (channel, item) in input.iter().enumerate() {
            self.input[channel][self.index] = item.to_f32() * window_value;
        }
    }

    /// Read output for current index.
    #[inline]
    pub(crate) fn read<T: Float, N: Size<T>>(&self, window_value: f32) -> Frame<T, N> {
        Frame::generate(|channel| convert(self.output[channel][self.index] * window_value))
    }

    /// Set FFT outputs to all zeros.
    pub fn clear_output(&mut self) {
        for i in 0..self.outputs() {
            self.output_fft[i].fill(Complex32::default());
        }
    }

    /// Current read and write index.
    #[inline]
    pub(crate) fn index(&self) -> usize {
        self.index
    }

    /// Reset the window to an empty state.
    pub(crate) fn reset(&mut self, start_index: usize) {
        self.samples = 0;
        self.index = start_index;
        for channel in 0..self.inputs() {
            self.input[channel].fill(0.0);
        }
        for channel in 0..self.outputs() {
            self.output[channel].fill(0.0);
        }
    }

    /// Advance index to the next sample.
    #[inline]
    pub(crate) fn advance(&mut self) {
        self.samples += 1;
        self.index = (self.index + 1) & (self.length - 1);
    }

    /// Return whether we should do FFT processing right now.
    #[inline]
    pub(crate) fn is_fft_time(&self) -> bool {
        self.index == 0 && self.samples >= self.length as u64
    }
}

/// Frequency domain resynthesizer. Processes windows of input samples with an overlap of four.
/// Each window is Fourier transformed and then processed into output spectra
/// by the user supplied processing function.
/// The output windows are finally inverse transformed into the outputs.
/// The latency is equal to the window length.
/// If any output is a copy of an input, then the input will be reconstructed exactly once
/// the windows are all overlapping, which happens one window length beyond latency.
#[derive(Clone)]
pub struct Resynth<I, O, F>
where
    I: Size<f32>,
    O: Size<f32>,
    F: FnMut(&mut FftWindow) + Clone + Send + Sync,
{
    _marker: core::marker::PhantomData<(I, O)>,
    /// FFT windows.
    window: [FftWindow; WINDOWS],
    /// Window length.
    window_length: usize,
    /// Hann window function.
    window_function: Vec<f32>,
    /// Processing function.
    processing: F,
    /// Sample rate.
    sample_rate: f64,
    /// Temporary vector for FFT.
    scratch: Vec<Complex32>,
    /// Number of processed samples.
    samples: u64,
    /// Normalizing term for FFT and overlap-add.
    z: f32,
}

impl<I, O, F> Resynth<I, O, F>
where
    I: Size<f32>,
    O: Size<f32>,
    F: FnMut(&mut FftWindow) + Clone + Send + Sync,
{
    /// Number of FFT bins. Equals the length of each frequency domain vector in FFT windows.
    #[inline]
    pub fn bins(&self) -> usize {
        (self.window_length >> 1) + 1
    }

    /// Window length in samples.
    #[inline]
    pub fn window_length(&self) -> usize {
        self.window_length
    }

    /// Create new resynthesizer. Window length must be a power of two between 4 and 32768.
    pub fn new(window_length: usize, processing: F) -> Self {
        assert!(window_length >= 4 && window_length.is_power_of_two());

        let mut window_function = Vec::with_capacity(window_length);

        for i in 0..window_length {
            let hann = 0.5
                + 0.5
                    * cos((i as i32 - (window_length >> 1) as i32) as f32 * f32::TAU
                        / window_length as f32);
            window_function.push(hann);
        }

        let window = [
            FftWindow::new(window_length, 0, I::USIZE, O::USIZE),
            FftWindow::new(window_length, window_length >> 2, I::USIZE, O::USIZE),
            FftWindow::new(window_length, window_length >> 1, I::USIZE, O::USIZE),
            FftWindow::new(
                window_length,
                (window_length >> 1) + (window_length >> 2),
                I::USIZE,
                O::USIZE,
            ),
        ];

        let scratch = vec![Complex32::default(); window_length];

        Self {
            _marker: core::marker::PhantomData,
            window,
            window_length,
            window_function,
            processing,
            sample_rate: DEFAULT_SR,
            scratch,
            samples: 0,
            z: 2.0 / 3.0,
        }
    }
}

impl<I, O, F> AudioNode for Resynth<I, O, F>
where
    I: Size<f32>,
    O: Size<f32>,
    F: FnMut(&mut FftWindow) + Clone + Send + Sync,
{
    const ID: u64 = 80;
    type Inputs = I;
    type Outputs = O;

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
        for i in 0..WINDOWS {
            self.window[i].set_sample_rate(sample_rate as f32);
        }
    }

    fn reset(&mut self) {
        self.samples = 0;
        for i in 0..WINDOWS {
            self.window[i].reset(i * (self.window_length >> 2));
        }
    }

    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let mut output = Frame::default();

        for i in 0..WINDOWS {
            let window_value = self.window_function[self.window[i].index()];
            self.window[i].write(input, window_value);
            output += self.window[i].read(window_value * self.z);
            self.window[i].advance();
        }

        self.samples += 1;

        if self.samples & ((self.window_length as u64 >> 2) - 1) == 0 {
            for i in 0..WINDOWS {
                if self.window[i].is_fft_time() {
                    for channel in 0..I::USIZE {
                        let (input, input_fft) = self.window[i].forward_vectors(channel);
                        super::fft::real_fft(input, input_fft);
                    }

                    self.window[i].clear_output();

                    (self.processing)(&mut self.window[i]);

                    self.window[i].extend_values();

                    for channel in 0..O::USIZE {
                        let (output_fft, output_scalar) = self.window[i].inverse_vectors(channel);
                        super::fft::inverse_fft(output_fft, &mut self.scratch);
                        for (x, y) in output_scalar.iter_mut().zip(self.scratch.iter()) {
                            *x = y.re;
                        }
                    }
                }
            }
        }
        output
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Arbitrary(self.window_length as f64).route(input, self.outputs())
    }
}
