//! Oversampling.

use super::audionode::*;
use super::math::*;
use super::signal::*;
use super::*;
use numeric_array::typenum::*;

// Coefficients from https://fiiir.com/, a linear phase Kaiser windowed filter with
// normalized frequency cutoff 0.22, transition band 0.06 and 80 dB stopband attenuation.
// Gain is -1.5 dB at 0.21 (18522 Hz @ 88.2 kHz) and -79 dB at 0.25.
const _HALFBAND_LINEAR_LEN: usize = 85;
#[allow(clippy::excessive_precision)]
const _HALFBAND_LINEAR: [f32; _HALFBAND_LINEAR_LEN] = [
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

// Minimum phase version of the linear phase filter. Generated with scipy at:
// https://www.tutorialspoint.com/execute_scipy_online.php
// from scipy.signal import minimum_phase
// min_phase = minimum_phase(linear_phase, method='homomorphic')
const HALFBAND_MIN_LEN: usize = 43;
#[allow(clippy::excessive_precision)]
const HALFBAND_MIN: [f32; HALFBAND_MIN_LEN] = [
    4.73552339e-02,
    1.81988040e-01,
    3.49148434e-01,
    3.92748135e-01,
    2.18230867e-01,
    -5.31842843e-02,
    -1.79186566e-01,
    -7.34488007e-02,
    8.94524103e-02,
    1.00868556e-01,
    -2.08681451e-02,
    -8.82510989e-02,
    -2.07640777e-02,
    6.22587555e-02,
    4.07776255e-02,
    -3.52258090e-02,
    -4.57407870e-02,
    1.27033444e-02,
    4.14376136e-02,
    3.30799834e-03,
    -3.24608206e-02,
    -1.27856355e-02,
    2.21659033e-02,
    1.67803711e-02,
    -1.27406974e-02,
    -1.68177367e-02,
    5.35518220e-03,
    1.44761581e-02,
    -3.70651781e-04,
    -1.11140183e-02,
    -2.40622311e-03,
    7.71596027e-03,
    3.48227062e-03,
    -4.86763558e-03,
    -3.45536353e-03,
    2.79880054e-03,
    2.86736431e-03,
    -1.48746153e-03,
    -2.11827989e-03,
    7.72684113e-04,
    1.44384114e-03,
    -4.49807048e-04,
    -9.41945265e-04,
];

#[inline]
fn tick_even(v: &Frame<f32, U128>, j: usize) -> f32 {
    let j = j + 0x80 - HALFBAND_MIN_LEN;
    let mut output = 0.0;
    for i in 0..HALFBAND_MIN_LEN / 2 + 1 {
        output += v[(j + i * 2) & 0x7f] * HALFBAND_MIN[i * 2];
    }
    output * 2.0
}

#[inline]
fn tick_odd(v: &Frame<f32, U128>, j: usize) -> f32 {
    let j = j + 0x80 - HALFBAND_MIN_LEN;
    let mut output = 0.0;
    for i in 0..HALFBAND_MIN_LEN / 2 {
        output += v[(j + i * 2 + 1) & 0x7f] * HALFBAND_MIN[i * 2 + 1];
    }
    output * 2.0
}

#[inline]
fn tick(v: &Frame<f32, U128>, j: usize) -> f32 {
    let j = j + 0x80 - HALFBAND_MIN_LEN;
    let mut output = 0.0;
    for i in 0..HALFBAND_MIN_LEN {
        output += v[(j + i) & 0x7f] * HALFBAND_MIN[i];
    }
    output
}

#[derive(Clone)]
pub struct Oversampler<X>
where
    X: AudioNode,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
    X::Inputs: Size<Frame<f32, U128>>,
    X::Outputs: Size<Frame<f32, U128>>,
{
    x: X,
    inv: Frame<Frame<f32, U128>, X::Inputs>,
    outv: Frame<Frame<f32, U128>, X::Outputs>,
    j: usize,
}

impl<X> Oversampler<X>
where
    X: AudioNode,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
    X::Inputs: Size<Frame<f32, U128>>,
    X::Outputs: Size<Frame<f32, U128>>,
{
    /// Create new oversampler. 2x oversamples enclosed node.
    pub fn new(sample_rate: f64, mut node: X) -> Self {
        let inner_sr = sample_rate * 2.0;
        node.set_sample_rate(inner_sr);
        let hash = node.ping(true, AttoHash::new(Self::ID));
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

impl<X> AudioNode for Oversampler<X>
where
    X: AudioNode,
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
    X::Inputs: Size<Frame<f32, U128>>,
    X::Outputs: Size<Frame<f32, U128>>,
{
    const ID: u64 = 51;
    type Inputs = X::Inputs;
    type Outputs = X::Outputs;

    fn reset(&mut self) {
        self.x.reset();
        self.inv = Frame::default();
        self.outv = Frame::default();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        let inner_sr = sample_rate * 2.0;
        self.x.set_sample_rate(inner_sr);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        for channel in 0..Self::Inputs::USIZE {
            self.inv[channel][self.j] = input[channel];
        }
        let over_input: Frame<f32, Self::Inputs> =
            Frame::generate(|channel| tick_even(&self.inv[channel], self.j + 1));
        let over_output = self.x.tick(&over_input);
        for channel in 0..Self::Outputs::USIZE {
            self.outv[channel][self.j] = over_output[channel];
        }
        self.j = (self.j + 1) & 0x7f;
        for channel in 0..Self::Inputs::USIZE {
            self.inv[channel][self.j] = 0.0;
        }
        let over_input2: Frame<f32, Self::Inputs> =
            Frame::generate(|channel| tick_odd(&self.inv[channel], self.j + 1));
        let over_output2 = self.x.tick(&over_input2);
        for channel in 0..Self::Outputs::USIZE {
            self.outv[channel][self.j] = over_output2[channel];
        }
        let output: Frame<f32, Self::Outputs> =
            Frame::generate(|channel| tick(&self.outv[channel], self.j));
        self.j = (self.j + 1) & 0x7f;
        output
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.x.route(input, frequency)
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.x.ping(probe, hash.hash(Self::ID))
    }

    fn allocate(&mut self) {
        self.x.allocate();
    }
}
