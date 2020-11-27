use super::audionode::*;
use super::math::*;
use super::*;
use std::marker::PhantomData;

/// Identity op.
#[derive(Clone, Default)]
pub struct FrameId<T: Float, N: Size<T>> {
    _marker: PhantomData<(T, N)>,
}

impl<T: Float, N: Size<T>> FrameId<T, N> {
    pub fn new() -> FrameId<T, N> {
        FrameId::default()
    }
}

impl<T: Float, N: Size<T>> FrameUnop<T, N> for FrameId<T, N> {
    #[inline]
    fn unop(x: &Frame<T, N>) -> Frame<T, N> {
        x.clone()
    }
}

/// Diffusive Hadamard feedback matrix.
#[derive(Clone, Default)]
pub struct FrameHadamard<T: Float, N: Size<T>> {
    _marker: PhantomData<(T, N)>,
}

impl<T: Float, N: Size<T>> FrameHadamard<T, N> {
    pub fn new() -> FrameHadamard<T, N> {
        FrameHadamard::default()
    }
}

impl<T: Float, N: Size<T>> FrameUnop<T, N> for FrameHadamard<T, N> {
    #[inline]
    fn unop(x: &Frame<T, N>) -> Frame<T, N> {
        let mut output = x.clone();
        let mut h = 1;
        while h < N::USIZE {
            let mut i = 0;
            while i < N::USIZE {
                for j in i..i + h {
                    let x = output[j];
                    let y = output[j + h];
                    output[j] = x + y;
                    output[j + h] = x - y;
                }
                i += h * 2;
            }
            h *= 2;
        }
        // Normalization for up to 511 channels.
        if N::USIZE >= 256 {
            return output * Frame::splat(T::from_f64(1.0 / 16.0));
        }
        if N::USIZE >= 128 {
            return output * Frame::splat(T::from_f64(1.0 / (SQRT_2 * 8.0)));
        }
        if N::USIZE >= 64 {
            return output * Frame::splat(T::from_f64(1.0 / 8.0));
        }
        if N::USIZE >= 32 {
            return output * Frame::splat(T::from_f64(1.0 / (SQRT_2 * 4.0)));
        }
        if N::USIZE >= 16 {
            return output * Frame::splat(T::from_f64(1.0 / 4.0));
        }
        if N::USIZE >= 8 {
            return output * Frame::splat(T::from_f64(1.0 / (SQRT_2 * 2.0)));
        }
        if N::USIZE >= 4 {
            return output * Frame::splat(T::from_f64(1.0 / 2.0));
        }
        if N::USIZE >= 2 {
            return output * Frame::splat(T::from_f64(1.0 / SQRT_2));
        }
        output
    }
}

/// Mix back output of contained node to its input.
/// The contained node must have an equal number of inputs and outputs.
#[derive(Clone)]
pub struct FeedbackNode<T, X, N, U>
where
    T: Float,
    X: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    N: Size<T>,
    U: FrameUnop<T, X::Outputs>,
{
    x: X,
    // Current feedback value.
    value: Frame<T, N>,
    // Feedback operator.
    feedback: U,
}

impl<T, X, N, U> FeedbackNode<T, X, N, U>
where
    T: Float,
    X: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    N: Size<T>,
    U: FrameUnop<T, X::Outputs>,
{
    pub fn new(x: X, feedback: U) -> Self {
        let mut node = FeedbackNode {
            x,
            value: Frame::default(),
            feedback,
        };
        let hash = node.ping(true, AttoRand::new(Self::ID));
        node.ping(false, hash);
        node
    }
}

impl<T, X, N, U> AudioNode for FeedbackNode<T, X, N, U>
where
    T: Float,
    X: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    N: Size<T>,
    U: FrameUnop<T, X::Outputs>,
{
    const ID: u64 = 11;
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
        self.value = U::unop(&output);
        output
    }

    fn latency(&self) -> Option<f64> {
        self.x.latency()
    }

    #[inline]
    fn ping(&mut self, probe: bool, hash: AttoRand) -> AttoRand {
        self.x.ping(probe, hash.hash(Self::ID))
    }
}
