/// AudioUnit processes audio data block by block at a synchronous rate.
/// It has a fixed number of inputs and outputs that can be queried.
pub trait AudioUnit {

    /// Resets the unit to an initial state where it has not computed any data. 
    fn reset(&mut self) {}

    /// Computes a block of data, reading from input buffers and writing to output buffers.
    /// Buffers are supplied as slices. All buffers must have the same, non-zero size.
    /// The caller decides the length of the block.
    /// The number of input and output buffers must correspond to inputs() and outputs(), respectively.
    fn process(&mut self, input : &[&[f32]], output : &[&mut[f32]]);

    /// Number of inputs to this unit. Size of the input arg in compute().
    /// This should be fixed after construction.
    fn inputs(&self) -> usize;

    /// Number of outputs from this unit. Size of the output arg in compute().
    /// This should be fixed after construction.
    fn outputs(&self) -> usize;

    /// Causal latency from input to output, in (fractional) samples.
    /// After a reset, we can discard this many samples from the output to avoid incurring a pre-delay.
    /// This applies only to components that have both inputs and outputs.
    /// The latency can depend on the sample rate and is allowed to change after a reset.
    fn latency(&self) -> f64 { 0.0 }
}
