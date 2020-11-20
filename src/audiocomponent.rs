use super::math::*;
use super::*;
use generic_array::sequence::*;
use numeric_array::typenum::*;
use std::marker::PhantomData;

/// Type-level integer.
pub trait Size<T>: numeric_array::ArrayLength<T> {}

impl<T, A: numeric_array::ArrayLength<T>> Size<T> for A {}

/// Frames transport audio data between AudioNodes.
pub type Frame<T, Size> = numeric_array::NumericArray<T, Size>;

/// AudioNode processes audio data sample by sample.
/// It has a static number of inputs and outputs known at compile time.
pub trait AudioNode: Clone {
    type Sample: Float;
    type Inputs: Size<Self::Sample>;
    type Outputs: Size<Self::Sample>;

    /// Resets the input state of the component to an initial state where it has not processed any samples.
    /// In other words, resets time to zero.
    fn reset(&mut self, _sample_rate: Option<f64>) {}

    /// Processes one sample.
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs>;

    /// Causal latency from input to output, in (fractional) samples.
    /// After a reset, we can discard this many samples from the output to avoid incurring a pre-delay.
    /// This applies only to components that have both inputs and outputs; others should return None.
    /// The latency can depend on the sample rate and is allowed to change after a reset.
    fn latency(&self) -> Option<f64> {
        // Default latency is zero.
        if self.inputs() > 0 && self.outputs() > 0 {
            Some(0.0)
        } else {
            None
        }
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

    /// Retrieves the next mono sample from an all-zero input.
    /// If there are many outputs, chooses the first.
    /// This is an infallible convenience method.
    #[inline]
    fn get_mono(&mut self) -> Self::Sample {
        assert!(self.outputs() >= 1);
        let output = self.tick(&Frame::default());
        output[0]
    }

    /// Retrieves the next stereo sample pair (left, right) from an all-zero input.
    /// If there are more outputs, chooses the first two. If there is just one output, duplicates it.
    /// This is an infallible convenience method.
    #[inline]
    fn get_stereo(&mut self) -> (Self::Sample, Self::Sample) {
        assert!(self.outputs() >= 1);
        let output = self.tick(&Frame::default());
        (output[0], output[if self.outputs() > 1 { 1 } else { 0 }])
    }

    /// Filters the next mono sample.
    /// Broadcasts the input to as many channels as are needed.
    /// If there are many outputs, chooses the first.
    /// This is an infallible convenience method.
    #[inline]
    fn filter_mono(&mut self, x: Self::Sample) -> Self::Sample {
        assert!(self.outputs() >= 1);
        let output = self.tick(&Frame::splat(x));
        output[0]
    }

    /// Filters the next stereo sample pair.
    /// Broadcasts the input by wrapping to as many channels as are needed.
    /// If there are more outputs, chooses the first two. If there is just one output, duplicates it.
    /// This is an infallible convenience method.
    #[inline]
    fn filter_stereo(&mut self, x: Self::Sample, y: Self::Sample) -> (Self::Sample, Self::Sample) {
        assert!(self.outputs() >= 1);
        let output = self.tick(&Frame::generate(|i| if i & 1 == 0 { x } else { y }));
        (output[0], output[if self.outputs() > 1 { 1 } else { 0 }])
    }
}

/// Combined latency of parallel components a and b.
fn parallel_latency(a: Option<f64>, b: Option<f64>) -> Option<f64> {
    match (a, b) {
        (Some(x), Some(y)) => Some(min(x, y)),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        _ => None,
    }
}

/// Combined latency of serial components a and b.
fn serial_latency(a: Option<f64>, b: Option<f64>) -> Option<f64> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x + y),
        _ => None,
    }
}

/// PassNode passes through its inputs unchanged.
#[derive(Clone)]
pub struct PassNode<T, N> {
    _marker: PhantomData<(T, N)>,
}

impl<T: Float, N: Size<T>> PassNode<T, N> {
    pub fn new() -> Self {
        PassNode {
            _marker: PhantomData,
        }
    }
}

impl<T: Float, N: Size<T>> AudioNode for PassNode<T, N> {
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

/// SinkNode consumes its inputs.
#[derive(Clone)]
pub struct SinkNode<T, N> {
    _marker: PhantomData<(T, N)>,
}

impl<T: Float, N: Size<T>> SinkNode<T, N> {
    pub fn new() -> Self {
        SinkNode {
            _marker: PhantomData,
        }
    }
}

impl<T: Float, N: Size<T>> AudioNode for SinkNode<T, N> {
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

/// ConstantNode outputs a constant value.
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
#[derive(Clone)]
pub struct FrameAdd<T: Float, N: Size<T>> {
    _marker: PhantomData<(T, N)>,
}

impl<T: Float, N: Size<T>> FrameAdd<T, N> {
    pub fn new() -> FrameAdd<T, N> {
        FrameAdd {
            _marker: PhantomData,
        }
    }
}

impl<T: Float, N: Size<T>> FrameBinop<T, N> for FrameAdd<T, N> {
    #[inline]
    fn binop(x: &Frame<T, N>, y: &Frame<T, N>) -> Frame<T, N> {
        x + y
    }
}

#[derive(Clone)]
pub struct FrameSub<T: Float, N: Size<T>> {
    _marker: PhantomData<(T, N)>,
}

impl<T: Float, N: Size<T>> FrameSub<T, N> {
    pub fn new() -> FrameSub<T, N> {
        FrameSub {
            _marker: PhantomData,
        }
    }
}

impl<T: Float, N: Size<T>> FrameBinop<T, N> for FrameSub<T, N> {
    #[inline]
    fn binop(x: &Frame<T, N>, y: &Frame<T, N>) -> Frame<T, N> {
        x - y
    }
}

#[derive(Clone)]
pub struct FrameMul<T: Float, N: Size<T>> {
    _marker: PhantomData<(T, N)>,
}

impl<T: Float, N: Size<T>> FrameMul<T, N> {
    pub fn new() -> FrameMul<T, N> {
        FrameMul {
            _marker: PhantomData,
        }
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
#[derive(Clone)]
pub struct FrameNeg<T: Float, N: Size<T>> {
    _marker: PhantomData<(T, N)>,
}

impl<T: Float, N: Size<T>> FrameNeg<T, N> {
    pub fn new() -> FrameNeg<T, N> {
        FrameNeg {
            _marker: PhantomData,
        }
    }
}

impl<T: Float, N: Size<T>> FrameUnop<T, N> for FrameNeg<T, N> {
    #[inline]
    fn unop(x: &Frame<T, N>) -> Frame<T, N> {
        -x
    }
}

/// BinopNode combines outputs of two components, channel-wise, with a binary operation.
/// The components must have the same number of outputs.
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
        BinopNode {
            _marker: PhantomData,
            x,
            y,
            b,
        }
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
}

/// UnopNode applies an unary operator to its inputs.
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
        UnopNode {
            _marker: PhantomData,
            x,
            u,
        }
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
}

#[derive(Clone)]
pub struct Map<T, F, I, O> {
    f: F,
    _marker: PhantomData<(T, I, O)>,
}

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

/// PipeNode pipes the output of X to Y.
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
        PipeNode {
            _marker: PhantomData,
            x,
            y,
        }
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
}

//// StackNode stacks X and Y in parallel.
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
        StackNode {
            _marker: PhantomData,
            x,
            y,
        }
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
}

/// BranchNode sends the same input to X and Y and concatenates the outputs.
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
        BranchNode {
            _marker: PhantomData,
            x,
            y,
        }
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
}

/// TickNode is a single sample delay.
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

/// BusNode mixes together a set of nodes sourcing from the same inputs.
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
        BusNode {
            _marker: PhantomData,
            x,
            y,
        }
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
}

/// FeedbackNode encloses a feedback circuit.
/// The feedback circuit must have an equal number of inputs and outputs.
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
        FeedbackNode {
            x,
            value: Frame::default(),
        }
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
}

/// FitNode adapts a filter to a pipeline.
#[derive(Clone)]
pub struct FitNode<X> {
    x: X,
}

impl<X: AudioNode> FitNode<X> {
    pub fn new(x: X) -> Self {
        FitNode { x }
    }
}

impl<X: AudioNode> AudioNode for FitNode<X> {
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
}
