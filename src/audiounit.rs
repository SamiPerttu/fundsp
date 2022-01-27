use super::audionode::*;
use super::combinator::*;
use super::signal::*;

/// AudioUnit is an audio processor with an object safe interface.
/// Once constructed, it has a fixed number of inputs and outputs.
pub trait AudioUnit {
    /// Reset the input state of the unit to an initial state where it has not processed any data.
    /// In other words, reset time to zero.
    fn reset(&mut self, _sample_rate: Option<f64>) {}

    /// Process one double precision sample.
    /// The length of `input` and `output` must be equal to `inputs` and `outputs`, respectively.
    fn tick64(&mut self, input: &[f64], output: &mut [f64]);

    /// Process up to 64 (MAX_BUFFER_SIZE) double precision samples.
    /// Buffers are supplied as slices. All buffers must have room for at least `size` samples.
    /// The number of input and output buffers must be equal to `inputs` and `outputs`, respectively.
    fn process64(&mut self, size: usize, input: &[&[f64]], output: &mut [&mut [f64]]);

    /// Number of inputs to this unit. Size of the input argument in `compute`.
    /// This should be fixed after construction.
    fn inputs(&self) -> usize;

    /// Number of outputs from this unit. Size of the output argument in `compute`.
    /// This should be fixed after construction.
    fn outputs(&self) -> usize;

    /// Propagate constants, latencies and frequency responses at `frequency`.
    /// Return output signal.
    fn propagate(&self, _input: &SignalFrame, _frequency: f64) -> SignalFrame;

    // End of interface. No need to override the following.
}

impl<X: AudioNode<Sample = f64>> AudioUnit for An<X>
where
    X::Inputs: Size<f64>,
    X::Outputs: Size<f64>,
{
    fn reset(&mut self, sample_rate: Option<f64>) {
        self.0.reset(sample_rate);
    }
    fn tick64(&mut self, input: &[f64], output: &mut [f64]) {
        debug_assert!(input.len() == self.inputs());
        debug_assert!(output.len() == self.outputs());
        output.copy_from_slice(self.0.tick(Frame::from_slice(input)).as_slice());
    }
    fn process64(&mut self, size: usize, input: &[&[f64]], output: &mut [&mut [f64]]) {
        self.0.process(size, input, output);
    }
    fn inputs(&self) -> usize {
        self.0.inputs()
    }
    fn outputs(&self) -> usize {
        self.0.outputs()
    }
    fn propagate(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.0.propagate(input, frequency)
    }
}

/// AudioUnit wrapper.
pub struct Au(pub Box<dyn AudioUnit>);

impl core::ops::Deref for Au {
    type Target = Box<dyn AudioUnit>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for Au {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
