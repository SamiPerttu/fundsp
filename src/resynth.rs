//! Frequency domain resynthesis.

// For more information on this technique, see
// "Fourier analysis and reconstruction of audio signals" at
// http://msp.ucsd.edu/techniques/v0.11/book-html/node172.html

use super::audionode::*;
use super::math::*;
use super::signal::*;
use super::*;
use num_complex::Complex32;
use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};
use std::sync::Arc;

/// Number of overlapping FFT windows.
const WINDOWS: usize = 4;

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

    /// Number of output channels.
    #[inline]
    pub fn outputs(&self) -> usize {
        self.output.len()
    }

    /// Get forward vectors for forward FFT.
    pub fn forward_vectors(&mut self, channel: usize) -> (&mut Vec<f32>, &mut Vec<Complex32>) {
        (&mut self.input[channel], &mut self.input_fft[channel])
    }

    /// Get inverse vectors for inverse FFT.
    pub fn inverse_vectors(&mut self, channel: usize) -> (&mut Vec<Complex32>, &mut Vec<f32>) {
        (&mut self.output_fft[channel], &mut self.output[channel])
    }

    /// FFT window length. This is a power of two and at least four.
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
            .resize(outputs, vec![Complex32::default(); window.bins()]);
        window
    }

    /// Set the sample rate.
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    /// Write input for current index.
    #[inline]
    pub fn write<T: Float, N: Size<T>>(&mut self, input: &Frame<T, N>, window_value: f32) {
        for (channel, item) in input.iter().enumerate() {
            self.input[channel][self.index] = item.to_f32() * window_value;
        }
    }

    /// Read output for current index.
    #[inline]
    pub fn read<T: Float, N: Size<T>>(&self, window_value: f32) -> Frame<T, N> {
        Frame::generate(|channel| convert(self.output[channel][self.index] * window_value))
    }

    /// Set outputs to all zeros.
    pub fn clear_output(&mut self) {
        for i in 0..self.outputs() {
            self.output_fft[i].fill(Complex32::default());
        }
    }

    /// Current read and write index.
    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }

    /// Reset the window to an empty state.
    pub fn reset(&mut self, start_index: usize) {
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
    pub fn advance(&mut self) {
        self.samples += 1;
        self.index = (self.index + 1) & (self.length - 1);
    }

    /// Return whether we should do FFT processing right now.
    #[inline]
    pub fn is_fft_time(&self) -> bool {
        self.index == 0 && self.samples >= self.length as u64
    }
}

/// Frequency domain resynthesizer. Processes windows of input samples with an overlap of four.
/// Each window is Fourier transformed and then processed into output spectra by the processing function.
/// The output windows are then inverse transformed into the outputs.
/// The latency is equal to the window length.
/// If any output is a copy of an input, then the input will be reconstructed exactly once
/// the windows are all overlapping, which takes one window length.
#[derive(Clone)]
pub struct Resynth<I, O, T, F>
where
    I: Size<T>,
    O: Size<T>,
    T: Float,
    F: FnMut(f32, &mut FftWindow) + Clone + Send + Sync,
{
    _marker: std::marker::PhantomData<(T, I, O)>,
    /// FFT windows.
    window: [FftWindow; WINDOWS],
    /// Window length.
    window_length: usize,
    /// Hann window function.
    window_function: Vec<f32>,
    /// Processing function is a function of (time, window).
    processing: F,
    /// Sample rate.
    sample_rate: f64,
    /// Forward transform.
    forward: Arc<dyn RealToComplex<f32>>,
    /// Inverse transform.
    inverse: Arc<dyn ComplexToReal<f32>>,
    /// Temporary vector for FFT.
    scratch: Vec<Complex32>,
    /// Number of processed samples.
    samples: u64,
}

impl<I, O, T, F> Resynth<I, O, T, F>
where
    I: Size<T>,
    O: Size<T>,
    T: Float,
    F: FnMut(f32, &mut FftWindow) + Clone + Send + Sync,
{
    /// Create new resynthesizer.
    pub fn new(window_length: usize, processing: F) -> Self {
        assert!(window_length >= 4 && window_length.is_power_of_two());

        let mut planner = RealFftPlanner::<f32>::new();
        let forward = planner.plan_fft_forward(window_length);
        let inverse = planner.plan_fft_inverse(window_length);

        let mut window_function = Vec::new();

        for i in 0..window_length {
            let hann = 0.5
                + 0.5
                    * cos((i as i32 - (window_length >> 1) as i32) as f32 * TAU as f32
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

        let scratch =
            vec![Complex32::default(); max(forward.get_scratch_len(), inverse.get_scratch_len())];

        Self {
            _marker: std::marker::PhantomData,
            window,
            window_length,
            window_function,
            processing,
            sample_rate: DEFAULT_SR,
            forward,
            inverse,
            scratch,
            samples: 0,
        }
    }
}

impl<I, O, T, F> AudioNode for Resynth<I, O, T, F>
where
    I: Size<T>,
    O: Size<T>,
    T: Float,
    F: FnMut(f32, &mut FftWindow) + Clone + Send + Sync,
{
    const ID: u64 = 80;
    type Sample = T;
    type Inputs = I;
    type Outputs = O;
    type Setting = ();

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

    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let mut output = Frame::default();

        let z = 2.0 / (3.0 * self.window_length as f32);

        for i in 0..WINDOWS {
            let window_value = self.window_function[self.window[i].index()];
            self.window[i].write(input, window_value);
            output += self.window[i].read(window_value * z);
            self.window[i].advance();
        }

        self.samples += 1;

        if self.samples & ((self.window_length as u64 >> 2) - 1) == 0 {
            for i in 0..WINDOWS {
                if self.window[i].is_fft_time() {
                    for channel in 0..I::USIZE {
                        let (input, input_fft) = self.window[i].forward_vectors(channel);
                        self.forward
                            .process_with_scratch(input, input_fft, &mut self.scratch)
                            .expect("Internal error");
                    }

                    self.window[i].clear_output();
                    (self.processing)(
                        (self.samples as f64 / self.sample_rate) as f32,
                        &mut self.window[i],
                    );

                    for channel in 0..O::USIZE {
                        let (output_fft, output) = self.window[i].inverse_vectors(channel);
                        self.inverse
                            .process_with_scratch(output_fft, output, &mut self.scratch)
                            .expect("Internal error");
                    }
                }
            }
        }
        output
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Arbitrary(self.window_length as f64).propagate(input, self.outputs())
    }
}
