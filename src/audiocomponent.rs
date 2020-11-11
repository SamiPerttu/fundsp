use utd::math::*;
use super::*;
use super::frame::*;

/// AudioComponent processes audio data sample by sample.
/// It has a static number of inputs and outputs known at compile time.
/// If not set otherwise, the sample rate is the system default DEFAULT_SR.
pub trait AudioComponent {
    type Sample: AudioFloat;
    type Input: Frame<Sample = Self::Sample>;
    type Output: Frame<Sample = Self::Sample>;

    /// Resets the input state of the component to an initial state where it has not processed any samples.
    fn reset(&mut self, _sample_rate: Option<f64>) {}

    /// Processes one sample.
    fn tick(&mut self, input: Self::Input) -> Self::Output;

    /// Causal latency from input to output, in (fractional) samples.
    /// After a reset, we can discard this many samples from the output to avoid incurring a pre-delay.
    /// This applies only to components that have both inputs and outputs.
    fn latency(&self) -> f64 { 0.0 }

    /// Processes one sample from an all-zeros input.
    fn next(&mut self) -> Self::Output { self.tick(Self::Input::from_fn(|_| Self::Sample::zero())) }
}

/// Component that outputs a constant value.
#[derive(Clone)]
pub struct ConstantComponent<F: Frame> {
    output: F,
}

impl<F: Frame> ConstantComponent<F> {
    pub fn new(output: F) -> Self { ConstantComponent { output } }
}

impl<F: Frame> AudioComponent for ConstantComponent<F> where F::Sample: AudioFloat {
    type Sample = F::Sample;
    type Input = [F::Sample; 0];
    type Output = F;

    fn tick(&mut self, _input: Self::Input) -> Self::Output { self.output }
}

/// BinopComponent combines outputs of two components, channel-wise, with a binary operation.
/// The components must have the same number of inputs and outputs.
/// The same input is sent to both components.
#[derive(Clone)]
pub struct BinopComponent<X, Y, B> where
    X: AudioComponent<Sample = F32>,
    Y: AudioComponent<Input = X::Input, Output = X::Output, Sample = F32>,
    B: Fn(F32, F32) -> F32,
{
    x: X,
    y: Y,
    binop: B,
}

impl<X, Y, B> BinopComponent<X, Y, B> where
    X: AudioComponent<Sample = F32>,
    Y: AudioComponent<Input = X::Input, Output = X::Output, Sample = F32>,
    B: Fn(F32, F32) -> F32,
{
    pub fn new(x: X, y: Y, binop: B) -> Self { BinopComponent { x, y, binop } }
}

impl<X, Y, B> AudioComponent for BinopComponent<X, Y, B> where
    X: AudioComponent<Sample = F32>,
    Y: AudioComponent<Input = X::Input, Output = X::Output, Sample = F32>,
    B: Fn(F32, F32) -> F32,
{
    type Sample = F32;
    type Input = X::Input;
    type Output = X::Output;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.y.reset(sample_rate);
    }
    fn tick(&mut self, input: Self::Input) -> Self::Output {
        let x = self.x.tick(input);
        let y = self.y.tick(input);
        Self::Output::from_fn(|i| (self.binop)(x.channel(i), y.channel(i)))
    }
    fn latency(&self) -> f64 { self.x.latency().min(self.y.latency()) }
}


#[derive(Clone)]
pub enum Binop { Add, Mul }

/// FixedBinopComponent combines outputs of two components, channel-wise, with a binary operation.
/// The components must have the same number of inputs and outputs.
/// The same input is sent to both components.
#[derive(Clone)]
pub struct FixedBinopComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Sample = X::Sample, Input = X::Input, Output = X::Output>,
{
    x: X,
    y: Y,
    b: Binop,
}

impl<X, Y> FixedBinopComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Sample = X::Sample, Input = X::Input, Output = X::Output>,
{
    pub fn new(x: X, y: Y, b: Binop) -> Self { FixedBinopComponent { x, y, b } }
}

impl<X, Y> AudioComponent for FixedBinopComponent<X, Y> where
    X: AudioComponent,
    Y: AudioComponent<Sample = X::Sample, Input = X::Input, Output = X::Output>,
{
    type Sample = X::Sample;
    type Input = X::Input;
    type Output = X::Output;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.y.reset(sample_rate);
    }
    fn tick(&mut self, input: Self::Input) -> Self::Output {
        let x = self.x.tick(input);
        let y = self.y.tick(input);
        match self.b {
            Binop::Add => Self::Output::from_fn(|i| x.channel(i) + y.channel(i)),
            Binop::Mul => Self::Output::from_fn(|i| x.channel(i) * y.channel(i)),
        }
    }
    fn latency(&self) -> f64 { self.x.latency().min(self.y.latency()) }
}

/// AudioComponent wrapper.
pub struct Ac<X: AudioComponent>(X);
pub fn acomp<X: AudioComponent>(x: X) -> Ac<X> { Ac(x) }

impl<X: AudioComponent> core::ops::Deref for Ac<X> {
    type Target = X;
    #[inline] fn deref(&self) -> &Self::Target { &self.0 }
}

impl<X: AudioComponent> core::ops::DerefMut for Ac<X> {
    #[inline] fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl<X, Y> std::ops::Add<Ac<Y>> for Ac<X> where
    X: AudioComponent,
    Y: AudioComponent<Sample = X::Sample, Input = X::Input, Output = X::Output>,
{
    type Output = Ac<FixedBinopComponent<X, Y>>;
    #[inline] fn add(self, y: Ac<Y>) -> Self::Output { Ac(FixedBinopComponent::new(self.0, y.0, Binop::Add)) }
}

impl<X: AudioComponent> Iterator for Ac<X> {
    type Item = X::Output;
    /// Processes one sample from an all-zeros input.
    fn next(&mut self) -> Option<Self::Item> { 
        Some(self.tick(X::Input::from_fn(|_| X::Sample::zero())))
    }
}

pub fn constant(x: F32) -> Ac<ConstantComponent<[F32; 1]>> { Ac(ConstantComponent::new([x])) }
