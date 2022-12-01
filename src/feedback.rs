//! Feedback component.

use super::audionode::*;
use super::math::*;
use super::signal::*;
use super::*;
use std::marker::PhantomData;

/// Diffusive Hadamard feedback matrix.
#[derive(Default, Clone)]
pub struct FrameHadamard<N: Size<T>, T: Float> {
    _marker: PhantomData<(N, T)>,
}

impl<N: Size<T>, T: Float> FrameHadamard<N, T> {
    pub fn new() -> FrameHadamard<N, T> {
        FrameHadamard::default()
    }
}

impl<N: Size<T>, T: Float> FrameUnop<N, T> for FrameHadamard<N, T> {
    #[inline]
    fn unop(&self, x: &Frame<T, N>) -> Frame<T, N> {
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
    // Not implemented.
    // TODO: Hadamard is a special op because of interchannel dependencies.
    #[inline]
    fn propagate(&self, _: Signal) -> Signal {
        panic!()
    }
    fn assign(&self, _size: usize, _x: &mut [T]) {
        panic!()
    }
}

/// Mix back output of contained node to its input.
/// The contained node must have an equal number of inputs and outputs.
#[derive(Clone)]
pub struct Feedback<N, T, X, U>
where
    N: Size<T>,
    T: Float,
    X: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    U: FrameUnop<X::Outputs, T>,
{
    x: X,
    // Current feedback value.
    value: Frame<T, N>,
    // Feedback operator.
    #[allow(dead_code)]
    feedback: U,
}

impl<N, T, X, U> Feedback<N, T, X, U>
where
    N: Size<T>,
    T: Float,
    X: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    U: FrameUnop<X::Outputs, T>,
{
    pub fn new(x: X, feedback: U) -> Self {
        let mut node = Feedback {
            x,
            value: Frame::default(),
            feedback,
        };
        let hash = node.ping(true, AttoRand::new(Self::ID));
        node.ping(false, hash);
        node
    }
}

impl<N, T, X, U> AudioNode for Feedback<N, T, X, U>
where
    N: Size<T>,
    T: Float,
    X: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    U: FrameUnop<X::Outputs, T>,
{
    const ID: u64 = 11;
    type Sample = T;
    type Inputs = N;
    type Outputs = N;
    type Setting = ();

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
        self.value = self.feedback.unop(&output);
        output
    }

    fn route(&self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Arbitrary.propagate(input, self.outputs())
    }

    fn ping(&mut self, probe: bool, hash: AttoRand) -> AttoRand {
        self.x.ping(probe, hash.hash(Self::ID))
    }
}

/// Mix back output of contained node `X` to its input, with extra feedback processing `Y`.
/// The contained nodes must have an equal number of inputs and outputs.
#[derive(Clone)]
pub struct Feedback2<N, T, X, Y, U>
where
    N: Size<T>,
    T: Float,
    X: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    Y: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    Y::Inputs: Size<T>,
    Y::Outputs: Size<T>,
    U: FrameUnop<X::Outputs, T>,
{
    x: X,
    // Feedback processing.
    y: Y,
    // Current feedback value.
    value: Frame<T, N>,
    // Feedback operator.
    #[allow(dead_code)]
    feedback: U,
}

impl<N, T, X, Y, U> Feedback2<N, T, X, Y, U>
where
    N: Size<T>,
    T: Float,
    X: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    Y: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    Y::Inputs: Size<T>,
    Y::Outputs: Size<T>,
    U: FrameUnop<X::Outputs, T>,
{
    pub fn new(x: X, y: Y, feedback: U) -> Self {
        let mut node = Feedback2 {
            x,
            y,
            value: Frame::default(),
            feedback,
        };
        let hash = node.ping(true, AttoRand::new(Self::ID));
        node.ping(false, hash);
        node
    }
}

impl<N, T, X, Y, U> AudioNode for Feedback2<N, T, X, Y, U>
where
    N: Size<T>,
    T: Float,
    X: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    Y: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    Y::Inputs: Size<T>,
    Y::Outputs: Size<T>,
    U: FrameUnop<X::Outputs, T>,
{
    const ID: u64 = 66;
    type Sample = T;
    type Inputs = N;
    type Outputs = N;
    type Setting = ();

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.y.reset(sample_rate);
        self.value = Frame::default();
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let output = self.x.tick(&(input + self.value.clone()));
        self.value = self.feedback.unop(&self.y.tick(&output));
        output
    }

    fn route(&self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Arbitrary.propagate(input, self.outputs())
    }

    fn ping(&mut self, probe: bool, hash: AttoRand) -> AttoRand {
        self.y.ping(probe, self.x.ping(probe, hash.hash(Self::ID)))
    }
}
