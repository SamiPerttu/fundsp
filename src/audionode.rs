//! The central `AudioNode` abstraction and basic components.

use super::buffer::*;
use super::combinator::*;
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

/*
Order of type arguments in nodes:
1. Basic input and output arities excepting filter input selector arities.
2. Interface float type.
3. Processing float type.
4. Unary or binary operation type.
5. Filter input selector arity.
6. Contained node types.
7. The rest in any order.
*/

/// Generic audio processor.
/// `AudioNode` has a static number of inputs (`AudioNode::Inputs`) and outputs (`AudioNode::Outputs`).
/// `AudioNode` processes samples of type `AudioNode::Sample`, chosen statically.
pub trait AudioNode: Clone {
    /// Unique ID for hashing.
    const ID: u64;
    /// Sample type for input and output.
    type Sample: Float;
    /// Input arity.
    type Inputs: Size<Self::Sample>;
    /// Output arity.
    type Outputs: Size<Self::Sample>;
    /// Setting type. Settings are parameters that do not have a dedicated input.
    /// This is the unit type if there are no settings.
    type Setting: Sync + Send + Clone + Default;

    /// Reset the input state of the component to an initial state where it has
    /// not processed any samples. In other words, reset time to zero.
    /// The default implementation does nothing.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// let mut node = saw_hz(440.0);
    /// let sample1 = node.get_mono();
    /// node.reset();
    /// let sample2 = node.get_mono();
    /// assert_eq!(sample1, sample2);
    /// ```
    #[allow(unused_variables)]
    fn reset(&mut self) {}

    /// Set the sample rate of the unit.
    /// The default sample rate is 44100 Hz.
    /// The unit is allowed to reset its state here in response to sample rate changes.
    ///
    /// ### Example (Changing The Sample Rate)
    /// ```
    /// use fundsp::hacker::*;
    /// let mut node = saw_hz(440.0);
    /// node.set_sample_rate(48_000.0);
    /// ```
    #[allow(unused_variables)]
    fn set_sample_rate(&mut self, sample_rate: f64) {}

    /// Apply setting.
    /// The default implementation does nothing.
    #[allow(unused_variables)]
    fn set(&mut self, setting: Self::Setting) {}

    /// Process one sample.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// assert_eq!(pass().tick(&Frame::from([2.0])), Frame::from([2.0]));
    /// ```
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs>;

    /// Process up to 64 (`MAX_BUFFER_SIZE`) samples.
    /// The number of input and output buffers must match the number of inputs and outputs, respectively.
    /// All input and output buffers must be at least as large as `size`.
    /// If `size` is zero then this is a no-op, which is permitted.
    /// The default implementation is a fallback that calls into `tick`.
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

    /// Set node pseudorandom phase hash. Override this to use the hash.
    /// This is called from `ping` (only). It should not be called by users.
    /// The default implementation does nothing.
    #[allow(unused_variables)]
    fn set_hash(&mut self, hash: u64) {}

    /// Ping contained `AudioNode`s to obtain a deterministic pseudorandom hash.
    /// The local hash includes children, too.
    /// Leaf nodes should not need to override this.
    /// If `probe` is true, then this is a probe for computing the network hash
    /// and `set_hash` should not be called yet.
    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        if !probe {
            self.set_hash(hash.state());
        }
        hash.hash(Self::ID)
    }

    /// Route constants, latencies and frequency responses at `frequency` Hz
    /// from inputs to outputs. Return output signal.
    /// Default implementation marks all outputs unknown.
    #[allow(unused_variables)]
    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        new_signal_frame(self.outputs())
    }

    /// Preallocate all needed memory, including buffers for block processing.
    /// The default implementation does nothing.
    fn allocate(&mut self) {}

    // End of interface. There is no need to override the following.

    /// Number of inputs.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// assert_eq!(sink().inputs(), 1);
    /// ```
    #[inline]
    fn inputs(&self) -> usize {
        Self::Inputs::USIZE
    }

    /// Number of outputs.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// assert_eq!(zero().outputs(), 1);
    /// ```
    #[inline]
    fn outputs(&self) -> usize {
        Self::Outputs::USIZE
    }

    /// Evaluate frequency response of `output` at `frequency` Hz.
    /// Any linear response can be composed.
    /// Return `None` if there is no response or it could not be calculated.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// use num_complex::Complex64;
    /// assert_eq!(pass().response(0, 440.0), Some(Complex64::new(1.0, 0.0)));
    /// ```
    fn response(&mut self, output: usize, frequency: f64) -> Option<Complex64> {
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

    /// Evaluate frequency response of `output` in dB at `frequency` Hz.
    /// Any linear response can be composed.
    /// Return `None` if there is no response or it could not be calculated.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// let db = pass().response_db(0, 440.0).unwrap();
    /// assert!(db < 1.0e-7 && db > -1.0e-7);
    /// ```
    fn response_db(&mut self, output: usize, frequency: f64) -> Option<f64> {
        assert!(output < self.outputs());
        self.response(output, frequency).map(|r| amp_db(r.norm()))
    }

    /// Causal latency in (fractional) samples.
    /// After a reset, we can discard this many samples from the output to avoid incurring a pre-delay.
    /// The latency can depend on the sample rate and is allowed to change after `reset`.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// assert_eq!(pass().latency(), Some(0.0));
    /// assert_eq!(tick().latency(), Some(1.0));
    /// assert_eq!(sink().latency(), None);
    /// assert_eq!(lowpass_hz(440.0, 1.0).latency(), Some(0.0));
    /// ```
    fn latency(&mut self) -> Option<f64> {
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
    /// The node must have no inputs and 1 or 2 outputs.
    /// If there are two outputs, average the channels.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// assert_eq!(dc(2.0).get_mono(), 2.0);
    /// assert_eq!(dc((3.0, 4.0)).get_mono(), 3.5);
    /// ```
    #[inline]
    fn get_mono(&mut self) -> Self::Sample {
        assert!(
            Self::Inputs::USIZE == 0 && (Self::Outputs::USIZE == 1 || Self::Outputs::USIZE == 2)
        );
        let output = self.tick(&Frame::default());
        if self.outputs() == 1 {
            output[0]
        } else {
            (output[0] + output[1]) / Self::Sample::new(2)
        }
    }

    /// Retrieve the next stereo sample (left, right) from a generator.
    /// The node must have no inputs and 1 or 2 outputs.
    /// If there is just one output, duplicate it.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// assert_eq!(dc((5.0, 6.0)).get_stereo(), (5.0, 6.0));
    /// assert_eq!(dc(7.0).get_stereo(), (7.0, 7.0));
    /// ```
    #[inline]
    fn get_stereo(&mut self) -> (Self::Sample, Self::Sample) {
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
    /// use fundsp::hacker::*;
    /// assert_eq!(add(1.0).filter_mono(1.0), 2.0);
    /// ```
    #[inline]
    fn filter_mono(&mut self, x: Self::Sample) -> Self::Sample {
        assert!(Self::Inputs::USIZE == 1 && Self::Outputs::USIZE == 1);
        let output = self.tick(&Frame::splat(x));
        output[0]
    }

    /// Filter the next stereo sample `(x, y)`.
    /// The node must have exactly 2 inputs and 2 outputs.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// assert_eq!(add((2.0, 3.0)).filter_stereo(4.0, 5.0), (6.0, 8.0));
    /// ```
    #[inline]
    fn filter_stereo(&mut self, x: Self::Sample, y: Self::Sample) -> (Self::Sample, Self::Sample) {
        assert!(Self::Inputs::USIZE == 2 && Self::Outputs::USIZE == 2);
        let output = self.tick(&Frame::generate(|i| if i == 0 { x } else { y }));
        (output[0], output[1])
    }
}

/// Pass through inputs unchanged.
#[derive(Default, Clone)]
pub struct MultiPass<N, T> {
    _marker: PhantomData<(N, T)>,
}

impl<N: Size<T>, T: Float> MultiPass<N, T> {
    pub fn new() -> Self {
        MultiPass::default()
    }
}

impl<N: Size<T>, T: Float> AudioNode for MultiPass<N, T> {
    const ID: u64 = 0;
    type Sample = T;
    type Inputs = N;
    type Outputs = N;
    type Setting = ();

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
            output[i][..size].clone_from_slice(&input[i][..size]);
        }
    }
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        input.clone()
    }
}

/// Pass through input unchanged.
#[derive(Default, Clone)]
pub struct Pass<T> {
    _marker: PhantomData<T>,
}

impl<T: Float> Pass<T> {
    pub fn new() -> Self {
        Pass::default()
    }
}

// Note. We have separate Pass and MultiPass structs
// because it helps a little with type inference.
impl<T: Float> AudioNode for Pass<T> {
    const ID: u64 = 48;
    type Sample = T;
    type Inputs = U1;
    type Outputs = U1;
    type Setting = ();

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        *input
    }
    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        output[0][..size].clone_from_slice(&input[0][..size]);
    }
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        input.clone()
    }
}

/// Discard inputs.
#[derive(Default, Clone)]
pub struct Sink<N, T> {
    _marker: PhantomData<(N, T)>,
}

impl<N: Size<T>, T: Float> Sink<N, T> {
    pub fn new() -> Self {
        Sink::default()
    }
}

impl<N: Size<T>, T: Float> AudioNode for Sink<N, T> {
    const ID: u64 = 1;
    type Sample = T;
    type Inputs = N;
    type Outputs = U0;
    type Setting = ();

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
pub struct Constant<N: Size<T>, T: Float> {
    output: Frame<T, N>,
}

impl<N: Size<T>, T: Float> Constant<N, T> {
    /// Construct constant.
    pub fn new(output: Frame<T, N>) -> Self {
        Constant { output }
    }
    /// Set the value of the constant.
    #[inline]
    pub fn set_value(&mut self, output: Frame<T, N>) {
        self.output = output;
    }
    /// Get the value of the constant.
    #[inline]
    pub fn value(&self) -> Frame<T, N> {
        self.output.clone()
    }
    /// Set a scalar value on all channels.
    #[inline]
    pub fn set_scalar(&mut self, output: T) {
        self.output = Frame::splat(output);
    }
}

impl<N: Size<T>, T: Float> AudioNode for Constant<N, T> {
    const ID: u64 = 2;
    type Sample = T;
    type Inputs = U0;
    type Outputs = N;
    type Setting = Frame<T, N>;

    #[inline]
    fn set(&mut self, setting: Self::Setting) {
        self.output = setting;
    }

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
            output[i][..size].fill(self.output[i]);
        }
    }

    fn route(&mut self, _input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        for i in 0..N::USIZE {
            output[i] = Signal::Value(self.output[i].to_f64());
        }
        output
    }
}

/// Split input into `N` channels.
#[derive(Clone)]
pub struct Split<N, T> {
    _marker: PhantomData<(N, T)>,
}

// Note. We have separate split and multisplit (and join and multijoin)
// implementations because it helps with type inference.
impl<N, T> Split<N, T>
where
    N: Size<T>,
    T: Float,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            _marker: PhantomData::default(),
        }
    }
}

impl<N, T> AudioNode for Split<N, T>
where
    N: Size<T>,
    T: Float,
{
    const ID: u64 = 40;
    type Sample = T;
    type Inputs = U1;
    type Outputs = N;
    type Setting = ();

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        Frame::splat(input[0])
    }
    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        for channel in 0..N::USIZE {
            output[channel][..size].clone_from_slice(&input[0][..size]);
        }
    }
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Split.propagate(input, self.outputs())
    }
}

/// Split `M` inputs into `N` branches, with `M` * `N` outputs.
#[derive(Clone)]
pub struct MultiSplit<M, N, T> {
    _marker: PhantomData<(M, N, T)>,
}

impl<M, N, T> MultiSplit<M, N, T>
where
    M: Size<T> + Mul<N>,
    N: Size<T>,
    <M as Mul<N>>::Output: Size<T>,
    T: Float,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            _marker: PhantomData::default(),
        }
    }
}

impl<M, N, T> AudioNode for MultiSplit<M, N, T>
where
    M: Size<T> + Mul<N>,
    N: Size<T>,
    <M as Mul<N>>::Output: Size<T>,
    T: Float,
{
    const ID: u64 = 38;
    type Sample = T;
    type Inputs = M;
    type Outputs = numeric_array::typenum::Prod<M, N>;
    type Setting = ();

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        Frame::generate(|i| input[i % M::USIZE])
    }
    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        for channel in 0..M::USIZE * N::USIZE {
            output[channel][..size].clone_from_slice(&input[channel % M::USIZE][..size]);
        }
    }
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Split.propagate(input, self.outputs())
    }
}

/// Join `N` channels into one by averaging. Inverse of `Split<N, T>`.
#[derive(Clone)]
pub struct Join<N, T> {
    _marker: PhantomData<(N, T)>,
}

impl<N, T> Join<N, T>
where
    N: Size<T>,
    T: Float,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            _marker: PhantomData::default(),
        }
    }
}

impl<N, T> AudioNode for Join<N, T>
where
    N: Size<T>,
    T: Float,
{
    const ID: u64 = 41;
    type Sample = T;
    type Inputs = N;
    type Outputs = U1;
    type Setting = ();

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let mut output = input[0];
        for i in 1..N::USIZE {
            output += input[i];
        }
        [output / T::new(N::I64)].into()
    }
    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        let z = T::one() / T::new(N::I64);
        for (o, i) in output[0][..size].iter_mut().zip(input[0][..size].iter()) {
            *o = *i * z;
        }
        for channel in 1..N::USIZE {
            for (o, i) in output[0][..size]
                .iter_mut()
                .zip(input[channel][..size].iter())
            {
                *o += *i * z;
            }
        }
    }
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Join.propagate(input, self.outputs())
    }
}

/// Average `N` branches of `M` channels into one branch with `M` channels.
/// The input has `M` * `N` channels. Inverse of `MultiSplit<M, N, T>`.
#[derive(Clone)]
pub struct MultiJoin<M, N, T> {
    _marker: PhantomData<(M, N, T)>,
}

impl<M, N, T> MultiJoin<M, N, T>
where
    M: Size<T> + Mul<N>,
    N: Size<T>,
    <M as Mul<N>>::Output: Size<T>,
    T: Float,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            _marker: PhantomData::default(),
        }
    }
}

impl<M, N, T> AudioNode for MultiJoin<M, N, T>
where
    M: Size<T> + Mul<N>,
    N: Size<T>,
    <M as Mul<N>>::Output: Size<T>,
    T: Float,
{
    const ID: u64 = 39;
    type Sample = T;
    type Inputs = numeric_array::typenum::Prod<M, N>;
    type Outputs = M;
    type Setting = ();

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        Frame::generate(|j| {
            let mut output = input[j];
            for i in 1..N::USIZE {
                output += input[j + i * M::USIZE];
            }
            output / T::new(N::I64)
        })
    }
    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        let z = T::one() / T::new(N::I64);
        for channel in 0..M::USIZE {
            for (o, i) in output[channel][..size]
                .iter_mut()
                .zip(input[channel][..size].iter())
            {
                *o = *i * z;
            }
        }
        for channel in M::USIZE..M::USIZE * N::USIZE {
            for (o, i) in output[channel % M::USIZE][..size]
                .iter_mut()
                .zip(input[channel][..size].iter())
            {
                *o += *i * z;
            }
        }
    }
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Join.propagate(input, self.outputs())
    }
}

/// Provides binary operator implementations to the `Binop` node.
pub trait FrameBinop<N: Size<T>, T: Float>: Clone {
    /// Do binary op (x op y) channelwise.
    fn binop(x: &Frame<T, N>, y: &Frame<T, N>) -> Frame<T, N>;
    /// Do binary op (x op y) on signals.
    fn propagate(x: Signal, y: Signal) -> Signal;
    /// Do binary op (x op y) in-place lengthwise. Size may be zero.
    fn assign(size: usize, x: &mut [T], y: &[T]);
}

/// Addition operator.
#[derive(Default, Clone)]
pub struct FrameAdd<N: Size<T>, T: Float> {
    _marker: PhantomData<(N, T)>,
}

impl<N: Size<T>, T: Float> FrameAdd<N, T> {
    pub fn new() -> FrameAdd<N, T> {
        FrameAdd::default()
    }
}

impl<N: Size<T>, T: Float> FrameBinop<N, T> for FrameAdd<N, T> {
    #[inline]
    fn binop(x: &Frame<T, N>, y: &Frame<T, N>) -> Frame<T, N> {
        x + y
    }
    fn propagate(x: Signal, y: Signal) -> Signal {
        x.combine_linear(y, 0.0, |x, y| x + y, |x, y| x + y)
    }
    #[inline]
    fn assign(size: usize, x: &mut [T], y: &[T]) {
        for (o, i) in x[..size].iter_mut().zip(y[..size].iter()) {
            *o += *i;
        }
    }
}

/// Subtraction operator.
#[derive(Default, Clone)]
pub struct FrameSub<N: Size<T>, T: Float> {
    _marker: PhantomData<(N, T)>,
}

impl<N: Size<T>, T: Float> FrameSub<N, T> {
    pub fn new() -> FrameSub<N, T> {
        FrameSub::default()
    }
}

impl<N: Size<T>, T: Float> FrameBinop<N, T> for FrameSub<N, T> {
    #[inline]
    fn binop(x: &Frame<T, N>, y: &Frame<T, N>) -> Frame<T, N> {
        x - y
    }
    fn propagate(x: Signal, y: Signal) -> Signal {
        x.combine_linear(y, 0.0, |x, y| x - y, |x, y| x - y)
    }
    #[inline]
    fn assign(size: usize, x: &mut [T], y: &[T]) {
        for (o, i) in x[..size].iter_mut().zip(y[..size].iter()) {
            *o -= *i;
        }
    }
}

/// Multiplication operator.
#[derive(Default, Clone)]
pub struct FrameMul<N: Size<T>, T: Float> {
    _marker: PhantomData<(N, T)>,
}

impl<N: Size<T>, T: Float> FrameMul<N, T> {
    pub fn new() -> FrameMul<N, T> {
        FrameMul::default()
    }
}

impl<N: Size<T>, T: Float> FrameBinop<N, T> for FrameMul<N, T> {
    #[inline]
    fn binop(x: &Frame<T, N>, y: &Frame<T, N>) -> Frame<T, N> {
        x * y
    }
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
    #[inline]
    fn assign(size: usize, x: &mut [T], y: &[T]) {
        for (o, i) in x[..size].iter_mut().zip(y[..size].iter()) {
            *o *= *i;
        }
    }
}

/// Combine outputs of two nodes with a binary operation.
/// Inputs are disjoint.
/// Outputs are combined channelwise.
/// The nodes must have the same number of outputs.
#[derive(Clone)]
pub struct Binop<T, B, X, Y>
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

impl<T, B, X, Y> Binop<T, B, X, Y>
where
    T: Float,
    X: AudioNode<Sample = T>,
    Y: AudioNode<Sample = T, Outputs = X::Outputs>,
    B: FrameBinop<X::Outputs, T>,
    X::Inputs: Size<T> + Add<Y::Inputs>,
    X::Outputs: Size<T>,
    Y::Inputs: Size<T>,
    <X::Inputs as Add<Y::Inputs>>::Output: Size<T>,
{
    pub fn new(x: X, y: Y, b: B) -> Self {
        let mut node = Binop {
            _marker: PhantomData,
            x,
            y,
            b,
            buffer: Buffer::new(),
        };
        let hash = node.ping(true, AttoHash::new(Self::ID));
        node.ping(false, hash);
        node
    }

    /// Access the left node of the binary operation.
    #[inline]
    pub fn left_mut(&mut self) -> &mut X {
        &mut self.x
    }

    /// Access the left node of the binary operation.
    #[inline]
    pub fn left(&self) -> &X {
        &self.x
    }

    /// Access the right node of the binary operation.
    #[inline]
    pub fn right_mut(&mut self) -> &mut Y {
        &mut self.y
    }

    /// Access the right node of the binary operation.
    #[inline]
    pub fn right(&self) -> &Y {
        &self.y
    }
}

impl<T, B, X, Y> AudioNode for Binop<T, B, X, Y>
where
    T: Float,
    X: AudioNode<Sample = T>,
    Y: AudioNode<Sample = T, Outputs = X::Outputs>,
    B: FrameBinop<X::Outputs, T>,
    X::Outputs: Size<T>,
    X::Inputs: Size<T> + Add<Y::Inputs>,
    Y::Inputs: Size<T>,
    <X::Inputs as Add<Y::Inputs>>::Output: Size<T>,
{
    const ID: u64 = 3;
    type Sample = T;
    type Inputs = Sum<X::Inputs, Y::Inputs>;
    type Outputs = X::Outputs;
    type Setting = Side<X::Setting, Y::Setting>;

    fn set(&mut self, setting: Self::Setting) {
        match setting {
            Side::Left(value) => self.x.set(value),
            Side::Right(value) => self.y.set(value),
        }
    }

    fn reset(&mut self) {
        self.x.reset();
        self.y.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x.set_sample_rate(sample_rate);
        self.y.set_sample_rate(sample_rate);
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
        self.x.process(size, &input[..X::Inputs::USIZE], output);
        self.y.process(
            size,
            &input[X::Inputs::USIZE..],
            self.buffer.get_mut(self.outputs()),
        );
        for i in 0..self.outputs() {
            B::assign(size, output[i], self.buffer.at(i));
        }
    }

    #[inline]
    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.y.ping(probe, self.x.ping(probe, hash.hash(Self::ID)))
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut signal_x = self.x.route(input, frequency);
        let signal_y = self.y.route(
            &copy_signal_frame(input, X::Inputs::USIZE, Y::Inputs::USIZE),
            frequency,
        );
        for i in 0..Self::Outputs::USIZE {
            signal_x[i] = B::propagate(signal_x[i], signal_y[i]);
        }
        signal_x
    }

    fn allocate(&mut self) {
        self.buffer.resize(self.outputs());
        self.x.allocate();
        self.y.allocate();
    }
}

/// Provides unary operator implementations to the `Unop` node.
pub trait FrameUnop<N: Size<T>, T: Float>: Clone {
    /// Do unary op channelwise.
    fn unop(&self, x: &Frame<T, N>) -> Frame<T, N>;
    /// Do unary op on signal.
    fn propagate(&self, x: Signal) -> Signal;
    /// Do unary op in-place lengthwise.
    fn assign(&self, size: usize, x: &mut [T]);
}

/// Negation operator.
#[derive(Default, Clone)]
pub struct FrameNeg<N: Size<T>, T: Float> {
    _marker: PhantomData<(N, T)>,
}

impl<N: Size<T>, T: Float> FrameNeg<N, T> {
    pub fn new() -> FrameNeg<N, T> {
        FrameNeg::default()
    }
}

impl<N: Size<T>, T: Float> FrameUnop<N, T> for FrameNeg<N, T> {
    #[inline]
    fn unop(&self, x: &Frame<T, N>) -> Frame<T, N> {
        -x
    }
    fn propagate(&self, x: Signal) -> Signal {
        match x {
            Signal::Value(vx) => Signal::Value(-vx),
            Signal::Response(rx, lx) => Signal::Response(-rx, lx),
            s => s,
        }
    }
    #[inline]
    fn assign(&self, size: usize, x: &mut [T]) {
        for o in x[..size].iter_mut() {
            *o = -*o;
        }
    }
}

/// Identity op.
#[derive(Default, Clone)]
pub struct FrameId<N: Size<T>, T: Float> {
    _marker: PhantomData<(N, T)>,
}

impl<N: Size<T>, T: Float> FrameId<N, T> {
    pub fn new() -> FrameId<N, T> {
        FrameId::default()
    }
}

impl<N: Size<T>, T: Float> FrameUnop<N, T> for FrameId<N, T> {
    #[inline]
    fn unop(&self, x: &Frame<T, N>) -> Frame<T, N> {
        x.clone()
    }
    fn propagate(&self, x: Signal) -> Signal {
        x
    }
    #[inline]
    fn assign(&self, _size: usize, _x: &mut [T]) {}
}

/// Add scalar op.
#[derive(Default, Clone)]
pub struct FrameAddScalar<N: Size<T>, T: Float> {
    scalar: T,
    _marker: PhantomData<N>,
}

impl<N: Size<T>, T: Float> FrameAddScalar<N, T> {
    pub fn new(scalar: T) -> Self {
        Self {
            scalar,
            _marker: PhantomData::default(),
        }
    }
}

impl<N: Size<T>, T: Float> FrameUnop<N, T> for FrameAddScalar<N, T> {
    #[inline]
    fn unop(&self, x: &Frame<T, N>) -> Frame<T, N> {
        x + Frame::splat(self.scalar)
    }
    fn propagate(&self, x: Signal) -> Signal {
        match x {
            Signal::Value(vx) => Signal::Value(vx + self.scalar.to_f64()),
            s => s,
        }
    }
    #[inline]
    fn assign(&self, size: usize, x: &mut [T]) {
        for o in x[..size].iter_mut() {
            *o += self.scalar;
        }
    }
}

/// Multiply with scalar op.
#[derive(Default, Clone)]
pub struct FrameMulScalar<N: Size<T>, T: Float> {
    scalar: T,
    _marker: PhantomData<N>,
}

impl<N: Size<T>, T: Float> FrameMulScalar<N, T> {
    pub fn new(scalar: T) -> Self {
        Self {
            scalar,
            _marker: PhantomData::default(),
        }
    }
}

impl<N: Size<T>, T: Float> FrameUnop<N, T> for FrameMulScalar<N, T> {
    #[inline]
    fn unop(&self, x: &Frame<T, N>) -> Frame<T, N> {
        x * Frame::splat(self.scalar)
    }
    fn propagate(&self, x: Signal) -> Signal {
        match x {
            Signal::Response(vx, lx) => Signal::Response(vx * self.scalar.to_f64(), lx),
            Signal::Value(vx) => Signal::Value(vx * self.scalar.to_f64()),
            s => s,
        }
    }
    #[inline]
    fn assign(&self, size: usize, x: &mut [T]) {
        for o in x[..size].iter_mut() {
            *o *= self.scalar;
        }
    }
}

/// Apply a unary operation to output of contained node.
#[derive(Clone)]
pub struct Unop<T, X, U> {
    _marker: PhantomData<T>,
    x: X,
    #[allow(dead_code)]
    u: U,
}

impl<T, X, U> Unop<T, X, U>
where
    T: Float,
    X: AudioNode<Sample = T>,
    U: FrameUnop<X::Outputs, T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
{
    pub fn new(x: X, u: U) -> Self {
        let mut node = Unop {
            _marker: PhantomData,
            x,
            u,
        };
        let hash = node.ping(true, AttoHash::new(Self::ID));
        node.ping(false, hash);
        node
    }
}

impl<T, X, U> AudioNode for Unop<T, X, U>
where
    T: Float,
    X: AudioNode<Sample = T>,
    U: FrameUnop<X::Outputs, T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
{
    const ID: u64 = 4;
    type Sample = T;
    type Inputs = X::Inputs;
    type Outputs = X::Outputs;
    type Setting = X::Setting;

    fn set(&mut self, setting: Self::Setting) {
        self.x.set(setting);
    }

    fn reset(&mut self) {
        self.x.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x.set_sample_rate(sample_rate);
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        self.u.unop(&self.x.tick(input))
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
            self.u.assign(size, output[i]);
        }
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.x.ping(probe, hash.hash(Self::ID))
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut signal_x = self.x.route(input, frequency);
        for i in 0..Self::Outputs::USIZE {
            signal_x[i] = self.u.propagate(signal_x[i]);
        }
        signal_x
    }

    fn allocate(&mut self) {
        self.x.allocate();
    }
}

/// Map any number of channels.
#[derive(Clone)]
pub struct Map<T, M, I, O> {
    f: M,
    routing: Routing,
    _marker: PhantomData<(T, I, O)>,
}

impl<T, M, I, O> Map<T, M, I, O>
where
    T: Float,
    M: Fn(&Frame<T, I>) -> O + Clone,
    I: Size<T>,
    O: ConstantFrame<Sample = T>,
    O::Size: Size<T>,
{
    pub fn new(f: M, routing: Routing) -> Self {
        Self {
            f,
            routing,
            _marker: PhantomData::default(),
        }
    }
}

impl<T, M, I, O> AudioNode for Map<T, M, I, O>
where
    T: Float,
    M: Fn(&Frame<T, I>) -> O + Clone,
    I: Size<T>,
    O: ConstantFrame<Sample = T>,
    O::Size: Size<T>,
{
    const ID: u64 = 5;
    type Sample = T;
    type Inputs = I;
    type Outputs = O::Size;
    type Setting = ();

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        (self.f)(input).convert()
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        self.routing.propagate(input, O::Size::USIZE)
    }
}

/// Use setting from left or right side of a binary operation.
#[derive(Clone)]
pub enum Side<L: Clone + Default, R: Clone + Default> {
    Left(L),
    Right(R),
}

impl<L: Clone + Default, R: Clone + Default> Default for Side<L, R> {
    fn default() -> Self {
        Side::Left(L::default())
    }
}

/// Return setting for left side of a binary operation.
pub fn left<L: Clone + Default, R: Clone + Default>(value: L) -> Side<L, R> {
    Side::<L, R>::Left(value)
}

/// Return setting for right side of a binary operation.
pub fn right<L: Clone + Default, R: Clone + Default>(value: R) -> Side<L, R> {
    Side::<L, R>::Right(value)
}

/// Pipe the output of `X` to `Y`.
#[derive(Clone)]
pub struct Pipe<T, X, Y>
where
    T: Float,
{
    _marker: PhantomData<T>,
    x: X,
    y: Y,
    buffer: Buffer<T>,
}

impl<T, X, Y> Pipe<T, X, Y>
where
    T: Float,
    X: AudioNode<Sample = T>,
    Y: AudioNode<Sample = T, Inputs = X::Outputs>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    Y::Outputs: Size<T>,
{
    pub fn new(x: X, y: Y) -> Self {
        let mut node = Pipe {
            _marker: PhantomData,
            x,
            y,
            buffer: Buffer::new(),
        };
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

impl<T, X, Y> AudioNode for Pipe<T, X, Y>
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
    type Setting = Side<X::Setting, Y::Setting>;

    fn set(&mut self, setting: Self::Setting) {
        match setting {
            Side::Left(left) => self.x.set(left),
            Side::Right(right) => self.y.set(right),
        }
    }

    fn reset(&mut self) {
        self.x.reset();
        self.y.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x.set_sample_rate(sample_rate);
        self.y.set_sample_rate(sample_rate);
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
        self.y.process(size, self.buffer.self_ref(), output);
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.y.ping(probe, self.x.ping(probe, hash.hash(Self::ID)))
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.y.route(&self.x.route(input, frequency), frequency)
    }

    fn allocate(&mut self) {
        self.buffer.resize(self.x.outputs());
        self.x.allocate();
        self.y.allocate();
    }
}

/// Stack `X` and `Y` in parallel.
#[derive(Clone)]
pub struct Stack<T, X, Y> {
    _marker: PhantomData<T>,
    x: X,
    y: Y,
}

impl<T, X, Y> Stack<T, X, Y>
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
        let mut node = Stack {
            _marker: PhantomData,
            x,
            y,
        };
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

impl<T, X, Y> AudioNode for Stack<T, X, Y>
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
    type Setting = Side<X::Setting, Y::Setting>;

    fn set(&mut self, setting: Self::Setting) {
        match setting {
            Side::Left(left) => self.x.set(left),
            Side::Right(right) => self.y.set(right),
        }
    }

    fn reset(&mut self) {
        self.x.reset();
        self.y.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x.set_sample_rate(sample_rate);
        self.y.set_sample_rate(sample_rate);
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
    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        self.x.process(
            size,
            &input[..X::Inputs::USIZE],
            &mut output[..X::Outputs::USIZE],
        );
        self.y.process(
            size,
            &input[X::Inputs::USIZE..],
            &mut output[X::Outputs::USIZE..],
        );
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.y.ping(probe, self.x.ping(probe, hash.hash(Self::ID)))
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut signal_x = self.x.route(input, frequency);
        let signal_y = self.y.route(
            &copy_signal_frame(input, X::Inputs::USIZE, Y::Inputs::USIZE),
            frequency,
        );
        signal_x.resize(self.outputs(), Signal::Unknown);
        signal_x[X::Outputs::USIZE..Self::Outputs::USIZE]
            .copy_from_slice(&signal_y[0..Y::Outputs::USIZE]);
        signal_x
    }

    fn allocate(&mut self) {
        self.x.allocate();
        self.y.allocate();
    }
}

/// Send the same input to `X` and `Y`. Concatenate outputs.
#[derive(Clone)]
pub struct Branch<T, X, Y> {
    _marker: PhantomData<T>,
    x: X,
    y: Y,
}

impl<T, X, Y> Branch<T, X, Y>
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
        let mut node = Branch {
            _marker: PhantomData,
            x,
            y,
        };
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

impl<T, X, Y> AudioNode for Branch<T, X, Y>
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
    type Setting = Side<X::Setting, Y::Setting>;

    fn set(&mut self, setting: Self::Setting) {
        match setting {
            Side::Left(left) => self.x.set(left),
            Side::Right(right) => self.y.set(right),
        }
    }

    fn reset(&mut self) {
        self.x.reset();
        self.y.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x.set_sample_rate(sample_rate);
        self.y.set_sample_rate(sample_rate);
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
    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        self.x
            .process(size, input, &mut output[..X::Outputs::USIZE]);
        self.y
            .process(size, input, &mut output[X::Outputs::USIZE..]);
    }
    #[inline]
    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.y.ping(probe, self.x.ping(probe, hash.hash(Self::ID)))
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut signal_x = self.x.route(input, frequency);
        let signal_y = self.y.route(input, frequency);
        signal_x.resize(self.outputs(), Signal::Unknown);
        signal_x[X::Outputs::USIZE..Self::Outputs::USIZE]
            .copy_from_slice(&signal_y[0..Y::Outputs::USIZE]);
        signal_x
    }
    fn allocate(&mut self) {
        self.x.allocate();
        self.y.allocate();
    }
}

/// Mix together `X` and `Y` sourcing from the same inputs.
#[derive(Clone)]
pub struct Bus<T, X, Y>
where
    T: Float,
{
    _marker: PhantomData<T>,
    x: X,
    y: Y,
    buffer: Buffer<T>,
}

impl<T, X, Y> Bus<T, X, Y>
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
        let mut node = Bus {
            _marker: PhantomData,
            x,
            y,
            buffer: Buffer::new(),
        };
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

impl<T, X, Y> AudioNode for Bus<T, X, Y>
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
    type Setting = Side<X::Setting, Y::Setting>;

    fn set(&mut self, setting: Self::Setting) {
        match setting {
            Side::Left(left) => self.x.set(left),
            Side::Right(right) => self.y.set(right),
        }
    }

    fn reset(&mut self) {
        self.x.reset();
        self.y.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x.set_sample_rate(sample_rate);
        self.y.set_sample_rate(sample_rate);
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

    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        self.x.process(size, input, output);
        self.y
            .process(size, input, self.buffer.get_mut(self.outputs()));
        for channel in 0..self.outputs() {
            for (o, i) in output[channel][..size]
                .iter_mut()
                .zip(self.buffer.at(channel)[..size].iter())
            {
                *o += *i;
            }
        }
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.y.ping(probe, self.x.ping(probe, hash.hash(Self::ID)))
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut signal_x = self.x.route(input, frequency);
        let signal_y = self.y.route(input, frequency);
        for i in 0..Self::Outputs::USIZE {
            signal_x[i] = signal_x[i].combine_linear(signal_y[i], 0.0, |x, y| x + y, |x, y| x + y);
        }
        signal_x
    }

    fn allocate(&mut self) {
        self.buffer.resize(self.outputs());
        self.x.allocate();
        self.y.allocate();
    }
}

/// Pass through inputs without matching outputs.
/// Adjusts output arity to match input arity, adapting a filter to a pipeline.
#[derive(Clone)]
pub struct Thru<X: AudioNode> {
    x: X,
    buffer: Buffer<X::Sample>,
}

impl<X: AudioNode> Thru<X> {
    pub fn new(x: X) -> Self {
        let mut node = Thru {
            x,
            buffer: Buffer::new(),
        };
        let hash = node.ping(true, AttoHash::new(Self::ID));
        node.ping(false, hash);
        node
    }
}

impl<X: AudioNode> AudioNode for Thru<X> {
    const ID: u64 = 12;
    type Sample = X::Sample;
    type Inputs = X::Inputs;
    type Outputs = X::Inputs;
    type Setting = X::Setting;

    fn set(&mut self, setting: Self::Setting) {
        self.x.set(setting);
    }

    fn reset(&mut self) {
        self.x.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x.set_sample_rate(sample_rate);
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let output = self.x.tick(input);
        Frame::generate(|channel| {
            if channel < X::Outputs::USIZE {
                output[channel]
            } else {
                input[channel]
            }
        })
    }
    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        if X::Inputs::USIZE == 0 {
            // This is an empty node.
            return;
        }
        if X::Inputs::USIZE < X::Outputs::USIZE {
            // The intermediate buffer is only used in this "degenerate" case where
            // we are not passing through inputs - we are cutting out some of them.
            self.x
                .process(size, input, self.buffer.get_mut(X::Outputs::USIZE));
            for channel in 0..X::Inputs::USIZE {
                output[channel][..size].clone_from_slice(&self.buffer.at(channel)[..size]);
            }
        } else {
            self.x
                .process(size, input, &mut output[..X::Outputs::USIZE]);
            for channel in X::Outputs::USIZE..X::Inputs::USIZE {
                output[channel][..size].clone_from_slice(&input[channel][..size]);
            }
        }
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.x.ping(probe, hash.hash(Self::ID))
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = self.x.route(input, frequency);
        output[X::Outputs::USIZE..Self::Outputs::USIZE]
            .copy_from_slice(&input[X::Outputs::USIZE..Self::Outputs::USIZE]);
        output
    }

    fn allocate(&mut self) {
        if X::Inputs::USIZE < X::Outputs::USIZE {
            self.buffer.resize(X::Outputs::USIZE);
        }
        self.x.allocate();
    }
}

/// Mix together a bunch of similar nodes sourcing from the same inputs.
#[derive(Clone)]
pub struct MultiBus<N, T, X>
where
    N: Size<T>,
    N: Size<X>,
    T: Float,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
{
    _marker: PhantomData<T>,
    x: Frame<X, N>,
    buffer: Buffer<T>,
}

impl<N, T, X> MultiBus<N, T, X>
where
    N: Size<T>,
    N: Size<X>,
    T: Float,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
{
    pub fn new(x: Frame<X, N>) -> Self {
        let mut node = MultiBus {
            _marker: PhantomData::default(),
            x,
            buffer: Buffer::new(),
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

impl<N, T, X> AudioNode for MultiBus<N, T, X>
where
    N: Size<T>,
    N: Size<X>,
    T: Float,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
{
    const ID: u64 = 28;
    type Sample = T;
    type Inputs = X::Inputs;
    type Outputs = X::Outputs;
    type Setting = (usize, X::Setting);

    #[inline]
    fn set(&mut self, setting: Self::Setting) {
        let (index, inner) = setting;
        self.x[index].set(inner);
    }

    fn reset(&mut self) {
        self.x.iter_mut().for_each(|node| node.reset());
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x
            .iter_mut()
            .for_each(|node| node.set_sample_rate(sample_rate));
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
    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        self.x[0].process(size, input, output);
        for i in 1..N::USIZE {
            self.x[i].process(size, input, self.buffer.get_mut(X::Outputs::USIZE));
            for channel in 0..X::Outputs::USIZE {
                for (o, i) in output[channel][..size]
                    .iter_mut()
                    .zip(self.buffer.at(channel)[..size].iter())
                {
                    *o += *i;
                }
            }
        }
    }
    #[inline]
    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        let mut hash = hash.hash(Self::ID);
        for x in &mut self.x {
            hash = x.ping(probe, hash);
        }
        hash
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        if self.x.is_empty() {
            return new_signal_frame(self.outputs());
        }
        let mut output = self.x[0].route(input, frequency);
        for i in 1..self.x.len() {
            let output_i = self.x[i].route(input, frequency);
            for channel in 0..Self::Outputs::USIZE {
                output[channel] = output[channel].combine_linear(
                    output_i[channel],
                    0.0,
                    |x, y| x + y,
                    |x, y| x + y,
                );
            }
        }
        output
    }

    fn allocate(&mut self) {
        self.buffer.resize(X::Outputs::USIZE);
        for x in &mut self.x {
            x.allocate();
        }
    }
}

/// Stack a bunch of similar nodes in parallel.
#[derive(Clone)]
pub struct MultiStack<N, T, X>
where
    N: Size<T>,
    N: Size<X>,
    T: Float,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T> + Mul<N>,
    X::Outputs: Size<T> + Mul<N>,
    <X::Inputs as Mul<N>>::Output: Size<T>,
    <X::Outputs as Mul<N>>::Output: Size<T>,
{
    _marker: PhantomData<(N, T)>,
    x: Frame<X, N>,
}

impl<N, T, X> MultiStack<N, T, X>
where
    N: Size<T>,
    N: Size<X>,
    T: Float,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T> + Mul<N>,
    X::Outputs: Size<T> + Mul<N>,
    <X::Inputs as Mul<N>>::Output: Size<T>,
    <X::Outputs as Mul<N>>::Output: Size<T>,
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

impl<N, T, X> AudioNode for MultiStack<N, T, X>
where
    N: Size<T>,
    N: Size<X>,
    T: Float,
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
    type Setting = (usize, X::Setting);

    fn set(&mut self, setting: Self::Setting) {
        let (index, inner) = setting;
        self.x[index].set(inner);
    }

    fn reset(&mut self) {
        self.x.iter_mut().for_each(|node| node.reset());
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x
            .iter_mut()
            .for_each(|node| node.set_sample_rate(sample_rate));
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
    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        let mut in_channel = 0;
        let mut out_channel = 0;
        for i in 0..N::USIZE {
            let next_in_channel = in_channel + X::Inputs::USIZE;
            let next_out_channel = out_channel + X::Outputs::USIZE;
            self.x[i].process(
                size,
                &input[in_channel..next_in_channel],
                &mut output[out_channel..next_out_channel],
            );
            in_channel = next_in_channel;
            out_channel = next_out_channel;
        }
    }
    #[inline]
    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        let mut hash = hash.hash(Self::ID);
        for x in self.x.iter_mut() {
            hash = x.ping(probe, hash);
        }
        hash
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        if self.x.is_empty() {
            return new_signal_frame(self.outputs());
        }
        let mut output = self.x[0].route(input, frequency);
        output.resize(self.outputs(), Signal::Unknown);
        for i in 1..N::USIZE {
            let output_i = self.x[i].route(
                &copy_signal_frame(input, i * X::Inputs::USIZE, X::Inputs::USIZE),
                frequency,
            );
            output[i * X::Outputs::USIZE..(i + 1) * X::Outputs::USIZE]
                .copy_from_slice(&output_i[0..X::Outputs::USIZE]);
        }
        output
    }

    fn allocate(&mut self) {
        for x in &mut self.x {
            x.allocate();
        }
    }
}

/// Combine outputs of a bunch of similar nodes with a binary operation.
/// Inputs are disjoint.
/// Outputs are combined channel-wise.
#[derive(Clone)]
pub struct Reduce<N, T, X, B>
where
    N: Size<T>,
    N: Size<X>,
    T: Float,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T> + Mul<N>,
    X::Outputs: Size<T>,
    <X::Inputs as Mul<N>>::Output: Size<T>,
    B: FrameBinop<X::Outputs, T>,
{
    x: Frame<X, N>,
    #[allow(dead_code)]
    b: B,
    buffer: Buffer<T>,
    _marker: PhantomData<T>,
}

impl<N, T, X, B> Reduce<N, T, X, B>
where
    N: Size<T>,
    N: Size<X>,
    T: Float,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T> + Mul<N>,
    X::Outputs: Size<T>,
    <X::Inputs as Mul<N>>::Output: Size<T>,
    B: FrameBinop<X::Outputs, T>,
{
    pub fn new(x: Frame<X, N>, b: B) -> Self {
        let mut node = Reduce {
            x,
            b,
            buffer: Buffer::new(),
            _marker: PhantomData,
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

impl<N, T, X, B> AudioNode for Reduce<N, T, X, B>
where
    N: Size<T>,
    N: Size<X>,
    T: Float,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T> + Mul<N>,
    X::Outputs: Size<T>,
    <X::Inputs as Mul<N>>::Output: Size<T>,
    B: FrameBinop<X::Outputs, T>,
{
    const ID: u64 = 32;
    type Sample = T;
    type Inputs = Prod<X::Inputs, N>;
    type Outputs = X::Outputs;
    type Setting = (usize, X::Setting);

    fn set(&mut self, setting: Self::Setting) {
        let (index, inner) = setting;
        self.x[index].set(inner);
    }

    fn reset(&mut self) {
        self.x.iter_mut().for_each(|node| node.reset());
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x
            .iter_mut()
            .for_each(|node| node.set_sample_rate(sample_rate));
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
    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        self.x[0].process(size, &input[..X::Inputs::USIZE], output);
        let mut in_channel = X::Inputs::USIZE;
        for i in 1..N::USIZE {
            let next_in_channel = in_channel + X::Inputs::USIZE;
            self.x[i].process(
                size,
                &input[in_channel..next_in_channel],
                self.buffer.get_mut(X::Outputs::USIZE),
            );
            in_channel = next_in_channel;
            for channel in 0..X::Outputs::USIZE {
                B::assign(size, output[channel], self.buffer.at(channel));
            }
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
            return new_signal_frame(self.outputs());
        }
        let mut output = self.x[0].route(input, frequency);
        for j in 1..self.x.len() {
            let output_j = self.x[j].route(
                &copy_signal_frame(input, j * X::Inputs::USIZE, X::Inputs::USIZE),
                frequency,
            );
            for i in 0..Self::Outputs::USIZE {
                output[i] = B::propagate(output[i], output_j[i]);
            }
        }
        output
    }

    fn allocate(&mut self) {
        self.buffer.resize(X::Outputs::USIZE);
        for x in &mut self.x {
            x.allocate();
        }
    }
}

/// Branch into a bunch of similar nodes in parallel.
#[derive(Clone)]
pub struct MultiBranch<N, T, X>
where
    N: Size<T>,
    N: Size<X>,
    T: Float,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T> + Mul<N>,
    <X::Outputs as Mul<N>>::Output: Size<T>,
{
    _marker: PhantomData<T>,
    x: Frame<X, N>,
}

impl<N, T, X> MultiBranch<N, T, X>
where
    N: Size<T>,
    N: Size<X>,
    T: Float,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T> + Mul<N>,
    <X::Outputs as Mul<N>>::Output: Size<T>,
{
    pub fn new(x: Frame<X, N>) -> Self {
        let mut node = MultiBranch {
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

impl<N, T, X> AudioNode for MultiBranch<N, T, X>
where
    N: Size<T>,
    N: Size<X>,
    T: Float,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T> + Mul<N>,
    <X::Outputs as Mul<N>>::Output: Size<T>,
{
    const ID: u64 = 33;
    type Sample = T;
    type Inputs = X::Inputs;
    type Outputs = Prod<X::Outputs, N>;
    type Setting = (usize, X::Setting);

    fn set(&mut self, setting: Self::Setting) {
        let (index, inner) = setting;
        self.x[index].set(inner);
    }

    fn reset(&mut self) {
        self.x.iter_mut().for_each(|node| node.reset());
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x
            .iter_mut()
            .for_each(|node| node.set_sample_rate(sample_rate));
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
    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        let mut out_channel = 0;
        for i in 0..N::USIZE {
            let next_out_channel = out_channel + X::Outputs::USIZE;
            self.x[i].process(size, input, &mut output[out_channel..next_out_channel]);
            out_channel = next_out_channel;
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
            return new_signal_frame(self.outputs());
        }
        let mut output = self.x[0].route(input, frequency);
        output.resize(self.outputs(), Signal::Unknown);
        for i in 1..N::USIZE {
            let output_i = self.x[i].route(input, frequency);
            output[i * X::Outputs::USIZE..(i + 1) * X::Outputs::USIZE]
                .copy_from_slice(&output_i[0..X::Outputs::USIZE]);
        }
        output
    }

    fn allocate(&mut self) {
        for x in &mut self.x {
            x.allocate();
        }
    }
}

/// Chain together a bunch of similar nodes.
#[derive(Clone)]
pub struct Chain<N, T, X>
where
    N: Size<T>,
    N: Size<X>,
    T: Float,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
{
    x: Frame<X, N>,
    buffer_a: Buffer<T>,
    buffer_b: Buffer<T>,
    _marker: PhantomData<T>,
}

impl<N, T, X> Chain<N, T, X>
where
    N: Size<T>,
    N: Size<X>,
    T: Float,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
{
    pub fn new(x: Frame<X, N>) -> Self {
        let mut node = Chain {
            x,
            buffer_a: Buffer::new(),
            buffer_b: Buffer::new(),
            _marker: PhantomData,
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

impl<N, T, X> AudioNode for Chain<N, T, X>
where
    N: Size<T>,
    N: Size<X>,
    T: Float,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
{
    const ID: u64 = 32;
    type Sample = T;
    // TODO. We'd like to require that X::Inputs equals X::Outputs but
    // I don't know how to write such a trait bound.
    type Inputs = X::Inputs;
    type Outputs = X::Outputs;
    type Setting = (usize, X::Setting);

    fn set(&mut self, setting: Self::Setting) {
        let (index, inner) = setting;
        self.x[index].set(inner);
    }

    fn reset(&mut self) {
        self.x.iter_mut().for_each(|node| node.reset());
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x
            .iter_mut()
            .for_each(|node| node.set_sample_rate(sample_rate));
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let mut output = self.x[0].tick(input);
        for i in 1..N::USIZE {
            output = self.x[i].tick(&Frame::generate(|i| output[i]));
        }
        output
    }

    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        if N::USIZE == 1 {
            self.x[0].process(size, input, output);
        } else {
            self.x[0].process(size, input, self.buffer_a.get_mut(X::Outputs::USIZE));
            self.buffer_b.resize(X::Outputs::USIZE);
            for i in 1..N::USIZE - 1 {
                if i & 1 > 0 {
                    self.x[i].process(size, self.buffer_a.self_ref(), self.buffer_b.self_mut());
                } else {
                    self.x[i].process(size, self.buffer_b.self_ref(), self.buffer_a.self_mut());
                }
            }
            if (N::USIZE - 1) & 1 > 0 {
                self.x[N::USIZE - 1].process(size, self.buffer_a.self_ref(), output);
            } else {
                self.x[N::USIZE - 1].process(size, self.buffer_b.self_ref(), output);
            }
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
        for i in 1..self.x.len() {
            output = self.x[i].route(&output, frequency);
        }
        output
    }

    fn allocate(&mut self) {
        self.buffer_a.resize(X::Outputs::USIZE);
        self.buffer_b.resize(X::Outputs::USIZE);
        for x in &mut self.x {
            x.allocate();
        }
    }
}

/// Reverse channel order.
#[derive(Default, Clone)]
pub struct Reverse<N, T> {
    _marker: PhantomData<(N, T)>,
}

impl<N: Size<T>, T: Float> Reverse<N, T> {
    pub fn new() -> Self {
        Reverse::default()
    }
}

impl<N: Size<T>, T: Float> AudioNode for Reverse<N, T> {
    const ID: u64 = 45;
    type Sample = T;
    type Inputs = N;
    type Outputs = N;
    type Setting = ();

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        Frame::generate(|i| input[N::USIZE - 1 - i])
    }
    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        for i in 0..N::USIZE {
            output[i][..size].clone_from_slice(&input[N::USIZE - 1 - i][..size]);
        }
    }
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Reverse.propagate(input, N::USIZE)
    }
}
