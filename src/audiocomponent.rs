use super::*;
use super::math::*;
use generic_array::sequence::*;
use numeric_array::typenum::*;
use std::marker::PhantomData;

/// AudioComponent processes audio data sample by sample.
/// It has a static number of inputs and outputs known at compile time.
/// If not set otherwise, the sample rate is presumed the system default DEFAULT_SR.
pub trait AudioComponent: Clone
{
    type Inputs: Size;
    type Outputs: Size;

    /// Resets the input state of the component to an initial state where it has not processed any samples.
    /// In other words, resets time to zero.
    fn reset(&mut self, _sample_rate: Option<f64>) {}

    /// Processes one sample.
    fn tick(&mut self, input: &Frame<Self::Inputs>) -> Frame<Self::Outputs>;

    /// Causal latency from input to output, in (fractional) samples.
    /// After a reset, we can discard this many samples from the output to avoid incurring a pre-delay.
    /// This applies only to components that have both inputs and outputs; others should return None.
    /// The latency can depend on the sample rate and is allowed to change after a reset.
    fn latency(&self) -> Option<f64> {
        // Default latency is zero.
        if self.inputs() > 0 && self.outputs() > 0 { Some(0.0) } else { None }
    }

    /// Returns output, if any, where the input is routed to.
    /// This is used for the fit operator and other routing functionality.
    fn route_input(&self, input: u32) -> Option<u32> {
        // Default is to route matching inputs and outputs.
        if input < Self::Outputs::U32 { Some(input) } else { None }
    }
 
    /// Returns input, if any, where the output is routed to.
    /// This is used for the fit operator and other routing functionality.
    fn route_output(&self, output: u32) -> Option<u32> {
        // Default is to route matching inputs and outputs.
        if output < Self::Inputs::U32 { Some(output) } else { None }
    }
 
    // End of interface. There is no need to override the following.

    /// Number of inputs.
    #[inline] fn inputs(&self) -> usize { Self::Inputs::USIZE }

    /// Number of outputs.
    #[inline] fn outputs(&self) -> usize { Self::Outputs::USIZE }

    /// Retrieves the next mono sample from an all-zero input.
    /// If there are many outputs, chooses the first.
    /// This is an infallible convenience method.
    #[inline] fn get_mono(&mut self) -> f48 {
        assert!(self.outputs() >= 1);
        let output = self.tick(&Frame::default());
        output[0]
    }

    /// Retrieves the next stereo sample pair (left, right) from an all-zero input.
    /// If there are more outputs, chooses the first two. If there is just one output, duplicates it.
    /// This is an infallible convenience method.
    #[inline] fn get_stereo(&mut self) -> (f48, f48) {
        assert!(self.outputs() >= 1);
        let output = self.tick(&Frame::default());
        (output[0], output[ if self.outputs() > 1 { 1 } else { 0 } ])
    }

    /// Filters the next mono sample.
    /// Broadcasts the input to as many channels as are needed.
    /// If there are many outputs, chooses the first.
    /// This is an infallible convenience method.
    #[inline] fn filter_mono(&mut self, x: f48) -> f48 {
        assert!(self.outputs() >= 1);
        let output = self.tick(&Frame::splat(x));
        output[0]
    }

    /// Filters the next stereo sample pair.
    /// Broadcasts the input by wrapping to as many channels as are needed.
    /// If there are more outputs, chooses the first two. If there is just one output, duplicates it.
    /// This is an infallible convenience method.
    #[inline] fn filter_stereo(&mut self, x: f48, y: f48) -> (f48, f48) {
        assert!(self.outputs() >= 1);
        let output = self.tick(&Frame::generate(|i| if i & 1 == 0 { x } else { y }));
        (output[0], output[ if self.outputs() > 1 { 1 } else { 0 } ])
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

/// PassComponent passes through its inputs unchanged.
#[derive(Clone)]
pub struct PassComponent<N: Size>
{
    _length: PhantomData<N>,
}

impl<N: Size> PassComponent<N>
{
    pub fn new() -> Self { PassComponent { _length: PhantomData::default() } }
}

impl<N: Size> AudioComponent for PassComponent<N>
{
    type Inputs = N;
    type Outputs = N;

    #[inline] fn tick(&mut self, input: &Frame<Self::Inputs>) -> Frame<Self::Outputs> {
        input.clone()
    }
}

/// SinkComponent consumes its inputs.
#[derive(Clone)]
pub struct SinkComponent<N: Size>
{
    _length: PhantomData<N>,
}

impl<N: Size> SinkComponent<N>
{
    pub fn new() -> Self { SinkComponent { _length: PhantomData::default() } }
}

impl<N: Size> AudioComponent for SinkComponent<N>
{
    type Inputs = N;
    type Outputs = U0;

    #[inline] fn tick(&mut self, _input: &Frame<Self::Inputs>) -> Frame<Self::Outputs> {
        Frame::default()
    }
}

/// ConstantComponent outputs a constant value.
#[derive(Clone)]
pub struct ConstantComponent<N: Size>
{
    output: Frame<N>,
}

impl<N: Size> ConstantComponent<N>
{
    pub fn new(output: Frame<N>) -> Self { ConstantComponent { output } }
}

impl<N: Size> AudioComponent for ConstantComponent<N>
{
    type Inputs = U0;
    type Outputs = N;

    #[inline] fn tick(&mut self, _input: &Frame<Self::Inputs>) -> Frame<Self::Outputs> {
        self.output.clone()
    }
}

#[derive(Clone)]
pub enum Binop { Add, Sub, Mul }

pub trait FrameBinop<S: Size>: Clone {
    fn binop(x: &Frame<S>, y: &Frame<S>) -> Frame<S>;
}
#[derive(Clone)]
pub struct FrameAdd<S: Size> { _size: PhantomData<S> }

impl<S: Size> FrameAdd<S> {
    pub fn new() -> FrameAdd<S> { FrameAdd { _size: PhantomData::default() } }
}

impl<S: Size> FrameBinop<S> for FrameAdd<S> {
    #[inline] fn binop(x: &Frame<S>, y: &Frame<S>) -> Frame<S> { x + y }
}

#[derive(Clone)]
pub struct FrameSub<S: Size> { _size: PhantomData<S> }

impl<S: Size> FrameSub<S> {
    pub fn new() -> FrameSub<S> { FrameSub { _size: PhantomData::default() } }
}

impl<S: Size> FrameBinop<S> for FrameSub<S> {
    #[inline] fn binop(x: &Frame<S>, y: &Frame<S>) -> Frame<S> { x - y }
}

#[derive(Clone)]
pub struct FrameMul<S: Size> { _size: PhantomData<S> }

impl<S: Size> FrameMul<S> {
    pub fn new() -> FrameMul<S> { FrameMul { _size: PhantomData::default() } }
}

impl<S: Size> FrameBinop<S> for FrameMul<S> {
    #[inline] fn binop(x: &Frame<S>, y: &Frame<S>) -> Frame<S> { x * y }
}

#[derive(Clone)]
pub enum Unop { Neg }

pub trait FrameUnop<S: Size>: Clone {
    fn unop(x: &Frame<S>) -> Frame<S>;
}
#[derive(Clone)]
pub struct FrameNeg<S: Size> { _size: PhantomData<S> }

impl<S: Size> FrameNeg<S> {
    pub fn new() -> FrameNeg<S> { FrameNeg { _size: PhantomData::default() } }
}

impl<S: Size> FrameUnop<S> for FrameNeg<S> {
    #[inline] fn unop(x: &Frame<S>) -> Frame<S> { -x }
}

/// BinopComponent combines outputs of two components, channel-wise, with a binary operation.
/// The components must have the same number of outputs.
#[derive(Clone)]
pub struct BinopComponent<X, Y, B> where
    X: AudioComponent,
    Y: AudioComponent<Outputs = X::Outputs>,
    B: FrameBinop<X::Outputs>,
    X::Inputs: Size + Add<Y::Inputs>,
    Y::Inputs: Size,
    <X::Inputs as Add<Y::Inputs>>::Output: Size,
{
    x: X,
    y: Y,
    b: B,
}

impl<X, Y, B> BinopComponent<X, Y, B> where
    X: AudioComponent,
    Y: AudioComponent<Outputs = X::Outputs>,
    B: FrameBinop<X::Outputs>,
    X::Inputs: Size + Add<Y::Inputs>,
    Y::Inputs: Size,
    <X::Inputs as Add<Y::Inputs>>::Output: Size,
{
    pub fn new(x: X, y: Y, b: B) -> Self { BinopComponent { x, y, b } }
}

impl<X, Y, B> AudioComponent for BinopComponent<X, Y, B> where
    X: AudioComponent,
    Y: AudioComponent<Outputs = X::Outputs>,
    B: FrameBinop<X::Outputs>,
    X::Inputs: Size + Add<Y::Inputs>,
    Y::Inputs: Size,
    <X::Inputs as Add<Y::Inputs>>::Output: Size,
{
    type Inputs = Sum<X::Inputs, Y::Inputs>;
    type Outputs = X::Outputs;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.y.reset(sample_rate);
    }
    #[inline] fn tick(&mut self, input: &Frame<Self::Inputs>) -> Frame<Self::Outputs> {
        let input_x = &input[0 .. X::Inputs::USIZE];
        let input_y = &input[Self::Inputs::USIZE - Y::Inputs::USIZE .. Self::Inputs::USIZE];
        let x = self.x.tick(input_x.into());
        let y = self.y.tick(input_y.into());
        B::binop(&x, &y)
    }
    fn latency(&self) -> Option<f64> {
        parallel_latency(self.x.latency(), self.y.latency())
    }
    fn route_input(&self, input: u32) -> Option<u32> {
        if input < X::Inputs::U32 { self.x.route_input(input) } else { self.y.route_input(input - X::Inputs::U32) }
    }
    fn route_output(&self, output: u32) -> Option<u32> {
        // Route to X arbitrarily.
        self.x.route_output(output)
    }
}

/// UnopComponent applies an unary operator to its inputs.
#[derive(Clone)]
pub struct UnopComponent<X, U: FrameUnop<X::Outputs>> where
    X: AudioComponent,
    U: FrameUnop<X::Outputs>,
    X::Outputs: Size,
{
    x: X,
    u: U,
}

impl<X, U> UnopComponent<X, U> where
    X: AudioComponent,
    U: FrameUnop<X::Outputs>,
    X::Outputs: Size,
{
    pub fn new(x: X, u: U) -> Self { UnopComponent { x, u } }
}

impl<X, U> AudioComponent for UnopComponent<X, U> where
    X: AudioComponent,
    U: FrameUnop<X::Outputs>,
    X::Outputs: Size,
{
    type Inputs = X::Inputs;
    type Outputs = X::Outputs;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
    }
    #[inline] fn tick(&mut self, input: &Frame<Self::Inputs>) -> Frame<Self::Outputs> {
        U::unop(&self.x.tick(input))
    }
    fn latency(&self) -> Option<f64> {
        self.x.latency()
    }
    fn route_input(&self, input: u32) -> Option<u32> {
        self.x.route_input(input)
    }
    fn route_output(&self, output: u32) -> Option<u32> {
        self.x.route_output(output)
    }
} 

/// PipeComponent pipes the output of X to Y.
#[derive(Clone)]
pub struct PipeComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Inputs = X::Outputs>,
    Y::Outputs: Size,
{
    x: X,
    y: Y,
}

impl<X, Y> PipeComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Inputs = X::Outputs>,
    Y::Outputs: Size,
{
    pub fn new(x: X, y: Y) -> Self { PipeComponent { x, y } }
}

impl<X, Y> AudioComponent for PipeComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Inputs = X::Outputs>,
    Y::Outputs: Size,
{
    type Inputs = X::Inputs;
    type Outputs = Y::Outputs;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.y.reset(sample_rate);
    }
    #[inline] fn tick(&mut self, input: &Frame<Self::Inputs>) -> Frame<Self::Outputs> {
        self.y.tick(&self.x.tick(input))
    }
    fn latency(&self) -> Option<f64> {
        serial_latency(self.x.latency(), self.y.latency())
    }
    fn route_input(&self, input: u32) -> Option<u32> {
        if input >= Self::Outputs::U32 { return None; }
        self.x.route_input(input).and_then(|y_input| self.y.route_input(y_input))
    }
    fn route_output(&self, output: u32) -> Option<u32> {
        if output >= Self::Inputs::U32 { return None; }
        self.y.route_output(output).and_then(|x_output| self.x.route_output(x_output))
    }
}

//// StackComponent stacks X and Y in parallel.
#[derive(Clone)]
pub struct StackComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent,
    X::Inputs: Size + Add<Y::Inputs>,
    X::Outputs: Size + Add<Y::Outputs>,
    Y::Inputs: Size,
    Y::Outputs: Size,
    <X::Inputs as Add<Y::Inputs>>::Output: Size,
    <X::Outputs as Add<Y::Outputs>>::Output: Size
{
    x: X,
    y: Y,
}

impl<X, Y> StackComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent,
    X::Inputs: Size + Add<Y::Inputs>,
    X::Outputs: Size + Add<Y::Outputs>,
    Y::Inputs: Size,
    Y::Outputs: Size,
    <X::Inputs as Add<Y::Inputs>>::Output: Size,
    <X::Outputs as Add<Y::Outputs>>::Output: Size
{
    pub fn new(x: X, y: Y) -> Self { StackComponent { x, y } }
}

impl<X, Y> AudioComponent for StackComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent,
    X::Inputs: Size + Add<Y::Inputs>,
    X::Outputs: Size + Add<Y::Outputs>,
    Y::Inputs: Size,
    Y::Outputs: Size,
    <X::Inputs as Add<Y::Inputs>>::Output: Size,
    <X::Outputs as Add<Y::Outputs>>::Output: Size
{
    type Inputs = Sum<X::Inputs, Y::Inputs>;
    type Outputs = Sum<X::Outputs, Y::Outputs>;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.y.reset(sample_rate);
    }
    #[inline] fn tick(&mut self, input: &Frame<Self::Inputs>) -> Frame<Self::Outputs> {
        let input_x = &input[0 .. X::Inputs::USIZE];
        let input_y = &input[Self::Inputs::USIZE - Y::Inputs::USIZE .. Self::Inputs::USIZE];
        let output_x = self.x.tick(input_x.into());
        let output_y = self.y.tick(input_y.into());
        Frame::generate(|i| if i < X::Outputs::USIZE { output_x[i] } else { output_y[i - X::Outputs::USIZE] })
    }
    fn latency(&self) -> Option<f64> {
        parallel_latency(self.x.latency(), self.y.latency())
    }
    fn route_input(&self, input: u32) -> Option<u32> {
        if input >= Self::Outputs::U32 { return None; }
        if input < X::Inputs::U32 { self.x.route_input(input) } else { self.y.route_input(input - X::Inputs::U32) }
    }
    fn route_output(&self, output: u32) -> Option<u32> {
        if output >= Self::Inputs::U32 { return None; }
        if output < X::Outputs::U32 { self.x.route_output(output) } else { self.y.route_output(output - X::Outputs::U32) }
    }
}

/// BranchComponent sends the same input to X and Y and concatenates the outputs.
#[derive(Clone)]
pub struct BranchComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Inputs = X::Inputs>,
    X::Outputs: Size + Add<Y::Outputs>,
    Y::Outputs: Size,
    <X::Outputs as Add<Y::Outputs>>::Output: Size
{
    x: X,
    y: Y,
}

impl<X, Y> BranchComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Inputs = X::Inputs>,
    X::Outputs: Size + Add<Y::Outputs>,
    Y::Outputs: Size,
    <X::Outputs as Add<Y::Outputs>>::Output: Size
{
    pub fn new(x: X, y: Y) -> Self { BranchComponent { x, y } }
}

impl<X, Y> AudioComponent for BranchComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Inputs = X::Inputs>,
    X::Outputs: Size + Add<Y::Outputs>,
    Y::Outputs: Size,
    <X::Outputs as Add<Y::Outputs>>::Output: Size
{
    type Inputs = X::Inputs;
    type Outputs = Sum<X::Outputs, Y::Outputs>;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.y.reset(sample_rate);
    }
    #[inline] fn tick(&mut self, input: &Frame<Self::Inputs>) -> Frame<Self::Outputs> {
        let output_x = self.x.tick(input);
        let output_y = self.y.tick(input);
        Frame::generate(|i| if i < X::Outputs::USIZE { output_x[i] } else { output_y[i - X::Outputs::USIZE] })
    }
    fn latency(&self) -> Option<f64> {
        parallel_latency(self.x.latency(), self.y.latency())
    }
    fn route_input(&self, input: u32) -> Option<u32> {
        // Route to X arbitrarily.
        self.x.route_input(input)
    }
    fn route_output(&self, output: u32) -> Option<u32> {
        if output < X::Outputs::U32 { self.x.route_output(output) } else { self.y.route_output(output - X::Outputs::U32) }
    }
}

/// TickComponent is a single sample delay.
#[derive(Clone)]
pub struct TickComponent<N: Size>
{
    buffer: Frame<N>,
    sample_rate: f64,
}

impl<N: Size> TickComponent<N>
{
    pub fn new(sample_rate: f64) -> Self { TickComponent { buffer: Frame::default(), sample_rate } }
}

impl<N: Size> AudioComponent for TickComponent<N>
{
    type Inputs = N;
    type Outputs = N;

    #[inline] fn reset(&mut self, sample_rate: Option<f64>) {
        if let Some(sample_rate) = sample_rate { self.sample_rate = sample_rate; }
        self.buffer = Frame::default();
    }

    #[inline] fn tick(&mut self, input: &Frame<Self::Inputs>) -> Frame<Self::Outputs> {
        let output = self.buffer.clone();
        self.buffer = input.clone();
        output
    }
    fn latency(&self) -> Option<f64> {
        Some(1.0 / self.sample_rate)
    }
}

/// BusComponent is a non-reducing mixing component.
#[derive(Clone)]
pub struct BusComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Inputs = X::Inputs, Outputs = X::Outputs>,
    Y::Inputs: Size,
    Y::Outputs: Size,
{
    x: X,
    y: Y,
}

impl<X, Y> BusComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Inputs = X::Inputs, Outputs = X::Outputs>,
    Y::Inputs: Size,
    Y::Outputs: Size,
{
    pub fn new(x: X, y: Y) -> Self { BusComponent { x, y } }
}

impl<X, Y> AudioComponent for BusComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Inputs = X::Inputs, Outputs = X::Outputs>,
    Y::Inputs: Size,
    Y::Outputs: Size,
{
    type Inputs = X::Inputs;
    type Outputs = X::Outputs;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.y.reset(sample_rate);
    }
    #[inline] fn tick(&mut self, input: &Frame<Self::Inputs>) -> Frame<Self::Outputs> {
        let output_x = self.x.tick(input);
        let output_y = self.y.tick(input);
        output_x + output_y
    }
    fn latency(&self) -> Option<f64> {
        parallel_latency(self.x.latency(), self.y.latency())
    }
    fn route_input(&self, input: u32) -> Option<u32> {
        if input >= Self::Outputs::U32 { return None; }
        // TODO: Should we query Y if X cannot route the input?
        self.x.route_input(input)
    }
    fn route_output(&self, output: u32) -> Option<u32> {
        if output >= Self::Inputs::U32 { return None; }
        // TODO: Should we query Y if X cannot route the output?
        self.x.route_output(output)
    }
}

/// FeedbackComponent encloses a feedback circuit.
/// The feedback circuit must have an equal number of inputs and outputs.
#[derive(Clone)]
pub struct FeedbackComponent<X, S> where
    X: AudioComponent<Inputs = S, Outputs = S>,
    X::Inputs: Size,
    X::Outputs: Size,
    S: Size,
{
    x: X,
    // Current feedback value.
    value: Frame<S>,
}

impl<X, S> FeedbackComponent<X, S> where
    X: AudioComponent<Inputs = S, Outputs = S>,
    X::Inputs: Size,
    X::Outputs: Size,
    S: Size,
{
    pub fn new(x: X) -> Self { FeedbackComponent { x, value: Frame::default() } }
}

impl<X, S> AudioComponent for FeedbackComponent<X, S> where
    X: AudioComponent<Inputs = S, Outputs = S>,
    X::Inputs: Size,
    X::Outputs: Size,
    S: Size,
{
    type Inputs = S;
    type Outputs = S;

    #[inline] fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.value = Frame::default();
    }

    #[inline] fn tick(&mut self, input: &Frame<Self::Inputs>) -> Frame<Self::Outputs> {
        let output = self.x.tick(&(input + self.value.clone()));
        self.value = output.clone();
        output
    }

    fn latency(&self) -> Option<f64> {
        self.x.latency()
    }

    // Logically, feedback components should already be pipelines.
    // Therefore we do not query the circuit for routing.
}

/// FitComponent adapts a filter to a pipeline using routing information
/// from the component.
#[derive(Clone)]
pub struct FitComponent<X> where
    X: AudioComponent,
{
    x: X,
}

impl<X> FitComponent<X> where
    X: AudioComponent,
{
    pub fn new(x: X) -> Self { FitComponent { x } }
}

impl<X> AudioComponent for FitComponent<X> where
    X: AudioComponent,
{
    type Inputs = X::Inputs;
    type Outputs = X::Inputs;

    #[inline] fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
    }

    #[inline] fn tick(&mut self, input: &Frame<Self::Inputs>) -> Frame<Self::Outputs> {
        let output = self.x.tick(input);
        Frame::generate(|i| {
            match self.x.route_input(i as u32) {
                Some(j) => output[j as usize],
                None => if i < Self::Inputs::USIZE { input[i] } else { 0.0 }
            }
        })
    }

    fn latency(&self) -> Option<f64> { Some(0.0) }

    // FitComponent has identity routing because of tick(), so we do not need to override routing methods.
}
