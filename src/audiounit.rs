//! AudioUnit abstraction. WIP.

use super::audionode::*;
use super::combinator::*;
use super::math::*;
use super::signal::*;
use num_complex::Complex64;

/// AudioUnit64 is a double precision audio processor with an object safe interface.
/// Once constructed, it has a fixed number of inputs and outputs.
pub trait AudioUnit64 {
    /// Reset the input state of the unit to an initial state where it has not processed any data.
    /// In other words, reset time to zero.
    fn reset(&mut self, _sample_rate: Option<f64>);

    /// Process one double precision sample.
    /// The length of `input` and `output` must be equal to `inputs` and `outputs`, respectively.
    fn tick(&mut self, input: &[f64], output: &mut [f64]);

    /// Process up to 64 (MAX_BUFFER_SIZE) double precision samples.
    /// Buffers are supplied as slices. All buffers must have room for at least `size` samples.
    /// The number of input and output buffers must be equal to `inputs` and `outputs`, respectively.
    fn process(&mut self, size: usize, input: &[&[f64]], output: &mut [&mut [f64]]);

    /// Number of inputs to this unit. Size of the input argument in `tick` and `process`.
    /// This should be fixed after construction.
    fn inputs(&self) -> usize;

    /// Number of outputs from this unit. Size of the output argument in `tick` and `process`.
    /// This should be fixed after construction.
    fn outputs(&self) -> usize;

    /// Route constants, latencies and frequency responses at `frequency` Hz
    /// from inputs to outputs. Return output signal.
    fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame;

    // End of interface. No need to override the following.

    /// Evaluate frequency response of `output` at `frequency` Hz.
    /// Any linear response can be composed.
    /// Return `None` if there is no response or it could not be calculated.
    fn response(&self, output: usize, frequency: f64) -> Option<Complex64> {
        assert!(output < self.outputs());
        let mut input = new_signal_frame(self.inputs());
        for i in 0..self.inputs() {
            input[i] = Signal::Response(Complex64::new(1.0, 0.0), 0.0);
        }
        let response = self.route(&input, frequency);
        match response[output] {
            Signal::Response(rx, _) => Some(rx),
            _ => None,
        }
    }

    /// Evaluate frequency response of `output` in dB at `frequency Hz`.
    /// Any linear response can be composed.
    /// Return `None` if there is no response or it could not be calculated.
    fn response_db(&self, output: usize, frequency: f64) -> Option<f64> {
        assert!(output < self.outputs());
        self.response(output, frequency).map(|r| amp_db(r.norm()))
    }

    /// Causal latency in (fractional) samples.
    /// After a reset, we can discard this many samples from the output to avoid incurring a pre-delay.
    /// The latency can depend on the sample rate and is allowed to change after `reset`.
    fn latency(&self) -> Option<f64> {
        if self.outputs() == 0 {
            return None;
        }
        let mut input = new_signal_frame(self.inputs());
        for i in 0..self.inputs() {
            input[i] = Signal::Latency(0.0);
        }
        // The frequency argument can be anything as there are no responses to propagate,
        // only latencies. Latencies are never promoted to responses during signal routing.
        let response = self.route(&input, 1.0);
        // Return the minimum latency.
        let mut result: Option<f64> = None;
        for output in 0..self.outputs() {
            match (result, response[output]) {
                (None, Signal::Latency(x)) => result = Some(x),
                (Some(r), Signal::Latency(x)) => result = Some(r.min(x)),
                _ => (),
            }
        }
        result
    }

    /// Retrieve the next mono sample from a generator.
    /// The node must have no inputs and 1 output.
    #[inline]
    fn get_mono(&mut self) -> f64 {
        assert!(self.inputs() == 0 && self.outputs() == 1);
        let mut output = [0.0];
        self.tick(&[], &mut output);
        output[0]
    }

    /// Retrieve the next stereo sample (left, right) from a generator.
    /// The node must have no inputs and 1 or 2 outputs.
    /// If there is just one output, duplicate it.
    #[inline]
    fn get_stereo(&mut self) -> (f64, f64) {
        assert!(self.inputs() == 0);
        match self.outputs() {
            1 => {
                let mut output = [0.0];
                self.tick(&[], &mut output);
                (output[0], output[0])
            }
            2 => {
                let mut output = [0.0, 0.0];
                self.tick(&[], &mut output);
                (output[0], output[1])
            }
            _ => panic!("AudioUnit64::get_stereo(): Unit must have 1 or 2 outputs"),
        }
    }

    /// Filter the next mono sample `x`.
    /// The node must have exactly 1 input and 1 output.
    #[inline]
    fn filter_mono(&mut self, x: f64) -> f64 {
        assert!(self.inputs() == 1 && self.outputs() == 1);
        let mut output = [0.0];
        self.tick(&[x], &mut output);
        output[0]
    }

    /// Filter the next stereo sample `(x, y)`.
    /// The node must have exactly 2 inputs and 2 outputs.
    #[inline]
    fn filter_stereo(&mut self, x: f64, y: f64) -> (f64, f64) {
        assert!(self.inputs() == 2 && self.outputs() == 2);
        let mut output = [0.0, 0.0];
        self.tick(&[x, y], &mut output);
        (output[0], output[1])
    }
}

/// AudioUnit32 is a single precision audio processor with an object safe interface.
/// Once constructed, it has a fixed number of inputs and outputs.
pub trait AudioUnit32 {
    /// Reset the input state of the unit to an initial state where it has not processed any data.
    /// In other words, reset time to zero.
    fn reset(&mut self, _sample_rate: Option<f64>);

    /// Process one double precision sample.
    /// The length of `input` and `output` must be equal to `inputs` and `outputs`, respectively.
    fn tick(&mut self, input: &[f32], output: &mut [f32]);

    /// Process up to 64 (MAX_BUFFER_SIZE) double precision samples.
    /// Buffers are supplied as slices. All buffers must have room for at least `size` samples.
    /// The number of input and output buffers must be equal to `inputs` and `outputs`, respectively.
    fn process(&mut self, size: usize, input: &[&[f32]], output: &mut [&mut [f32]]);

    /// Number of inputs to this unit. Size of the input argument in `tick` and `process`.
    /// This should be fixed after construction.
    fn inputs(&self) -> usize;

    /// Number of outputs from this unit. Size of the output argument in `tick` and `process`.
    /// This should be fixed after construction.
    fn outputs(&self) -> usize;

    /// Route constants, latencies and frequency responses at `frequency` Hz
    /// from inputs to outputs. Return output signal.
    fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame;

    // End of interface. No need to override the following.
}

impl<X: AudioNode<Sample = f64>> AudioUnit64 for An<X>
where
    X::Inputs: Size<f64>,
    X::Outputs: Size<f64>,
{
    fn reset(&mut self, sample_rate: Option<f64>) {
        self.0.reset(sample_rate);
    }
    fn tick(&mut self, input: &[f64], output: &mut [f64]) {
        debug_assert!(input.len() == self.inputs());
        debug_assert!(output.len() == self.outputs());
        output.copy_from_slice(self.0.tick(Frame::from_slice(input)).as_slice());
    }
    fn process(&mut self, size: usize, input: &[&[f64]], output: &mut [&mut [f64]]) {
        self.0.process(size, input, output);
    }
    fn inputs(&self) -> usize {
        self.0.inputs()
    }
    fn outputs(&self) -> usize {
        self.0.outputs()
    }
    fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.0.route(input, frequency)
    }
}

impl<X: AudioNode<Sample = f32>> AudioUnit32 for An<X>
where
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
{
    fn reset(&mut self, sample_rate: Option<f64>) {
        self.0.reset(sample_rate);
    }
    fn tick(&mut self, input: &[f32], output: &mut [f32]) {
        debug_assert!(input.len() == self.inputs());
        debug_assert!(output.len() == self.outputs());
        output.copy_from_slice(self.0.tick(Frame::from_slice(input)).as_slice());
    }
    fn process(&mut self, size: usize, input: &[&[f32]], output: &mut [&mut [f32]]) {
        self.0.process(size, input, output);
    }
    fn inputs(&self) -> usize {
        self.0.inputs()
    }
    fn outputs(&self) -> usize {
        self.0.outputs()
    }
    fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.0.route(input, frequency)
    }
}

/*
/// AudioUnit64 wrapper.
pub struct Au64(pub Box<dyn AudioUnit64>);

impl core::ops::Deref for Au64 {
    type Target = Box<dyn AudioUnit64>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for Au64 {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn au64<X: 'static + AudioUnit64>(x: X) -> Au64 {
    Au64(Box::new(x))
}
*/
