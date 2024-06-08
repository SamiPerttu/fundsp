//! `AudioNode` wrapper `An` and operators, methods and traits.

// FunDSP Composable Graph Notation defined here was developed by Sami Perttu,
// with contributions from Benjamin Saunders.

use super::audionode::*;
use super::buffer::*;
use super::math::*;
use super::signal::*;
use super::*;
use core::ops::{Add, BitAnd, BitOr, BitXor, Mul, Neg, Shr, Sub};
use numeric_array::typenum::*;

/// Trait for multi-channel constants.
pub trait ConstantFrame: Clone + Sync + Send {
    type Sample: Float;
    type Size: Size<Self::Sample>;
    fn frame(self) -> Frame<Self::Sample, Self::Size>;
}

impl<T: Float, N: Size<T>> ConstantFrame for Frame<T, N> {
    type Sample = T;
    type Size = N;
    fn frame(self) -> Frame<Self::Sample, Self::Size> {
        self
    }
}

impl<T: Float> ConstantFrame for T {
    type Sample = T;
    type Size = U1;
    fn frame(self) -> Frame<Self::Sample, Self::Size> {
        [self].into()
    }
}

impl<T: Float> ConstantFrame for (T, T) {
    type Sample = T;
    type Size = U2;
    fn frame(self) -> Frame<Self::Sample, Self::Size> {
        [self.0, self.1].into()
    }
}

impl<T: Float> ConstantFrame for (T, T, T) {
    type Sample = T;
    type Size = U3;
    fn frame(self) -> Frame<Self::Sample, Self::Size> {
        [self.0, self.1, self.2].into()
    }
}

impl<T: Float> ConstantFrame for (T, T, T, T) {
    type Sample = T;
    type Size = U4;
    fn frame(self) -> Frame<Self::Sample, Self::Size> {
        [self.0, self.1, self.2, self.3].into()
    }
}

impl<T: Float> ConstantFrame for (T, T, T, T, T) {
    type Sample = T;
    type Size = U5;
    fn frame(self) -> Frame<Self::Sample, Self::Size> {
        [self.0, self.1, self.2, self.3, self.4].into()
    }
}

impl<T: Float> ConstantFrame for (T, T, T, T, T, T) {
    type Sample = T;
    type Size = U6;
    fn frame(self) -> Frame<Self::Sample, Self::Size> {
        [self.0, self.1, self.2, self.3, self.4, self.5].into()
    }
}

impl<T: Float> ConstantFrame for (T, T, T, T, T, T, T) {
    type Sample = T;
    type Size = U7;
    fn frame(self) -> Frame<Self::Sample, Self::Size> {
        [self.0, self.1, self.2, self.3, self.4, self.5, self.6].into()
    }
}

impl<T: Float> ConstantFrame for (T, T, T, T, T, T, T, T) {
    type Sample = T;
    type Size = U8;
    fn frame(self) -> Frame<Self::Sample, Self::Size> {
        [
            self.0, self.1, self.2, self.3, self.4, self.5, self.6, self.7,
        ]
        .into()
    }
}

impl<T: Float> ConstantFrame for (T, T, T, T, T, T, T, T, T) {
    type Sample = T;
    type Size = U9;
    fn frame(self) -> Frame<Self::Sample, Self::Size> {
        [
            self.0, self.1, self.2, self.3, self.4, self.5, self.6, self.7, self.8,
        ]
        .into()
    }
}

impl<T: Float> ConstantFrame for (T, T, T, T, T, T, T, T, T, T) {
    type Sample = T;
    type Size = U10;
    fn frame(self) -> Frame<Self::Sample, Self::Size> {
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
    pub fn tick(&mut self, input: &Frame<f32, X::Inputs>) -> Frame<f32, X::Outputs> {
        self.0.tick(input)
    }
    #[inline]
    pub fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
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
    pub fn get_mono(&mut self) -> f32 {
        self.0.get_mono()
    }
    #[inline]
    pub fn get_stereo(&mut self) -> (f32, f32) {
        self.0.get_stereo()
    }
    #[inline]
    pub fn filter_mono(&mut self, x: f32) -> f32 {
        self.0.filter_mono(x)
    }
    #[inline]
    pub fn filter_stereo(&mut self, x: f32, y: f32) -> (f32, f32) {
        self.0.filter_stereo(x, y)
    }
}

impl<X> Neg for An<X>
where
    X: AudioNode,
    X::Outputs: Size<f32>,
{
    type Output = An<Unop<X, FrameNeg<X::Outputs>>>;
    #[inline]
    fn neg(self) -> Self::Output {
        An(Unop::new(self.0, FrameNeg::new()))
    }
}

/// The thru operator makes output arity match input arity
/// and passes through missing outputs.
impl<X> Not for An<X>
where
    X: AudioNode,
{
    type Output = An<Thru<X>>;
    #[inline]
    fn not(self) -> Self::Output {
        An(Thru::new(self.0))
    }
}

impl<X, Y> Add<An<Y>> for An<X>
where
    X: AudioNode,
    Y: AudioNode<Outputs = X::Outputs>,
    X::Inputs: Add<Y::Inputs>,
    <X::Inputs as Add<Y::Inputs>>::Output: Size<f32>,
{
    type Output = An<Binop<FrameAdd<X::Outputs>, X, Y>>;
    #[inline]
    fn add(self, y: An<Y>) -> Self::Output {
        An(Binop::new(FrameAdd::new(), self.0, y.0))
    }
}

impl<X, Y> Sub<An<Y>> for An<X>
where
    X: AudioNode,
    Y: AudioNode<Outputs = X::Outputs>,
    X::Outputs: Size<f32>,
    X::Inputs: Size<f32> + Add<Y::Inputs>,
    Y::Inputs: Size<f32>,
    <X::Inputs as Add<Y::Inputs>>::Output: Size<f32>,
{
    type Output = An<Binop<FrameSub<X::Outputs>, X, Y>>;
    #[inline]
    fn sub(self, y: An<Y>) -> Self::Output {
        An(Binop::new(FrameSub::new(), self.0, y.0))
    }
}

impl<X, Y> Mul<An<Y>> for An<X>
where
    X: AudioNode,
    Y: AudioNode<Outputs = X::Outputs>,
    X::Inputs: Add<Y::Inputs>,
    <X::Inputs as Add<Y::Inputs>>::Output: Size<f32>,
{
    type Output = An<Binop<FrameMul<X::Outputs>, X, Y>>;
    #[inline]
    fn mul(self, y: An<Y>) -> Self::Output {
        An(Binop::new(FrameMul::new(), self.0, y.0))
    }
}

impl<X, Y> Shr<An<Y>> for An<X>
where
    X: AudioNode,
    Y: AudioNode<Inputs = X::Outputs>,
{
    type Output = An<Pipe<X, Y>>;
    #[inline]
    fn shr(self, y: An<Y>) -> Self::Output {
        An(Pipe::new(self.0, y.0))
    }
}

impl<X, Y> BitAnd<An<Y>> for An<X>
where
    X: AudioNode,
    Y: AudioNode<Inputs = X::Inputs, Outputs = X::Outputs>,
{
    type Output = An<Bus<X, Y>>;
    #[inline]
    fn bitand(self, y: An<Y>) -> Self::Output {
        An(Bus::new(self.0, y.0))
    }
}

impl<X, Y> BitXor<An<Y>> for An<X>
where
    X: AudioNode,
    Y: AudioNode<Inputs = X::Inputs>,
    X::Outputs: Add<Y::Outputs>,
    <X::Outputs as Add<Y::Outputs>>::Output: Size<f32>,
{
    type Output = An<Branch<X, Y>>;
    #[inline]
    fn bitxor(self, y: An<Y>) -> Self::Output {
        An(Branch::new(self.0, y.0))
    }
}

impl<X, Y> BitOr<An<Y>> for An<X>
where
    X: AudioNode,
    Y: AudioNode,
    X::Inputs: Add<Y::Inputs>,
    X::Outputs: Add<Y::Outputs>,
    <X::Inputs as Add<Y::Inputs>>::Output: Size<f32>,
    <X::Outputs as Add<Y::Outputs>>::Output: Size<f32>,
{
    type Output = An<Stack<X, Y>>;
    #[inline]
    fn bitor(self, y: An<Y>) -> Self::Output {
        An(Stack::new(self.0, y.0))
    }
}

impl<X> Add<f32> for An<X>
where
    X: AudioNode,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
{
    type Output = An<Unop<X, FrameAddScalar<X::Outputs>>>;
    #[inline]
    fn add(self, y: f32) -> Self::Output {
        An(Unop::new(self.0, FrameAddScalar::new(y)))
    }
}

impl<X> Add<An<X>> for f32
where
    X: AudioNode,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
{
    type Output = An<Unop<X, FrameAddScalar<X::Outputs>>>;
    #[inline]
    fn add(self, y: An<X>) -> Self::Output {
        An(Unop::new(y.0, FrameAddScalar::new(self)))
    }
}

impl<X> Sub<f32> for An<X>
where
    X: AudioNode,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
{
    type Output = An<Unop<X, FrameAddScalar<X::Outputs>>>;
    #[inline]
    fn sub(self, y: f32) -> Self::Output {
        An(Unop::new(self.0, FrameAddScalar::new(-y)))
    }
}

impl<X> Sub<An<X>> for f32
where
    X: AudioNode,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
{
    type Output = An<Unop<X, FrameNegAddScalar<X::Outputs>>>;
    #[inline]
    fn sub(self, y: An<X>) -> Self::Output {
        An(Unop::new(y.0, FrameNegAddScalar::new(self)))
    }
}

impl<X> Mul<f32> for An<X>
where
    X: AudioNode,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
{
    type Output = An<Unop<X, FrameMulScalar<X::Outputs>>>;
    #[inline]
    fn mul(self, y: f32) -> Self::Output {
        An(Unop::new(self.0, FrameMulScalar::new(y)))
    }
}

impl<X> Mul<An<X>> for f32
where
    X: AudioNode,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
{
    type Output = An<Unop<X, FrameMulScalar<X::Outputs>>>;
    #[inline]
    fn mul(self, y: An<X>) -> Self::Output {
        An(Unop::new(y.0, FrameMulScalar::new(self)))
    }
}
