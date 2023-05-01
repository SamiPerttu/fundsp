//! `AudioNode` wrapper `An` and operators, methods and traits.

// FunDSP Composable Graph Notation defined here was developed by Sami Perttu,
// with contributions from Benjamin Saunders.

use super::audionode::*;
use super::math::*;
use super::signal::*;
use super::*;
use duplicate::duplicate_item;
use numeric_array::typenum::*;

/// Trait for multi-channel constants.
pub trait ConstantFrame: Clone {
    type Sample: Float;
    type Size: Size<Self::Sample>;
    fn convert(self) -> Frame<Self::Sample, Self::Size>;
}

impl<T: Float, N: Size<T>> ConstantFrame for Frame<T, N> {
    type Sample = T;
    type Size = N;
    fn convert(self) -> Frame<Self::Sample, Self::Size> {
        self
    }
}

impl<T: Float> ConstantFrame for T {
    type Sample = T;
    type Size = U1;
    fn convert(self) -> Frame<Self::Sample, Self::Size> {
        [self].into()
    }
}

impl<T: Float> ConstantFrame for (T, T) {
    type Sample = T;
    type Size = U2;
    fn convert(self) -> Frame<Self::Sample, Self::Size> {
        [self.0, self.1].into()
    }
}

impl<T: Float> ConstantFrame for (T, T, T) {
    type Sample = T;
    type Size = U3;
    fn convert(self) -> Frame<Self::Sample, Self::Size> {
        [self.0, self.1, self.2].into()
    }
}

impl<T: Float> ConstantFrame for (T, T, T, T) {
    type Sample = T;
    type Size = U4;
    fn convert(self) -> Frame<Self::Sample, Self::Size> {
        [self.0, self.1, self.2, self.3].into()
    }
}

impl<T: Float> ConstantFrame for (T, T, T, T, T) {
    type Sample = T;
    type Size = U5;
    fn convert(self) -> Frame<Self::Sample, Self::Size> {
        [self.0, self.1, self.2, self.3, self.4].into()
    }
}

impl<T: Float> ConstantFrame for (T, T, T, T, T, T) {
    type Sample = T;
    type Size = U6;
    fn convert(self) -> Frame<Self::Sample, Self::Size> {
        [self.0, self.1, self.2, self.3, self.4, self.5].into()
    }
}

impl<T: Float> ConstantFrame for (T, T, T, T, T, T, T) {
    type Sample = T;
    type Size = U7;
    fn convert(self) -> Frame<Self::Sample, Self::Size> {
        [self.0, self.1, self.2, self.3, self.4, self.5, self.6].into()
    }
}

impl<T: Float> ConstantFrame for (T, T, T, T, T, T, T, T) {
    type Sample = T;
    type Size = U8;
    fn convert(self) -> Frame<Self::Sample, Self::Size> {
        [
            self.0, self.1, self.2, self.3, self.4, self.5, self.6, self.7,
        ]
        .into()
    }
}

impl<T: Float> ConstantFrame for (T, T, T, T, T, T, T, T, T) {
    type Sample = T;
    type Size = U9;
    fn convert(self) -> Frame<Self::Sample, Self::Size> {
        [
            self.0, self.1, self.2, self.3, self.4, self.5, self.6, self.7, self.8,
        ]
        .into()
    }
}

impl<T: Float> ConstantFrame for (T, T, T, T, T, T, T, T, T, T) {
    type Sample = T;
    type Size = U10;
    fn convert(self) -> Frame<Self::Sample, Self::Size> {
        [
            self.0, self.1, self.2, self.3, self.4, self.5, self.6, self.7, self.8, self.9,
        ]
        .into()
    }
}

/// Trait for 1-way/2-way distinctions, such as symmetric/asymmetric response times.
pub trait ScalarOrPair: Clone + Default + Send + Sync {
    type Sample: Float;
    /// Construct new item from broadcast pair.
    fn construct(x: Self::Sample, y: Self::Sample) -> Self;
    /// Return pair, broadcasting scalar into pair if needed.
    fn broadcast(&self) -> (Self::Sample, Self::Sample);
    /// Symmetric or asymmetric 1-pole filter where factors are derived from the broadcast pair.
    fn filter_pole(
        &self,
        input: Self::Sample,
        current: Self::Sample,
        afactor: Self::Sample,
        rfactor: Self::Sample,
    ) -> Self::Sample;
}

impl<T: Float> ScalarOrPair for T {
    type Sample = T;
    fn construct(x: Self::Sample, _y: Self::Sample) -> Self {
        x
    }
    fn broadcast(&self) -> (Self::Sample, Self::Sample) {
        (*self, *self)
    }
    fn filter_pole(
        &self,
        input: Self::Sample,
        current: Self::Sample,
        afactor: Self::Sample,
        _rfactor: Self::Sample,
    ) -> Self::Sample {
        // We know afactor == rfactor.
        current + (input - current) * afactor
    }
}

impl<T: Float> ScalarOrPair for (T, T) {
    type Sample = T;
    fn construct(x: Self::Sample, y: Self::Sample) -> Self {
        (x, y)
    }
    fn broadcast(&self) -> (Self::Sample, Self::Sample) {
        *self
    }
    fn filter_pole(
        &self,
        input: Self::Sample,
        current: Self::Sample,
        afactor: Self::Sample,
        rfactor: Self::Sample,
    ) -> Self::Sample {
        current + max(T::zero(), input - current) * afactor
            - max(T::zero(), current - input) * rfactor
    }
}

/// AudioNode wrapper that implements operators and traits.
#[derive(Clone)]
pub struct An<X: AudioNode>(pub X);

impl<X: AudioNode> core::ops::Deref for An<X> {
    type Target = X;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<X: AudioNode> core::ops::DerefMut for An<X> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// We relay some calls preferentially to the underlying AudioNode
// - otherwise the AudioUnit implementation would be picked.
impl<X: AudioNode> An<X> {
    #[inline]
    pub fn reset(&mut self) {
        self.0.reset();
    }
    #[inline]
    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.0.set_sample_rate(sample_rate);
    }
    #[inline]
    pub fn tick(&mut self, input: &Frame<X::Sample, X::Inputs>) -> Frame<X::Sample, X::Outputs> {
        self.0.tick(input)
    }
    #[inline]
    pub fn process(
        &mut self,
        size: usize,
        input: &[&[X::Sample]],
        output: &mut [&mut [X::Sample]],
    ) {
        self.0.process(size, input, output);
    }
    #[inline]
    pub fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.0.route(input, frequency)
    }
    #[inline]
    pub fn inputs(&self) -> usize {
        self.0.inputs()
    }
    #[inline]
    pub fn outputs(&self) -> usize {
        self.0.outputs()
    }
    #[inline]
    pub fn set_hash(&mut self, hash: u64) {
        self.0.set_hash(hash);
    }
    #[inline]
    pub fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.0.ping(probe, hash)
    }
    #[inline]
    pub fn get_mono(&mut self) -> X::Sample {
        self.0.get_mono()
    }
    #[inline]
    pub fn get_stereo(&mut self) -> (X::Sample, X::Sample) {
        self.0.get_stereo()
    }
    #[inline]
    pub fn filter_mono(&mut self, x: X::Sample) -> X::Sample {
        self.0.filter_mono(x)
    }
    #[inline]
    pub fn filter_stereo(&mut self, x: X::Sample, y: X::Sample) -> (X::Sample, X::Sample) {
        self.0.filter_stereo(x, y)
    }
}

/// `-` unary operator: Negates node outputs. Any node can be negated.
impl<X> std::ops::Neg for An<X>
where
    X: AudioNode,
    X::Outputs: Size<X::Sample>,
{
    type Output = An<Unop<X::Sample, X, FrameNeg<X::Outputs, X::Sample>>>;
    #[inline]
    fn neg(self) -> Self::Output {
        An(Unop::new(self.0, FrameNeg::new()))
    }
}

/// `!` unary operator: The thru operator makes output arity match input arity
/// and passes through missing outputs.
impl<X> std::ops::Not for An<X>
where
    X: AudioNode,
{
    type Output = An<Thru<X>>;
    #[inline]
    fn not(self) -> Self::Output {
        An(Thru::new(self.0))
    }
}

/// `+` binary operator: Sums outputs of two nodes with disjoint inputs.
/// The nodes must have the same number of outputs.
impl<X, Y> std::ops::Add<An<Y>> for An<X>
where
    X: AudioNode,
    Y: AudioNode<Sample = X::Sample, Outputs = X::Outputs>,
    X::Inputs: Size<X::Sample> + Add<Y::Inputs>,
    Y::Inputs: Size<Y::Sample>,
    <X::Inputs as Add<Y::Inputs>>::Output: Size<X::Sample>,
{
    type Output = An<Binop<X::Sample, FrameAdd<X::Outputs, X::Sample>, X, Y>>;
    #[inline]
    fn add(self, y: An<Y>) -> Self::Output {
        An(Binop::new(self.0, y.0, FrameAdd::new()))
    }
}

/// `X + constant` binary operator: Adds `constant` to outputs of `X`.
/// Broadcasts `constant` to an arbitrary number of channels.
#[duplicate_item(
    f48;
    [ f64 ];
    [ f32 ];
)]
impl<X> std::ops::Add<f48> for An<X>
where
    X: AudioNode<Sample = f48>,
    X::Inputs: Size<f48>,
    X::Outputs: Size<f48>,
{
    type Output = An<Unop<f48, X, FrameAddScalar<X::Outputs, X::Sample>>>;
    #[inline]
    fn add(self, y: f48) -> Self::Output {
        An(Unop::new(self.0, FrameAddScalar::new(y)))
    }
}

/// `constant + X` binary operator: Adds `constant` to outputs of `X`.
/// Broadcasts `constant` to an arbitrary number of channels.
#[duplicate_item(
    f48;
    [ f64 ];
    [ f32 ];
)]
impl<X> std::ops::Add<An<X>> for f48
where
    X: AudioNode<Sample = f48>,
    X::Inputs: Size<f48>,
    X::Outputs: Size<f48>,
{
    type Output = An<Unop<f48, X, FrameAddScalar<X::Outputs, f48>>>;
    #[inline]
    fn add(self, y: An<X>) -> Self::Output {
        An(Unop::new(y.0, FrameAddScalar::new(self)))
    }
}

/// `-` binary operator: The difference of outputs of two nodes with disjoint inputs.
/// The nodes must have the same number of outputs.
impl<X, Y> std::ops::Sub<An<Y>> for An<X>
where
    X: AudioNode,
    Y: AudioNode<Sample = X::Sample, Outputs = X::Outputs>,
    X::Inputs: Size<X::Sample> + Add<Y::Inputs>,
    Y::Inputs: Size<Y::Sample>,
    <X::Inputs as Add<Y::Inputs>>::Output: Size<X::Sample>,
{
    type Output = An<Binop<X::Sample, FrameSub<X::Outputs, X::Sample>, X, Y>>;
    #[inline]
    fn sub(self, y: An<Y>) -> Self::Output {
        An(Binop::new(self.0, y.0, FrameSub::new()))
    }
}

/// `X - constant` binary operator: Subtracts `constant` from outputs of `X`.
/// Broadcasts `constant` to an arbitrary number of channels.
#[duplicate_item(
    f48;
    [ f64 ];
    [ f32 ];
)]
impl<X> std::ops::Sub<f48> for An<X>
where
    X: AudioNode<Sample = f48>,
    X::Inputs: Size<f48>,
    X::Outputs: Size<f48>,
{
    type Output = An<Unop<f48, X, FrameAddScalar<X::Outputs, f48>>>;
    #[inline]
    fn sub(self, y: f48) -> Self::Output {
        An(Unop::new(self.0, FrameAddScalar::new(-y)))
    }
}

/// `constant - X` binary operator: Negates `X` and adds `constant` to its outputs.
/// Broadcasts `constant` to an arbitrary number of channels.
#[duplicate_item(
    f48;
    [ f64 ];
    [ f32 ];
)]
impl<X> std::ops::Sub<An<X>> for f48
where
    X: AudioNode<Sample = f48>,
    X::Inputs: Size<f48> + Add<U0>,
    X::Outputs: Size<f48>,
    <X::Inputs as Add<U0>>::Output: Size<f48>,
{
    type Output = An<Binop<f48, FrameSub<X::Outputs, f48>, Constant<X::Outputs, f48>, X>>;
    #[inline]
    fn sub(self, y: An<X>) -> Self::Output {
        An(Binop::new(
            Constant::new(Frame::splat(self)),
            y.0,
            FrameSub::new(),
        ))
    }
}

/// `*` binary operator: Multiplies outputs of two nodes with disjoint inputs.
/// The nodes must have the same number of outputs.
impl<X, Y> std::ops::Mul<An<Y>> for An<X>
where
    X: AudioNode,
    Y: AudioNode<Sample = X::Sample, Outputs = X::Outputs>,
    X::Inputs: Size<X::Sample> + Add<Y::Inputs>,
    Y::Inputs: Size<Y::Sample>,
    <X::Inputs as Add<Y::Inputs>>::Output: Size<X::Sample>,
{
    type Output = An<Binop<X::Sample, FrameMul<X::Outputs, X::Sample>, X, Y>>;
    #[inline]
    fn mul(self, y: An<Y>) -> Self::Output {
        An(Binop::new(self.0, y.0, FrameMul::new()))
    }
}

/// `X * constant` binary operator: Multiplies outputs of `X` with `constant`.
/// Broadcasts `constant` to an arbitrary number of channels.
#[duplicate_item(
    f48;
    [ f64 ];
    [ f32 ];
)]
impl<X> std::ops::Mul<f48> for An<X>
where
    X: AudioNode<Sample = f48>,
    X::Inputs: Size<f48>,
    X::Outputs: Size<f48>,
{
    type Output = An<Unop<f48, X, FrameMulScalar<X::Outputs, f48>>>;
    #[inline]
    fn mul(self, y: f48) -> Self::Output {
        An(Unop::new(self.0, FrameMulScalar::new(y)))
    }
}

/// `constant * X` binary operator: Multiplies outputs of `X` with `constant`.
/// Broadcasts `constant` to an arbitrary number of channels.
#[duplicate_item(
    f48;
    [ f64 ];
    [ f32 ];
)]
impl<X> std::ops::Mul<An<X>> for f48
where
    X: AudioNode<Sample = f48>,
    X::Inputs: Size<f48>,
    X::Outputs: Size<f48>,
{
    type Output = An<Unop<f48, X, FrameMulScalar<X::Outputs, f48>>>;
    #[inline]
    fn mul(self, y: An<X>) -> Self::Output {
        An(Unop::new(y.0, FrameMulScalar::new(self)))
    }
}

/// `>>` binary operator: The pipe operator pipes outputs of left node to inputs of right node.
/// Number of outputs on the left side and number of inputs on the right side must match.
impl<T, X, Y> std::ops::Shr<An<Y>> for An<X>
where
    T: Float,
    X: AudioNode<Sample = T>,
    Y: AudioNode<Sample = T, Inputs = X::Outputs>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    Y::Outputs: Size<T>,
{
    type Output = An<Pipe<T, X, Y>>;
    #[inline]
    fn shr(self, y: An<Y>) -> Self::Output {
        An(Pipe::new(self.0, y.0))
    }
}

/// `&` binary operator: The bus operator mixes together units with similar connectivity that share inputs and outputs.
impl<T, X, Y> std::ops::BitAnd<An<Y>> for An<X>
where
    T: Float,
    X: AudioNode<Sample = T>,
    Y: AudioNode<Sample = T, Inputs = X::Inputs, Outputs = X::Outputs>,
    Y::Inputs: Size<T>,
    Y::Outputs: Size<T>,
{
    type Output = An<Bus<T, X, Y>>;
    #[inline]
    fn bitand(self, y: An<Y>) -> Self::Output {
        An(Bus::new(self.0, y.0))
    }
}

/// `^` binary operator: The branch operator sources two nodes from the same inputs and concatenates their outputs.
impl<T, X, Y> std::ops::BitXor<An<Y>> for An<X>
where
    T: Float,
    X: AudioNode<Sample = T>,
    Y: AudioNode<Sample = T, Inputs = X::Inputs>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T> + Add<Y::Outputs>,
    Y::Outputs: Size<T>,
    <X::Outputs as Add<Y::Outputs>>::Output: Size<T>,
{
    type Output = An<Branch<T, X, Y>>;
    #[inline]
    fn bitxor(self, y: An<Y>) -> Self::Output {
        An(Branch::new(self.0, y.0))
    }
}

/// `|` binary operator: The stack operator stacks inputs and outputs of two nodes running in parallel.
impl<T, X, Y> std::ops::BitOr<An<Y>> for An<X>
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
    type Output = An<Stack<T, X, Y>>;
    #[inline]
    fn bitor(self, y: An<Y>) -> Self::Output {
        An(Stack::new(self.0, y.0))
    }
}
