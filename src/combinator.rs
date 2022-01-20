use super::audionode::*;
use super::math::*;
use super::*;
use numeric_array::typenum::*;

/// Trait for multi-channel constants.
pub trait ConstantFrame: Clone {
    type Sample: Float;
    type Size: Size<Self::Sample>;
    fn convert(self) -> Frame<Self::Sample, Self::Size>;
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
pub trait ScalarOrPair: Clone + Default {
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

/// `-` unary operator: Negates node outputs. Any node can be negated.
impl<X> std::ops::Neg for An<X>
where
    X: AudioNode,
    X::Outputs: Size<X::Sample>,
{
    type Output = An<UnopNode<X::Sample, X, FrameNeg<X::Sample, X::Outputs>>>;
    #[inline]
    fn neg(self) -> Self::Output {
        An(UnopNode::new(self.0, FrameNeg::new()))
    }
}

/// `!` unary operator: The fit operator converts output arity to match input arity and passes through missing outputs.
impl<X> std::ops::Not for An<X>
where
    X: AudioNode,
{
    type Output = An<ThruNode<X>>;
    #[inline]
    fn not(self) -> Self::Output {
        An(ThruNode::new(self.0))
    }
}

/// `+` binary operator: Sums outputs of two nodes with disjoint inputs. The nodes must have the same number of outputs.
impl<X, Y> std::ops::Add<An<Y>> for An<X>
where
    X: AudioNode,
    Y: AudioNode<Sample = X::Sample, Outputs = X::Outputs>,
    X::Inputs: Size<X::Sample> + Add<Y::Inputs>,
    Y::Inputs: Size<Y::Sample>,
    <X::Inputs as Add<Y::Inputs>>::Output: Size<X::Sample>,
{
    type Output = An<BinopNode<X::Sample, X, Y, FrameAdd<X::Sample, X::Outputs>>>;
    #[inline]
    fn add(self, y: An<Y>) -> Self::Output {
        An(BinopNode::new(self.0, y.0, FrameAdd::new()))
    }
}

/// `X + constant` binary operator: Adds `constant` to outputs of `X`. Broadcasts `constant` to an arbitrary number of channels.
impl<X> std::ops::Add<f64> for An<X>
where
    X: AudioNode<Sample = f64>,
    X::Inputs: Size<f64> + Add<U0>,
    X::Outputs: Size<f64>,
    <X::Inputs as Add<U0>>::Output: Size<f64>,
{
    type Output =
        An<BinopNode<f64, X, ConstantNode<f64, X::Outputs>, FrameAdd<X::Sample, X::Outputs>>>;
    #[inline]
    fn add(self, y: f64) -> Self::Output {
        An(BinopNode::new(
            self.0,
            ConstantNode::new(Frame::splat(y)),
            FrameAdd::new(),
        ))
    }
}

/// `constant + X` binary operator: Adds `constant` to outputs of `X`. Broadcasts `constant` to an arbitrary number of channels.
impl<X> std::ops::Add<An<X>> for f64
where
    X: AudioNode<Sample = f64>,
    X::Inputs: Size<f64> + Add<U0>,
    X::Outputs: Size<f64>,
    <X::Inputs as Add<U0>>::Output: Size<f64>,
{
    type Output = An<BinopNode<f64, ConstantNode<f64, X::Outputs>, X, FrameAdd<f64, X::Outputs>>>;
    #[inline]
    fn add(self, y: An<X>) -> Self::Output {
        An(BinopNode::new(
            ConstantNode::new(Frame::splat(self)),
            y.0,
            FrameAdd::new(),
        ))
    }
}

/// `X + constant` binary operator: Adds `constant` to outputs of `X`. Broadcasts `constant` to an arbitrary number of channels.
impl<X> std::ops::Add<f32> for An<X>
where
    X: AudioNode<Sample = f32>,
    X::Inputs: Size<f32> + Add<U0>,
    X::Outputs: Size<f32>,
    <X::Inputs as Add<U0>>::Output: Size<f32>,
{
    type Output =
        An<BinopNode<f32, X, ConstantNode<f32, X::Outputs>, FrameAdd<X::Sample, X::Outputs>>>;
    #[inline]
    fn add(self, y: f32) -> Self::Output {
        An(BinopNode::new(
            self.0,
            ConstantNode::new(Frame::splat(y)),
            FrameAdd::new(),
        ))
    }
}

/// `constant + X` binary operator: Adds `constant` to outputs of `X`. Broadcasts `constant` to an arbitrary number of channels.
impl<X> std::ops::Add<An<X>> for f32
where
    X: AudioNode<Sample = f32>,
    X::Inputs: Size<f32> + Add<U0>,
    X::Outputs: Size<f32>,
    <X::Inputs as Add<U0>>::Output: Size<f32>,
{
    type Output = An<BinopNode<f32, ConstantNode<f32, X::Outputs>, X, FrameAdd<f32, X::Outputs>>>;
    #[inline]
    fn add(self, y: An<X>) -> Self::Output {
        An(BinopNode::new(
            ConstantNode::new(Frame::splat(self)),
            y.0,
            FrameAdd::new(),
        ))
    }
}

/// `-` binary operator: The difference of outputs of two nodes with disjoint inputs. The nodes must have the same number of outputs.
impl<X, Y> std::ops::Sub<An<Y>> for An<X>
where
    X: AudioNode,
    Y: AudioNode<Sample = X::Sample, Outputs = X::Outputs>,
    X::Inputs: Size<X::Sample> + Add<Y::Inputs>,
    Y::Inputs: Size<Y::Sample>,
    <X::Inputs as Add<Y::Inputs>>::Output: Size<X::Sample>,
{
    type Output = An<BinopNode<X::Sample, X, Y, FrameSub<X::Sample, X::Outputs>>>;
    #[inline]
    fn sub(self, y: An<Y>) -> Self::Output {
        An(BinopNode::new(self.0, y.0, FrameSub::new()))
    }
}

/// `X - constant` binary operator: Subtracts `constant` from outputs of `X`. Broadcasts `constant` to an arbitrary number of channels.
impl<X> std::ops::Sub<f64> for An<X>
where
    X: AudioNode<Sample = f64>,
    X::Inputs: Size<f64> + Add<U0>,
    X::Outputs: Size<f64>,
    <X::Inputs as Add<U0>>::Output: Size<f64>,
{
    type Output = An<BinopNode<f64, X, ConstantNode<f64, X::Outputs>, FrameSub<f64, X::Outputs>>>;
    #[inline]
    fn sub(self, y: f64) -> Self::Output {
        An(BinopNode::new(
            self.0,
            ConstantNode::new(Frame::splat(y)),
            FrameSub::new(),
        ))
    }
}

/// `constant - X` binary operator: Negates `X` and adds `constant` to its outputs. Broadcasts `constant` to an arbitrary number of channels.
impl<X> std::ops::Sub<An<X>> for f64
where
    X: AudioNode<Sample = f64>,
    X::Inputs: Size<f64> + Add<U0>,
    X::Outputs: Size<f64>,
    <X::Inputs as Add<U0>>::Output: Size<f64>,
{
    type Output = An<BinopNode<f64, ConstantNode<f64, X::Outputs>, X, FrameSub<f64, X::Outputs>>>;
    #[inline]
    fn sub(self, y: An<X>) -> Self::Output {
        An(BinopNode::new(
            ConstantNode::new(Frame::splat(self)),
            y.0,
            FrameSub::new(),
        ))
    }
}

/// `X - constant` binary operator: Subtracts `constant` from outputs of `X`. Broadcasts `constant` to an arbitrary number of channels.
impl<X> std::ops::Sub<f32> for An<X>
where
    X: AudioNode<Sample = f32>,
    X::Inputs: Size<f32> + Add<U0>,
    X::Outputs: Size<f32>,
    <X::Inputs as Add<U0>>::Output: Size<f32>,
{
    type Output = An<BinopNode<f32, X, ConstantNode<f32, X::Outputs>, FrameSub<f32, X::Outputs>>>;
    #[inline]
    fn sub(self, y: f32) -> Self::Output {
        An(BinopNode::new(
            self.0,
            ConstantNode::new(Frame::splat(y)),
            FrameSub::new(),
        ))
    }
}

/// `constant - X` binary operator: Negates `X` and adds `constant` to its outputs. Broadcasts `constant` to an arbitrary number of channels.
impl<X> std::ops::Sub<An<X>> for f32
where
    X: AudioNode<Sample = f32>,
    X::Inputs: Size<f32> + Add<U0>,
    X::Outputs: Size<f32>,
    <X::Inputs as Add<U0>>::Output: Size<f32>,
{
    type Output = An<BinopNode<f32, ConstantNode<f32, X::Outputs>, X, FrameSub<f32, X::Outputs>>>;
    #[inline]
    fn sub(self, y: An<X>) -> Self::Output {
        An(BinopNode::new(
            ConstantNode::new(Frame::splat(self)),
            y.0,
            FrameSub::new(),
        ))
    }
}

/// `*` binary operator: Multiplies outputs of two nodes with disjoint inputs. The nodes must have the same number of outputs.
impl<X, Y> std::ops::Mul<An<Y>> for An<X>
where
    X: AudioNode,
    Y: AudioNode<Sample = X::Sample, Outputs = X::Outputs>,
    X::Inputs: Size<X::Sample> + Add<Y::Inputs>,
    Y::Inputs: Size<Y::Sample>,
    <X::Inputs as Add<Y::Inputs>>::Output: Size<X::Sample>,
{
    type Output = An<BinopNode<X::Sample, X, Y, FrameMul<X::Sample, X::Outputs>>>;
    #[inline]
    fn mul(self, y: An<Y>) -> Self::Output {
        An(BinopNode::new(self.0, y.0, FrameMul::new()))
    }
}

/// `X * constant` binary operator: Multplies outputs of `X` with `constant`. Broadcasts `constant` to an arbitrary number of channels.
impl<X> std::ops::Mul<f64> for An<X>
where
    X: AudioNode<Sample = f64>,
    X::Inputs: Size<f64> + Add<U0>,
    X::Outputs: Size<f64>,
    <X::Inputs as Add<U0>>::Output: Size<f64>,
{
    type Output = An<BinopNode<f64, X, ConstantNode<f64, X::Outputs>, FrameMul<f64, X::Outputs>>>;
    #[inline]
    fn mul(self, y: f64) -> Self::Output {
        An(BinopNode::new(
            self.0,
            ConstantNode::new(Frame::splat(y)),
            FrameMul::new(),
        ))
    }
}

/// `constant * X` binary operator: Multplies outputs of `X` with `constant`. Broadcasts `constant` to an arbitrary number of channels.
impl<X> std::ops::Mul<An<X>> for f64
where
    X: AudioNode<Sample = f64>,
    X::Inputs: Size<f64> + Add<U0>,
    X::Outputs: Size<f64>,
    <X::Inputs as Add<U0>>::Output: Size<f64>,
{
    type Output = An<BinopNode<f64, ConstantNode<f64, X::Outputs>, X, FrameMul<f64, X::Outputs>>>;
    #[inline]
    fn mul(self, y: An<X>) -> Self::Output {
        An(BinopNode::new(
            ConstantNode::new(Frame::splat(self)),
            y.0,
            FrameMul::new(),
        ))
    }
}

/// `X * constant` binary operator: Multplies outputs of `X` with `constant`. Broadcasts `constant` to an arbitrary number of channels.
impl<X> std::ops::Mul<f32> for An<X>
where
    X: AudioNode<Sample = f32>,
    X::Inputs: Size<f32> + Add<U0>,
    X::Outputs: Size<f32>,
    <X::Inputs as Add<U0>>::Output: Size<f32>,
{
    type Output = An<BinopNode<f32, X, ConstantNode<f32, X::Outputs>, FrameMul<f32, X::Outputs>>>;
    #[inline]
    fn mul(self, y: f32) -> Self::Output {
        An(BinopNode::new(
            self.0,
            ConstantNode::new(Frame::splat(y)),
            FrameMul::new(),
        ))
    }
}

/// `constant * X` binary operator: Multplies outputs of `X` with `constant`. Broadcasts `constant` to an arbitrary number of channels.
impl<X> std::ops::Mul<An<X>> for f32
where
    X: AudioNode<Sample = f32>,
    X::Inputs: Size<f32> + Add<U0>,
    X::Outputs: Size<f32>,
    <X::Inputs as Add<U0>>::Output: Size<f32>,
{
    type Output = An<BinopNode<f32, ConstantNode<f32, X::Outputs>, X, FrameMul<f32, X::Outputs>>>;
    #[inline]
    fn mul(self, y: An<X>) -> Self::Output {
        An(BinopNode::new(
            ConstantNode::new(Frame::splat(self)),
            y.0,
            FrameMul::new(),
        ))
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
    type Output = An<PipeNode<T, X, Y>>;
    #[inline]
    fn shr(self, y: An<Y>) -> Self::Output {
        An(PipeNode::new(self.0, y.0))
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
    type Output = An<BusNode<T, X, Y>>;
    #[inline]
    fn bitand(self, y: An<Y>) -> Self::Output {
        An(BusNode::new(self.0, y.0))
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
    type Output = An<BranchNode<T, X, Y>>;
    #[inline]
    fn bitxor(self, y: An<Y>) -> Self::Output {
        An(BranchNode::new(self.0, y.0))
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
    type Output = An<StackNode<T, X, Y>>;
    #[inline]
    fn bitor(self, y: An<Y>) -> Self::Output {
        An(StackNode::new(self.0, y.0))
    }
}

impl<X: AudioNode> Iterator for An<X> {
    type Item = Frame<X::Sample, X::Outputs>;
    /// Processes a sample from an all-zeros input.
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.tick(&Frame::default()))
    }
}

pub struct MonoIter<X>
where
    X: AudioNode<Outputs = U1>,
    X::Sample: Float,
    X::Inputs: Size<X::Sample>,
{
    node: X,
}

impl<X> MonoIter<X>
where
    X: AudioNode<Outputs = U1>,
    X::Sample: Float,
    X::Inputs: Size<X::Sample>,
{
    pub fn new(node: X) -> Self {
        MonoIter { node }
    }
}

impl<X> Iterator for MonoIter<X>
where
    X: AudioNode<Outputs = U1>,
    X::Sample: Float,
    X::Inputs: Size<X::Sample>,
{
    type Item = X::Sample;
    /// Processes a sample from an all-zeros input.
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.node.tick(&Frame::default())[0])
    }
}

impl<X: AudioNode> An<X> {
    /// Consumes and returns the component as an FnMut closure
    /// that yields mono samples via AudioNode::get_mono.
    pub fn into_mono_fn(self) -> impl FnMut() -> X::Sample {
        let mut c = self;
        move || c.get_mono()
    }

    /// Consumes and returns the component as an FnMut closure
    /// that filters mono samples via AudioNode::filter_mono.
    pub fn into_mono_filter_fn(self) -> impl FnMut(X::Sample) -> X::Sample {
        let mut c = self;
        move |x| c.filter_mono(x)
    }

    /// Consumes and returns the component as an FnMut closure
    /// that yields stereo samples via AudioNode::get_stereo.
    pub fn into_stereo_fn(self) -> impl FnMut() -> (X::Sample, X::Sample) {
        let mut c = self;
        move || c.get_stereo()
    }

    /// Consumes and returns the component as an FnMut closure
    /// that filters stereo samples via AudioNode::filter_stereo.
    pub fn into_stereo_filter_fn(
        self,
    ) -> impl FnMut(X::Sample, X::Sample) -> (X::Sample, X::Sample) {
        let mut c = self;
        move |x, y| c.filter_stereo(x, y)
    }
}

// TODO: Constrain get_ and other into_ methods similarly.
impl<X> An<X>
where
    X: AudioNode<Outputs = U1>,
    X::Sample: Float,
    X::Inputs: Size<X::Sample>,
{
    /// Return node as an iterator that returns mono samples directly.
    pub fn into_mono_iter(self) -> MonoIter<X> {
        MonoIter::new(self.0)
    }
}
