use super::math::*;
use super::*;
use numeric_array::typenum::*;
use std::marker::PhantomData;

/// Type-level integer.
pub trait Size<T>: numeric_array::ArrayLength<T> {}

impl<T, A: numeric_array::ArrayLength<T>> Size<T> for A {}

/// Transports audio data between `AudioNode` instances.
pub type Frame<T, Size> = numeric_array::NumericArray<T, Size>;

/// An audio processor that processes audio data sample by sample.
/// `AudioNode` has a static number of inputs (`AudioNode::Inputs`) and outputs (`AudioNode::Outputs`)
/// known at compile time (they are encoded in the types as type-level integers).
/// `AudioNode` processes samples of type `AudioNode::Sample`, chosen statically.
pub trait AudioNode: Clone {
    /// Unique ID for hashing.
    const ID: u32;
    /// Sample type for input and output.
    type Sample: Float;
    /// Input arity.
    type Inputs: Size<Self::Sample>;
    /// Output arity.
    type Outputs: Size<Self::Sample>;

    /// Reset the input state of the component to an initial state where it has not processed any samples.
    /// In other words, reset time to zero.
    fn reset(&mut self, _sample_rate: Option<f64>) {}

    /// Process one sample.
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs>;

    /// Causal latency from input to output, in (fractional) samples.
    /// After a reset, we can discard this many samples from the output to avoid incurring a pre-delay.
    /// This applies only to nodes that have both inputs and outputs; others should return `None`.
    /// The latency can depend on the sample rate and is allowed to change after `reset`.
    fn latency(&self) -> Option<f64> {
        // Default latency is zero.
        if self.inputs() > 0 && self.outputs() > 0 {
            Some(0.0)
        } else {
            None
        }
    }

    /// Set node hash. Override this to use the hash. This is called from `ping`. It should not be called by users.
    fn set_hash(&mut self, _hash: u32) {}

    /// Ping contained `AudioNode`s to obtain a deterministic pseudorandom seed.
    /// The local hash includes children, too.
    /// Leaf nodes should not need to override this.
    fn ping(&mut self, hash: u32) -> u32 {
        hashw(Self::ID ^ hash)
    }

    // End of interface. There is no need to override the following.

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
    /// The node must have exactly 2 inputs.
    /// The node must have 1 or 2 outputs. If there is just one output, duplicate it.
    #[inline]
    fn filter_stereo(&mut self, x: Self::Sample, y: Self::Sample) -> (Self::Sample, Self::Sample) {
        assert!(Self::Inputs::USIZE == 2);
        assert!(Self::Outputs::USIZE == 1 || Self::Outputs::USIZE == 2);
        let output = self.tick(&Frame::generate(|i| if i & 1 == 0 { x } else { y }));
        (output[0], output[if self.outputs() > 1 { 1 } else { 0 }])
    }
}

/// Combined latency of parallel components `a` and `b`.
fn parallel_latency(a: Option<f64>, b: Option<f64>) -> Option<f64> {
    match (a, b) {
        (Some(x), Some(y)) => Some(min(x, y)),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        _ => None,
    }
}

/// Combined latency of serial components `a` and `b`.
fn serial_latency(a: Option<f64>, b: Option<f64>) -> Option<f64> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x + y),
        _ => None,
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
    const ID: u32 = 0;
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
    const ID: u32 = 1;
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
    const ID: u32 = 2;
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
}

#[derive(Clone)]
pub enum Binop {
    Add,
    Sub,
    Mul,
}

pub trait FrameBinop<T: Float, N: Size<T>>: Clone {
    fn binop(x: &Frame<T, N>, y: &Frame<T, N>) -> Frame<T, N>;
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
}

#[derive(Clone)]
pub enum Unop {
    Neg,
}

pub trait FrameUnop<T: Float, N: Size<T>>: Clone {
    fn unop(x: &Frame<T, N>) -> Frame<T, N>;
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
}

/// Combine outputs of two nodes with a binary operation.
/// Outputs are combined channel-wise.
/// The nodes must have the same number of outputs.
#[derive(Clone)]
pub struct BinopNode<T, X, Y, B> {
    _marker: PhantomData<T>,
    x: X,
    y: Y,
    b: B,
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
        };
        node.ping(Self::ID);
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
    const ID: u32 = 3;
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
        let input_x = &input[0..X::Inputs::USIZE];
        let input_y = &input[Self::Inputs::USIZE - Y::Inputs::USIZE..Self::Inputs::USIZE];
        let x = self.x.tick(input_x.into());
        let y = self.y.tick(input_y.into());
        B::binop(&x, &y)
    }
    fn latency(&self) -> Option<f64> {
        parallel_latency(self.x.latency(), self.y.latency())
    }
    #[inline]
    fn ping(&mut self, hash: u32) -> u32 {
        let hash = self.x.ping(hash);
        let hash = self.y.ping(hash);
        hashw(Self::ID ^ hash)
    }
}

/// Apply a unary operation to output of contained node.
#[derive(Clone)]
pub struct UnopNode<T, X, U> {
    _marker: PhantomData<T>,
    x: X,
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
        node.ping(Self::ID);
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
    const ID: u32 = 4;
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
    fn latency(&self) -> Option<f64> {
        self.x.latency()
    }
    #[inline]
    fn ping(&mut self, hash: u32) -> u32 {
        let hash = self.x.ping(hash);
        hashw(Self::ID ^ hash)
    }
}

#[derive(Clone)]
pub struct Map<T, F, I, O> {
    f: F,
    _marker: PhantomData<(T, I, O)>,
}

/// Map any number of channels.
impl<T, F, I, O> Map<T, F, I, O>
where
    T: Float,
    F: Clone + FnMut(&Frame<T, I>) -> Frame<T, O>,
    I: Size<T>,
    O: Size<T>,
{
    pub fn new(f: F) -> Self {
        Self {
            f,
            _marker: PhantomData,
        }
    }
}

impl<T, F, I, O> AudioNode for Map<T, F, I, O>
where
    T: Float,
    F: Clone + FnMut(&Frame<T, I>) -> Frame<T, O>,
    I: Size<T>,
    O: Size<T>,
{
    const ID: u32 = 5;
    type Sample = T;
    type Inputs = I;
    type Outputs = O;

    // TODO: Implement reset() by storing initial state?

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        (self.f)(input)
    }
}

/// Pipe the output of `X` to `Y`.
#[derive(Clone)]
pub struct PipeNode<T, X, Y> {
    _marker: PhantomData<T>,
    x: X,
    y: Y,
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
        };
        node.ping(Self::ID);
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
    const ID: u32 = 6;
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
    fn latency(&self) -> Option<f64> {
        serial_latency(self.x.latency(), self.y.latency())
    }
    #[inline]
    fn ping(&mut self, hash: u32) -> u32 {
        let hash = self.x.ping(hash);
        let hash = self.y.ping(hash);
        hashw(Self::ID ^ hash)
    }
}

//// Stack `X` and `Y` in parallel.
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
        node.ping(Self::ID);
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
    const ID: u32 = 7;
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
        let input_x = &input[0..X::Inputs::USIZE];
        let input_y = &input[Self::Inputs::USIZE - Y::Inputs::USIZE..Self::Inputs::USIZE];
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
    fn latency(&self) -> Option<f64> {
        parallel_latency(self.x.latency(), self.y.latency())
    }
    #[inline]
    fn ping(&mut self, hash: u32) -> u32 {
        let hash = self.x.ping(hash);
        let hash = self.y.ping(hash);
        hashw(Self::ID ^ hash)
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
        node.ping(Self::ID);
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
    const ID: u32 = 8;
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
    fn latency(&self) -> Option<f64> {
        parallel_latency(self.x.latency(), self.y.latency())
    }
    #[inline]
    fn ping(&mut self, hash: u32) -> u32 {
        let hash = self.x.ping(hash);
        let hash = self.y.ping(hash);
        hashw(0x00A ^ hash)
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
    const ID: u32 = 9;
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
    fn latency(&self) -> Option<f64> {
        Some(1.0 / self.sample_rate)
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
        node.ping(Self::ID);
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
    const ID: u32 = 10;
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
    fn latency(&self) -> Option<f64> {
        parallel_latency(self.x.latency(), self.y.latency())
    }
    #[inline]
    fn ping(&mut self, hash: u32) -> u32 {
        let hash = self.x.ping(hash);
        let hash = self.y.ping(hash);
        hashw(Self::ID ^ hash)
    }
}

/// Mix back output of contained node to its input.
/// The contained node must have an equal number of inputs and outputs.
#[derive(Clone)]
pub struct FeedbackNode<T, X, N>
where
    T: Float,
    X: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    N: Size<T>,
{
    x: X,
    // Current feedback value.
    value: Frame<T, N>,
}

impl<T, X, N> FeedbackNode<T, X, N>
where
    T: Float,
    X: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    N: Size<T>,
{
    pub fn new(x: X) -> Self {
        let mut node = FeedbackNode {
            x,
            value: Frame::default(),
        };
        node.ping(Self::ID);
        node
    }
}

impl<T, X, N> AudioNode for FeedbackNode<T, X, N>
where
    T: Float,
    X: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    N: Size<T>,
{
    const ID: u32 = 11;
    type Sample = T;
    type Inputs = N;
    type Outputs = N;

    #[inline]
    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.value = Frame::default();
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let output = self.x.tick(&(input + self.value.clone()));
        self.value = output.clone();
        output
    }

    fn latency(&self) -> Option<f64> {
        self.x.latency()
    }

    #[inline]
    fn ping(&mut self, hash: u32) -> u32 {
        let hash = self.x.ping(hash);
        hashw(Self::ID ^ hash)
    }
}

/// Adapt a filter to a pipeline.
#[derive(Clone)]
pub struct FitNode<X> {
    x: X,
}

impl<X: AudioNode> FitNode<X> {
    pub fn new(x: X) -> Self {
        let mut node = FitNode { x };
        node.ping(Self::ID);
        node
    }
}

impl<X: AudioNode> AudioNode for FitNode<X> {
    const ID: u32 = 12;
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

    fn latency(&self) -> Option<f64> {
        self.x.latency()
    }

    #[inline]
    fn ping(&mut self, hash: u32) -> u32 {
        let hash = self.x.ping(hash);
        hashw(Self::ID ^ hash)
    }
}
