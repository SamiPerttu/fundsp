use super::audionode::*;
use super::signal::*;
use super::*;
use numeric_array::typenum::*;
use std::marker::PhantomData;

/// Waveshaper.
pub struct Shaper<T, S> {
    f: S,
    _marker: PhantomData<T>,
}

impl<T, S> Shaper<T, S>
where
    T: Float,
    S: Fn(T) -> T,
{
    pub fn new(f: S) -> Self {
        Self {
            f,
            _marker: PhantomData::default(),
        }
    }
}

impl<T, S> AudioNode for Shaper<T, S>
where
    T: Float,
    S: Fn(T) -> T,
{
    const ID: u64 = 37;
    type Sample = T;
    type Inputs = U1;
    type Outputs = U1;

    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        [(self.f)(input[0])].into()
    }

    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        let input = input[0];
        let output = &mut *output[0];
        for i in 0..size {
            output[i] = (self.f)(input[i]);
        }
    }

    fn propagate(&self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].distort(0.0);
        output
    }
}
