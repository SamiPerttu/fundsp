use super::audionode::*;
use super::*;

/// AudioUnit processes audio data block by block at a synchronous rate.
/// Once constructed, it has a fixed number of inputs and outputs that can be queried.
pub trait AudioUnit {
    /// Resets the input state of the unit to an initial state where it has not processed any data.
    /// In other words, resets time to zero.
    fn reset(&mut self, _sample_rate: Option<f64>) {}

    /// Computes a block of data, reading from input buffers and writing to output buffers.
    /// Buffers are supplied as slices. All buffers must have the same size.
    /// The caller decides the length of the block.
    /// Using the same length for many consecutive calls is presumed to be efficient.
    /// The number of input and output buffers must correspond to inputs() and outputs(), respectively.
    fn process(&mut self, input: &[&[f32]], output: &mut [&mut [f32]]);

    /// Number of inputs to this unit. Size of the input argument in compute().
    /// This should be fixed after construction.
    fn inputs(&self) -> usize;

    /// Number of outputs from this unit. Size of the output argument in compute().
    /// This should be fixed after construction.
    fn outputs(&self) -> usize;

    /// Causal latency from input to output, in (fractional) samples.
    /// After a reset, we can discard this many samples from the output to avoid incurring a pre-delay.
    /// This applies only to components that have both inputs and outputs. Others should return None.
    /// The latency can depend on the sample rate and is allowed to change after a reset.
    fn latency(&self) -> Option<f64> {
        // Default latency is zero.
        if self.inputs() > 0 && self.outputs() > 0 {
            Some(0.0)
        } else {
            None
        }
    }
}

/// Adapts an AudioNode into an AudioUnit.
pub struct AnUnit<X: AudioNode>(pub X);

impl<X: AudioNode> AudioUnit for AnUnit<X> {
    fn reset(&mut self, sample_rate: Option<f64>) {
        self.0.reset(sample_rate);
    }
    fn process(&mut self, input: &[&[f32]], output: &mut [&mut [f32]]) {
        assert!(input.len() == self.inputs());
        assert!(output.len() == self.outputs());
        // Assume we have at least one input or output.
        let buffer_size = if input.len() > 0 {
            input[0].len()
        } else {
            output[0].len()
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
    fn inputs(&self) -> usize {
        self.0.inputs()
    }
    fn outputs(&self) -> usize {
        self.0.outputs()
    }
    fn latency(&self) -> Option<f64> {
        self.0.latency()
    }
}
