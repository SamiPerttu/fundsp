use super::*;
use numeric_array::typenum::*;
use super::audiocomponent::*;

/// Trait for multi-channel constants.
pub trait ConstantFrame {
    type Size: Size;
    fn convert(self) -> Frame<Self::Size>;
}

impl ConstantFrame for f48 {
    type Size = U1;
    fn convert(self) -> Frame<Self::Size> { [self].into() }
}

impl ConstantFrame for (f48, f48) {
    type Size = U2;
    fn convert(self) -> Frame<Self::Size> { [self.0, self.1].into() }
}

impl ConstantFrame for (f48, f48, f48) {
    type Size = U3;
    fn convert(self) -> Frame<Self::Size> { [self.0, self.1, self.2].into() }
}

impl ConstantFrame for (f48, f48, f48, f48) {
    type Size = U4;
    fn convert(self) -> Frame<Self::Size> { [self.0, self.1, self.2, self.3].into() }
}

impl ConstantFrame for (f48, f48, f48, f48, f48) {
    type Size = U5;
    fn convert(self) -> Frame<Self::Size> { [self.0, self.1, self.2, self.3, self.4].into() }
}
/// AudioComponent wrapper that implements operators and traits.
#[derive(Clone)]
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

/// X + Y: sum signal.
impl<X, Y> std::ops::Add<Ac<Y>> for Ac<X> where
    X: AudioComponent,
    Y: AudioComponent<Outputs = X::Outputs>,
    X::Inputs: Size + Add<Y::Inputs>,
    Y::Inputs: Size,
    <X::Inputs as Add<Y::Inputs>>::Output: Size,
{
    type Output = Ac<BinopComponent<X, Y, FrameAdd<X::Outputs>>>;
    #[inline] fn add(self, y: Ac<Y>) -> Self::Output {
        Ac(BinopComponent::new(self.0, y.0, FrameAdd::new()))
    }
}

/// -X: negated signal.
impl<X> std::ops::Neg for Ac<X> where
    X: AudioComponent,
    X::Outputs: Size,
{
    type Output = Ac<UnopComponent<X, FrameNeg<X::Outputs>>>;
    #[inline] fn neg(self) -> Self::Output {
        Ac(UnopComponent::new(self.0, FrameNeg::new()))
    }
}

/// !X: monitor signal.
impl<X> std::ops::Not for Ac<X> where
    X: AudioComponent,
{
    type Output = Ac<MonitorComponent<X>>;
    #[inline] fn not(self) -> Self::Output {
        Ac(MonitorComponent::new(self.0))
    }
}

/// X + constant: offset signal.
impl<X> std::ops::Add<f48> for Ac<X> where
    X: AudioComponent,
    X::Inputs: Size + Add<U0>,
    X::Outputs: Size,
    <X::Inputs as Add<U0>>::Output: Size
{
    type Output = Ac<BinopComponent<X, ConstantComponent<X::Outputs>, FrameAdd<X::Outputs>>>;
    #[inline] fn add(self, y: f48) -> Self::Output {
        Ac(BinopComponent::new(self.0, ConstantComponent::new(Frame::splat(y)), FrameAdd::new()))
    }
}

/// constant + X: offset signal.
impl<X> std::ops::Add<Ac<X>> for f48 where
    X: AudioComponent,
    X::Inputs: Size + Add<U0>,
    X::Outputs: Size,
    <X::Inputs as Add<U0>>::Output: Size
{
    type Output = Ac<BinopComponent<ConstantComponent<X::Outputs>, X, FrameAdd<X::Outputs>>>;
    #[inline] fn add(self, y: Ac<X>) -> Self::Output {
        Ac(BinopComponent::new(ConstantComponent::new(Frame::splat(self)), y.0, FrameAdd::new()))
    }
}

/// X - Y: difference signal.
impl<X, Y> std::ops::Sub<Ac<Y>> for Ac<X> where
    X: AudioComponent,
    Y: AudioComponent<Outputs = X::Outputs>,
    X::Inputs: Size + Add<Y::Inputs>,
    Y::Inputs: Size,
    <X::Inputs as Add<Y::Inputs>>::Output: Size,
{
    type Output = Ac<BinopComponent<X, Y, FrameSub<X::Outputs>>>;
    #[inline] fn sub(self, y: Ac<Y>) -> Self::Output {
        Ac(BinopComponent::new(self.0, y.0, FrameSub::new()))
    }
}

/// X - constant: offset signal.
impl<X> std::ops::Sub<f48> for Ac<X> where
    X: AudioComponent,
    X::Inputs: Size + Add<U0>,
    X::Outputs: Size,
    <X::Inputs as Add<U0>>::Output: Size
{
    type Output = Ac<BinopComponent<X, ConstantComponent<X::Outputs>, FrameSub<X::Outputs>>>;
    #[inline] fn sub(self, y: f48) -> Self::Output {
        Ac(BinopComponent::new(self.0, ConstantComponent::new(Frame::splat(y)), FrameSub::new()))
    }
}

/// constant - X: inverted offset signal.
impl<X> std::ops::Sub<Ac<X>> for f48 where
    X: AudioComponent,
    X::Inputs: Size + Add<U0>,
    X::Outputs: Size,
    <X::Inputs as Add<U0>>::Output: Size
{
    type Output = Ac<BinopComponent<ConstantComponent<X::Outputs>, X, FrameSub<X::Outputs>>>;
    #[inline] fn sub(self, y: Ac<X>) -> Self::Output {
        Ac(BinopComponent::new(ConstantComponent::new(Frame::splat(self)), y.0, FrameSub::new()))
    }
}

/// X * Y: product signal.
impl<X, Y> std::ops::Mul<Ac<Y>> for Ac<X> where
    X: AudioComponent,
    Y: AudioComponent<Outputs = X::Outputs>,
    X::Inputs: Size + Add<Y::Inputs>,
    Y::Inputs: Size,
    <X::Inputs as Add<Y::Inputs>>::Output: Size,
{
    type Output = Ac<BinopComponent<X, Y, FrameMul<X::Outputs>>>;
    #[inline] fn mul(self, y: Ac<Y>) -> Self::Output {
        Ac(BinopComponent::new(self.0, y.0, FrameMul::new()))
    }
}

/// X * constant: amplified signal.
impl<X> std::ops::Mul<f48> for Ac<X> where
    X: AudioComponent,
    X::Inputs: Size + Add<U0>,
    X::Outputs: Size,
    <X::Inputs as Add<U0>>::Output: Size
{
    type Output = Ac<BinopComponent<X, ConstantComponent<X::Outputs>, FrameMul<X::Outputs>>>;
    #[inline] fn mul(self, y: f48) -> Self::Output {
        Ac(BinopComponent::new(self.0, ConstantComponent::new(Frame::splat(y)), FrameMul::new()))
    }
}

/// constant * X: amplified signal.
impl<X> std::ops::Mul<Ac<X>> for f48 where
    X: AudioComponent,
    X::Inputs: Size + Add<U0>,
    X::Outputs: Size,
    <X::Inputs as Add<U0>>::Output: Size
{
    type Output = Ac<BinopComponent<ConstantComponent<X::Outputs>, X, FrameMul<X::Outputs>>>;
    #[inline] fn mul(self, y: Ac<X>) -> Self::Output {
        Ac(BinopComponent::new(ConstantComponent::new(Frame::splat(self)), y.0, FrameMul::new()))
    }
}

/// X / Y: serial cascade.
impl<X, Y> std::ops::Div<Ac<Y>> for Ac<X> where
    X: AudioComponent,
    Y: AudioComponent<Inputs = X::Inputs>,
    Y::Outputs: Size,
{
    type Output = Ac<CascadeComponent<X, Y>>;
    #[inline] fn div(self, y: Ac<Y>) -> Self::Output {
        Ac(CascadeComponent::new(self.0, y.0))
    }
}

/// X >> Y: serial pipe.
impl<X, Y> std::ops::Shr<Ac<Y>> for Ac<X> where
    X: AudioComponent,
    Y: AudioComponent<Inputs = X::Outputs>,
    Y::Outputs: Size,
{
    type Output = Ac<PipeComponent<X, Y>>;
    #[inline] fn shr(self, y: Ac<Y>) -> Self::Output {
        Ac(PipeComponent::new(self.0, y.0))
    }
}

/// X & Y: parallel bus.
impl<X, Y> std::ops::BitAnd<Ac<Y>> for Ac<X> where
    X: AudioComponent,
    Y: AudioComponent<Inputs = X::Inputs, Outputs = X::Outputs>,
    Y::Inputs: Size,
    Y::Outputs: Size,
{
    type Output = Ac<BusComponent<X, Y>>;
    #[inline] fn bitand(self, y: Ac<Y>) -> Self::Output {
        Ac(BusComponent::new(self.0, y.0))
    }
}

/// X ^ Y: parallel branch.
impl<X, Y> std::ops::BitXor<Ac<Y>> for Ac<X> where
    X: AudioComponent,
    Y: AudioComponent<Inputs = X::Inputs>,
    X::Outputs: Size + Add<Y::Outputs>,
    Y::Outputs: Size,
    <X::Outputs as Add<Y::Outputs>>::Output: Size
{
    type Output = Ac<BranchComponent<X, Y>>;
    #[inline] fn bitxor(self, y: Ac<Y>) -> Self::Output {
        Ac(BranchComponent::new(self.0, y.0))
    }
}

/// X | Y: parallel stack.
impl<X, Y> std::ops::BitOr<Ac<Y>> for Ac<X> where
    X: AudioComponent,
    Y: AudioComponent,
    X::Inputs: Size + Add<Y::Inputs>,
    X::Outputs: Size + Add<Y::Outputs>,
    Y::Inputs: Size,
    Y::Outputs: Size,
    <X::Inputs as Add<Y::Inputs>>::Output: Size,
    <X::Outputs as Add<Y::Outputs>>::Output: Size
{
    type Output = Ac<StackComponent<X, Y>>;
    #[inline] fn bitor(self, y: Ac<Y>) -> Self::Output {
        Ac(StackComponent::new(self.0, y.0))
    }
}

impl<X: AudioComponent> Iterator for Ac<X>
{
    type Item = Frame<X::Outputs>;
    /// Processes a sample from an all-zeros input.
    #[inline] fn next(&mut self) -> Option<Self::Item> { 
        Some(self.tick(&Frame::default()))
    }
}

impl<X: AudioComponent> Ac<X> {
    /// Consumes and returns the component as an FnMut closure
    /// that yields mono samples via AudioComponent::get_mono.
    pub fn as_mono_fn(self) -> impl FnMut() -> f48 {
        let mut c = self;
        move || c.get_mono()
    }

    /// Consumes and returns the component as an FnMut closure
    /// that filters mono samples via AudioComponent::filter_mono.
    /// Broadcasts the mono input if applicable.
    pub fn as_mono_filter_fn(self) -> impl FnMut(f48) -> f48 {
        let mut c = self;
        move |x| c.filter_mono(x)
    }

    /// Consumes and returns the component as an FnMut closure
    /// that yields stereo samples via AudioComponent::get_stereo.
    pub fn as_stereo_fn(self) -> impl FnMut() -> (f48, f48) {
        let mut c = self;
        move || c.get_stereo()
    }

    /// Consumes and returns the component as an FnMut closure
    /// that filters stereo samples via AudioComponent::filter_stereo.
    /// Broadcasts the stereo input if applicable.
    pub fn as_stereo_filter_fn(self) -> impl FnMut(f48, f48) -> (f48, f48) {
        let mut c = self;
        move |x, y| c.filter_stereo(x, y)
    }
}
