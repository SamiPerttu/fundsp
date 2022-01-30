//! Feedback component.

use super::audionode::*;
use super::math::*;
use super::signal::*;
use super::*;
use std::marker::PhantomData;

/// Diffusive Hadamard feedback matrix.
#[derive(Default)]
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
    // Not implemented.
    // TODO: Hadamard is a special op because of interchannel dependencies.
    #[inline]
    fn propagate(_: Signal) -> Signal {
        panic!()
    }
    fn assign(_size: usize, _x: &mut [T]) {
        panic!()
    }
}

/// Mix back output of contained node to its input.
/// The contained node must have an equal number of inputs and outputs.
pub struct Feedback<T, X, N, U>
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
    #[allow(dead_code)]
    feedback: U,
}

impl<T, X, N, U> Feedback<T, X, N, U>
where
    T: Float,
    X: AudioNode<Sample = T, Inputs = N, Outputs = N>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    N: Size<T>,
    U: FrameUnop<T, X::Outputs>,
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

impl<T, X, N, U> AudioNode for Feedback<T, X, N, U>
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

    fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = self.x.route(input, frequency);
        for i in 0..N::USIZE {
            output[i] = input[i].distort(0.0);
        }
        output
    }

    #[inline]
    fn ping(&mut self, probe: bool, hash: AttoRand) -> AttoRand {
        self.x.ping(probe, hash.hash(Self::ID))
    }
}
