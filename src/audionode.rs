//! The central `AudioNode` abstraction and basic components.

use super::buffer::*;
use super::combinator::*;
use super::graph::*;
use super::math::*;
use super::setting::*;
use super::signal::*;
use super::*;
use core::marker::PhantomData;
use num_complex::Complex64;
use numeric_array::typenum::*;
extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

/*
Order of type arguments in nodes:
1. Basic input and output arities excepting filter input selector arities.
2. Processing float type.
3. Unary or binary operation type.
4. Filter input selector arity.
5. Contained node types.
6. The rest in any order.
*/

/// Generic audio processor.
/// `AudioNode` has a static number of inputs (`AudioNode::Inputs`) and outputs (`AudioNode::Outputs`).
pub trait AudioNode: Clone + Sync + Send {
    /// Unique ID for hashing.
    const ID: u64;
    /// Input arity.
    type Inputs: Size<f32>;
    /// Output arity.
    type Outputs: Size<f32>;

    /// Reset the input state of the component and all its children to an initial state
    /// where it has not processed any samples. In other words, reset time to zero.
    /// If `allocate` has been called previously, and the sample rate is unchanged,
    /// then it is expected that no memory allocation or deallocation takes place here.
    ///
    /// ### Example
    /// ```
    /// use fundsp::prelude64::*;
    /// let mut node = saw_hz(440.0);
    /// let sample1 = node.get_mono();
    /// node.reset();
    /// let sample2 = node.get_mono();
    /// assert_eq!(sample1, sample2);
    /// ```
    #[allow(unused_variables)]
    fn reset(&mut self) {
        // The default implementation does nothing.
    }

    /// Set the sample rate of the node and all its children.
    /// The default sample rate is 44100 Hz.
    /// The unit is allowed to reset its state here in response to sample rate changes.
    /// If the sample rate stays unchanged, then the goal is to maintain current state.
    ///
    /// ### Example (Changing The Sample Rate)
    /// ```
    /// use fundsp::prelude64::*;
    /// let mut node = saw_hz(440.0);
    /// node.set_sample_rate(48_000.0);
    /// ```
    #[allow(unused_variables)]
    fn set_sample_rate(&mut self, sample_rate: f64) {
        // The default implementation does nothing.
    }

    /// Process one sample.
    ///
    /// ### Example
    /// ```
    /// use fundsp::prelude64::*;
    /// assert_eq!(pass().tick(&Frame::from([2.0])), Frame::from([2.0]));
    /// ```
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs>;

    /// Process up to 64 (`MAX_BUFFER_SIZE`) samples.
    /// If `size` is zero then this is a no-op, which is permitted.
    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        // The default implementation is a fallback that calls into `tick`.
        debug_assert!(size <= MAX_BUFFER_SIZE);
        debug_assert!(input.channels() == self.inputs());
        debug_assert!(output.channels() == self.outputs());

        // Note. We could build `tick` inputs from `[f32; 8]` or `f32x8` temporary
        // values to make index arithmetic easier but according to benchmarks
        // it doesn't make a difference.
        let mut input_frame: Frame<f32, Self::Inputs> = Frame::default();

        for i in 0..size {
            for channel in 0..self.inputs() {
                input_frame[channel] = input.at_f32(channel, i);
            }
            let output_frame = self.tick(&input_frame);
            for channel in 0..self.outputs() {
                output.set_f32(channel, i, output_frame[channel]);
            }
        }
    }

    /// Process samples left over using `tick` after processing all full SIMD items.
    /// This is a convenience method for implementers.
    #[inline]
    fn process_remainder(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        debug_assert!(size <= MAX_BUFFER_SIZE);
        debug_assert!(input.channels() == self.inputs());
        debug_assert!(output.channels() == self.outputs());

        let mut input_frame: Frame<f32, Self::Inputs> = Frame::default();

        for i in (size & !SIMD_M)..size {
            for channel in 0..self.inputs() {
                input_frame[channel] = input.at_f32(channel, i);
            }
            let output_frame = self.tick(&input_frame);
            for channel in 0..self.outputs() {
                output.set_f32(channel, i, output_frame[channel]);
            }
        }
    }

    /// Set a parameter. What formats are recognized depends on the component.
    #[allow(unused_variables)]
    fn set(&mut self, setting: Setting) {}

    /// Set node pseudorandom phase hash.
    /// This is called from `ping` (only). It should not be called by users.
    /// The node is allowed to reset itself here.
    #[allow(unused_variables)]
    fn set_hash(&mut self, hash: u64) {
        // Override this to use the hash.
        // The default implementation does nothing.
    }

    /// Ping contained `AudioNode`s to obtain a deterministic pseudorandom hash.
    /// The local hash includes children, too.
    /// Leaf nodes should not need to override this.
    /// If `probe` is true, then this is a probe for computing the network hash
    /// and `set_hash` should not be called yet.
    /// To set a custom hash for a graph, call this method with `probe`
    /// set to false and `hash` initialized with the custom hash.
    /// The node is allowed to reset itself here.
    ///
    /// ### Example (Setting a Custom Hash)
    /// ```
    /// use fundsp::prelude32::*;
    /// let mut node = square_hz(440.0);
    /// node.ping(false, AttoHash::new(1));
    /// ```
    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        if !probe {
            self.set_hash(hash.state());
        }
        hash.hash(Self::ID)
    }

    /// Preallocate all needed memory.
    fn allocate(&mut self) {
        // The default implementation does nothing.
    }

    /// Route constants, latencies and frequency responses at `frequency` Hz
    /// from inputs to outputs. Return output signal.
    /// If there are no frequency responses in `input`, then `frequency` is ignored.
    #[allow(unused_variables)]
    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        // The default implementation marks all outputs unknown.
        SignalFrame::new(self.outputs())
    }

    /// Get edge source to output `index`.
    fn source_edge(&self, output: usize, path: Path) -> Path {
        path.with_index(output)
    }

    /// Get edge targets from input `index`.
    fn target_edges(&self, input: usize, path: Path) -> Vec<Path> {
        vec![path.with_index(input)]
    }

    /// Fill inner structure of the node with nodes and edges.
    fn fill_graph(&self, graph: &mut Graph, path: Path) {
        graph.push_node(Node::new(path, Self::ID, self.inputs(), self.outputs()));
    }

    // End of interface. There is no need to override the following.

    /// Number of inputs.
    ///
    /// ### Example
    /// ```
    /// use fundsp::prelude64::*;
    /// assert_eq!(sink().inputs(), 1);
    /// ```
    #[inline(always)]
    fn inputs(&self) -> usize {
        Self::Inputs::USIZE
    }

    /// Number of outputs.
    ///
    /// ### Example
    /// ```
    /// use fundsp::prelude64::*;
    /// assert_eq!(zero().outputs(), 1);
    /// ```
    #[inline(always)]
    fn outputs(&self) -> usize {
        Self::Outputs::USIZE
    }

    /// Retrieve the next mono sample from a generator.
    /// The node must have no inputs and 1 or 2 outputs.
    /// If there are two outputs, average the channels.
    ///
    /// ### Example
    /// ```
    /// use fundsp::prelude64::*;
    /// assert_eq!(dc(2.0).get_mono(), 2.0);
    /// assert_eq!(dc((3.0, 4.0)).get_mono(), 3.5);
    /// ```
    #[inline]
    fn get_mono(&mut self) -> f32 {
        assert!(
            Self::Inputs::USIZE == 0 && (Self::Outputs::USIZE == 1 || Self::Outputs::USIZE == 2)
        );
        let output = self.tick(&Frame::default());
        if self.outputs() == 1 {
            output[0]
        } else {
            (output[0] + output[1]) / f32::new(2)
        }
    }

    /// Retrieve the next stereo sample (left, right) from a generator.
    /// The node must have no inputs and 1 or 2 outputs.
    /// If there is just one output, duplicate it.
    ///
    /// ### Example
    /// ```
    /// use fundsp::prelude64::*;
    /// assert_eq!(dc((5.0, 6.0)).get_stereo(), (5.0, 6.0));
    /// assert_eq!(dc(7.0).get_stereo(), (7.0, 7.0));
    /// ```
    #[inline]
    fn get_stereo(&mut self) -> (f32, f32) {
        assert!(
            Self::Inputs::USIZE == 0 && (Self::Outputs::USIZE == 1 || Self::Outputs::USIZE == 2)
        );
        let output = self.tick(&Frame::default());
        (output[0], output[(Self::Outputs::USIZE > 1) as usize])
    }

    /// Filter the next mono sample `x`.
    /// The node must have exactly 1 input and 1 output.
    ///
    /// ### Example
    /// ```
    /// use fundsp::prelude64::*;
    /// assert_eq!(add(1.0).filter_mono(1.0), 2.0);
    /// ```
    #[inline]
    fn filter_mono(&mut self, x: f32) -> f32 {
        assert!(Self::Inputs::USIZE == 1 && Self::Outputs::USIZE == 1);
        let output = self.tick(&Frame::splat(x));
        output[0]
    }

    /// Filter the next stereo sample `(x, y)`.
    /// The node must have exactly 2 inputs and 2 outputs.
    ///
    /// ### Example
    /// ```
    /// use fundsp::prelude64::*;
    /// assert_eq!(add((2.0, 3.0)).filter_stereo(4.0, 5.0), (6.0, 8.0));
    /// ```
    #[inline]
    fn filter_stereo(&mut self, x: f32, y: f32) -> (f32, f32) {
        assert!(Self::Inputs::USIZE == 2 && Self::Outputs::USIZE == 2);
        let output = self.tick(&Frame::generate(|i| if i == 0 { x } else { y }));
        (output[0], output[1])
    }

    /// Evaluate frequency response of `output` at `frequency` Hz.
    /// Any linear response can be composed.
    /// Return `None` if there is no response or it could not be calculated.
    ///
    /// ### Example
    /// ```
    /// use fundsp::prelude64::*;
    /// assert_eq!(pass().response(0, 440.0), Some(Complex64::new(1.0, 0.0)));
    /// ```
    fn response(&mut self, output: usize, frequency: f64) -> Option<Complex64> {
        assert!(output < self.outputs());
        let mut input = SignalFrame::new(self.inputs());
        for i in 0..self.inputs() {
            input.set(i, Signal::Response(Complex64::new(1.0, 0.0), 0.0));
        }
        let response = self.route(&input, frequency);
        match response.at(output) {
            Signal::Response(rx, _) => Some(rx),
            _ => None,
        }
    }

    /// Evaluate frequency response of `output` in dB at `frequency` Hz.
    /// Any linear response can be composed.
    /// Return `None` if there is no response or it could not be calculated.
    ///
    /// ### Example
    /// ```
    /// use fundsp::prelude64::*;
    /// let db = pass().response_db(0, 440.0).unwrap();
    /// assert!(db < 1.0e-7 && db > -1.0e-7);
    /// ```
    fn response_db(&mut self, output: usize, frequency: f64) -> Option<f64> {
        assert!(output < self.outputs());
        self.response(output, frequency).map(|r| amp_db(r.norm()))
    }

    /// Causal latency in (fractional) samples, if any.
    /// After a reset, we can discard this many samples from the output to avoid incurring a pre-delay.
    /// The latency may depend on the sample rate.
    /// Voluntary latencies, such as delays, are not counted as latency.
    ///
    /// ### Example
    /// ```
    /// use fundsp::prelude64::*;
    /// assert_eq!(pass().latency(), Some(0.0));
    /// assert_eq!(tick().latency(), Some(0.0));
    /// assert_eq!(sink().latency(), None);
    /// assert_eq!(lowpass_hz(440.0, 1.0).latency(), Some(0.0));
    /// assert_eq!(limiter(0.01, 0.01).latency(), Some(441.0));
    /// ```
    fn latency(&mut self) -> Option<f64> {
        if self.outputs() == 0 {
            return None;
        }
        let mut input = SignalFrame::new(self.inputs());
        for i in 0..self.inputs() {
            input.set(i, Signal::Latency(0.0));
        }
        // The frequency argument can be anything as there are no responses to propagate,
        // only latencies. Latencies are never promoted to responses during signal routing.
        let response = self.route(&input, 1.0);
        // Return the minimum latency.
        let mut result: Option<f64> = None;
        for output in 0..self.outputs() {
            match (result, response.at(output)) {
                (None, Signal::Latency(x)) => result = Some(x),
                (Some(r), Signal::Latency(x)) => result = Some(r.min(x)),
                _ => (),
            }
        }
        result
    }
}

/// Pass through inputs unchanged.
#[derive(Default, Clone)]
pub struct MultiPass<N> {
    _marker: PhantomData<N>,
}

impl<N: Size<f32>> MultiPass<N> {
    pub fn new() -> Self {
        MultiPass::default()
    }
}

impl<N: Size<f32>> AudioNode for MultiPass<N> {
    const ID: u64 = 0;
    type Inputs = N;
    type Outputs = N;

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        input.clone()
    }
    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        for channel in 0..self.outputs() {
            for i in 0..simd_items(size) {
                output.set(channel, i, input.at(channel, i));
            }
        }
    }
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        input.clone()
    }
}

/// Pass through input unchanged.
#[derive(Default, Clone)]
pub struct Pass {}

impl Pass {
    pub fn new() -> Self {
        Pass::default()
    }
}

// Note. We have separate Pass and MultiPass structs
// because it helps a little with type inference.
impl AudioNode for Pass {
    const ID: u64 = 48;
    type Inputs = U1;
    type Outputs = U1;

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        *input
    }
    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        for i in 0..simd_items(size) {
            output.set(0, i, input.at(0, i));
        }
    }
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        input.clone()
    }
}

/// Discard inputs.
#[derive(Default, Clone)]
pub struct Sink<N> {
    _marker: PhantomData<N>,
}

impl<N: Size<f32>> Sink<N> {
    pub fn new() -> Self {
        Sink::default()
    }
}

impl<N: Size<f32>> AudioNode for Sink<N> {
    const ID: u64 = 1;
    type Inputs = N;
    type Outputs = U0;

    #[inline]
    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        Frame::default()
    }
    fn process(&mut self, _size: usize, _input: &BufferRef, _output: &mut BufferMut) {}

    fn route(&mut self, _input: &SignalFrame, _frequency: f64) -> SignalFrame {
        SignalFrame::new(self.outputs())
    }
}

/// Output a constant value.
#[derive(Clone)]
pub struct Constant<N: Size<f32>> {
    output: Frame<f32, N>,
}

impl<N: Size<f32>> Constant<N> {
    /// Construct constant.
    pub fn new(output: Frame<f32, N>) -> Self {
        Constant { output }
    }
    /// Set the value of the constant.
    #[inline]
    pub fn set_value(&mut self, output: Frame<f32, N>) {
        self.output = output;
    }
    /// Get the value of the constant.
    #[inline]
    pub fn value(&self) -> Frame<f32, N> {
        self.output.clone()
    }
    /// Set a scalar value on all channels.
    #[inline]
    pub fn set_scalar(&mut self, output: f32) {
        self.output = Frame::splat(output);
    }
}

impl<N: Size<f32>> AudioNode for Constant<N> {
    const ID: u64 = 2;
    type Inputs = U0;
    type Outputs = N;

    #[inline]
    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        self.output.clone()
    }

    fn process(&mut self, size: usize, _input: &BufferRef, output: &mut BufferMut) {
        for channel in 0..self.outputs() {
            let channel_value = F32x::splat(self.output[channel].to_f32());
            for j in 0..simd_items(size) {
                output.set(channel, j, channel_value);
            }
        }
    }

    fn set(&mut self, setting: Setting) {
        if let Parameter::Value(value) = setting.parameter() {
            self.set_scalar(*value);
        }
    }

    fn route(&mut self, _input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = SignalFrame::new(self.outputs());
        for i in 0..N::USIZE {
            output.set(i, Signal::Value(self.output[i].to_f64()));
        }
        output
    }
}

/// Split input into `N` channels.
#[derive(Clone)]
pub struct Split<N> {
    _marker: PhantomData<N>,
}

// Note. We have separate split and multisplit (and join and multijoin)
// implementations because it helps with type inference.
impl<N> Split<N>
where
    N: Size<f32>,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<N> AudioNode for Split<N>
where
    N: Size<f32>,
{
    const ID: u64 = 40;
    type Inputs = U1;
    type Outputs = N;

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        Frame::splat(input[0])
    }
    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        for channel in 0..N::USIZE {
            for i in 0..simd_items(size) {
                output.set(channel, i, input.at(0, i));
            }
        }
    }
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Split.route(input, self.outputs())
    }
}

/// Split `M` inputs into `N` branches, with `M` * `N` outputs.
#[derive(Clone)]
pub struct MultiSplit<M, N> {
    _marker: PhantomData<(M, N)>,
}

impl<M, N> MultiSplit<M, N>
where
    M: Size<f32> + Mul<N>,
    N: Size<f32>,
    <M as Mul<N>>::Output: Size<f32>,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<M, N> AudioNode for MultiSplit<M, N>
where
    M: Size<f32> + Mul<N>,
    N: Size<f32>,
    <M as Mul<N>>::Output: Size<f32>,
{
    const ID: u64 = 38;
    type Inputs = M;
    type Outputs = numeric_array::typenum::Prod<M, N>;

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        Frame::generate(|i| input[i % M::USIZE])
    }
    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        for channel in 0..M::USIZE * N::USIZE {
            for i in 0..simd_items(size) {
                output.set(channel, i, input.at(channel % M::USIZE, i));
            }
        }
    }
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Split.route(input, self.outputs())
    }
}

/// Join `N` channels into one by averaging. Inverse of `Split<N>`.
#[derive(Clone)]
pub struct Join<N> {
    _marker: PhantomData<N>,
}

impl<N> Join<N>
where
    N: Size<f32>,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<N> AudioNode for Join<N>
where
    N: Size<f32>,
{
    const ID: u64 = 41;
    type Inputs = N;
    type Outputs = U1;

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let mut output = input[0];
        for i in 1..N::USIZE {
            output += input[i];
        }
        [output / N::I64 as f32].into()
    }
    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        let z = 1.0 / N::U64 as f32;
        for i in 0..simd_items(size) {
            output.set(0, i, input.at(0, i) * z);
        }
        for channel in 1..N::USIZE {
            for i in 0..simd_items(size) {
                output.add(0, i, input.at(channel, i) * z);
            }
        }
    }
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Join.route(input, self.outputs())
    }
}

/// Average `N` branches of `M` channels into one branch with `M` channels.
/// The input has `M` * `N` channels. Inverse of `MultiSplit<M, N>`.
#[derive(Clone)]
pub struct MultiJoin<M, N> {
    _marker: PhantomData<(M, N)>,
}

impl<M, N> MultiJoin<M, N>
where
    M: Size<f32> + Mul<N>,
    N: Size<f32>,
    <M as Mul<N>>::Output: Size<f32>,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<M, N> AudioNode for MultiJoin<M, N>
where
    M: Size<f32> + Mul<N>,
    N: Size<f32>,
    <M as Mul<N>>::Output: Size<f32>,
{
    const ID: u64 = 39;
    type Inputs = numeric_array::typenum::Prod<M, N>;
    type Outputs = M;

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        Frame::generate(|j| {
            let mut output = input[j];
            for i in 1..N::USIZE {
                output += input[j + i * M::USIZE];
            }
            output / N::I64 as f32
        })
    }
    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        let z = 1.0 / N::U64 as f32;
        for channel in 0..M::USIZE {
            for i in 0..simd_items(size) {
                output.set(channel, i, input.at(channel, i) * z);
            }
        }
        for channel in M::USIZE..M::USIZE * N::USIZE {
            for i in 0..simd_items(size) {
                output.add(channel % M::USIZE, i, input.at(channel, i) * z);
            }
        }
    }
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Join.route(input, self.outputs())
    }
}

/// Provides binary operator implementations to the `Binop` node.
pub trait FrameBinop<N: Size<f32>>: Clone + Sync + Send {
    /// Do binary op (`x` op `y`) channelwise.
    fn binop(&self, x: F32x, y: F32x) -> F32x;
    /// Do binary op (`x` op `y`) channelwise.
    fn frame(&self, x: &Frame<f32, N>, y: &Frame<f32, N>) -> Frame<f32, N>;
    /// Do binary op (`x` op `y`) in-place lengthwise. `size` may be zero.
    fn assign(&self, size: usize, x: &mut [f32], y: &[f32]);
    /// Do binary op (`x` op `y`) on signals.
    fn route(&self, x: Signal, y: Signal) -> Signal;
}

/// Addition operator.
#[derive(Default, Clone)]
pub struct FrameAdd<N: Size<f32>> {
    _marker: PhantomData<N>,
}

impl<N: Size<f32>> FrameAdd<N> {
    pub fn new() -> FrameAdd<N> {
        FrameAdd::default()
    }
}

impl<N: Size<f32>> FrameBinop<N> for FrameAdd<N> {
    #[inline]
    fn binop(&self, x: F32x, y: F32x) -> F32x {
        x + y
    }
    #[inline]
    fn frame(&self, x: &Frame<f32, N>, y: &Frame<f32, N>) -> Frame<f32, N> {
        x + y
    }
    #[inline]
    fn assign(&self, size: usize, x: &mut [f32], y: &[f32]) {
        for (o, i) in x[..size].iter_mut().zip(y[..size].iter()) {
            *o += *i;
        }
    }
    fn route(&self, x: Signal, y: Signal) -> Signal {
        x.combine_linear(y, 0.0, |x, y| x + y, |x, y| x + y)
    }
}

/// Subtraction operator.
#[derive(Default, Clone)]
pub struct FrameSub<N: Size<f32>> {
    _marker: PhantomData<N>,
}

impl<N: Size<f32>> FrameSub<N> {
    pub fn new() -> FrameSub<N> {
        FrameSub::default()
    }
}

impl<N: Size<f32>> FrameBinop<N> for FrameSub<N> {
    #[inline]
    fn binop(&self, x: F32x, y: F32x) -> F32x {
        x - y
    }
    #[inline]
    fn frame(&self, x: &Frame<f32, N>, y: &Frame<f32, N>) -> Frame<f32, N> {
        x - y
    }
    #[inline]
    fn assign(&self, size: usize, x: &mut [f32], y: &[f32]) {
        for (o, i) in x[..size].iter_mut().zip(y[..size].iter()) {
            *o -= *i;
        }
    }
    fn route(&self, x: Signal, y: Signal) -> Signal {
        x.combine_linear(y, 0.0, |x, y| x - y, |x, y| x - y)
    }
}

/// Multiplication operator.
#[derive(Default, Clone)]
pub struct FrameMul<N: Size<f32>> {
    _marker: PhantomData<N>,
}

impl<N: Size<f32>> FrameMul<N> {
    pub fn new() -> FrameMul<N> {
        FrameMul::default()
    }
}

impl<N: Size<f32>> FrameBinop<N> for FrameMul<N> {
    #[inline]
    fn binop(&self, x: F32x, y: F32x) -> F32x {
        x * y
    }
    #[inline]
    fn frame(&self, x: &Frame<f32, N>, y: &Frame<f32, N>) -> Frame<f32, N> {
        x * y
    }
    #[inline]
    fn assign(&self, size: usize, x: &mut [f32], y: &[f32]) {
        for (o, i) in x[..size].iter_mut().zip(y[..size].iter()) {
            *o *= *i;
        }
    }
    fn route(&self, x: Signal, y: Signal) -> Signal {
        match (x, y) {
            (Signal::Value(vx), Signal::Value(vy)) => Signal::Value(vx * vy),
            (Signal::Latency(lx), Signal::Latency(ly)) => Signal::Latency(min(lx, ly)),
            (Signal::Response(_, lx), Signal::Response(_, ly)) => Signal::Latency(min(lx, ly)),
            (Signal::Response(_, lx), Signal::Latency(ly)) => Signal::Latency(min(lx, ly)),
            (Signal::Latency(lx), Signal::Response(_, ly)) => Signal::Latency(min(lx, ly)),
            (Signal::Response(rx, lx), Signal::Value(vy)) => {
                Signal::Response(rx * Complex64::new(vy, 0.0), lx)
            }
            (Signal::Value(vx), Signal::Response(ry, ly)) => {
                Signal::Response(ry * Complex64::new(vx, 0.0), ly)
            }
            (Signal::Latency(lx), _) => Signal::Latency(lx),
            (Signal::Response(_, lx), _) => Signal::Latency(lx),
            (_, Signal::Latency(ly)) => Signal::Latency(ly),
            (_, Signal::Response(_, ly)) => Signal::Latency(ly),
            _ => Signal::Unknown,
        }
    }
}

#[derive(Clone)]
pub struct Binop<B, X, Y>
where
    B: FrameBinop<X::Outputs>,
    X: AudioNode,
    Y: AudioNode<Outputs = X::Outputs>,
    X::Inputs: Add<Y::Inputs>,
    <X::Inputs as Add<Y::Inputs>>::Output: Size<f32>,
{
    x: X,
    y: Y,
    binop: B,
}

impl<B, X, Y> Binop<B, X, Y>
where
    B: FrameBinop<X::Outputs>,
    X: AudioNode,
    Y: AudioNode<Outputs = X::Outputs>,
    X::Inputs: Add<Y::Inputs>,
    <X::Inputs as Add<Y::Inputs>>::Output: Size<f32>,
{
    pub fn new(binop: B, x: X, y: Y) -> Self {
        let mut node = Self { x, y, binop };
        let hash = node.ping(true, AttoHash::new(Self::ID));
        node.ping(false, hash);
        node
    }

    /// Access the left node of the sum.
    #[inline]
    pub fn left_mut(&mut self) -> &mut X {
        &mut self.x
    }

    /// Access the left node of the sum.
    #[inline]
    pub fn left(&self) -> &X {
        &self.x
    }

    /// Access the right node of the sum.
    #[inline]
    pub fn right_mut(&mut self) -> &mut Y {
        &mut self.y
    }

    /// Access the right node of the sum.
    #[inline]
    pub fn right(&self) -> &Y {
        &self.y
    }
}

impl<B, X, Y> AudioNode for Binop<B, X, Y>
where
    B: FrameBinop<X::Outputs>,
    X: AudioNode,
    Y: AudioNode<Outputs = X::Outputs>,
    X::Inputs: Add<Y::Inputs>,
    <X::Inputs as Add<Y::Inputs>>::Output: Size<f32>,
{
    const ID: u64 = 3;
    type Inputs = Sum<X::Inputs, Y::Inputs>;
    type Outputs = X::Outputs;

    fn reset(&mut self) {
        self.x.reset();
        self.y.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x.set_sample_rate(sample_rate);
        self.y.set_sample_rate(sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let input_x = &input[..X::Inputs::USIZE];
        let input_y = &input[X::Inputs::USIZE..];
        self.binop
            .frame(&self.x.tick(input_x.into()), &self.y.tick(input_y.into()))
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        let mut buffer = BufferArray::<X::Outputs>::uninitialized();
        self.x.process(
            size,
            &input.subset(0, self.x.inputs()),
            &mut buffer.buffer_mut(),
        );
        self.y.process(
            size,
            &input.subset(self.x.inputs(), self.y.inputs()),
            output,
        );
        for channel in 0..self.outputs() {
            for i in 0..simd_items(size) {
                output.set(
                    channel,
                    i,
                    self.binop
                        .binop(buffer.at(channel, i), output.at(channel, i)),
                );
            }
        }
    }

    fn set(&mut self, setting: Setting) {
        match setting.direction() {
            Address::Left => self.x.set(setting.peel()),
            Address::Right => self.y.set(setting.peel()),
            _ => (),
        }
    }

    #[inline]
    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.y.ping(probe, self.x.ping(probe, hash.hash(Self::ID)))
    }

    fn allocate(&mut self) {
        self.x.allocate();
        self.y.allocate();
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut signal_x = self
            .x
            .route(&SignalFrame::copy(input, 0, X::Inputs::USIZE), frequency);
        let signal_y = self.y.route(
            &SignalFrame::copy(input, X::Inputs::USIZE, Y::Inputs::USIZE),
            frequency,
        );
        for i in 0..Self::Outputs::USIZE {
            signal_x.set(i, self.binop.route(signal_x.at(i), signal_y.at(i)));
        }
        signal_x
    }

    fn source_edge(&self, output: usize, path: Path) -> Path {
        path.with_index(output)
    }

    fn target_edges(&self, input: usize, mut path: Path) -> Vec<Path> {
        if input < self.x.inputs() {
            path.push(0);
            self.x.target_edges(input, path)
        } else {
            path.push(1);
            self.y.target_edges(input - self.x.inputs(), path)
        }
    }

    fn fill_graph(&self, graph: &mut Graph, path: Path) {
        graph.push_node(Node::new(
            path.clone(),
            Self::ID,
            self.outputs(),
            self.outputs(),
        ));
        let x_path = path.clone().with_suffix(0);
        for i in 0..self.x.outputs() {
            graph.push_edge(Edge::new(
                self.x.source_edge(i, x_path.clone()),
                path.clone().with_index(i),
            ));
        }
        self.x.fill_graph(graph, x_path);
        let y_path = path.clone().with_suffix(1);
        for i in 0..self.y.outputs() {
            graph.push_edge(Edge::new(
                self.y.source_edge(i, y_path.clone()),
                path.clone().with_index(i),
            ));
        }
        self.y.fill_graph(graph, y_path);
    }
}

/// Provides unary operator implementations to the `Unop` node.
pub trait FrameUnop<N: Size<f32>>: Clone + Sync + Send {
    /// Do unary op channelwise.
    fn unop(&self, x: F32x) -> F32x;
    /// Do unary op channelwise.
    fn frame(&self, x: &Frame<f32, N>) -> Frame<f32, N>;
    /// Do unary op in-place lengthwise.
    fn assign(&self, size: usize, x: &mut [f32]);
    /// Do unary op on signal.
    fn route(&self, x: Signal) -> Signal;
}

/// Negation operator.
#[derive(Default, Clone)]
pub struct FrameNeg<N: Size<f32>> {
    _marker: PhantomData<N>,
}

impl<N: Size<f32>> FrameNeg<N> {
    pub fn new() -> FrameNeg<N> {
        FrameNeg::default()
    }
}

impl<N: Size<f32>> FrameUnop<N> for FrameNeg<N> {
    #[inline]
    fn unop(&self, x: F32x) -> F32x {
        -x
    }
    #[inline]
    fn frame(&self, x: &Frame<f32, N>) -> Frame<f32, N> {
        -x
    }
    #[inline]
    fn assign(&self, size: usize, x: &mut [f32]) {
        for o in x[..size].iter_mut() {
            *o = -*o;
        }
    }
    fn route(&self, x: Signal) -> Signal {
        match x {
            Signal::Value(vx) => Signal::Value(-vx),
            Signal::Response(rx, lx) => Signal::Response(-rx, lx),
            s => s,
        }
    }
}

/// Identity op.
#[derive(Default, Clone)]
pub struct FrameId<N: Size<f32>> {
    _marker: PhantomData<N>,
}

impl<N: Size<f32>> FrameId<N> {
    pub fn new() -> FrameId<N> {
        FrameId::default()
    }
}

impl<N: Size<f32>> FrameUnop<N> for FrameId<N> {
    #[inline]
    fn unop(&self, x: F32x) -> F32x {
        x
    }
    #[inline]
    fn frame(&self, x: &Frame<f32, N>) -> Frame<f32, N> {
        x.clone()
    }
    #[inline]
    fn assign(&self, _size: usize, _x: &mut [f32]) {}
    fn route(&self, x: Signal) -> Signal {
        x
    }
}

/// Add scalar op.
#[derive(Default, Clone)]
pub struct FrameAddScalar<N: Size<f32>> {
    scalar: f32,
    splat: F32x,
    _marker: PhantomData<N>,
}

impl<N: Size<f32>> FrameAddScalar<N> {
    pub fn new(scalar: f32) -> Self {
        Self {
            scalar,
            splat: F32x::splat(scalar),
            _marker: PhantomData,
        }
    }
}

impl<N: Size<f32>> FrameUnop<N> for FrameAddScalar<N> {
    #[inline]
    fn unop(&self, x: F32x) -> F32x {
        x + self.splat
    }
    #[inline]
    fn frame(&self, x: &Frame<f32, N>) -> Frame<f32, N> {
        x + Frame::splat(self.scalar)
    }
    #[inline]
    fn assign(&self, size: usize, x: &mut [f32]) {
        for o in x[..size].iter_mut() {
            *o += self.scalar;
        }
    }
    fn route(&self, x: Signal) -> Signal {
        match x {
            Signal::Value(vx) => Signal::Value(vx + self.scalar.to_f64()),
            s => s,
        }
    }
}

/// Negate and add scalar op.
#[derive(Default, Clone)]
pub struct FrameNegAddScalar<N: Size<f32>> {
    scalar: f32,
    splat: F32x,
    _marker: PhantomData<N>,
}

impl<N: Size<f32>> FrameNegAddScalar<N> {
    pub fn new(scalar: f32) -> Self {
        Self {
            scalar,
            splat: F32x::splat(scalar),
            _marker: PhantomData,
        }
    }
}

impl<N: Size<f32>> FrameUnop<N> for FrameNegAddScalar<N> {
    #[inline]
    fn unop(&self, x: F32x) -> F32x {
        -x + self.splat
    }
    #[inline]
    fn frame(&self, x: &Frame<f32, N>) -> Frame<f32, N> {
        -x + Frame::splat(self.scalar)
    }
    #[inline]
    fn assign(&self, size: usize, x: &mut [f32]) {
        for o in x[..size].iter_mut() {
            *o = -*o + self.scalar;
        }
    }
    fn route(&self, x: Signal) -> Signal {
        match x {
            Signal::Value(vx) => Signal::Value(-vx + self.scalar.to_f64()),
            Signal::Response(rx, lx) => Signal::Response(-rx, lx),
            s => s,
        }
    }
}

/// Multiply with scalar op.
#[derive(Default, Clone)]
pub struct FrameMulScalar<N: Size<f32>> {
    scalar: f32,
    splat: F32x,
    _marker: PhantomData<N>,
}

impl<N: Size<f32>> FrameMulScalar<N> {
    pub fn new(scalar: f32) -> Self {
        Self {
            scalar,
            splat: F32x::splat(scalar),
            _marker: PhantomData,
        }
    }
}

impl<N: Size<f32>> FrameUnop<N> for FrameMulScalar<N> {
    #[inline]
    fn unop(&self, x: F32x) -> F32x {
        x * self.splat
    }
    #[inline]
    fn frame(&self, x: &Frame<f32, N>) -> Frame<f32, N> {
        x * Frame::splat(self.scalar)
    }
    #[inline]
    fn assign(&self, size: usize, x: &mut [f32]) {
        for o in x[..size].iter_mut() {
            *o *= self.scalar;
        }
    }
    fn route(&self, x: Signal) -> Signal {
        match x {
            Signal::Response(vx, lx) => Signal::Response(vx * self.scalar.to_f64(), lx),
            Signal::Value(vx) => Signal::Value(vx * self.scalar.to_f64()),
            s => s,
        }
    }
}

/// Apply a unary operation to output of contained node.
#[derive(Clone)]
pub struct Unop<X, U> {
    x: X,
    u: U,
}

impl<X, U> Unop<X, U>
where
    X: AudioNode,
    U: FrameUnop<X::Outputs>,
{
    pub fn new(x: X, u: U) -> Self {
        let mut node = Unop { x, u };
        let hash = node.ping(true, AttoHash::new(Self::ID));
        node.ping(false, hash);
        node
    }
}

impl<X, U> AudioNode for Unop<X, U>
where
    X: AudioNode,
    U: FrameUnop<X::Outputs>,
{
    const ID: u64 = 4;
    type Inputs = X::Inputs;
    type Outputs = X::Outputs;

    fn reset(&mut self) {
        self.x.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x.set_sample_rate(sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        self.u.frame(&self.x.tick(input))
    }

    #[inline]
    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        self.x.process(size, input, output);
        for channel in 0..self.outputs() {
            for i in 0..simd_items(size) {
                output.set(channel, i, self.u.unop(output.at(channel, i)));
            }
        }
    }

    fn set(&mut self, setting: Setting) {
        self.x.set(setting);
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.x.ping(probe, hash.hash(Self::ID))
    }

    fn allocate(&mut self) {
        self.x.allocate();
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut signal_x = self.x.route(input, frequency);
        for i in 0..Self::Outputs::USIZE {
            signal_x.set(i, self.u.route(signal_x.at(i)));
        }
        signal_x
    }

    fn source_edge(&self, output: usize, path: Path) -> Path {
        path.with_index(output)
    }

    fn target_edges(&self, input: usize, path: Path) -> Vec<Path> {
        self.x.target_edges(input, path.with_suffix(0))
    }

    fn fill_graph(&self, graph: &mut Graph, path: Path) {
        graph.push_node(Node::new(
            path.clone(),
            Self::ID,
            self.outputs(),
            self.outputs(),
        ));
        let x_path = path.clone().with_suffix(0);
        for i in 0..self.x.outputs() {
            graph.push_edge(Edge::new(
                self.x.source_edge(i, x_path.clone()),
                path.clone().with_index(i),
            ));
        }
        self.x.fill_graph(graph, x_path);
    }
}

/// Map any number of channels.
#[derive(Clone)]
pub struct Map<M, I, O> {
    f: M,
    routing: Routing,
    _marker: PhantomData<(I, O)>,
}

impl<M, I, O> Map<M, I, O>
where
    M: Fn(&Frame<f32, I>) -> O + Clone + Send + Sync,
    I: Size<f32>,
    O: ConstantFrame<Sample = f32>,
    O::Size: Size<f32>,
{
    pub fn new(f: M, routing: Routing) -> Self {
        Self {
            f,
            routing,
            _marker: PhantomData,
        }
    }
}

impl<M, I, O> AudioNode for Map<M, I, O>
where
    M: Fn(&Frame<f32, I>) -> O + Clone + Send + Sync,
    I: Size<f32>,
    O: ConstantFrame<Sample = f32>,
    O::Size: Size<f32>,
{
    const ID: u64 = 5;
    type Inputs = I;
    type Outputs = O::Size;

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        (self.f)(input).frame()
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        self.routing.route(input, O::Size::USIZE)
    }
}

/// Pipe the output of `X` to `Y`.
#[derive(Clone)]
pub struct Pipe<X, Y>
where
    X: AudioNode,
    Y: AudioNode<Inputs = X::Outputs>,
{
    x: X,
    y: Y,
}

impl<X, Y> Pipe<X, Y>
where
    X: AudioNode,
    Y: AudioNode<Inputs = X::Outputs>,
{
    pub fn new(x: X, y: Y) -> Self {
        let mut node = Pipe { x, y };
        let hash = node.ping(true, AttoHash::new(Self::ID));
        node.ping(false, hash);
        node
    }

    /// Access the left node of the pipe.
    #[inline]
    pub fn left_mut(&mut self) -> &mut X {
        &mut self.x
    }

    /// Access the left node of the pipe.
    #[inline]
    pub fn left(&self) -> &X {
        &self.x
    }

    /// Access the right node of the pipe.
    #[inline]
    pub fn right_mut(&mut self) -> &mut Y {
        &mut self.y
    }

    /// Access the right node of the pipe.
    #[inline]
    pub fn right(&self) -> &Y {
        &self.y
    }
}

impl<X, Y> AudioNode for Pipe<X, Y>
where
    X: AudioNode,
    Y: AudioNode<Inputs = X::Outputs>,
{
    const ID: u64 = 6;
    type Inputs = X::Inputs;
    type Outputs = Y::Outputs;

    fn reset(&mut self) {
        self.x.reset();
        self.y.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x.set_sample_rate(sample_rate);
        self.y.set_sample_rate(sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        self.y.tick(&self.x.tick(input))
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        let mut buffer = BufferArray::<X::Outputs>::uninitialized();
        self.x.process(size, input, &mut buffer.buffer_mut());
        self.y.process(size, &buffer.buffer_ref(), output);
    }

    fn set(&mut self, setting: Setting) {
        match setting.direction() {
            Address::Left => self.x.set(setting.peel()),
            Address::Right => self.y.set(setting.peel()),
            _ => (),
        }
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.y.ping(probe, self.x.ping(probe, hash.hash(Self::ID)))
    }

    fn allocate(&mut self) {
        self.x.allocate();
        self.y.allocate();
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.y.route(&self.x.route(input, frequency), frequency)
    }

    fn source_edge(&self, output: usize, path: Path) -> Path {
        self.y.source_edge(output, path.with_suffix(1))
    }

    fn target_edges(&self, input: usize, path: Path) -> Vec<Path> {
        self.x.target_edges(input, path.with_suffix(0))
    }

    fn fill_graph(&self, graph: &mut Graph, path: Path) {
        let x_path = path.clone().with_suffix(0);
        let y_path = path.clone().with_suffix(1);
        for i in 0..self.x.outputs() {
            graph.push_edges(
                self.x.source_edge(i, x_path.clone()),
                self.y.target_edges(i, y_path.clone()),
            );
        }
        self.x.fill_graph(graph, x_path);
        self.y.fill_graph(graph, y_path);
    }
}

/// Stack `X` and `Y` in parallel.
#[derive(Clone)]
pub struct Stack<X, Y> {
    x: X,
    y: Y,
}

impl<X, Y> Stack<X, Y>
where
    X: AudioNode,
    Y: AudioNode,
    X::Inputs: Add<Y::Inputs>,
    X::Outputs: Add<Y::Outputs>,
    <X::Inputs as Add<Y::Inputs>>::Output: Size<f32>,
    <X::Outputs as Add<Y::Outputs>>::Output: Size<f32>,
{
    pub fn new(x: X, y: Y) -> Self {
        let mut node = Stack { x, y };
        let hash = node.ping(true, AttoHash::new(Self::ID));
        node.ping(false, hash);
        node
    }

    /// Access the left node of the stack.
    #[inline]
    pub fn left_mut(&mut self) -> &mut X {
        &mut self.x
    }

    /// Access the left node of the stack.
    #[inline]
    pub fn left(&self) -> &X {
        &self.x
    }

    /// Access the right node of the stack.
    #[inline]
    pub fn right_mut(&mut self) -> &mut Y {
        &mut self.y
    }

    /// Access the right node of the stack.
    #[inline]
    pub fn right(&self) -> &Y {
        &self.y
    }
}

impl<X, Y> AudioNode for Stack<X, Y>
where
    X: AudioNode,
    Y: AudioNode,
    X::Inputs: Add<Y::Inputs>,
    X::Outputs: Add<Y::Outputs>,
    <X::Inputs as Add<Y::Inputs>>::Output: Size<f32>,
    <X::Outputs as Add<Y::Outputs>>::Output: Size<f32>,
{
    const ID: u64 = 7;
    type Inputs = Sum<X::Inputs, Y::Inputs>;
    type Outputs = Sum<X::Outputs, Y::Outputs>;

    fn reset(&mut self) {
        self.x.reset();
        self.y.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x.set_sample_rate(sample_rate);
        self.y.set_sample_rate(sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let input_x = &input[..X::Inputs::USIZE];
        let input_y = &input[X::Inputs::USIZE..];
        let output_x = self.x.tick(input_x.into());
        let output_y = self.y.tick(input_y.into());
        Frame::generate(|i| {
            if i < X::Outputs::USIZE {
                output_x[i]
            } else {
                output_y[i - X::Outputs::USIZE]
            }
        })
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        self.x.process(
            size,
            &input.subset(0, X::Inputs::USIZE),
            &mut output.subset(0, X::Outputs::USIZE),
        );
        self.y.process(
            size,
            &input.subset(X::Inputs::USIZE, Y::Inputs::USIZE),
            &mut output.subset(X::Outputs::USIZE, Y::Outputs::USIZE),
        );
    }

    fn set(&mut self, setting: Setting) {
        match setting.direction() {
            Address::Left => self.x.set(setting.peel()),
            Address::Right => self.y.set(setting.peel()),
            _ => (),
        }
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.y.ping(probe, self.x.ping(probe, hash.hash(Self::ID)))
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut signal_x = self
            .x
            .route(&SignalFrame::copy(input, 0, X::Inputs::USIZE), frequency);
        let signal_y = self.y.route(
            &SignalFrame::copy(input, X::Inputs::USIZE, Y::Inputs::USIZE),
            frequency,
        );
        signal_x.resize(self.outputs());
        for i in 0..Y::Outputs::USIZE {
            signal_x.set(X::Outputs::USIZE + i, signal_y.at(i));
        }
        signal_x
    }

    fn allocate(&mut self) {
        self.x.allocate();
        self.y.allocate();
    }

    fn source_edge(&self, output: usize, path: Path) -> Path {
        if output < X::Outputs::USIZE {
            self.x.source_edge(output, path.with_suffix(0))
        } else {
            self.y
                .source_edge(output - X::Outputs::USIZE, path.with_suffix(1))
        }
    }

    fn target_edges(&self, input: usize, path: Path) -> Vec<Path> {
        if input < X::Inputs::USIZE {
            self.x.target_edges(input, path.with_suffix(0))
        } else {
            self.y
                .target_edges(input - X::Inputs::USIZE, path.with_suffix(1))
        }
    }

    fn fill_graph(&self, graph: &mut Graph, path: Path) {
        let x_path = path.clone().with_suffix(0);
        self.x.fill_graph(graph, x_path);
        let y_path = path.clone().with_suffix(1);
        self.y.fill_graph(graph, y_path);
    }
}

/// Send the same input to `X` and `Y`. Concatenate outputs.
#[derive(Clone)]
pub struct Branch<X, Y> {
    x: X,
    y: Y,
}

impl<X, Y> Branch<X, Y>
where
    X: AudioNode,
    Y: AudioNode<Inputs = X::Inputs>,
    X::Outputs: Add<Y::Outputs>,
    <X::Outputs as Add<Y::Outputs>>::Output: Size<f32>,
{
    pub fn new(x: X, y: Y) -> Self {
        let mut node = Branch { x, y };
        let hash = node.ping(true, AttoHash::new(Self::ID));
        node.ping(false, hash);
        node
    }

    /// Access the left node of the branch.
    #[inline]
    pub fn left_mut(&mut self) -> &mut X {
        &mut self.x
    }

    /// Access the left node of the branch.
    #[inline]
    pub fn left(&self) -> &X {
        &self.x
    }

    /// Access the right node of the branch.
    #[inline]
    pub fn right_mut(&mut self) -> &mut Y {
        &mut self.y
    }

    /// Access the right node of the branch.
    #[inline]
    pub fn right(&self) -> &Y {
        &self.y
    }
}

impl<X, Y> AudioNode for Branch<X, Y>
where
    X: AudioNode,
    Y: AudioNode<Inputs = X::Inputs>,
    X::Outputs: Add<Y::Outputs>,
    <X::Outputs as Add<Y::Outputs>>::Output: Size<f32>,
{
    const ID: u64 = 8;
    type Inputs = X::Inputs;
    type Outputs = Sum<X::Outputs, Y::Outputs>;

    fn reset(&mut self) {
        self.x.reset();
        self.y.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x.set_sample_rate(sample_rate);
        self.y.set_sample_rate(sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let output_x = self.x.tick(input);
        let output_y = self.y.tick(input);
        Frame::generate(|i| {
            if i < X::Outputs::USIZE {
                output_x[i]
            } else {
                output_y[i - X::Outputs::USIZE]
            }
        })
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        self.x
            .process(size, input, &mut output.subset(0, X::Outputs::USIZE));
        self.y.process(
            size,
            input,
            &mut output.subset(X::Outputs::USIZE, Y::Outputs::USIZE),
        );
    }

    fn set(&mut self, setting: Setting) {
        match setting.direction() {
            Address::Left => self.x.set(setting.peel()),
            Address::Right => self.y.set(setting.peel()),
            _ => (),
        }
    }

    #[inline]
    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.y.ping(probe, self.x.ping(probe, hash.hash(Self::ID)))
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut signal_x = self.x.route(input, frequency);
        let signal_y = self.y.route(input, frequency);
        signal_x.resize(self.outputs());
        for i in 0..Y::Outputs::USIZE {
            signal_x.set(X::Outputs::USIZE + i, signal_y.at(i));
        }
        signal_x
    }

    fn allocate(&mut self) {
        self.x.allocate();
        self.y.allocate();
    }

    fn source_edge(&self, output: usize, path: Path) -> Path {
        if output < X::Outputs::USIZE {
            self.x.source_edge(output, path.with_suffix(0))
        } else {
            self.y
                .source_edge(output - X::Outputs::USIZE, path.with_suffix(1))
        }
    }

    fn target_edges(&self, input: usize, path: Path) -> Vec<Path> {
        let x_path = path.clone().with_suffix(0);
        let y_path = path.clone().with_suffix(1);
        let mut edges = self.x.target_edges(input, x_path);
        edges.append(&mut self.y.target_edges(input, y_path));
        edges
    }

    fn fill_graph(&self, graph: &mut Graph, path: Path) {
        let x_path = path.clone().with_suffix(0);
        self.x.fill_graph(graph, x_path);
        let y_path = path.clone().with_suffix(1);
        self.y.fill_graph(graph, y_path);
    }
}

/// Mix together `X` and `Y` sourcing from the same inputs.
#[derive(Clone)]
pub struct Bus<X, Y>
where
    X: AudioNode,
    Y: AudioNode<Inputs = X::Inputs, Outputs = X::Outputs>,
{
    x: X,
    y: Y,
}

impl<X, Y> Bus<X, Y>
where
    X: AudioNode,
    Y: AudioNode<Inputs = X::Inputs, Outputs = X::Outputs>,
{
    pub fn new(x: X, y: Y) -> Self {
        let mut node = Bus { x, y };
        let hash = node.ping(true, AttoHash::new(Self::ID));
        node.ping(false, hash);
        node
    }

    /// Access the left node of the bus.
    #[inline]
    pub fn left_mut(&mut self) -> &mut X {
        &mut self.x
    }

    /// Access the left node of the bus.
    #[inline]
    pub fn left(&self) -> &X {
        &self.x
    }

    /// Access the right node of the bus.
    #[inline]
    pub fn right_mut(&mut self) -> &mut Y {
        &mut self.y
    }

    /// Access the right node of the bus.
    #[inline]
    pub fn right(&self) -> &Y {
        &self.y
    }
}

impl<X, Y> AudioNode for Bus<X, Y>
where
    X: AudioNode,
    Y: AudioNode<Inputs = X::Inputs, Outputs = X::Outputs>,
{
    const ID: u64 = 10;
    type Inputs = X::Inputs;
    type Outputs = X::Outputs;

    fn reset(&mut self) {
        self.x.reset();
        self.y.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x.set_sample_rate(sample_rate);
        self.y.set_sample_rate(sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let output_x = self.x.tick(input);
        let output_y = self.y.tick(input);
        output_x + output_y
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        let mut buffer = BufferArray::<X::Outputs>::uninitialized();
        self.x.process(size, input, output);
        self.y.process(size, input, &mut buffer.buffer_mut());
        for channel in 0..self.outputs() {
            for i in 0..simd_items(size) {
                output.add(channel, i, buffer.at(channel, i));
            }
        }
    }

    fn set(&mut self, setting: Setting) {
        match setting.direction() {
            Address::Left => self.x.set(setting.peel()),
            Address::Right => self.y.set(setting.peel()),
            _ => (),
        }
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.y.ping(probe, self.x.ping(probe, hash.hash(Self::ID)))
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut signal_x = self.x.route(input, frequency);
        let signal_y = self.y.route(input, frequency);
        for i in 0..Self::Outputs::USIZE {
            signal_x.set(
                i,
                signal_x
                    .at(i)
                    .combine_linear(signal_y.at(i), 0.0, |x, y| x + y, |x, y| x + y),
            );
        }
        signal_x
    }

    fn allocate(&mut self) {
        self.x.allocate();
        self.y.allocate();
    }

    fn source_edge(&self, output: usize, path: Path) -> Path {
        path.with_index(output)
    }

    fn target_edges(&self, input: usize, path: Path) -> Vec<Path> {
        let x_path = path.clone().with_suffix(0);
        let y_path = path.clone().with_suffix(1);
        let mut edges = self.x.target_edges(input, x_path);
        edges.append(&mut self.y.target_edges(input, y_path));
        edges
    }

    fn fill_graph(&self, graph: &mut Graph, path: Path) {
        graph.push_node(Node::new(
            path.clone(),
            Self::ID,
            self.outputs(),
            self.outputs(),
        ));
        let x_path = path.clone().with_suffix(0);
        for x_output in 0..self.x.outputs() {
            graph.push_edge(Edge::new(
                self.x.source_edge(x_output, x_path.clone()),
                path.clone().with_index(x_output),
            ));
        }
        self.x.fill_graph(graph, x_path);
        let y_path = path.clone().with_suffix(1);
        for y_output in 0..self.y.outputs() {
            graph.push_edge(Edge::new(
                self.y.source_edge(y_output, y_path.clone()),
                path.clone().with_index(y_output),
            ));
        }
        self.y.fill_graph(graph, y_path);
    }
}

/// Pass through inputs without matching outputs.
/// Adjusts output arity to match input arity, adapting a filter to a pipeline.
#[derive(Clone)]
pub struct Thru<X: AudioNode> {
    x: X,
}

impl<X: AudioNode> Thru<X> {
    pub fn new(x: X) -> Self {
        let mut node = Thru { x };
        let hash = node.ping(true, AttoHash::new(Self::ID));
        node.ping(false, hash);
        node
    }
}

impl<X: AudioNode> AudioNode for Thru<X> {
    const ID: u64 = 12;
    type Inputs = X::Inputs;
    type Outputs = X::Inputs;

    fn reset(&mut self) {
        self.x.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x.set_sample_rate(sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let output = self.x.tick(input);
        Frame::generate(|channel| {
            if channel < X::Outputs::USIZE {
                output[channel]
            } else {
                input[channel]
            }
        })
    }
    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        if X::Inputs::USIZE == 0 {
            // This is an empty node.
            return;
        }
        if X::Inputs::USIZE < X::Outputs::USIZE {
            // An intermediate buffer is only used in this "degenerate" case where
            // we are not passing through inputs - we are cutting out some of them.
            let mut buffer = BufferArray::<X::Outputs>::uninitialized();
            self.x.process(size, input, &mut buffer.buffer_mut());
            for channel in 0..X::Inputs::USIZE {
                for i in 0..simd_items(size) {
                    output.set(channel, i, buffer.at(channel, i));
                }
            }
        } else {
            self.x
                .process(size, input, &mut output.subset(0, X::Outputs::USIZE));
            for channel in X::Outputs::USIZE..X::Inputs::USIZE {
                for i in 0..simd_items(size) {
                    output.set(channel, i, input.at(channel, i));
                }
            }
        }
    }

    fn set(&mut self, setting: Setting) {
        self.x.set(setting);
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.x.ping(probe, hash.hash(Self::ID))
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = self.x.route(input, frequency);
        output.resize(self.outputs());
        for i in X::Outputs::USIZE..Self::Outputs::USIZE {
            output.set(i, input.at(i));
        }
        output
    }

    fn allocate(&mut self) {
        self.x.allocate();
    }

    fn source_edge(&self, output: usize, path: Path) -> Path {
        if output < self.x.outputs() {
            let x_path = path.clone().with_suffix(0);
            self.x.source_edge(output, x_path)
        } else {
            path.with_index(output)
        }
    }

    fn target_edges(&self, input: usize, path: Path) -> Vec<Path> {
        let x_path = path.clone().with_suffix(0);
        let mut edges = self.x.target_edges(input, x_path);
        edges.push(path.with_index(input));
        edges
    }

    fn fill_graph(&self, graph: &mut Graph, path: Path) {
        graph.push_node(Node::new(
            path.clone(),
            Self::ID,
            self.inputs(),
            self.outputs(),
        ));
        let x_path = path.clone().with_suffix(0);
        self.x.fill_graph(graph, x_path);
    }
}

/// Mix together a bunch of similar nodes sourcing from the same inputs.
#[derive(Clone)]
pub struct MultiBus<N, X>
where
    N: Size<f32> + Size<X>,
    X: AudioNode,
{
    x: Frame<X, N>,
}

impl<N, X> MultiBus<N, X>
where
    N: Size<f32> + Size<X>,
    X: AudioNode,
{
    pub fn new(x: Frame<X, N>) -> Self {
        let mut node = MultiBus { x };
        let hash = node.ping(true, AttoHash::new(Self::ID));
        node.ping(false, hash);
        node
    }

    /// Access a contained node.
    #[inline]
    pub fn node_mut(&mut self, index: usize) -> &mut X {
        &mut self.x[index]
    }

    /// Access a contained node.
    #[inline]
    pub fn node(&self, index: usize) -> &X {
        &self.x[index]
    }
}

impl<N, X> AudioNode for MultiBus<N, X>
where
    N: Size<f32> + Size<X>,
    X: AudioNode,
{
    const ID: u64 = 28;
    type Inputs = X::Inputs;
    type Outputs = X::Outputs;

    fn reset(&mut self) {
        self.x.iter_mut().for_each(|node| node.reset());
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x
            .iter_mut()
            .for_each(|node| node.set_sample_rate(sample_rate));
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        self.x
            .iter_mut()
            .fold(Frame::splat(0.0), |acc, x| acc + x.tick(input))
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        let mut buffer = BufferArray::<X::Outputs>::uninitialized();
        self.x[0].process(size, input, output);
        for i in 1..N::USIZE {
            self.x[i].process(size, input, &mut buffer.buffer_mut());
            for channel in 0..X::Outputs::USIZE {
                for j in 0..simd_items(size) {
                    output.add(channel, j, buffer.at(channel, j));
                }
            }
        }
    }

    fn set(&mut self, setting: Setting) {
        if let Address::Index(index) = setting.direction() {
            self.x[index].set(setting.peel());
        }
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        let mut hash = hash.hash(Self::ID);
        for x in &mut self.x {
            hash = x.ping(probe, hash);
        }
        hash
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = self.x[0].route(input, frequency);
        for i in 1..self.x.len() {
            let output_i = self.x[i].route(input, frequency);
            for channel in 0..Self::Outputs::USIZE {
                output.set(
                    channel,
                    output.at(channel).combine_linear(
                        output_i.at(channel),
                        0.0,
                        |x, y| x + y,
                        |x, y| x + y,
                    ),
                );
            }
        }
        output
    }

    fn allocate(&mut self) {
        for x in &mut self.x {
            x.allocate();
        }
    }

    fn source_edge(&self, output: usize, path: Path) -> Path {
        path.with_index(output)
    }

    fn target_edges(&self, input: usize, path: Path) -> Vec<Path> {
        let mut edges = Vec::new();
        for x_index in 0..N::USIZE {
            let x_path = path.clone().with_suffix(x_index as u32);
            edges.append(&mut self.x[x_index].target_edges(input, x_path));
        }
        edges
    }

    fn fill_graph(&self, graph: &mut Graph, path: Path) {
        graph.push_node(Node::new(
            path.clone(),
            Self::ID,
            self.outputs(),
            self.outputs(),
        ));
        for x_index in 0..N::USIZE {
            let x_path = path.clone().with_suffix(x_index as u32);
            self.x[x_index].fill_graph(graph, x_path.clone());
            for output in 0..self.x[x_index].outputs() {
                graph.push_edge(Edge::new(
                    self.x[x_index].source_edge(output, x_path.clone()),
                    path.clone().with_index(output),
                ));
            }
        }
    }
}

/// Stack a bunch of similar nodes in parallel.
#[derive(Clone)]
pub struct MultiStack<N, X>
where
    N: Size<f32> + Size<X>,
    X: AudioNode,
    X::Inputs: Size<f32> + Mul<N>,
    X::Outputs: Size<f32> + Mul<N>,
    <X::Inputs as Mul<N>>::Output: Size<f32>,
    <X::Outputs as Mul<N>>::Output: Size<f32>,
{
    _marker: PhantomData<N>,
    x: Frame<X, N>,
}

impl<N, X> MultiStack<N, X>
where
    N: Size<f32> + Size<X>,
    X: AudioNode,
    X::Inputs: Size<f32> + Mul<N>,
    X::Outputs: Size<f32> + Mul<N>,
    <X::Inputs as Mul<N>>::Output: Size<f32>,
    <X::Outputs as Mul<N>>::Output: Size<f32>,
{
    pub fn new(x: Frame<X, N>) -> Self {
        let mut node = MultiStack {
            _marker: PhantomData,
            x,
        };
        let hash = node.ping(true, AttoHash::new(Self::ID));
        node.ping(false, hash);
        node
    }

    /// Access a contained node.
    #[inline]
    pub fn node_mut(&mut self, index: usize) -> &mut X {
        &mut self.x[index]
    }

    /// Access a contained node.
    #[inline]
    pub fn node(&self, index: usize) -> &X {
        &self.x[index]
    }
}

impl<N, X> AudioNode for MultiStack<N, X>
where
    N: Size<f32> + Size<X>,
    X: AudioNode,
    X::Inputs: Size<f32> + Mul<N>,
    X::Outputs: Size<f32> + Mul<N>,
    <X::Inputs as Mul<N>>::Output: Size<f32>,
    <X::Outputs as Mul<N>>::Output: Size<f32>,
{
    const ID: u64 = 30;
    type Inputs = Prod<X::Inputs, N>;
    type Outputs = Prod<X::Outputs, N>;

    fn reset(&mut self) {
        self.x.iter_mut().for_each(|node| node.reset());
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x
            .iter_mut()
            .for_each(|node| node.set_sample_rate(sample_rate));
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let mut output: Frame<f32, Self::Outputs> = Frame::splat(0.0);
        for (i, node) in self.x.iter_mut().enumerate() {
            let node_input = &input[i * X::Inputs::USIZE..(i + 1) * X::Inputs::USIZE];
            let node_output = node.tick(node_input.into());
            output[i * X::Outputs::USIZE..(i + 1) * X::Outputs::USIZE]
                .copy_from_slice(node_output.as_slice());
        }
        output
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        let mut in_channel = 0;
        let mut out_channel = 0;
        for i in 0..N::USIZE {
            self.x[i].process(
                size,
                &input.subset(in_channel, X::Inputs::USIZE),
                &mut output.subset(out_channel, X::Outputs::USIZE),
            );
            in_channel += X::Inputs::USIZE;
            out_channel += X::Outputs::USIZE;
        }
    }

    fn set(&mut self, setting: Setting) {
        if let Address::Index(index) = setting.direction() {
            self.x[index].set(setting.peel());
        }
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        let mut hash = hash.hash(Self::ID);
        for x in self.x.iter_mut() {
            hash = x.ping(probe, hash);
        }
        hash
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        if self.x.is_empty() {
            return SignalFrame::new(self.outputs());
        }
        let mut output = self.x[0].route(input, frequency);
        output.resize(self.outputs());
        for i in 1..N::USIZE {
            let output_i = self.x[i].route(
                &SignalFrame::copy(input, i * X::Inputs::USIZE, X::Inputs::USIZE),
                frequency,
            );
            for channel in 0..X::Outputs::USIZE {
                output.set(channel + i * X::Outputs::USIZE, output_i.at(channel));
            }
        }
        output
    }

    fn allocate(&mut self) {
        for x in &mut self.x {
            x.allocate();
        }
    }

    fn source_edge(&self, output: usize, path: Path) -> Path {
        let x_index = output / X::Outputs::USIZE;
        let x_output = output % X::Outputs::USIZE;
        self.x[x_index].source_edge(x_output, path.with_suffix(x_index as u32))
    }

    fn target_edges(&self, input: usize, path: Path) -> Vec<Path> {
        let x_index = input / X::Inputs::USIZE;
        let x_input = input % X::Inputs::USIZE;
        self.x[x_index].target_edges(x_input, path.with_suffix(x_index as u32))
    }

    fn fill_graph(&self, graph: &mut Graph, path: Path) {
        for x_index in 0..N::USIZE {
            self.x[x_index].fill_graph(graph, path.clone().with_suffix(x_index as u32));
        }
    }
}

/// Combine outputs of a bunch of similar nodes with a binary operation.
/// Inputs are disjoint.
/// Outputs are combined channel-wise.
#[derive(Clone)]
pub struct Reduce<N, X, B>
where
    N: Size<f32> + Size<X>,
    X: AudioNode,
    X::Inputs: Mul<N>,
    <X::Inputs as Mul<N>>::Output: Size<f32>,
    B: FrameBinop<X::Outputs>,
{
    x: Frame<X, N>,
    b: B,
}

impl<N, X, B> Reduce<N, X, B>
where
    N: Size<f32> + Size<X>,
    X: AudioNode,
    X::Inputs: Mul<N>,
    <X::Inputs as Mul<N>>::Output: Size<f32>,
    B: FrameBinop<X::Outputs>,
{
    pub fn new(x: Frame<X, N>, b: B) -> Self {
        let mut node = Reduce { x, b };
        let hash = node.ping(true, AttoHash::new(Self::ID));
        node.ping(false, hash);
        node
    }

    /// Access a contained node.
    #[inline]
    pub fn node_mut(&mut self, index: usize) -> &mut X {
        &mut self.x[index]
    }

    /// Access a contained node.
    #[inline]
    pub fn node(&self, index: usize) -> &X {
        &self.x[index]
    }
}

impl<N, X, B> AudioNode for Reduce<N, X, B>
where
    N: Size<f32> + Size<X>,
    X: AudioNode,
    X::Inputs: Mul<N>,
    <X::Inputs as Mul<N>>::Output: Size<f32>,
    B: FrameBinop<X::Outputs>,
{
    const ID: u64 = 31;
    type Inputs = Prod<X::Inputs, N>;
    type Outputs = X::Outputs;

    fn reset(&mut self) {
        self.x.iter_mut().for_each(|node| node.reset());
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x
            .iter_mut()
            .for_each(|node| node.set_sample_rate(sample_rate));
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let mut output: Frame<f32, Self::Outputs> = Frame::splat(0.0);
        for (i, node) in self.x.iter_mut().enumerate() {
            let node_input = &input[i * X::Inputs::USIZE..(i + 1) * X::Inputs::USIZE];
            let node_output = node.tick(node_input.into());
            if i > 0 {
                output = self.b.frame(&output, &node_output);
            } else {
                output = node_output;
            }
        }
        output
    }
    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        let mut buffer = BufferArray::<X::Outputs>::uninitialized();
        self.x[0].process(size, &input.subset(0, X::Inputs::USIZE), output);
        let mut in_channel = X::Inputs::USIZE;
        for i in 1..N::USIZE {
            self.x[i].process(
                size,
                &input.subset(in_channel, X::Inputs::USIZE),
                &mut buffer.buffer_mut(),
            );
            in_channel += X::Inputs::USIZE;
            for channel in 0..X::Outputs::USIZE {
                for j in 0..simd_items(size) {
                    output.set(
                        channel,
                        j,
                        self.b.binop(output.at(channel, j), buffer.at(channel, j)),
                    );
                }
            }
        }
    }

    fn set(&mut self, setting: Setting) {
        if let Address::Index(index) = setting.direction() {
            self.x[index].set(setting.peel());
        }
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        let mut hash = hash.hash(Self::ID);
        for x in self.x.iter_mut() {
            hash = x.ping(probe, hash);
        }
        hash
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = self.x[0].route(input, frequency);
        for j in 1..self.x.len() {
            let output_j = self.x[j].route(
                &SignalFrame::copy(input, j * X::Inputs::USIZE, X::Inputs::USIZE),
                frequency,
            );
            for i in 0..Self::Outputs::USIZE {
                output.set(i, self.b.route(output.at(i), output_j.at(i)));
            }
        }
        output
    }

    fn allocate(&mut self) {
        for x in &mut self.x {
            x.allocate();
        }
    }

    fn source_edge(&self, output: usize, path: Path) -> Path {
        path.with_index(output)
    }

    fn target_edges(&self, input: usize, path: Path) -> Vec<Path> {
        let x_index = input / X::Inputs::USIZE;
        let x_input = input % X::Inputs::USIZE;
        self.x[x_index].target_edges(x_input, path.with_suffix(x_index as u32))
    }

    fn fill_graph(&self, graph: &mut Graph, path: Path) {
        graph.push_node(Node::new(
            path.clone(),
            Self::ID,
            self.outputs(),
            self.outputs(),
        ));
        let mut x_path = path.clone().with_suffix(0);
        for x_index in 0..N::USIZE {
            x_path.set_suffix(x_index as u32);
            self.x[x_index].fill_graph(graph, x_path.clone());
            for output in 0..X::Outputs::USIZE {
                graph.push_edge(Edge::new(
                    self.x[x_index].source_edge(output, x_path.clone()),
                    path.clone().with_index(output),
                ));
            }
        }
    }
}

/// Branch into a bunch of similar nodes in parallel.
#[derive(Clone)]
pub struct MultiBranch<N, X>
where
    N: Size<f32> + Size<X>,
    X: AudioNode,
    X::Outputs: Mul<N>,
    <X::Outputs as Mul<N>>::Output: Size<f32>,
{
    x: Frame<X, N>,
}

impl<N, X> MultiBranch<N, X>
where
    N: Size<f32> + Size<X>,
    X: AudioNode,
    X::Outputs: Mul<N>,
    <X::Outputs as Mul<N>>::Output: Size<f32>,
{
    pub fn new(x: Frame<X, N>) -> Self {
        let mut node = MultiBranch { x };
        let hash = node.ping(true, AttoHash::new(Self::ID));
        node.ping(false, hash);
        node
    }

    /// Access a contained node.
    #[inline]
    pub fn node_mut(&mut self, index: usize) -> &mut X {
        &mut self.x[index]
    }

    /// Access a contained node.
    #[inline]
    pub fn node(&self, index: usize) -> &X {
        &self.x[index]
    }
}

impl<N, X> AudioNode for MultiBranch<N, X>
where
    N: Size<f32> + Size<X>,
    X: AudioNode,
    X::Outputs: Mul<N>,
    <X::Outputs as Mul<N>>::Output: Size<f32>,
{
    const ID: u64 = 33;
    type Inputs = X::Inputs;
    type Outputs = Prod<X::Outputs, N>;

    fn reset(&mut self) {
        self.x.iter_mut().for_each(|node| node.reset());
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x
            .iter_mut()
            .for_each(|node| node.set_sample_rate(sample_rate));
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let mut output: Frame<f32, Self::Outputs> = Frame::splat(0.0);
        for (i, node) in self.x.iter_mut().enumerate() {
            let node_output = node.tick(input);
            output[i * X::Outputs::USIZE..(i + 1) * X::Outputs::USIZE]
                .copy_from_slice(node_output.as_slice());
        }
        output
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        let mut out_channel = 0;
        for i in 0..N::USIZE {
            self.x[i].process(
                size,
                input,
                &mut output.subset(out_channel, X::Outputs::USIZE),
            );
            out_channel += X::Outputs::USIZE;
        }
    }

    fn set(&mut self, setting: Setting) {
        if let Address::Index(index) = setting.direction() {
            self.x[index].set(setting.peel());
        }
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        let mut hash = hash.hash(Self::ID);
        for x in self.x.iter_mut() {
            hash = x.ping(probe, hash);
        }
        hash
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        if self.x.is_empty() {
            return SignalFrame::new(self.outputs());
        }
        let mut output = self.x[0].route(input, frequency);
        output.resize(self.outputs());
        for i in 1..N::USIZE {
            let output_i = self.x[i].route(input, frequency);
            for j in 0..X::Outputs::USIZE {
                output.set(i * X::Outputs::USIZE + j, output_i.at(j));
            }
        }
        output
    }

    fn allocate(&mut self) {
        for x in &mut self.x {
            x.allocate();
        }
    }

    fn source_edge(&self, output: usize, path: Path) -> Path {
        let x_index = output / X::Outputs::USIZE;
        let x_output = output % X::Outputs::USIZE;
        self.x[x_index].source_edge(x_output, path.with_suffix(x_index as u32))
    }

    fn target_edges(&self, input: usize, path: Path) -> Vec<Path> {
        let mut edges = Vec::new();
        for x_index in 0..N::USIZE {
            let x_path = path.clone().with_suffix(x_index as u32);
            edges.append(&mut self.x[x_index].target_edges(input, x_path));
        }
        edges
    }

    fn fill_graph(&self, graph: &mut Graph, path: Path) {
        for x_index in 0..N::USIZE {
            let x_path = path.clone().with_suffix(x_index as u32);
            self.x[x_index].fill_graph(graph, x_path);
        }
    }
}

/// A pipeline of multiple nodes.
#[derive(Clone)]
pub struct Chain<N, X>
where
    N: Size<f32> + Size<X>,
    X: AudioNode,
{
    x: Frame<X, N>,
}

impl<N, X> Chain<N, X>
where
    N: Size<f32> + Size<X>,
    X: AudioNode,
{
    pub fn new(x: Frame<X, N>) -> Self {
        // TODO. We'd like to require statically that X::Inputs equals X::Outputs
        // but I don't know how to write such a trait bound.
        assert_eq!(x[0].inputs(), x[0].outputs());
        let mut node = Chain { x };
        let hash = node.ping(true, AttoHash::new(Self::ID));
        node.ping(false, hash);
        node
    }

    /// Access a contained node.
    #[inline]
    pub fn node_mut(&mut self, index: usize) -> &mut X {
        &mut self.x[index]
    }

    /// Access a contained node.
    #[inline]
    pub fn node(&self, index: usize) -> &X {
        &self.x[index]
    }
}

impl<N, X> AudioNode for Chain<N, X>
where
    N: Size<f32> + Size<X>,
    X: AudioNode,
{
    const ID: u64 = 32;
    type Inputs = X::Inputs;
    type Outputs = X::Outputs;

    fn reset(&mut self) {
        self.x.iter_mut().for_each(|node| node.reset());
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x
            .iter_mut()
            .for_each(|node| node.set_sample_rate(sample_rate));
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let mut output = self.x[0].tick(input);
        for i in 1..N::USIZE {
            output = self.x[i].tick(&Frame::generate(|i| output[i]));
        }
        output
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        let mut buffer = BufferArray::<X::Outputs>::uninitialized();
        if N::USIZE & 1 > 0 {
            self.x[0].process(size, input, output);
        } else {
            self.x[0].process(size, input, &mut buffer.buffer_mut());
        }
        for i in 1..N::USIZE {
            if (N::USIZE ^ i) & 1 > 0 {
                self.x[i].process(size, &buffer.buffer_ref(), output);
            } else {
                self.x[i].process(size, &output.buffer_ref(), &mut buffer.buffer_mut());
            }
        }
    }

    fn set(&mut self, setting: Setting) {
        if let Address::Index(index) = setting.direction() {
            self.x[index].set(setting.peel());
        }
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        let mut hash = hash.hash(Self::ID);
        for x in self.x.iter_mut() {
            hash = x.ping(probe, hash);
        }
        hash
    }

    fn allocate(&mut self) {
        for x in &mut self.x {
            x.allocate();
        }
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = self.x[0].route(input, frequency);
        for i in 1..self.x.len() {
            output = self.x[i].route(&output, frequency);
        }
        output
    }

    fn source_edge(&self, output: usize, path: Path) -> Path {
        self.x[N::USIZE - 1].source_edge(output, path.with_suffix(N::U32 - 1))
    }

    fn target_edges(&self, input: usize, path: Path) -> Vec<Path> {
        self.x[0].target_edges(input, path.with_suffix(0))
    }

    fn fill_graph(&self, graph: &mut Graph, path: Path) {
        for x_index in 0..N::USIZE {
            let x_path = path.clone().with_suffix(x_index as u32);
            self.x[x_index].fill_graph(graph, x_path.clone());
            if x_index + 1 < N::USIZE {
                let x_1_path = path.clone().with_suffix(x_index as u32 + 1);
                for x_output in 0..X::Outputs::USIZE {
                    graph.push_edges(
                        self.x[x_index].source_edge(x_output, x_path.clone()),
                        self.x[x_index + 1].target_edges(x_output, x_1_path.clone()),
                    );
                }
            }
        }
    }
}

/// Reverse channel order.
#[derive(Default, Clone)]
pub struct Reverse<N> {
    _marker: PhantomData<N>,
}

impl<N: Size<f32>> Reverse<N> {
    pub fn new() -> Self {
        Reverse::default()
    }
}

impl<N: Size<f32>> AudioNode for Reverse<N> {
    const ID: u64 = 45;
    type Inputs = N;
    type Outputs = N;

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        Frame::generate(|i| input[N::USIZE - 1 - i])
    }
    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        for channel in 0..N::USIZE {
            for i in 0..simd_items(size) {
                output.set(channel, i, input.at(N::USIZE - 1 - channel, i));
            }
        }
    }
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Reverse.route(input, N::USIZE)
    }
}

/// `N`-channel impulse. First sample on each channel is one, the rest are zero.
#[derive(Default, Clone)]
pub struct Impulse<N: Size<f32>> {
    value: f32,
    _marker: PhantomData<N>,
}

impl<N: Size<f32>> Impulse<N> {
    pub fn new() -> Self {
        Self {
            value: 1.0,
            _marker: PhantomData,
        }
    }
}

impl<N: Size<f32>> AudioNode for Impulse<N> {
    const ID: u64 = 81;
    type Inputs = U0;
    type Outputs = N;

    fn reset(&mut self) {
        self.value = 1.0;
    }

    #[inline]
    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let output = Frame::splat(self.value);
        self.value = 0.0;
        output
    }
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Generator(0.0).route(input, N::USIZE)
    }
}
