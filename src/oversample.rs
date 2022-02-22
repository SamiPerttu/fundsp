//! Oversampling.

use super::audionode::*;
use super::math::*;
use super::signal::*;
use super::*;
use numeric_array::typenum::*;

/*
Coefficients from https://fiiir.com/, a Kaiser windowed filter with
normalized frequency cutoff 0.22, transition band 0.06 and 80 dB stopband attenuation.
Gain is -1.5 dB at 0.21 (18522 Hz @ 88.2 kHz) and -79 dB at 0.25.
*/
const HALFBAND_LEN: usize = 85;
#[allow(clippy::excessive_precision)]
const HALFBAND: [f32; HALFBAND_LEN] = [
    0.000020220200441046,
    0.000004861974285292,
    -0.000061492255405391,
    -0.000047947308542579,
    0.000111949286674030,
    0.000157238183884534,
    -0.000134029973010597,
    -0.000352195617970385,
    0.000060566750523008,
    0.000618881801452382,
    0.000195044985122935,
    -0.000886256007067166,
    -0.000711008113349721,
    0.001012498615544184,
    0.001513402890270113,
    -0.000793210142009061,
    -0.002525875150117118,
    0.000000000000000015,
    0.003528671213843895,
    0.001549877506957102,
    -0.004145791705015911,
    -0.003902794383274251,
    0.003875807190602988,
    0.006876814923619958,
    -0.002172107493248859,
    -0.009993447571056116,
    -0.001436081065240459,
    0.012454100941225989,
    0.007205685895214254,
    -0.013166435667418973,
    -0.015059319569344419,
    0.010793941877221932,
    0.024518894667885167,
    -0.003738570533651831,
    -0.034721013736356068,
    -0.010206021619584520,
    0.044523346752473138,
    0.035517283003499218,
    -0.052687647461759073,
    -0.087922447018580221,
    0.058103086744721588,
    0.312021697722786151,
    0.439999638527507508,
    0.312021697722786151,
    0.058103086744721588,
    -0.087922447018580221,
    -0.052687647461759073,
    0.035517283003499218,
    0.044523346752473138,
    -0.010206021619584520,
    -0.034721013736356068,
    -0.003738570533651831,
    0.024518894667885167,
    0.010793941877221932,
    -0.015059319569344403,
    -0.013166435667418973,
    0.007205685895214254,
    0.012454100941225989,
    -0.001436081065240459,
    -0.009993447571056116,
    -0.002172107493248859,
    0.006876814923619958,
    0.003875807190602985,
    -0.003902794383274251,
    -0.004145791705015916,
    0.001549877506957102,
    0.003528671213843895,
    0.000000000000000015,
    -0.002525875150117118,
    -0.000793210142009061,
    0.001513402890270113,
    0.001012498615544184,
    -0.000711008113349721,
    -0.000886256007067166,
    0.000195044985122935,
    0.000618881801452382,
    0.000060566750523008,
    -0.000352195617970385,
    -0.000134029973010597,
    0.000157238183884534,
    0.000111949286674031,
    -0.000047947308542579,
    -0.000061492255405391,
    0.000004861974285292,
    0.000020220200441046,
];

fn tick_even<T: Float>(v: &Frame<T, U128>, j: usize) -> T {
    let j = j + 0x80 - HALFBAND_LEN;
    let mut output = T::zero();
    for i in 0..HALFBAND_LEN / 2 + 1 {
        output += v[(j + i * 2) & 0x7f] * T::from_f32(HALFBAND[i * 2]);
    }
    output * T::new(2)
}

fn tick_odd<T: Float>(v: &Frame<T, U128>, j: usize) -> T {
    let j = j + 0x80 - HALFBAND_LEN;
    let mut output = T::zero();
    for i in 0..HALFBAND_LEN / 2 {
        output += v[(j + i * 2 + 1) & 0x7f] * T::from_f32(HALFBAND[i * 2 + 1]);
    }
    output * T::new(2)
}

fn tick<T: Float>(v: &Frame<T, U128>, j: usize) -> T {
    let j = j + 0x80 - HALFBAND_LEN;
    let mut output = T::zero();
    for i in 0..HALFBAND_LEN {
        output += v[(j + i) & 0x7f] * T::from_f32(HALFBAND[i]);
    }
    output
}

pub struct Oversampler<T, X>
where
    T: Float,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    X::Inputs: Size<Frame<T, U128>>,
    X::Outputs: Size<Frame<T, U128>>,
{
    x: X,
    inv: Frame<Frame<T, U128>, X::Inputs>,
    outv: Frame<Frame<T, U128>, X::Outputs>,
    j: usize,
}

impl<T, X> Oversampler<T, X>
where
    T: Float,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    X::Inputs: Size<Frame<T, U128>>,
    X::Outputs: Size<Frame<T, U128>>,
{
    /// Create new oversampler. 2x oversamples enclosed node.
    pub fn new(sample_rate: f64, mut node: X) -> Self {
        let inner_sr = sample_rate * 2.0;
        node.reset(Some(inner_sr));
        let hash = node.ping(true, AttoRand::new(Self::ID));
        node.ping(false, hash);
        Self {
            x: node,
            inv: Frame::default(),
            outv: Frame::default(),
            j: 0,
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

impl<T, X> AudioNode for Oversampler<T, X>
where
    T: Float,
    X: AudioNode<Sample = T>,
    X::Inputs: Size<T>,
    X::Outputs: Size<T>,
    X::Inputs: Size<Frame<T, U128>>,
    X::Outputs: Size<Frame<T, U128>>,
{
    const ID: u64 = 51;
    type Sample = T;
    type Inputs = X::Inputs;
    type Outputs = X::Outputs;

    fn reset(&mut self, sample_rate: Option<f64>) {
        let inner_sr = sample_rate.map(|sr| sr * 2.0);
        self.x.reset(inner_sr);
        self.inv = Frame::default();
        self.outv = Frame::default();
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        for channel in 0..Self::Inputs::USIZE {
            self.inv[channel][self.j] = input[channel];
        }
        let over_input: Frame<T, Self::Inputs> =
            Frame::generate(|channel| tick_even(&self.inv[channel], self.j + 1));
        let over_output = self.x.tick(&over_input);
        for channel in 0..Self::Outputs::USIZE {
            self.outv[channel][self.j] = over_output[channel];
        }
        self.j = (self.j + 1) & 0x7f;
        for channel in 0..Self::Inputs::USIZE {
            self.inv[channel][self.j] = T::zero();
        }
        let over_input2: Frame<T, Self::Inputs> =
            Frame::generate(|channel| tick_odd(&self.inv[channel], self.j + 1));
        let over_output2 = self.x.tick(&over_input2);
        for channel in 0..Self::Outputs::USIZE {
            self.outv[channel][self.j] = over_output2[channel];
        }
        let output: Frame<T, Self::Outputs> =
            Frame::generate(|channel| tick(&self.outv[channel], self.j));
        self.j = (self.j + 1) & 0x7f;
        output
    }

    fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.x.route(input, frequency)
    }

    fn ping(&mut self, probe: bool, hash: AttoRand) -> AttoRand {
        self.x.ping(probe, hash.hash(Self::ID))
    }

    fn set(&mut self, parameter: Tag, value: f64) {
        self.x.set(parameter, value);
    }

    fn get(&self, parameter: Tag) -> Option<f64> {
        self.x.get(parameter)
    }
}
