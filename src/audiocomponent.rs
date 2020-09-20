use crate::prelude::*;

/// AudioComponent processes audio data sample by sample.
/// It has a fixed number of inputs (M) and outputs (N).
/// The sample rate is the system default prelude::DEFAULT_SR, if not set otherwise.
pub trait AudioComponent<const M: usize, const N: usize> {

    /// Resets the input state of the component to an initial state where it has not computed any data. 
    fn reset(&mut self, sample_rate: Option<f64>) {}

    /// Processes one sample.
    fn tick(&mut self, input: [F32; M]) -> [F32; N];

    /// Causal latency from input to output, in (fractional) samples.
    /// After a reset, we can discard this many samples from the output to avoid incurring a pre-delay.
    /// This applies only to components that have both inputs and outputs.
    fn latency(&self) -> f64 { 0.0 }
}

#[derive(Copy, Clone)]
pub struct ConstantComponent<const N: usize> {
    output: [F32; N],
}

impl<const N: usize> ConstantComponent<N> {
    pub fn new(output: [F32; N]) -> Self { ConstantComponent::<N> { output } }
}

impl<const N: usize> AudioComponent<0, N> for ConstantComponent<N> {
    fn tick(&mut self, input: [F32; 0]) -> [F32; N] {
        self.output
    }
}

// Note. Const generics type arithmetic is on hold at the time of writing.
// I do not know how to express binop AudioComponent bounds generically where both
// subcomponents have inputs, something like AudioComponent<{M1+M2},N>.
// It is possible to just specify the input arity of the result, but inelegant,
// as users would have to type it out.

#[derive(Copy, Clone)]
pub struct BinopComponent<X: AudioComponent<M, N>, Y: AudioComponent<0, N>, B: Copy + Fn(F32, F32) -> F32, const M: usize, const N: usize> {
    x: X,
    y: Y,
    binop: B,
}

// TODO. We can use function pointer 
// fn(f32, f32) -> f32 above to avoid being generic over B.

impl<X: AudioComponent<M, N>, Y: AudioComponent<0, N>, B: Copy + Fn(F32, F32) -> F32, const M: usize, const N: usize> AudioComponent<M, N> for BinopComponent<X, Y, B, M, N> {
    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.y.reset(sample_rate);
    }
    fn tick(&mut self, input: [F32; M]) -> [F32; N] {
        let x = self.x.tick(input);
        let y = self.y.tick([]);

        let mut output: [F32; N] = [0.0; N];

        x.iter()
        .zip(y.iter())
        .enumerate()
        .for_each(|(i, (&x, &y))| output[i] = (self.binop)(x, y) );
        
        output
    }
}

#[derive(Copy, Clone)]
pub struct FixedBinopComponent<X: AudioComponent<M, N>, Y: AudioComponent<0, N>, const M: usize, const N: usize> {
    x: X,
    y: Y,
}

impl<X: AudioComponent<M, N>, Y: AudioComponent<0, N>, const M: usize, const N: usize> AudioComponent<M, N> for FixedBinopComponent<X, Y, M, N> {
    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        self.y.reset(sample_rate);
    }
    fn tick(&mut self, input: [F32; M]) -> [F32; N] {
        let x = self.x.tick(input);
        let y = self.y.tick([]);

        let mut output: [F32; N] = [0.0; N];

        x.iter()
        .zip(y.iter())
        .enumerate()
        .for_each(|(i, (&x, &y))| output[i] = x + y );
        
        output
    }
}

impl<Y: AudioComponent<0, N>, const N: usize> std::ops::Add<Y> for ConstantComponent<N> {
    type Output = FixedBinopComponent<Self, Y, 0, N>;
    fn add(self, y: Y) -> Self::Output {
        FixedBinopComponent::<Self, Y, 0, N> { x: self, y }
    }
}

//impl<Y: AudioComponent<M, 1>, const M: usize> std::ops::Shr<Y> for ConstantComponent<M> {
//}

// We would like a simple combinatory style for creating audio processing graphs.
// Below, envelope implies subsampling with linear interpolation, a so-called
// control rate signal, or control signal.
// envelope(|t| exp(-t * 10.0)) * (constant(440.0) >> sine())
