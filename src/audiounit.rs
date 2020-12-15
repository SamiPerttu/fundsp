use super::audionode::*;
use super::math::*;
use super::signal::*;
use super::*;
use num_complex::Complex64;

/// AudioUnit is an audio processor with an object safe interface.
/// Once constructed, it has a fixed number of inputs and outputs that can be queried.
pub trait AudioUnit {
    /// Resets the input state of the unit to an initial state where it has not processed any data.
    /// In other words, resets time to zero.
    fn reset(&mut self, _sample_rate: Option<f64>) {}

    /// Process one 32-bit sample. The length of `input` and `output` must be equal to `inputs` and `outputs`,
    /// respectively.
    fn tick32(&mut self, input: &[f32], output: &mut [f32]);

    /// Process one 64-bit sample. The length of `input` and `output` must be equal to `inputs` and `outputs`,
    /// respectively.
    fn tick64(&mut self, input: &[f64], output: &mut [f64]);

    /// Computes a block of 32-bit data, reading from input buffers and writing to output buffers.
    /// Buffers are supplied as slices. All buffers must have the same size.
    /// The caller decides the length of the block.
    /// Using the same length for many consecutive calls is presumed to be efficient.
    /// The number of input and output buffers must be equal to `inputs` and `outputs`, respectively.
    fn process32(&mut self, input: &[&[f32]], output: &mut [&mut [f32]]);

    /// Computes a block of 32-bit data, reading from input buffers and writing to output buffers.
    /// Buffers are supplied as slices. All buffers must have the same size.
    /// The caller decides the length of the block.
    /// Using the same length for many consecutive calls is presumed to be efficient.
    /// The number of input and output buffers must be equal to `inputs` and `outputs`, respectively.
    fn process64(&mut self, input: &[&[f64]], output: &mut [&mut [f64]]);

    /// Number of inputs to this unit. Size of the input argument in compute().
    /// This should be fixed after construction.
    fn inputs(&self) -> usize;

    /// Number of outputs from this unit. Size of the output argument in compute().
    /// This should be fixed after construction.
    fn outputs(&self) -> usize;

    /// Propagate constants, latencies and frequency responses at `frequency`. Return output signal.
    /// Default implementation marks all outputs unknown.
    fn propagate(&self, _input: &SignalFrame, _frequency: f64) -> SignalFrame {
        new_signal_frame()
    }

    /// Evaluate frequency response at `output`. Any linear response can be composed.
    /// Return `None` if there is no response or it could not be calculated.
    fn response(&self, output: usize, frequency: f64) -> Option<Complex64> {
        assert!(output < self.outputs());
        let mut input = new_signal_frame();
        for i in 0..self.inputs() {
            input[i] = Signal::Response(Complex64::new(1.0, 0.0), 0.0);
        }
        let response = self.propagate(&input, frequency);
        match response[output] {
            Signal::Response(rx, _) => Some(rx),
            _ => None,
        }
    }

    /// Evaluate frequency response at `output` in dB. Any linear response can be composed.
    /// Return `None` if there is no response or it could not be calculated.
    fn response_db(&self, output: usize, frequency: f64) -> Option<f64> {
        assert!(output < self.outputs());
        self.response(output, frequency).map(|r| amp_db(r.norm()))
    }

    /// Causal latency at `output`, in (fractional) samples.
    /// After a reset, we can discard this many samples from the output to avoid incurring a pre-delay.
    /// The latency can depend on the sample rate and is allowed to change after `reset`.
    fn latency(&self, output: usize) -> Option<f64> {
        assert!(output < self.outputs());
        let mut input = new_signal_frame();
        for i in 0..self.inputs() {
            input[i] = Signal::Latency(0.0);
        }
        // The frequency argument can be anything as there are no responses to propagate, only latencies.
        let response = self.propagate(&input, 1.0);
        match response[output] {
            Signal::Latency(latency) => Some(latency),
            _ => None,
        }
    }
}

/// Adapts an AudioNode into an AudioUnit.
pub struct AnUnit<X: AudioNode>(pub X);

impl<X: AudioNode<Sample = f64>> AudioUnit for AnUnit<X>
where
    X::Inputs: Size<f64>,
    X::Outputs: Size<f64>,
{
    fn reset(&mut self, sample_rate: Option<f64>) {
        self.0.reset(sample_rate);
    }
    fn tick32(&mut self, input: &[f32], output: &mut [f32]) {
        assert!(input.len() == self.inputs());
        assert!(output.len() == self.outputs());
        let out = self.0.tick(&Frame::generate(|i| convert(input[i])));
        for i in 0..self.outputs() {
            output[i] = convert(out[i]);
        }
    }
    fn tick64(&mut self, input: &[f64], output: &mut [f64]) {
        assert!(input.len() == self.inputs());
        assert!(output.len() == self.outputs());
        output.copy_from_slice(self.0.tick(&Frame::from_slice(input)).as_slice());
    }
    fn process32(&mut self, input: &[&[f32]], output: &mut [&mut [f32]]) {
        assert!(input.len() == self.inputs());
        assert!(output.len() == self.outputs());
        let buffer_size = if input.len() > 0 {
            input[0].len()
        } else if output.len() > 0 {
            output[0].len()
        } else {
            // TODO. This will be replaced with an explicit length argument.
            return;
        };
        assert!(input.iter().all(|x| x.len() == buffer_size));
        assert!(output.iter().all(|x| x.len() == buffer_size));
        for i in 0..buffer_size {
            let result = self.0.tick(&Frame::generate(|j| convert(input[j][i])));
            for (j, &x) in result.iter().enumerate() {
                output[j][i] = convert(x);
            }
        }
    }
    fn process64(&mut self, input: &[&[f64]], output: &mut [&mut [f64]]) {
        assert!(input.len() == self.inputs());
        assert!(output.len() == self.outputs());
        let buffer_size = if input.len() > 0 {
            input[0].len()
        } else if output.len() > 0 {
            output[0].len()
        } else {
            // TODO. This will be replaced with an explicit length argument.
            return;
        };
        assert!(input.iter().all(|x| x.len() == buffer_size));
        assert!(output.iter().all(|x| x.len() == buffer_size));
        for i in 0..buffer_size {
            let result = self.0.tick(&Frame::generate(|j| input[j][i]));
            for (j, &x) in result.iter().enumerate() {
                output[j][i] = x;
            }
        }
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
