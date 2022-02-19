//! Oversampling.

use super::audionode::*;
use super::filter::*;
use super::math::*;
use super::signal::*;
use super::*;
use numeric_array::typenum::*;

/// Lowpass filtering for oversampling.
struct Lowpass<T: Float, F: Real> {
    lp1: ButterLowpass<T, F, U1>,
    lp2: ButterLowpass<T, F, U1>,
    lp3: ButterLowpass<T, F, U1>,
    lp4: ButterLowpass<T, F, U1>,
    lp5: ButterLowpass<T, F, U1>,
}

impl<T: Float, F: Real> Lowpass<T, F> {
    pub fn new(sample_rate: f64) -> Self {
        let cutoff = F::new(20_000);
        Self {
            lp1: ButterLowpass::new(sample_rate, cutoff),
            lp2: ButterLowpass::new(sample_rate, cutoff),
            lp3: ButterLowpass::new(sample_rate, cutoff),
            lp4: ButterLowpass::new(sample_rate, cutoff),
            lp5: ButterLowpass::new(sample_rate, cutoff),
        }
    }
    pub fn tick(&mut self, x: T) -> T {
        self.lp5.tick(
            &self
                .lp4
                .tick(&self.lp3.tick(&self.lp2.tick(&self.lp1.tick(&[x].into())))),
        )[0]
    }
    pub fn reset(&mut self, sample_rate: Option<f64>) {
        self.lp1.reset(sample_rate);
        self.lp2.reset(sample_rate);
        self.lp3.reset(sample_rate);
        self.lp4.reset(sample_rate);
        self.lp5.reset(sample_rate);
    }
}

pub struct Oversampler<T, F, X>
where
    T: Float,
    F: Real,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
{
    ratio: i64,
    x: X,
    inp: Vec<Lowpass<T, F>>,
    outp: Vec<Lowpass<T, F>>,
}

impl<T, F, X> Oversampler<T, F, X>
where
    T: Float,
    F: Real,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
{
    /// Create new oversampler. Oversamples enclosed node by `ratio` (`ratio` > 1).
    pub fn new(sample_rate: f64, ratio: i64, mut node: X) -> Self {
        let inner_sr = sample_rate * ratio as f64;
        node.reset(Some(inner_sr));
        let mut inp = Vec::new();
        for _ in 0..X::Inputs::USIZE {
            inp.push(Lowpass::new(inner_sr));
        }
        let mut outp = Vec::new();
        for _ in 0..X::Outputs::USIZE {
            outp.push(Lowpass::new(inner_sr));
        }
        let hash = node.ping(true, AttoRand::new(Self::ID));
        node.ping(false, hash);
        Self {
            ratio,
            x: node,
            inp,
            outp,
        }
    }

    // Access enclosed node.
    pub fn node(&self) -> &X {
        &self.x
    }

    // Access enclosed node.
    pub fn node_mut(&mut self) -> &mut X {
        &mut self.x
    }
}

impl<T, F, X> AudioNode for Oversampler<T, F, X>
where
    T: Float,
    F: Real,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
{
    const ID: u64 = 51;
    type Sample = T;
    type Inputs = X::Inputs;
    type Outputs = X::Outputs;

    fn reset(&mut self, sample_rate: Option<f64>) {
        let inner_sr = sample_rate.map(|sr| sr * self.ratio as f64);
        self.x.reset(inner_sr);
        for lp in self.inp.iter_mut() {
            lp.reset(inner_sr);
        }
        for lp in self.outp.iter_mut() {
            lp.reset(inner_sr);
        }
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let in1 = Frame::generate(|i| self.inp[i].tick(input[i] * T::new(self.ratio)));
        let out1 = self.x.tick(&in1);
        let out2 = Frame::generate(|i| self.outp[i].tick(out1[i]));
        for _ in 1..self.ratio {
            let in2 = Frame::generate(|i| self.inp[i].tick(T::zero()));
            let out3 = self.x.tick(&in2);
            let _out4: Frame<T, X::Outputs> = Frame::generate(|i| self.outp[i].tick(out3[i]));
        }
        out2
    }

    fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.x.route(input, frequency)
    }

    fn ping(&mut self, probe: bool, hash: AttoRand) -> AttoRand {
        self.x.ping(probe, hash.hash(Self::ID))
    }
}
