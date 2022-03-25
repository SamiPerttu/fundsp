//! `AudioUnit64` and `AudioUnit32` abstractions and a common wrapper `Au`.

use super::audionode::*;
use super::combinator::*;
use super::math::*;
use super::signal::*;
use super::*;
use duplicate::duplicate_item;
use num_complex::Complex64;
use rsor::Slice;

/// An audio processor with an object safe interface.
/// Once constructed, it has a fixed number of inputs and outputs.
#[duplicate_item(
    f48       AudioUnit48;
    [ f64 ]   [ AudioUnit64 ];
    [ f32 ]   [ AudioUnit32 ];
)]
pub trait AudioUnit48 {
    /// Reset the input state of the unit to an initial state where it has not processed any data.
    /// In other words, reset time to zero.
    fn reset(&mut self, sample_rate: Option<f64>);

    /// Process one sample.
    /// The length of `input` and `output` must be equal to `inputs` and `outputs`, respectively.
    fn tick(&mut self, input: &[f48], output: &mut [f48]);

    /// Process up to 64 (MAX_BUFFER_SIZE) samples.
    /// Buffers are supplied as slices. All buffers must have room for at least `size` samples.
    /// The number of input and output buffers must be equal to `inputs` and `outputs`, respectively.
    fn process(&mut self, size: usize, input: &[&[f48]], output: &mut [&mut [f48]]);

    /// Number of inputs to this unit.
    /// Equals size of the input argument in `tick` and `process`.
    /// This should be fixed after construction.
    fn inputs(&self) -> usize;

    /// Number of outputs from this unit.
    /// Equals size of the output argument in `tick` and `process`.
    /// This should be fixed after construction.
    fn outputs(&self) -> usize;

    /// Route constants, latencies and frequency responses at `frequency` Hz
    /// from inputs to outputs. Return output signal.
    fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame;

    /// Set parameter to value. The default implementation does nothing.
    fn set(&mut self, _parameter: Tag, _value: f64) {}

    /// Query parameter value. The first matching parameter is returned.
    /// The default implementation returns None.
    fn get(&self, _parameter: Tag) -> Option<f64> {
        None
    }

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
    fn get_mono(&mut self) -> f48 {
        debug_assert!(self.inputs() == 0 && self.outputs() == 1);
        let mut output = [0.0];
        self.tick(&[], &mut output);
        output[0]
    }

    /// Retrieve the next stereo sample (left, right) from a generator.
    /// The node must have no inputs and 1 or 2 outputs.
    /// If there is just one output, duplicate it.
    #[inline]
    fn get_stereo(&mut self) -> (f48, f48) {
        debug_assert!(self.inputs() == 0);
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
            _ => panic!("AudioUnit48::get_stereo(): Unit must have 1 or 2 outputs"),
        }
    }

    /// Filter the next mono sample `x`.
    /// The node must have exactly 1 input and 1 output.
    #[inline]
    fn filter_mono(&mut self, x: f48) -> f48 {
        debug_assert!(self.inputs() == 1 && self.outputs() == 1);
        let mut output = [0.0];
        self.tick(&[x], &mut output);
        output[0]
    }

    /// Filter the next stereo sample `(x, y)`.
    /// The node must have exactly 2 inputs and 2 outputs.
    #[inline]
    fn filter_stereo(&mut self, x: f48, y: f48) -> (f48, f48) {
        debug_assert!(self.inputs() == 2 && self.outputs() == 2);
        let mut output = [0.0, 0.0];
        self.tick(&[x, y], &mut output);
        (output[0], output[1])
    }
}

#[duplicate_item(
    f48       AudioUnit48;
    [ f64 ]   [ AudioUnit64 ];
    [ f32 ]   [ AudioUnit32 ];
)]
impl<X: AudioNode<Sample = f48>> AudioUnit48 for An<X>
where
    X::Inputs: Size<f48>,
    X::Outputs: Size<f48>,
{
    fn reset(&mut self, sample_rate: Option<f64>) {
        self.0.reset(sample_rate);
    }
    fn tick(&mut self, input: &[f48], output: &mut [f48]) {
        debug_assert!(input.len() == self.inputs());
        debug_assert!(output.len() == self.outputs());
        output.copy_from_slice(self.0.tick(Frame::from_slice(input)).as_slice());
    }
    fn process(&mut self, size: usize, input: &[&[f48]], output: &mut [&mut [f48]]) {
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
    fn set(&mut self, parameter: Tag, value: f64) {
        self.0.set(parameter, value);
    }
    fn get(&self, parameter: Tag) -> Option<f64> {
        self.0.get(parameter)
    }
}

/// AudioUnit64/32 wrapper.
pub enum Au {
    Unit64(Box<dyn AudioUnit64>),
    Unit32(Box<dyn AudioUnit32>),
}

impl Au {
    pub fn reset(&mut self, sample_rate: Option<f64>) {
        match self {
            Au::Unit64(x) => x.reset(sample_rate),
            Au::Unit32(x) => x.reset(sample_rate),
        }
    }
    pub fn tick64(&mut self, input: &[f64], output: &mut [f64]) {
        if let Au::Unit64(x) = self {
            x.tick(input, output);
        }
    }
    pub fn tick32(&mut self, input: &[f32], output: &mut [f32]) {
        if let Au::Unit32(x) = self {
            x.tick(input, output);
        }
    }
    pub fn process64(&mut self, size: usize, input: &[&[f64]], output: &mut [&mut [f64]]) {
        if let Au::Unit64(x) = self {
            x.process(size, input, output);
        }
    }
    pub fn process32(&mut self, size: usize, input: &[&[f32]], output: &mut [&mut [f32]]) {
        if let Au::Unit32(x) = self {
            x.process(size, input, output);
        }
    }
    pub fn inputs(&self) -> usize {
        match self {
            Au::Unit64(x) => x.inputs(),
            Au::Unit32(x) => x.inputs(),
        }
    }
    pub fn outputs(&self) -> usize {
        match self {
            Au::Unit64(x) => x.outputs(),
            Au::Unit32(x) => x.outputs(),
        }
    }
    pub fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        match self {
            Au::Unit64(x) => x.route(input, frequency),
            Au::Unit32(x) => x.route(input, frequency),
        }
    }

    pub fn set(&mut self, parameter: Tag, value: f64) {
        match self {
            Au::Unit64(x) => x.set(parameter, value),
            Au::Unit32(x) => x.set(parameter, value),
        }
    }

    pub fn get(&self, parameter: Tag) -> Option<f64> {
        match self {
            Au::Unit64(x) => x.get(parameter),
            Au::Unit32(x) => x.get(parameter),
        }
    }
}

/// A big block adapter.
/// The adapter enables calls to `process` with arbitrary buffer sizes.
#[duplicate_item(
    f48       BigBlockAdapter48       AudioUnit48;
    [ f64 ]   [ BigBlockAdapter64 ]   [ AudioUnit64 ];
    [ f32 ]   [ BigBlockAdapter32 ]   [ AudioUnit32 ];
)]
pub struct BigBlockAdapter48 {
    source: Box<dyn AudioUnit48>,
    input: Vec<Vec<f48>>,
    output: Vec<Vec<f48>>,
    input_slice: Slice<[f48]>,
    output_slice: Slice<[f48]>,
}

#[duplicate_item(
    f48       BigBlockAdapter48       AudioUnit48;
    [ f64 ]   [ BigBlockAdapter64 ]   [ AudioUnit64 ];
    [ f32 ]   [ BigBlockAdapter32 ]   [ AudioUnit32 ];
)]
impl BigBlockAdapter48 {
    /// Create a new big block adapter.
    pub fn new(source: Box<dyn AudioUnit48>) -> Self {
        let input = vec![Vec::new(); source.inputs()];
        let output = vec![Vec::new(); source.outputs()];
        Self {
            source,
            input,
            output,
            input_slice: Slice::new(),
            output_slice: Slice::new(),
        }
    }
}

#[duplicate_item(
    f48       BigBlockAdapter48       AudioUnit48;
    [ f64 ]   [ BigBlockAdapter64 ]   [ AudioUnit64 ];
    [ f32 ]   [ BigBlockAdapter32 ]   [ AudioUnit32 ];
)]
impl AudioUnit48 for BigBlockAdapter48 {
    fn reset(&mut self, sample_rate: Option<f64>) {
        self.source.reset(sample_rate);
    }
    fn tick(&mut self, input: &[f48], output: &mut [f48]) {
        self.source.tick(input, output);
    }
    fn process(&mut self, size: usize, input: &[&[f48]], output: &mut [&mut [f48]]) {
        if size > MAX_BUFFER_SIZE {
            for input_buffer in self.input.iter_mut() {
                input_buffer.resize(MAX_BUFFER_SIZE, 0.0);
            }
            for output_buffer in self.output.iter_mut() {
                output_buffer.resize(MAX_BUFFER_SIZE, 0.0);
            }
            let mut i = 0;
            while i < size {
                let n = min(size - i, MAX_BUFFER_SIZE);
                for input_i in 0..self.input.len() {
                    for j in 0..n {
                        self.input[input_i][j] = input[input_i][i + j];
                    }
                }
                self.source.process(
                    n,
                    self.input_slice.from_refs(&self.input),
                    self.output_slice.from_muts(&mut self.output),
                );
                for output_i in 0..self.output.len() {
                    for j in 0..n {
                        output[output_i][i + j] = self.output[output_i][j];
                    }
                }
                i += n;
            }
        } else {
            self.source.process(size, input, output);
        }
    }
    fn inputs(&self) -> usize {
        self.source.inputs()
    }
    fn outputs(&self) -> usize {
        self.source.outputs()
    }
    fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.source.route(input, frequency)
    }
    fn set(&mut self, parameter: Tag, value: f64) {
        self.source.set(parameter, value);
    }
    fn get(&self, parameter: Tag) -> Option<f64> {
        self.source.get(parameter)
    }
}
