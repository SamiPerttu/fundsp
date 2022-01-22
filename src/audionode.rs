use super::buffer::*;
use super::math::*;
use super::signal::*;
use super::*;
use num_complex::Complex64;
use numeric_array::typenum::*;
use std::marker::PhantomData;

/// Type-level integer.
pub trait Size<T>: numeric_array::ArrayLength<T> {}

impl<T, A: numeric_array::ArrayLength<T>> Size<T> for A {}

/// Frames are used to transport audio data between `AudioNode` instances.
pub type Frame<T, Size> = numeric_array::NumericArray<T, Size>;

/// Generic audio processor.
/// `AudioNode` has a static number of inputs (`AudioNode::Inputs`) and outputs (`AudioNode::Outputs`).
/// `AudioNode` processes samples of type `AudioNode::Sample`, chosen statically.
pub trait AudioNode {
    /// Unique ID for hashing.
    const ID: u64;
    /// Sample type for input and output.
    type Sample: Float;
    /// Input arity.
    type Inputs: Size<Self::Sample>;
    /// Output arity.
    type Outputs: Size<Self::Sample>;

    /// Reset the input state of the component to an initial state where it has
    /// not processed any samples. In other words, reset time to zero.
    /// The sample rate can be set optionally. The default sample rate is 44.1 kHz.
    fn reset(&mut self, _sample_rate: Option<f64>) {}

    /// Process one sample.
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs>;

    /// Process up to 64 (`MAX_BUFFER_SIZE`) samples.
    /// The number of input and output buffers must match the number of inputs and outputs, respectively.
    /// All input and output buffers must be at least as large as `size`.
    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        debug_assert!(size <= MAX_BUFFER_SIZE);
        debug_assert!(input.len() == self.inputs());
        debug_assert!(output.len() == self.outputs());
        debug_assert!(input.iter().all(|x| x.len() >= size));
        debug_assert!(output.iter().all(|x| x.len() >= size));
        for i in 0..size {
            let result = self.tick(&Frame::generate(|j| input[j][i]));
            for (j, &x) in result.iter().enumerate() {
                output[j][i] = x;
            }
        }
    }

    /// Set node hash. Override this to use the hash.
    /// This is called from `ping`. It should not be called by users.
    fn set_hash(&mut self, _hash: u32) {}

    /// Ping contained `AudioNode`s to obtain a deterministic pseudorandom hash.
    /// The local hash includes children, too.
    /// Leaf nodes should not need to override this.
    /// If `probe` is true, then this is a probe for computing the network hash
    /// and `set_hash` should not be called yet.
    fn ping(&mut self, probe: bool, hash: AttoRand) -> AttoRand {
        if !probe {
            self.set_hash(hash.value());
        }
        hash.hash(Self::ID)
    }

    /// Propagate constants, latencies and frequency responses at `frequency`.
    /// Return output signal. Default implementation marks all outputs with zero latencies.
    fn propagate(&self, _input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut frame = new_signal_frame(self.outputs());
        for i in 0..self.outputs() {
            frame[i] = Signal::Latency(0.0)
        }
        frame
    }

    // End of interface. There is no need to override the following.

    /// Evaluate frequency response at `output`. Any linear response can be composed.
    /// Return `None` if there is no response or it could not be calculated.
    fn response(&self, output: usize, frequency: f64) -> Option<Complex64> {
        assert!(output < Self::Outputs::USIZE);
        let mut input = new_signal_frame(self.inputs());
        for i in 0..Self::Inputs::USIZE {
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
        assert!(output < Self::Outputs::USIZE);
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
        for i in 0..Self::Inputs::USIZE {
            input[i] = Signal::Latency(0.0);
        }
        // The frequency argument can be anything as there are no responses to propagate, only latencies.
        let response = self.propagate(&input, 1.0);
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

    /// Number of inputs.
    #[inline]
    fn inputs(&self) -> usize {
        Self::Inputs::USIZE
    }

    /// Number of outputs.
    #[inline]
    fn outputs(&self) -> usize {
        Self::Outputs::USIZE
    }

    /// Retrieve the next mono sample from a zero input.
    /// The node must have exactly 1 output.
    #[inline]
    fn get_mono(&mut self) -> Self::Sample {
        // TODO. Is there some way to make this constraint static.
        assert!(Self::Outputs::USIZE == 1);
        let output = self.tick(&Frame::default());
        output[0]
    }

    /// Retrieve the next stereo sample (left, right) from a zero input.
    /// The node must have 1 or 2 outputs. If there is just one output, duplicate it.
    #[inline]
    fn get_stereo(&mut self) -> (Self::Sample, Self::Sample) {
        assert!(Self::Outputs::USIZE == 1 || Self::Outputs::USIZE == 2);
        let output = self.tick(&Frame::default());
        (
            output[0],
            output[if Self::Outputs::USIZE > 1 { 1 } else { 0 }],
        )
    }

    /// Filter the next mono sample `x`.
    /// The node must have exactly 1 input and 1 output.
    #[inline]
    fn filter_mono(&mut self, x: Self::Sample) -> Self::Sample {
        assert!(Self::Inputs::USIZE == 1 && Self::Outputs::USIZE == 1);
        let output = self.tick(&Frame::splat(x));
        output[0]
    }

    /// Filter the next stereo sample `(x, y)`.
    /// The node must have exactly 2 inputs and 2 outputs.
    #[inline]
    fn filter_stereo(&mut self, x: Self::Sample, y: Self::Sample) -> (Self::Sample, Self::Sample) {
        assert!(Self::Inputs::USIZE == 2 && Self::Outputs::USIZE == 2);
        let output = self.tick(&Frame::generate(|i| if i == 0 { x } else { y }));
        (output[0], output[1])
    }
}

/// Pass through inputs unchanged.
#[derive(Clone, Default)]
pub struct PassNode<T, N> {
    _marker: PhantomData<(T, N)>,
}

impl<T: Float, N: Size<T>> PassNode<T, N> {
    pub fn new() -> Self {
        PassNode::default()
    }
}

impl<T: Float, N: Size<T>> AudioNode for PassNode<T, N> {
    const ID: u64 = 0;
    type Sample = T;
    type Inputs = N;
    type Outputs = N;

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        input.clone()
    }
    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        for i in 0..self.inputs() {
            for j in 0..size {
                output[i][j] = input[i][j];
            }
        }
    }
    fn propagate(&self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        input.clone()
    }
}

/// Consume inputs.
#[derive(Clone, Default)]
pub struct SinkNode<T, N> {
    _marker: PhantomData<(T, N)>,
}

impl<T: Float, N: Size<T>> SinkNode<T, N> {
    pub fn new() -> Self {
        SinkNode::default()
    }
}

impl<T: Float, N: Size<T>> AudioNode for SinkNode<T, N> {
    const ID: u64 = 1;
    type Sample = T;
    type Inputs = N;
    type Outputs = U0;

    #[inline]
    fn tick(
        &mut self,
        _input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        Frame::default()
    }
    fn process(
        &mut self,
        _size: usize,
        _input: &[&[Self::Sample]],
        _output: &mut [&mut [Self::Sample]],
    ) {
    }
}

/// Output a constant value.
#[derive(Clone)]
pub struct ConstantNode<T: Float, N: Size<T>> {
    output: Frame<T, N>,
}

impl<T: Float, N: Size<T>> ConstantNode<T, N> {
    pub fn new(output: Frame<T, N>) -> Self {
        ConstantNode { output }
    }
}

impl<T: Float, N: Size<T>> AudioNode for ConstantNode<T, N> {
    const ID: u64 = 2;
    type Sample = T;
    type Inputs = U0;
    type Outputs = N;

    #[inline]
    fn tick(
        &mut self,
        _input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        self.output.clone()
    }
    fn process(
        &mut self,
        size: usize,
        _input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        for i in 0..self.outputs() {
            for j in 0..size {
                output[i][j] = self.output[i];
            }
        }
    }
    fn propagate(&self, _input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        for i in 0..N::USIZE {
            output[i] = Signal::Value(self.output[i].to_f64());
        }
        output
    }
}

pub trait FrameBinop<T: Float, N: Size<T>>: Clone {
    fn binop(x: &Frame<T, N>, y: &Frame<T, N>) -> Frame<T, N>;
    fn propagate(x: Signal, y: Signal) -> Signal;
    /// Do binary op (x op y) in-place.
    fn assign(size: usize, x: &mut [T], y: &[T]);
}
#[derive(Clone, Default)]
pub struct FrameAdd<T: Float, N: Size<T>> {
    _marker: PhantomData<(T, N)>,
}

impl<T: Float, N: Size<T>> FrameAdd<T, N> {
    pub fn new() -> FrameAdd<T, N> {
        FrameAdd::default()
    }
}

impl<T: Float, N: Size<T>> FrameBinop<T, N> for FrameAdd<T, N> {
    #[inline]
    fn binop(x: &Frame<T, N>, y: &Frame<T, N>) -> Frame<T, N> {
        x + y
    }
    #[inline]
    fn propagate(x: Signal, y: Signal) -> Signal {
        combine_linear(x, y, 0.0, |x, y| x + y, |x, y| x + y)
    }
    fn assign(size: usize, x: &mut [T], y: &[T]) {
        for i in 0..size {
            x[i] += y[i];
        }
    }
}

#[derive(Clone, Default)]
pub struct FrameSub<T: Float, N: Size<T>> {
    _marker: PhantomData<(T, N)>,
}

impl<T: Float, N: Size<T>> FrameSub<T, N> {
    pub fn new() -> FrameSub<T, N> {
        FrameSub::default()
    }
}

impl<T: Float, N: Size<T>> FrameBinop<T, N> for FrameSub<T, N> {
    #[inline]
    fn binop(x: &Frame<T, N>, y: &Frame<T, N>) -> Frame<T, N> {
        x - y
    }
    #[inline]
    fn propagate(x: Signal, y: Signal) -> Signal {
        combine_linear(x, y, 0.0, |x, y| x - y, |x, y| x - y)
    }
    fn assign(size: usize, x: &mut [T], y: &[T]) {
        for i in 0..size {
            x[i] -= y[i];
        }
    }
}

#[derive(Clone, Default)]
pub struct FrameMul<T: Float, N: Size<T>> {
    _marker: PhantomData<(T, N)>,
}

impl<T: Float, N: Size<T>> FrameMul<T, N> {
    pub fn new() -> FrameMul<T, N> {
        FrameMul::default()
    }
}

impl<T: Float, N: Size<T>> FrameBinop<T, N> for FrameMul<T, N> {
    #[inline]
    fn binop(x: &Frame<T, N>, y: &Frame<T, N>) -> Frame<T, N> {
        x * y
    }
    #[inline]
    fn propagate(x: Signal, y: Signal) -> Signal {
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
    fn assign(size: usize, x: &mut [T], y: &[T]) {
        for i in 0..size {
            x[i] *= y[i];
        }
    }
}

/// Combine outputs of two nodes with a binary operation.
/// Inputs are disjoint.
/// Outputs are combined channel-wise.
/// The nodes must have the same number of outputs.
//#[derive(Clone)]
pub struct BinopNode<T, X, Y, B>
where
    T: Float,
{
    _marker: PhantomData<T>,
    x: X,
    y: Y,
    #[allow(dead_code)]
    b: B,
    buffer: Buffer<T>,
}

impl<T, X, Y, B> BinopNode<T, X, Y, B>
where
    T: Float,
    X: AudioNode<Sample = T>,
    Y: AudioNode<Sample = T, Outputs = X::Outputs>,
    B: FrameBinop<T, X::Outputs>,
    X::Inputs: Size<T> + Add<Y::Inputs>,
    X::Outputs: Size<T>,
    Y::Inputs: Size<T>,
    <X::Inputs as Add<Y::Inputs>>::Output: Size<T>,
{
    pub fn new(x: X, y: Y, b: B) -> Self {
        let mut node = BinopNode {
            _marker: PhantomData,
            x,
            y,
            b,
            buffer: Buffer::new(),
        };
        let hash = node.ping(true, AttoRand::new(Self::ID));
        node.ping(false, hash);
        node
    }
}

impl<T, X, Y, B> AudioNode for BinopNode<T, X, Y, B>
where
    T: Float,
    X: AudioNode<Sample = T>,
    Y: AudioNode<Sample = T, Outputs = X::Outputs>,
    B: FrameBinop<T, X::Outputs>,
    X::Outputs: Size<T>,
    X::Inputs: Size<T> + Add<Y::Inputs>,
    Y::Inputs: Size<T>,
    <X::Inputs as Add<Y::Inputs>>::Output: Size<T>,
{
    const ID: u64 = 3;
    type Sample = T;
    type Inputs = Sum<X::Inputs, Y::Inputs>;
    type Outputs = X::Outputs;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.y.reset(sample_rate);
    }
    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let input_x = &input[..X::Inputs::USIZE];
        let input_y = &input[X::Inputs::USIZE..];
        let x = self.x.tick(input_x.into());
        let y = self.y.tick(input_y.into());
        B::binop(&x, &y)
    }
    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        self.x.process(size, input, output);
        self.y
            .process(size, input, self.buffer.get_mut(self.outputs()));
        for i in 0..self.outputs() {
            B::assign(size, output[i], self.buffer.at(i));
        }
    }
    #[inline]
    fn ping(&mut self, probe: bool, hash: AttoRand) -> AttoRand {
        self.y.ping(probe, self.x.ping(probe, hash.hash(Self::ID)))
    }

    fn propagate(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut signal_x = self.x.propagate(input, frequency);
        let signal_y = self.y.propagate(
            &copy_signal_frame(input, X::Inputs::USIZE, Y::Inputs::USIZE),
            frequency,
        );
        for i in 0..Self::Outputs::USIZE {
            signal_x[i] = B::propagate(signal_x[i], signal_y[i]);
        }
        signal_x
    }
}

pub trait FrameUnop<T: Float, N: Size<T>>: Clone {
    fn unop(x: &Frame<T, N>) -> Frame<T, N>;
    fn propagate(x: Signal) -> Signal;
    /// Do unary op in-place.
    fn assign(size: usize, x: &mut [T]);
}

#[derive(Clone, Default)]
pub struct FrameNeg<T: Float, N: Size<T>> {
    _marker: PhantomData<(T, N)>,
}

impl<T: Float, N: Size<T>> FrameNeg<T, N> {
    pub fn new() -> FrameNeg<T, N> {
        FrameNeg::default()
    }
}

impl<T: Float, N: Size<T>> FrameUnop<T, N> for FrameNeg<T, N> {
    #[inline]
    fn unop(x: &Frame<T, N>) -> Frame<T, N> {
        -x
    }
    fn propagate(x: Signal) -> Signal {
        match x {
            Signal::Value(vx) => Signal::Value(-vx),
            Signal::Response(rx, lx) => Signal::Response(-rx, lx),
            s => s,
        }
    }
    fn assign(size: usize, x: &mut [T]) {
        for i in 0..size {
            x[i] = -x[i];
        }
    }
}

/// Identity op.
#[derive(Clone, Default)]
pub struct FrameId<T: Float, N: Size<T>> {
    _marker: PhantomData<(T, N)>,
}

impl<T: Float, N: Size<T>> FrameId<T, N> {
    pub fn new() -> FrameId<T, N> {
        FrameId::default()
    }
}

impl<T: Float, N: Size<T>> FrameUnop<T, N> for FrameId<T, N> {
    #[inline]
    fn unop(x: &Frame<T, N>) -> Frame<T, N> {
        x.clone()
    }
    #[inline]
    fn propagate(x: Signal) -> Signal {
        x
    }
    fn assign(_size: usize, _x: &mut [T]) {}
}

/// Apply a unary operation to output of contained node.
#[derive(Clone)]
pub struct UnopNode<T, X, U> {
    _marker: PhantomData<T>,
    x: X,
    #[allow(dead_code)]
    u: U,
}

impl<T, X, U> UnopNode<T, X, U>
where
    T: Float,
    X: AudioNode<Sample = T>,
    U: FrameUnop<T, X::Outputs>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
{
    pub fn new(x: X, u: U) -> Self {
        let mut node = UnopNode {
            _marker: PhantomData,
            x,
            u,
        };
        let hash = node.ping(true, AttoRand::new(Self::ID));
        node.ping(false, hash);
        node
    }
}

impl<T, X, U> AudioNode for UnopNode<T, X, U>
where
    T: Float,
    X: AudioNode<Sample = T>,
    U: FrameUnop<T, X::Outputs>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
{
    const ID: u64 = 4;
    type Sample = T;
    type Inputs = X::Inputs;
    type Outputs = X::Outputs;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
    }
    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        U::unop(&self.x.tick(input))
    }
    #[inline]
    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        self.x.process(size, input, output);
        for i in 0..self.outputs() {
            U::assign(size, output[i]);
        }
    }
    fn ping(&mut self, probe: bool, hash: AttoRand) -> AttoRand {
        self.x.ping(probe, hash.hash(Self::ID))
    }

    fn propagate(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut signal_x = self.x.propagate(input, frequency);
        for i in 0..Self::Outputs::USIZE {
            signal_x[i] = U::propagate(signal_x[i]);
        }
        signal_x
    }
}

#[derive(Clone)]
pub struct MapNode<T, M, P, I, O> {
    f: M,
    propagate_f: P,
    _marker: PhantomData<(T, I, O)>,
}

/// Map any number of channels.
impl<T, M, P, I, O> MapNode<T, M, P, I, O>
where
    T: Float,
    M: Clone + Fn(&Frame<T, I>) -> Frame<T, O>,
    P: Clone + Fn(&SignalFrame, f64) -> SignalFrame,
    I: Size<T>,
    O: Size<T>,
{
    pub fn new(f: M, propagate_f: P) -> Self {
        Self {
            f,
            propagate_f,
            _marker: PhantomData,
        }
    }
}

impl<T, M, P, I, O> AudioNode for MapNode<T, M, P, I, O>
where
    T: Float,
    M: Clone + Fn(&Frame<T, I>) -> Frame<T, O>,
    P: Clone + Fn(&SignalFrame, f64) -> SignalFrame,
    I: Size<T>,
    O: Size<T>,
{
    const ID: u64 = 5;
    type Sample = T;
    type Inputs = I;
    type Outputs = O;

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        (self.f)(input)
    }

    fn propagate(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        (self.propagate_f)(input, frequency)
    }
}

/// Pipe the output of `X` to `Y`.
pub struct PipeNode<T, X, Y>
where
    T: Float,
{
    _marker: PhantomData<T>,
    x: X,
    y: Y,
    buffer: Buffer<T>,
}

impl<T, X, Y> PipeNode<T, X, Y>
where
    T: Float,
    X: AudioNode<Sample = T>,
    Y: AudioNode<Sample = T, Inputs = X::Outputs>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    Y::Outputs: Size<T>,
{
    pub fn new(x: X, y: Y) -> Self {
        let mut node = PipeNode {
            _marker: PhantomData,
            x,
            y,
            buffer: Buffer::new(),
        };
        let hash = node.ping(true, AttoRand::new(Self::ID));
        node.ping(false, hash);
        node
    }
}

impl<T, X, Y> AudioNode for PipeNode<T, X, Y>
where
    T: Float,
    X: AudioNode<Sample = T>,
    Y: AudioNode<Sample = T, Inputs = X::Outputs>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    Y::Outputs: Size<T>,
{
    const ID: u64 = 6;
    type Sample = T;
    type Inputs = X::Inputs;
    type Outputs = Y::Outputs;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.y.reset(sample_rate);
    }
    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        self.y.tick(&self.x.tick(input))
    }
    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        self.x
            .process(size, input, self.buffer.get_mut(self.x.outputs()));
        self.y
            .process(size, self.buffer.get_ref(self.y.inputs()), output);
    }
    #[inline]
    fn ping(&mut self, probe: bool, hash: AttoRand) -> AttoRand {
        self.y.ping(probe, self.x.ping(probe, hash.hash(Self::ID)))
    }
    fn propagate(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.y
            .propagate(&self.x.propagate(input, frequency), frequency)
    }
}

/// Stack `X` and `Y` in parallel.
#[derive(Clone)]
pub struct StackNode<T, X, Y> {
    _marker: PhantomData<T>,
    x: X,
    y: Y,
}

impl<T, X, Y> StackNode<T, X, Y>
where
    T: Float,
    X: AudioNode<Sample = T>,
    Y: AudioNode<Sample = T>,
    X::Inputs: Size<T> + Add<Y::Inputs>,
    X::Outputs: Size<T> + Add<Y::Outputs>,
    Y::Inputs: Size<T>,
    Y::Outputs: Size<T>,
    <X::Inputs as Add<Y::Inputs>>::Output: Size<T>,
    <X::Outputs as Add<Y::Outputs>>::Output: Size<T>,
{
    pub fn new(x: X, y: Y) -> Self {
        let mut node = StackNode {
            _marker: PhantomData,
            x,
            y,
        };
        let hash = node.ping(true, AttoRand::new(Self::ID));
        node.ping(false, hash);
        node
    }
}

impl<T, X, Y> AudioNode for StackNode<T, X, Y>
where
    T: Float,
    X: AudioNode<Sample = T>,
    Y: AudioNode<Sample = T>,
    X::Inputs: Size<T> + Add<Y::Inputs>,
    X::Outputs: Size<T> + Add<Y::Outputs>,
    Y::Inputs: Size<T>,
    Y::Outputs: Size<T>,
    <X::Inputs as Add<Y::Inputs>>::Output: Size<T>,
    <X::Outputs as Add<Y::Outputs>>::Output: Size<T>,
{
    const ID: u64 = 7;
    type Sample = T;
    type Inputs = Sum<X::Inputs, Y::Inputs>;
    type Outputs = Sum<X::Outputs, Y::Outputs>;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.y.reset(sample_rate);
    }
    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
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
    #[inline]
    fn ping(&mut self, probe: bool, hash: AttoRand) -> AttoRand {
        self.y.ping(probe, self.x.ping(probe, hash.hash(Self::ID)))
    }

    fn propagate(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut signal_x = self.x.propagate(input, frequency);
        let signal_y = self.y.propagate(
            &copy_signal_frame(input, X::Inputs::USIZE, Y::Inputs::USIZE),
            frequency,
        );
        signal_x.resize(self.outputs(), Signal::Unknown);
        signal_x[X::Outputs::USIZE..Self::Outputs::USIZE]
            .copy_from_slice(&signal_y[0..Y::Outputs::USIZE]);
        signal_x
    }
}

/// Send the same input to `X` and `Y`. Concatenate outputs.
#[derive(Clone)]
pub struct BranchNode<T, X, Y> {
    _marker: PhantomData<T>,
    x: X,
    y: Y,
}

impl<T, X, Y> BranchNode<T, X, Y>
where
    T: Float,
    X: AudioNode<Sample = T>,
    Y: AudioNode<Sample = T, Inputs = X::Inputs>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T> + Add<Y::Outputs>,
    Y::Outputs: Size<T>,
    <X::Outputs as Add<Y::Outputs>>::Output: Size<T>,
{
    pub fn new(x: X, y: Y) -> Self {
        let mut node = BranchNode {
            _marker: PhantomData,
            x,
            y,
        };
        let hash = node.ping(true, AttoRand::new(Self::ID));
        node.ping(false, hash);
        node
    }
}

impl<T, X, Y> AudioNode for BranchNode<T, X, Y>
where
    T: Float,
    X: AudioNode<Sample = T>,
    Y: AudioNode<Sample = T, Inputs = X::Inputs>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T> + Add<Y::Outputs>,
    Y::Outputs: Size<T>,
    <X::Outputs as Add<Y::Outputs>>::Output: Size<T>,
{
    const ID: u64 = 8;
    type Sample = T;
    type Inputs = X::Inputs;
    type Outputs = Sum<X::Outputs, Y::Outputs>;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.y.reset(sample_rate);
    }
    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
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
    #[inline]
    fn ping(&mut self, probe: bool, hash: AttoRand) -> AttoRand {
        self.y.ping(probe, self.x.ping(probe, hash.hash(Self::ID)))
    }

    fn propagate(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut signal_x = self.x.propagate(input, frequency);
        let signal_y = self.y.propagate(input, frequency);
        signal_x.resize(self.outputs(), Signal::Unknown);
        signal_x[X::Outputs::USIZE..Self::Outputs::USIZE]
            .copy_from_slice(&signal_y[0..Y::Outputs::USIZE]);
        signal_x
    }
}

/// Single sample delay.
#[derive(Clone)]
pub struct TickNode<T: Float, N: Size<T>> {
    buffer: Frame<T, N>,
    sample_rate: f64,
}

impl<T: Float, N: Size<T>> TickNode<T, N> {
    pub fn new(sample_rate: f64) -> Self {
        TickNode {
            buffer: Frame::default(),
            sample_rate,
        }
    }
}

impl<T: Float, N: Size<T>> AudioNode for TickNode<T, N> {
    const ID: u64 = 9;
    type Sample = T;
    type Inputs = N;
    type Outputs = N;

    #[inline]
    fn reset(&mut self, sample_rate: Option<f64>) {
        if let Some(sample_rate) = sample_rate {
            self.sample_rate = sample_rate;
        }
        self.buffer = Frame::default();
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let output = self.buffer.clone();
        self.buffer = input.clone();
        output
    }
    fn propagate(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        for i in 0..self.outputs() {
            output[i] = input[i].filter(1.0, |r| {
                r * Complex64::from_polar(1.0, -TAU * frequency / self.sample_rate)
            });
        }
        output
    }
}

/// Mix together `X` and `Y` sourcing from the same inputs.
#[derive(Clone)]
pub struct BusNode<T, X, Y> {
    _marker: PhantomData<T>,
    x: X,
    y: Y,
}

impl<T, X, Y> BusNode<T, X, Y>
where
    T: Float,
    X: AudioNode<Sample = T>,
    Y: AudioNode<Sample = T, Inputs = X::Inputs, Outputs = X::Outputs>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    Y::Inputs: Size<T>,
    Y::Outputs: Size<T>,
{
    pub fn new(x: X, y: Y) -> Self {
        let mut node = BusNode {
            _marker: PhantomData,
            x,
            y,
        };
        let hash = node.ping(true, AttoRand::new(Self::ID));
        node.ping(false, hash);
        node
    }
}

impl<T, X, Y> AudioNode for BusNode<T, X, Y>
where
    T: Float,
    X: AudioNode<Sample = T>,
    Y: AudioNode<Sample = T, Inputs = X::Inputs, Outputs = X::Outputs>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    Y::Inputs: Size<T>,
    Y::Outputs: Size<T>,
{
    const ID: u64 = 10;
    type Sample = T;
    type Inputs = X::Inputs;
    type Outputs = X::Outputs;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.y.reset(sample_rate);
    }
    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let output_x = self.x.tick(input);
        let output_y = self.y.tick(input);
        output_x + output_y
    }
    #[inline]
    fn ping(&mut self, probe: bool, hash: AttoRand) -> AttoRand {
        self.y.ping(probe, self.x.ping(probe, hash.hash(Self::ID)))
    }

    fn propagate(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut signal_x = self.x.propagate(input, frequency);
        let signal_y = self.y.propagate(input, frequency);
        for i in 0..Self::Outputs::USIZE {
            signal_x[i] = combine_linear(signal_x[i], signal_y[i], 0.0, |x, y| x + y, |x, y| x + y);
        }
        signal_x
    }
}

/// Pass through inputs without matching outputs.
/// Adjusts output arity to match input arity, adapting a filter to a pipeline.
#[derive(Clone)]
pub struct ThruNode<X> {
    x: X,
}

impl<X: AudioNode> ThruNode<X> {
    pub fn new(x: X) -> Self {
        let mut node = ThruNode { x };
        let hash = node.ping(true, AttoRand::new(Self::ID));
        node.ping(false, hash);
        node
    }
}

impl<X: AudioNode> AudioNode for ThruNode<X> {
    const ID: u64 = 12;
    type Sample = X::Sample;
    type Inputs = X::Inputs;
    type Outputs = X::Inputs;

    #[inline]
    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let output = self.x.tick(input);
        Frame::generate(|i| {
            if i < X::Outputs::USIZE {
                output[i]
            } else {
                input[i]
            }
        })
    }

    #[inline]
    fn ping(&mut self, probe: bool, hash: AttoRand) -> AttoRand {
        self.x.ping(probe, hash.hash(Self::ID))
    }

    fn propagate(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = self.x.propagate(input, frequency);
        output[X::Outputs::USIZE..Self::Outputs::USIZE]
            .copy_from_slice(&input[X::Outputs::USIZE..Self::Outputs::USIZE]);
        output
    }
}

/// Mix together a bunch of similar nodes sourcing from the same inputs.
#[derive(Clone, Default)]
pub struct MultiBusNode<T, N, X>
where
    T: Float,
    N: Size<T>,
    N: Size<X>,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
{
    _marker: PhantomData<T>,
    x: Frame<X, N>,
}

impl<T, N, X> MultiBusNode<T, N, X>
where
    T: Float,
    N: Size<T>,
    N: Size<X>,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
{
    pub fn new(x: Frame<X, N>) -> Self {
        let mut node = MultiBusNode {
            _marker: PhantomData::default(),
            x,
        };
        let hash = node.ping(true, AttoRand::new(Self::ID));
        node.ping(false, hash);
        node
    }
}

impl<T, N, X> AudioNode for MultiBusNode<T, N, X>
where
    T: Float,
    N: Size<T>,
    N: Size<X>,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
{
    const ID: u64 = 28;
    type Sample = T;
    type Inputs = X::Inputs;
    type Outputs = X::Outputs;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.iter_mut().for_each(|node| node.reset(sample_rate));
    }
    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        self.x
            .iter_mut()
            .fold(Frame::splat(T::zero()), |acc, x| acc + x.tick(input))
    }
    #[inline]
    fn ping(&mut self, probe: bool, hash: AttoRand) -> AttoRand {
        let mut hash = hash.hash(Self::ID);
        for x in &mut self.x {
            hash = x.ping(probe, hash);
        }
        hash
    }

    fn propagate(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        if self.x.is_empty() {
            return new_signal_frame(self.outputs());
        }
        let mut output = self.x[0].propagate(input, frequency);
        for j in 1..self.x.len() {
            let output_j = self.x[j].propagate(input, frequency);
            for i in 0..Self::Outputs::USIZE {
                output[i] = combine_linear(output[i], output_j[i], 0.0, |x, y| x + y, |x, y| x + y);
            }
        }
        output
    }
}

/// Stack a bunch of similar nodes in parallel.
#[derive(Clone)]
pub struct MultiStackNode<T, N, X>
where
    T: Float,
    N: Size<T>,
    N: Size<X>,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T> + Mul<N>,
    X::Outputs: Size<T> + Mul<N>,
    <X::Inputs as Mul<N>>::Output: Size<T>,
    <X::Outputs as Mul<N>>::Output: Size<T>,
{
    _marker: PhantomData<(T, N)>,
    x: Frame<X, N>,
}

impl<T, N, X> MultiStackNode<T, N, X>
where
    T: Float,
    N: Size<T>,
    N: Size<X>,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T> + Mul<N>,
    X::Outputs: Size<T> + Mul<N>,
    <X::Inputs as Mul<N>>::Output: Size<T>,
    <X::Outputs as Mul<N>>::Output: Size<T>,
{
    pub fn new(x: Frame<X, N>) -> Self {
        let mut node = MultiStackNode {
            _marker: PhantomData,
            x,
        };
        let hash = node.ping(true, AttoRand::new(Self::ID));
        node.ping(false, hash);
        node
    }
}

impl<T, N, X> AudioNode for MultiStackNode<T, N, X>
where
    T: Float,
    N: Size<T>,
    N: Size<X>,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T> + Mul<N>,
    X::Outputs: Size<T> + Mul<N>,
    <X::Inputs as Mul<N>>::Output: Size<T>,
    <X::Outputs as Mul<N>>::Output: Size<T>,
{
    const ID: u64 = 30;
    type Sample = T;
    type Inputs = Prod<X::Inputs, N>;
    type Outputs = Prod<X::Outputs, N>;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.iter_mut().for_each(|node| node.reset(sample_rate));
    }
    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let mut output: Frame<Self::Sample, Self::Outputs> = Frame::splat(T::zero());
        for (i, node) in self.x.iter_mut().enumerate() {
            let node_input = &input[i * X::Inputs::USIZE..(i + 1) * X::Inputs::USIZE];
            let node_output = node.tick(node_input.into());
            output[i * X::Outputs::USIZE..(i + 1) * X::Outputs::USIZE]
                .copy_from_slice(node_output.as_slice());
        }
        output
    }
    #[inline]
    fn ping(&mut self, probe: bool, hash: AttoRand) -> AttoRand {
        let mut hash = hash.hash(Self::ID);
        for x in self.x.iter_mut() {
            hash = x.ping(probe, hash);
        }
        hash
    }

    fn propagate(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        if self.x.is_empty() {
            return new_signal_frame(self.outputs());
        }
        let mut output = self.x[0].propagate(input, frequency);
        output.resize(self.outputs(), Signal::Unknown);
        for i in 1..N::USIZE {
            let output_i = self.x[i].propagate(
                &copy_signal_frame(input, i * X::Inputs::USIZE, X::Inputs::USIZE),
                frequency,
            );
            output[i * X::Outputs::USIZE..(i + 1) * X::Outputs::USIZE]
                .copy_from_slice(&output_i[0..X::Outputs::USIZE]);
        }
        output
    }
}

/// Combine outputs of a bunch of similar nodes with a binary operation.
/// Inputs are disjoint.
/// Outputs are combined channel-wise.
#[derive(Clone)]
pub struct ReduceNode<T, N, X, B>
where
    T: Float,
    N: Size<T>,
    N: Size<X>,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T> + Mul<N>,
    X::Outputs: Size<T>,
    <X::Inputs as Mul<N>>::Output: Size<T>,
    B: FrameBinop<T, X::Outputs>,
{
    x: Frame<X, N>,
    #[allow(dead_code)]
    b: B,
    _marker: PhantomData<T>,
}

impl<T, N, X, B> ReduceNode<T, N, X, B>
where
    T: Float,
    N: Size<T>,
    N: Size<X>,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T> + Mul<N>,
    X::Outputs: Size<T>,
    <X::Inputs as Mul<N>>::Output: Size<T>,
    B: FrameBinop<T, X::Outputs>,
{
    pub fn new(x: Frame<X, N>, b: B) -> Self {
        let mut node = ReduceNode {
            x,
            b,
            _marker: PhantomData,
        };
        let hash = node.ping(true, AttoRand::new(Self::ID));
        node.ping(false, hash);
        node
    }
}

impl<T, N, X, B> AudioNode for ReduceNode<T, N, X, B>
where
    T: Float,
    N: Size<T>,
    N: Size<X>,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T> + Mul<N>,
    X::Outputs: Size<T>,
    <X::Inputs as Mul<N>>::Output: Size<T>,
    B: FrameBinop<T, X::Outputs>,
{
    const ID: u64 = 32;
    type Sample = T;
    type Inputs = Prod<X::Inputs, N>;
    type Outputs = X::Outputs;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.iter_mut().for_each(|node| node.reset(sample_rate));
    }
    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let mut output: Frame<Self::Sample, Self::Outputs> = Frame::splat(T::zero());
        for (i, node) in self.x.iter_mut().enumerate() {
            let node_input = &input[i * X::Inputs::USIZE..(i + 1) * X::Inputs::USIZE];
            let node_output = node.tick(node_input.into());
            if i > 0 {
                output = B::binop(&output, &node_output);
            } else {
                output = node_output;
            }
        }
        output
    }
    #[inline]
    fn ping(&mut self, probe: bool, hash: AttoRand) -> AttoRand {
        let mut hash = hash.hash(Self::ID);
        for x in self.x.iter_mut() {
            hash = x.ping(probe, hash);
        }
        hash
    }

    fn propagate(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        if self.x.is_empty() {
            return new_signal_frame(self.outputs());
        }
        let mut output = self.x[0].propagate(input, frequency);
        for j in 1..self.x.len() {
            let output_j = self.x[j].propagate(
                &copy_signal_frame(input, j * X::Inputs::USIZE, X::Inputs::USIZE),
                frequency,
            );
            for i in 0..Self::Outputs::USIZE {
                output[i] = B::propagate(output[i], output_j[i]);
            }
        }
        output
    }
}

/// Branch into a bunch of similar nodes in parallel.
#[derive(Clone)]
pub struct MultiBranchNode<T, N, X>
where
    T: Float,
    N: Size<T>,
    N: Size<X>,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T> + Mul<N>,
    <X::Outputs as Mul<N>>::Output: Size<T>,
{
    _marker: PhantomData<(T, N)>,
    x: Frame<X, N>,
}

impl<T, N, X> MultiBranchNode<T, N, X>
where
    T: Float,
    N: Size<T>,
    N: Size<X>,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T> + Mul<N>,
    <X::Outputs as Mul<N>>::Output: Size<T>,
{
    pub fn new(x: Frame<X, N>) -> Self {
        let mut node = MultiBranchNode {
            _marker: PhantomData,
            x,
        };
        let hash = node.ping(true, AttoRand::new(Self::ID));
        node.ping(false, hash);
        node
    }
}

impl<T, N, X> AudioNode for MultiBranchNode<T, N, X>
where
    T: Float,
    N: Size<T>,
    N: Size<X>,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T> + Mul<N>,
    <X::Outputs as Mul<N>>::Output: Size<T>,
{
    const ID: u64 = 33;
    type Sample = T;
    type Inputs = X::Inputs;
    type Outputs = Prod<X::Outputs, N>;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.iter_mut().for_each(|node| node.reset(sample_rate));
    }
    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let mut output: Frame<Self::Sample, Self::Outputs> = Frame::splat(T::zero());
        for (i, node) in self.x.iter_mut().enumerate() {
            let node_output = node.tick(input);
            output[i * X::Outputs::USIZE..(i + 1) * X::Outputs::USIZE]
                .copy_from_slice(node_output.as_slice());
        }
        output
    }
    #[inline]
    fn ping(&mut self, probe: bool, hash: AttoRand) -> AttoRand {
        let mut hash = hash.hash(Self::ID);
        for x in self.x.iter_mut() {
            hash = x.ping(probe, hash);
        }
        hash
    }

    fn propagate(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        if self.x.is_empty() {
            return new_signal_frame(self.outputs());
        }
        let mut output = self.x[0].propagate(input, frequency);
        output.resize(self.outputs(), Signal::Unknown);
        for i in 1..N::USIZE {
            let output_i = self.x[i].propagate(input, frequency);
            output[i * X::Outputs::USIZE..(i + 1) * X::Outputs::USIZE]
                .copy_from_slice(&output_i[0..X::Outputs::USIZE]);
        }
        output
    }
}
