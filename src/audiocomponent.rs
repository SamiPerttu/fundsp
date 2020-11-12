use super::*;
use super::prelude::*;
use numeric_array::*;
use generic_array::sequence::*;
use numeric_array::typenum::*;
use std::marker::PhantomData;

/// AudioComponent processes audio data sample by sample.
/// It has a static number of inputs and outputs known at compile time.
/// If not set otherwise, the sample rate is the system default DEFAULT_SR.
pub trait AudioComponent
{
    type Sample: AudioFloat;
    type Inputs: ArrayLength<Self::Sample>;
    type Outputs: ArrayLength<Self::Sample>;

    /// Resets the input state of the component to an initial state where it has not processed any samples.
    fn reset(&mut self, _sample_rate: Option<f64>) {}

    /// Processes one sample.
    fn tick(&mut self, input: &NumericArray<Self::Sample, Self::Inputs>) -> NumericArray<Self::Sample, Self::Outputs>;

    /// Causal latency from input to output, in (fractional) samples.
    /// After a reset, we can discard this many samples from the output to avoid incurring a pre-delay.
    /// This applies only to components that have both inputs and outputs; others should return 0.0.
    /// The latency can depend on the sample rate and is allowed to change after a reset.
    fn latency(&self) -> f64 { 0.0 }

    /// Number of inputs.
    #[inline] fn inputs(&self) -> usize { Self::Inputs::USIZE }

    /// Number of outputs.
    #[inline] fn outputs(&self) -> usize { Self::Outputs::USIZE }

    /// Retrieves the next mono sample from an all-zero input. Convenience method.
    fn get_mono(&mut self) -> Self::Sample {
        self.tick(&NumericArray::default())[0]
    }

    /// Retrieves the next stereo sample pair (left, right) from an all-zero input. Convenience method.
    fn get_stereo(&mut self) -> (Self::Sample, Self::Sample) {
        let output = self.tick(&NumericArray::default());
        (output[0], output[1])
    }
}

pub struct MonoIter<A: AudioComponent>(pub A);

/// PassComponent passes through its inputs unchanged.
#[derive(Clone)]
pub struct PassComponent<S: AudioFloat, N: ArrayLength<S>>
{
    _sample: PhantomData<S>,
    _length: PhantomData<N>,
}

impl<S: AudioFloat, N: ArrayLength<S>> PassComponent<S, N>
{
    pub fn new() -> Self { PassComponent { _sample: PhantomData::default(), _length: PhantomData::default() } }
}

impl<S: AudioFloat, N: ArrayLength<S>> AudioComponent for PassComponent<S, N>
{
    type Sample = S;
    type Inputs = N;
    type Outputs = N;

    fn tick(&mut self, input: &NumericArray<S, Self::Inputs>) -> NumericArray<S, Self::Outputs> { input.clone() }
}

/// ConstantComponent outputs a constant value.
#[derive(Clone)]
pub struct ConstantComponent<S: AudioFloat, N: ArrayLength<S>>
{
    output: NumericArray<S, N>,
}

impl<S: AudioFloat, N: ArrayLength<S>> ConstantComponent<S, N>
{
    pub fn new(output: NumericArray<S, N>) -> Self { ConstantComponent { output } }
}

impl<S: AudioFloat, N: ArrayLength<S>> AudioComponent for ConstantComponent<S, N>
{
    type Sample = S;
    type Inputs = typenum::U0;
    type Outputs = N;

    fn tick(&mut self, _input: &NumericArray<S, Self::Inputs>) -> NumericArray<S, Self::Outputs> { self.output.clone() }
}

#[derive(Clone)]
pub enum Binop { Add, Sub, Mul }

/// BinaryComponent combines outputs of two components, channel-wise, with a binary operation.
/// The components must have the same number of outputs.
#[derive(Clone)]
pub struct BinaryComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Sample = X::Sample, Outputs = X::Outputs>,
    X::Inputs: ArrayLength<X::Sample> + Add<Y::Inputs>,
    Y::Inputs: ArrayLength<Y::Sample>,
    <<X as AudioComponent>::Inputs as Add<<Y as AudioComponent>::Inputs>>::Output: ArrayLength<<X as AudioComponent>::Sample>,
{
    x: X,
    y: Y,
    b: Binop,
}

impl<X, Y> BinaryComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Sample = X::Sample, Outputs = X::Outputs>,
    X::Inputs: ArrayLength<X::Sample> + Add<Y::Inputs>,
    Y::Inputs: ArrayLength<Y::Sample>,
    <<X as AudioComponent>::Inputs as Add<<Y as AudioComponent>::Inputs>>::Output: ArrayLength<<X as AudioComponent>::Sample>,
{
    pub fn new(x: X, y: Y, b: Binop) -> Self { BinaryComponent { x, y, b } }
}

impl<X, Y> AudioComponent for BinaryComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Sample = X::Sample, Outputs = X::Outputs>,
    X::Inputs: ArrayLength<X::Sample> + Add<Y::Inputs>,
    Y::Inputs: ArrayLength<Y::Sample>,
    <<X as AudioComponent>::Inputs as Add<<Y as AudioComponent>::Inputs>>::Output: ArrayLength<<X as AudioComponent>::Sample>,
{
    type Sample = X::Sample;
    type Inputs = Sum<X::Inputs, Y::Inputs>;
    type Outputs = X::Outputs;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.y.reset(sample_rate);
    }
    fn tick(&mut self, input: &NumericArray<Self::Sample, Self::Inputs>) -> NumericArray<Self::Sample, Self::Outputs> {
        let input_x = &input[0 .. X::Inputs::USIZE];
        let input_y = &input[Self::Inputs::USIZE - Y::Inputs::USIZE .. Self::Inputs::USIZE];
        let x = self.x.tick(input_x.into());
        let y = self.y.tick(input_y.into());
        match self.b {
            Binop::Add => x + y,
            Binop::Sub => x - y,
            Binop::Mul => x * y,
        }
    }
    fn latency(&self) -> f64 { self.x.latency().min(self.y.latency()) }
}

/// SerialComponent pipes the output of X to Y.
#[derive(Clone)]
pub struct SerialComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Sample = X::Sample, Inputs = X::Outputs>,
    Y::Outputs: ArrayLength<Y::Sample>,
{
    x: X,
    y: Y,
}

impl<X, Y> SerialComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Sample = X::Sample, Inputs = X::Outputs>,
    Y::Outputs: ArrayLength<Y::Sample>,
{
    pub fn new(x: X, y: Y) -> Self { SerialComponent { x, y } }
}

impl<X, Y> AudioComponent for SerialComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Sample = X::Sample, Inputs = X::Outputs>,
    Y::Outputs: ArrayLength<Y::Sample>,
{
    type Sample = X::Sample;
    type Inputs = X::Inputs;
    type Outputs = Y::Outputs;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.y.reset(sample_rate);
    }
    fn tick(&mut self, input: &NumericArray<Self::Sample, Self::Inputs>) -> NumericArray<Self::Sample, Self::Outputs> {
        self.y.tick(&self.x.tick(input))
    }
    fn latency(&self) -> f64 { self.x.latency() + self.y.latency() }
}

//// ParallelComponent combines X and Y in parallel.
#[derive(Clone)]
pub struct ParallelComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Sample = X::Sample>,
    X::Inputs: ArrayLength<X::Sample> + Add<Y::Inputs>,
    X::Outputs: ArrayLength<X::Sample> + Add<Y::Outputs>,
    Y::Inputs: ArrayLength<Y::Sample>,
    Y::Outputs: ArrayLength<Y::Sample>,
    <<X as AudioComponent>::Inputs as Add<<Y as AudioComponent>::Inputs>>::Output: ArrayLength<<X as AudioComponent>::Sample>,
    <<X as AudioComponent>::Outputs as Add<<Y as AudioComponent>::Outputs>>::Output: ArrayLength<<X as AudioComponent>::Sample>
{
    x: X,
    y: Y,
}

impl<X, Y> ParallelComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Sample = X::Sample>,
    X::Inputs: ArrayLength<X::Sample> + Add<Y::Inputs>,
    X::Outputs: ArrayLength<X::Sample> + Add<Y::Outputs>,
    Y::Inputs: ArrayLength<Y::Sample>,
    Y::Outputs: ArrayLength<Y::Sample>,
    <<X as AudioComponent>::Inputs as Add<<Y as AudioComponent>::Inputs>>::Output: ArrayLength<<X as AudioComponent>::Sample>,
    <<X as AudioComponent>::Outputs as Add<<Y as AudioComponent>::Outputs>>::Output: ArrayLength<<X as AudioComponent>::Sample>
{
    pub fn new(x: X, y: Y) -> Self { ParallelComponent { x, y } }
}

use core::ops::Add;

impl<X, Y> AudioComponent for ParallelComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Sample = X::Sample>,
    X::Inputs: ArrayLength<X::Sample> + Add<Y::Inputs>,
    X::Outputs: ArrayLength<X::Sample> + Add<Y::Outputs>,
    Y::Inputs: ArrayLength<Y::Sample>,
    Y::Outputs: ArrayLength<Y::Sample>,
    <<X as AudioComponent>::Inputs as Add<<Y as AudioComponent>::Inputs>>::Output: ArrayLength<<X as AudioComponent>::Sample>,
    <<X as AudioComponent>::Outputs as Add<<Y as AudioComponent>::Outputs>>::Output: ArrayLength<<X as AudioComponent>::Sample>
{
    type Sample = X::Sample;
    type Inputs = typenum::Sum<X::Inputs, Y::Inputs>;
    type Outputs = typenum::Sum<X::Outputs, Y::Outputs>;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.y.reset(sample_rate);
    }
    fn tick(&mut self, input: &NumericArray<Self::Sample, Self::Inputs>) -> NumericArray<Self::Sample, Self::Outputs> {
        let input_x = &input[0 .. X::Inputs::USIZE];
        let input_y = &input[Self::Inputs::USIZE - Y::Inputs::USIZE .. Self::Inputs::USIZE];
        let output_x = self.x.tick(input_x.into());
        let output_y = self.y.tick(input_y.into());
        NumericArray::generate(|i| if i < X::Outputs::USIZE { output_x[i] } else { output_y[i - X::Outputs::USIZE] })
    }
    fn latency(&self) -> f64 { self.x.latency().min(self.y.latency()) }
}

/// BranchComponent sends the same input to X and Y and concatenates the outputs.
#[derive(Clone)]
pub struct BranchComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Sample = X::Sample, Inputs = X::Inputs>,
    X::Outputs: ArrayLength<X::Sample> + Add<Y::Outputs>,
    Y::Outputs: ArrayLength<Y::Sample>,
    <<X as AudioComponent>::Outputs as Add<<Y as AudioComponent>::Outputs>>::Output: ArrayLength<<X as AudioComponent>::Sample>
{
    x: X,
    y: Y,
}

impl<X, Y> BranchComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Sample = X::Sample, Inputs = X::Inputs>,
    X::Outputs: ArrayLength<X::Sample> + Add<Y::Outputs>,
    Y::Outputs: ArrayLength<Y::Sample>,
    <<X as AudioComponent>::Outputs as Add<<Y as AudioComponent>::Outputs>>::Output: ArrayLength<<X as AudioComponent>::Sample>
{
    pub fn new(x: X, y: Y) -> Self { BranchComponent { x, y } }
}

impl<X, Y> AudioComponent for BranchComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Sample = X::Sample, Inputs = X::Inputs>,
    X::Outputs: ArrayLength<X::Sample> + Add<Y::Outputs>,
    Y::Outputs: ArrayLength<Y::Sample>,
    <<X as AudioComponent>::Outputs as Add<<Y as AudioComponent>::Outputs>>::Output: ArrayLength<<X as AudioComponent>::Sample>
{
    type Sample = X::Sample;
    type Inputs = X::Inputs;
    type Outputs = typenum::Sum<X::Outputs, Y::Outputs>;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.y.reset(sample_rate);
    }
    fn tick(&mut self, input: &NumericArray<Self::Sample, Self::Inputs>) -> NumericArray<Self::Sample, Self::Outputs> {
        let output_x = self.x.tick(input);
        let output_y = self.y.tick(input);
        NumericArray::generate(|i| if i < X::Outputs::USIZE { output_x[i] } else { output_y[i - X::Outputs::USIZE] })
    }
    fn latency(&self) -> f64 { self.x.latency().min(self.y.latency()) }
}

pub trait ConstantFrame {
    type Length: ArrayLength<F32>;
    fn convert(self) -> NumericArray<F32, Self::Length>;
}

impl ConstantFrame for F32 {
    type Length = typenum::U1;
    fn convert(self) -> NumericArray<F32, Self::Length> { [self].into() }
}

impl ConstantFrame for (F32, F32) {
    type Length = typenum::U2;
    fn convert(self) -> NumericArray<F32, Self::Length> { [self.0, self.1].into() }
}

impl ConstantFrame for (F32, F32, F32) {
    type Length = typenum::U3;
    fn convert(self) -> NumericArray<F32, Self::Length> { [self.0, self.1, self.2].into() }
}

/// AudioComponent wrapper.
pub struct Ac<X: AudioComponent>(pub X);

impl<X: AudioComponent> core::ops::Deref for Ac<X>
{
    type Target = X;
    #[inline] fn deref(&self) -> &Self::Target { &self.0 }
}

impl<X: AudioComponent> core::ops::DerefMut for Ac<X>
{
    #[inline] fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl<X, Y> std::ops::Add<Ac<Y>> for Ac<X> where
    X: AudioComponent,
    Y: AudioComponent<Sample = X::Sample, Outputs = X::Outputs>,
    X::Inputs: ArrayLength<X::Sample> + Add<Y::Inputs>,
    Y::Inputs: ArrayLength<Y::Sample>,
    <<X as AudioComponent>::Inputs as Add<<Y as AudioComponent>::Inputs>>::Output: ArrayLength<<X as AudioComponent>::Sample>,
{
    type Output = Ac<BinaryComponent<X, Y>>;
    #[inline] fn add(self, y: Ac<Y>) -> Self::Output { Ac(BinaryComponent::new(self.0, y.0, Binop::Add)) }
}

impl<X, Y> std::ops::Sub<Ac<Y>> for Ac<X> where
    X: AudioComponent,
    Y: AudioComponent<Sample = X::Sample, Outputs = X::Outputs>,
    X::Inputs: ArrayLength<X::Sample> + Add<Y::Inputs>,
    Y::Inputs: ArrayLength<Y::Sample>,
    <<X as AudioComponent>::Inputs as Add<<Y as AudioComponent>::Inputs>>::Output: ArrayLength<<X as AudioComponent>::Sample>,
{
    type Output = Ac<BinaryComponent<X, Y>>;
    #[inline] fn sub(self, y: Ac<Y>) -> Self::Output { Ac(BinaryComponent::new(self.0, y.0, Binop::Sub)) }
}

impl<X, Y> std::ops::Mul<Ac<Y>> for Ac<X> where
    X: AudioComponent,
    Y: AudioComponent<Sample = X::Sample, Outputs = X::Outputs>,
    X::Inputs: ArrayLength<X::Sample> + Add<Y::Inputs>,
    Y::Inputs: ArrayLength<Y::Sample>,
    <<X as AudioComponent>::Inputs as Add<<Y as AudioComponent>::Inputs>>::Output: ArrayLength<<X as AudioComponent>::Sample>,
{
    type Output = Ac<BinaryComponent<X, Y>>;
    #[inline] fn mul(self, y: Ac<Y>) -> Self::Output { Ac(BinaryComponent::new(self.0, y.0, Binop::Mul)) }
}

impl<X, Y> std::ops::Shr<Ac<Y>> for Ac<X> where
    X: AudioComponent,
    Y: AudioComponent<Sample = X::Sample, Inputs = X::Outputs>,
    Y::Outputs: ArrayLength<Y::Sample>,
{
    type Output = Ac<SerialComponent<X, Y>>;
    #[inline] fn shr(self, y: Ac<Y>) -> Self::Output { Ac(SerialComponent::new(self.0, y.0)) }
}

impl<X, Y> std::ops::BitOr<Ac<Y>> for Ac<X> where
    X: AudioComponent,
    Y: AudioComponent<Sample = X::Sample>,
    X::Inputs: ArrayLength<X::Sample> + Add<Y::Inputs>,
    X::Outputs: ArrayLength<X::Sample> + Add<Y::Outputs>,
    Y::Inputs: ArrayLength<Y::Sample>,
    Y::Outputs: ArrayLength<Y::Sample>,
    <<X as AudioComponent>::Inputs as Add<<Y as AudioComponent>::Inputs>>::Output: ArrayLength<<X as AudioComponent>::Sample>,
    <<X as AudioComponent>::Outputs as Add<<Y as AudioComponent>::Outputs>>::Output: ArrayLength<<X as AudioComponent>::Sample>
{
    type Output = Ac<ParallelComponent<X, Y>>;
    #[inline] fn bitor(self, y: Ac<Y>) -> Self::Output { Ac(ParallelComponent::new(self.0, y.0)) }
}

impl<X, Y> std::ops::BitAnd<Ac<Y>> for Ac<X> where
    X: AudioComponent,
    Y: AudioComponent<Sample = X::Sample, Inputs = X::Inputs>,
    X::Outputs: ArrayLength<X::Sample> + Add<Y::Outputs>,
    Y::Outputs: ArrayLength<Y::Sample>,
    <<X as AudioComponent>::Outputs as Add<<Y as AudioComponent>::Outputs>>::Output: ArrayLength<<X as AudioComponent>::Sample>
{
    type Output = Ac<BranchComponent<X, Y>>;
    #[inline] fn bitand(self, y: Ac<Y>) -> Self::Output { Ac(BranchComponent::new(self.0, y.0)) }
}

impl<X: AudioComponent> Iterator for Ac<X>
{
    type Item = NumericArray<X::Sample, X::Outputs>;
    /// Processes a sample from an all-zeros input.
    fn next(&mut self) -> Option<Self::Item> { 
        Some(self.tick(&NumericArray::default()))
    }
}

/// Makes a constant component. Synonymous with dc.
pub fn constant<X: ConstantFrame>(x: X) -> Ac<ConstantComponent<F32, X::Length>> { Ac(ConstantComponent::new(x.convert())) }

/// Makes a constant component. Synonymous with constant.
/// DC stands for "direct current", which is an electrical engineering term used with signals.
pub fn dc<X: ConstantFrame>(x: X) -> Ac<ConstantComponent<F32, X::Length>> { Ac(ConstantComponent::new(x.convert())) }

/// Makes a mono pass-through component.
pub fn pass() -> Ac<PassComponent<F32, typenum::U1>> { Ac(PassComponent::new()) }

/*
Operator precedences:
*
+, -
>>
&
|
These precedences work well except for >>, which should be lowest.
*/
