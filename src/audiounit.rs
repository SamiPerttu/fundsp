use super::*;
use super::prelude::*;
use super::audiocomponent::*;
use numeric_array::*;
use generic_array::sequence::*;

/// AudioUnit processes audio data block by block at a synchronous rate.
/// Once constructed, it has a fixed number of inputs and outputs that can be queried.
/// If not set otherwise, the sample rate is the system default DEFAULT_SR.
pub trait AudioUnit {

    /// Resets the input state of the unit to an initial state where it has not processed any data. 
    fn reset(&mut self, _sample_rate: Option<f64>) {}

    /// Computes a block of data, reading from input buffers and writing to output buffers.
    /// Buffers are supplied as slices. All buffers must have the same size.
    /// The caller decides the length of the block.
    /// Using the same length for many consecutive calls is presumed to be efficient.
    /// The number of input and output buffers must correspond to inputs() and outputs(), respectively.
    fn process(&mut self, input: &[&[F32]], output: &mut [&mut[F32]]);

    /// Number of inputs to this unit. Size of the input arg in compute().
    /// This should be fixed after construction.
    fn inputs(&self) -> usize;

    /// Number of outputs from this unit. Size of the output arg in compute().
    /// This should be fixed after construction.
    fn outputs(&self) -> usize;

    /// Causal latency from input to output, in (fractional) samples.
    /// After a reset, we can discard this many samples from the output to avoid incurring a pre-delay.
    /// This applies only to components that have both inputs and outputs; others should return 0.0.
    /// The latency can depend on the sample rate and is allowed to change after a reset.
    fn latency(&self) -> f64 { 0.0 }
}

/// Adapts an AudioComponent into an AudioUnit.
pub struct AsUnit<A: AudioComponent>(pub A);

impl<A: AudioComponent> AudioUnit for AsUnit<A> {
    fn reset(&mut self, sample_rate: Option<f64>) { self.0.reset(sample_rate); }
    fn process(&mut self, input: &[&[F32]], output: &mut [&mut[F32]])
    {
        assert!(input.len() == self.inputs());
        assert!(output.len() == self.outputs());
        let buffer_size = if input.len() > 0 { input[0].len() } else { output[0].len() };
        assert!(input.iter().all(|x| x.len() == buffer_size));
        assert!(output.iter().all(|x| x.len() == buffer_size));
        for i in 0 .. buffer_size {
            let result = self.0.tick(&NumericArray::generate( |j| afloat(input[j][i]) ));
            for (j, &x) in result.iter().enumerate() {
                output[j][i] = x.as_();
            }
        }
    }
    fn inputs(&self) -> usize { self.0.inputs() }
    fn outputs(&self) -> usize { self.0.outputs() }
    fn latency(&self) -> f64 { self.0.latency() }
}
