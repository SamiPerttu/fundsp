use super::audionode::*;
use super::*;
use numeric_array::typenum::*;

/// Trait for multi-channel constants.
pub trait ConstantFrame {
    type Sample: Float;
    type Size: Size<Self::Sample>;
    fn convert(self) -> Frame<Self::Sample, Self::Size>;
}

impl ConstantFrame for f64 {
    type Sample = f64;
    type Size = U1;
    fn convert(self) -> Frame<Self::Sample, Self::Size> {
        [self].into()
    }
}

impl ConstantFrame for (f64, f64) {
    type Sample = f64;
    type Size = U2;
    fn convert(self) -> Frame<Self::Sample, Self::Size> {
        [self.0, self.1].into()
    }
}

impl ConstantFrame for (f64, f64, f64) {
    type Sample = f64;
    type Size = U3;
    fn convert(self) -> Frame<Self::Sample, Self::Size> {
        [self.0, self.1, self.2].into()
    }
}

impl ConstantFrame for (f64, f64, f64, f64) {
    type Sample = f64;
    type Size = U4;
    fn convert(self) -> Frame<Self::Sample, Self::Size> {
        [self.0, self.1, self.2, self.3].into()
    }
}

impl ConstantFrame for (f64, f64, f64, f64, f64) {
    type Sample = f64;
    type Size = U5;
    fn convert(self) -> Frame<Self::Sample, Self::Size> {
        [self.0, self.1, self.2, self.3, self.4].into()
    }
}

impl ConstantFrame for f32 {
    type Sample = f32;
    type Size = U1;
    fn convert(self) -> Frame<Self::Sample, Self::Size> {
        [self].into()
    }
}

impl ConstantFrame for (f32, f32) {
    type Sample = f32;
    type Size = U2;
    fn convert(self) -> Frame<Self::Sample, Self::Size> {
        [self.0, self.1].into()
    }
}

impl ConstantFrame for (f32, f32, f32) {
    type Sample = f32;
    type Size = U3;
    fn convert(self) -> Frame<Self::Sample, Self::Size> {
        [self.0, self.1, self.2].into()
    }
}

impl ConstantFrame for (f32, f32, f32, f32) {
    type Sample = f32;
    type Size = U4;
    fn convert(self) -> Frame<Self::Sample, Self::Size> {
        [self.0, self.1, self.2, self.3].into()
    }
}

impl ConstantFrame for (f32, f32, f32, f32, f32) {
    type Sample = f32;
    type Size = U5;
    fn convert(self) -> Frame<Self::Sample, Self::Size> {
        [self.0, self.1, self.2, self.3, self.4].into()
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

/// -X: negated signal.
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

/// !X: fit signal.
impl<X> std::ops::Not for An<X>
where
    X: AudioNode,
{
    type Output = An<FitNode<X>>;
    #[inline]
    fn not(self) -> Self::Output {
        An(FitNode::new(self.0))
    }
}

/// X + Y: sum signal.
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

/// X + constant: offset signal.
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

/// constant + X: offset signal.
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

/// X + constant: offset signal.
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

/// constant + X: offset signal.
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

/// X - Y: difference signal.
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

/// X - constant: offset signal.
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

/// constant - X: inverted offset signal.
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

/// X - constant: offset signal.
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

/// constant - X: inverted offset signal.
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

/// X * Y: product signal.
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

/// X * constant: amplified signal.
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

/// constant * X: amplified signal.
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

/// X * constant: amplified signal.
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

/// constant * X: amplified signal.
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

/// X >> Y: serial pipe.
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

/// X & Y: parallel bus.
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

/// X ^ Y: parallel branch.
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

/// X | Y: parallel stack.
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

impl<X: AudioNode<Sample = f32>> An<X>
where
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
{
    /// Consumes and returns the component as an FnMut closure
    /// that yields mono samples via AudioNode::get_mono.
    pub fn as_mono_fn(self) -> impl FnMut() -> f32 {
        let mut c = self;
        move || c.get_mono()
    }

    /// Consumes and returns the component as an FnMut closure
    /// that filters mono samples via AudioNode::filter_mono.
    /// Broadcasts the mono input if applicable.
    pub fn as_mono_filter_fn(self) -> impl FnMut(f32) -> f32 {
        let mut c = self;
        move |x| c.filter_mono(x)
    }

    /// Consumes and returns the component as an FnMut closure
    /// that yields stereo samples via AudioNode::get_stereo.
    pub fn as_stereo_fn(self) -> impl FnMut() -> (f32, f32) {
        let mut c = self;
        move || c.get_stereo()
    }

    /// Consumes and returns the component as an FnMut closure
    /// that filters stereo samples via AudioNode::filter_stereo.
    /// Broadcasts the stereo input if applicable.
    pub fn as_stereo_filter_fn(self) -> impl FnMut(f32, f32) -> (f32, f32) {
        let mut c = self;
        move |x, y| c.filter_stereo(x, y)
    }
}
