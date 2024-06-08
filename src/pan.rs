//! Panning functionality.

use super::audionode::*;
use super::buffer::*;
use super::math::*;
use super::setting::*;
use super::signal::*;
use super::*;
use core::marker::PhantomData;
use numeric_array::*;

/// Return equal power pan weights for pan value in -1...1.
#[inline]
fn pan_weights<T: Real>(value: T) -> (T, T) {
    let angle = (clamp11(value) + T::one()) * (T::PI * T::from_f32(0.25));
    (cos(angle), sin(angle))
}

/// Mono-to-stereo equal power panner. Number of inputs is `N`, either 1 or 2.
/// Setting: pan value.
/// Input 0: mono audio
/// Input 1 (optional): pan value in -1...1
/// Output 0: left output
/// Output 1: right output
#[derive(Clone)]
pub struct Panner<N: Size<f32>> {
    _marker: PhantomData<N>,
    left_weight: f32,
    right_weight: f32,
}

impl<N: Size<f32>> Panner<N> {
    pub fn new(value: f32) -> Self {
        let (left_weight, right_weight) = pan_weights(value);
        Self {
            _marker: PhantomData,
            left_weight,
            right_weight,
        }
    }
    #[inline]
    pub fn set_pan(&mut self, value: f32) {
        let (left_weight, right_weight) = pan_weights(value);
        self.left_weight = left_weight;
        self.right_weight = right_weight;
    }
}

impl<N: Size<f32>> AudioNode for Panner<N> {
    const ID: u64 = 49;
    type Inputs = N;
    type Outputs = typenum::U2;

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        if N::USIZE > 1 {
            let value = input[1];
            self.set_pan(value);
        }
        [self.left_weight * input[0], self.right_weight * input[0]].into()
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        if N::USIZE == 1 {
            for i in 0..simd_items(size) {
                output.set(0, i, input.at(0, i) * self.left_weight);
                output.set(1, i, input.at(0, i) * self.right_weight);
            }
        } else {
            for i in 0..size {
                self.set_pan(input.at_f32(1, i));
                output.set_f32(0, i, input.at_f32(0, i) * self.left_weight);
                output.set_f32(1, i, input.at_f32(0, i) * self.right_weight);
            }
        }
    }

    fn set(&mut self, setting: Setting) {
        if let Parameter::Pan(pan) = setting.parameter() {
            self.set_pan(*pan);
        }
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = SignalFrame::new(self.outputs());
        // Pretend the pan value is constant.
        output.set(0, input.at(0).scale(self.left_weight.to_f64()));
        output.set(1, input.at(0).scale(self.right_weight.to_f64()));
        output
    }
}

/// Mixing matrix with `M` input channels and `N` output channels.
#[derive(Clone)]
pub struct Mixer<M, N>
where
    M: Size<f32>,
    N: Size<f32> + Size<Frame<f32, M>>,
{
    matrix: Frame<Frame<f32, M>, N>,
}

impl<M, N> Mixer<M, N>
where
    M: Size<f32>,
    N: Size<f32> + Size<Frame<f32, M>>,
{
    pub fn new(matrix: Frame<Frame<f32, M>, N>) -> Self {
        Self { matrix }
    }
}

impl<M, N> AudioNode for Mixer<M, N>
where
    M: Size<f32>,
    N: Size<f32> + Size<Frame<f32, M>>,
{
    const ID: u64 = 84;
    type Inputs = M;
    type Outputs = N;

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        Frame::generate(|i| {
            let mut value = 0.0;
            for (x, y) in input.iter().zip(self.matrix[i].iter()) {
                value += (*x) * (*y);
            }
            value
        })
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = SignalFrame::new(self.outputs());
        for i in 0..self.outputs() {
            output.set(i, input.at(0).scale(convert(self.matrix[i][0])));
            for j in 1..self.inputs() {
                output.set(
                    i,
                    output.at(i).combine_linear(
                        input.at(j).scale(convert(self.matrix[i][j])),
                        0.0,
                        |x, y| x + y,
                        |x, y| x + y,
                    ),
                );
            }
        }
        output
    }
}
