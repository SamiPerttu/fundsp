//! Resampling components.

use super::audionode::*;
use super::math::*;
use super::signal::*;
use super::*;
use numeric_array::typenum::*;
use resampler::{Attenuation, Latency, ResamplerFir, SampleRate};
extern crate alloc;
use alloc::vec;
use alloc::vec::*;

/// Resampling quality:
/// Medium = 32 taps (latency 16, attenuation 60 dB),
/// High = 64 taps (latency 32, attenuation 90 dB), and
/// Best = 128 taps (latency 64, attenuation 90 dB).
#[derive(Clone)]
pub enum Quality {
    Medium,
    High,
    Best,
}

impl Quality {
    pub fn latency(&self) -> f64 {
        match self {
            Quality::Medium => 16.0,
            Quality::High => 32.0,
            Quality::Best => 64.0,
        }
    }
    pub fn latency_enum(&self) -> Latency {
        match self {
            Quality::Medium => Latency::Sample16,
            Quality::High => Latency::Sample32,
            Quality::Best => Latency::Sample64,
        }
    }
    pub fn attenuation(&self) -> Attenuation {
        match self {
            Quality::Medium => Attenuation::Db60,
            Quality::High => Attenuation::Db90,
            Quality::Best => Attenuation::Db90,
        }
    }
}

/// FIR based sinc resampler. It supports these input and output sample rates:
/// 16 kHz, 22.05 kHz, 32 kHz, 44.1 kHz, 48 kHz, 88.2 kHz, 96 kHz, 176.4 kHz, 192 kHz, 384 kHz.
/// - Output(s): Resampled outputs of contained generator.
pub struct ResampleFir<X>
where
    X: AudioNode<Inputs = U0>,
    X::Outputs: Size<f32> + Size<Frame<f32, X::Outputs>>,
{
    x: X,
    quality: Quality,
    resampler: ResamplerFir,
    source_rate: f64,
    target_rate: f64,
    consumer: usize,
    producer: usize,
    i: usize,
    output: Vec<f32>,
}

fn rate_enum(sample_rate: f64) -> SampleRate {
    match sample_rate {
        16000.0 => SampleRate::Hz16000,
        22050.0 => SampleRate::Hz22050,
        32000.0 => SampleRate::Hz32000,
        44100.0 => SampleRate::Hz44100,
        48000.0 => SampleRate::Hz48000,
        88200.0 => SampleRate::Hz88200,
        96000.0 => SampleRate::Hz96000,
        176400.0 => SampleRate::Hz176400,
        192000.0 => SampleRate::Hz192000,
        384000.0 => SampleRate::Hz384000,
        _ => panic!("ResampleFir: Unsupported sample rate: {}", sample_rate),
    }
}

impl<X> ResampleFir<X>
where
    X: AudioNode<Inputs = U0>,
    X::Outputs: Size<f32> + Size<Frame<f32, X::Outputs>>,
{
    pub fn new(source_rate: f64, target_rate: f64, quality: Quality, mut node: X) -> Self {
        node.set_sample_rate(source_rate);
        Self {
            x: node,
            quality: quality.clone(),
            resampler: ResamplerFir::new(
                X::Outputs::USIZE,
                rate_enum(source_rate),
                rate_enum(target_rate),
                quality.latency_enum(),
                quality.attenuation(),
            ),
            source_rate,
            target_rate,
            consumer: 0,
            producer: 0,
            i: 0,
            output: vec![0.0; 24],
        }
    }

    // Access enclosed node.
    #[inline]
    pub fn node(&self) -> &X {
        &self.x
    }

    // Access enclosed node.
    #[inline]
    pub fn node_mut(&mut self) -> &mut X {
        &mut self.x
    }

    /// Number of input samples processed.
    #[inline]
    pub fn samples(&self) -> usize {
        self.consumer
    }

    /// Number of output samples processed.
    #[inline]
    pub fn produced(&self) -> usize {
        self.producer
    }
}

impl<X> Clone for ResampleFir<X>
where
    X: AudioNode<Inputs = U0>,
    X::Outputs: Size<f32> + Size<Frame<f32, X::Outputs>>,
{
    fn clone(&self) -> ResampleFir<X> {
        Self {
            x: self.x.clone(),
            quality: self.quality.clone(),
            resampler: ResamplerFir::new(
                X::Outputs::USIZE,
                rate_enum(self.source_rate),
                rate_enum(self.target_rate),
                self.quality.latency_enum(),
                self.quality.attenuation(),
            ),
            source_rate: self.source_rate,
            target_rate: self.target_rate,
            consumer: 0,
            producer: 0,
            i: 0,
            output: vec![0.0; 24],
        }
    }
}

impl<X> AudioNode for ResampleFir<X>
where
    X: AudioNode<Inputs = U0>,
    X::Outputs: Size<f32> + Size<Frame<f32, X::Outputs>>,
{
    const ID: u64 = 99;
    type Inputs = U0;
    type Outputs = X::Outputs;

    fn reset(&mut self) {
        self.x.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x.reset();
        self.target_rate = sample_rate;
        self.resampler = ResamplerFir::new(
            X::Outputs::USIZE,
            rate_enum(self.source_rate),
            rate_enum(self.target_rate),
            self.quality.latency_enum(),
            self.quality.attenuation(),
        );
    }

    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        while self.i < self.producer {
            let state = self
                .resampler
                .resample(&self.x.tick(&Frame::default()), &mut self.output)
                .unwrap();
            self.consumer += state.0;
            self.i += state.1;
        }
        let output = &self.output[(self.i - self.producer) * self.outputs()
            ..(self.i - self.producer + 1) * self.outputs()];
        self.producer += 1;
        Frame::generate(|i| output[i])
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Generator(0.0).route(input, Self::Outputs::USIZE)
    }
}

/// Resample enclosed generator `node` using cubic interpolation
/// at speed obtained from input 0, where 1 is the original speed.
/// - Input 0: Sampling speed.
/// - Output(s): Resampled outputs of contained generator.
#[derive(Clone)]
pub struct Resample<X>
where
    X: AudioNode<Inputs = U0>,
    X::Outputs: Size<f32> + Size<Frame<f32, U128>>,
{
    x: X,
    buffer: Frame<Frame<f32, U128>, X::Outputs>,
    consumer: f64,
    producer: usize,
}

impl<X> Resample<X>
where
    X: AudioNode<Inputs = U0>,
    X::Outputs: Size<f32> + Size<Frame<f32, U128>>,
{
    /// Create new resampler. Resamples enclosed generator node output(s)
    /// at speed obtained from the input, where 1 is the original speed.
    pub fn new(sample_rate: f64, mut node: X) -> Self {
        node.set_sample_rate(sample_rate);
        let hash = node.ping(true, AttoHash::new(Self::ID));
        node.ping(false, hash);
        Self {
            x: node,
            buffer: Frame::default(),
            consumer: 1.0,
            producer: 0,
        }
    }

    // Access enclosed node.
    #[inline]
    pub fn node(&self) -> &X {
        &self.x
    }

    // Access enclosed node.
    #[inline]
    pub fn node_mut(&mut self) -> &mut X {
        &mut self.x
    }

    /// Number of input samples processed.
    #[inline]
    pub fn samples(&self) -> usize {
        self.consumer.floor() as usize
    }
}

impl<X> AudioNode for Resample<X>
where
    X: AudioNode<Inputs = U0>,
    X::Outputs: Size<f32> + Size<Frame<f32, U128>>,
{
    const ID: u64 = 69;
    // The input is sampling speed where 1 is the original speed.
    type Inputs = U1;
    type Outputs = X::Outputs;

    fn reset(&mut self) {
        self.x.reset();
        // We start input at the second sample to get proper slope information.
        self.consumer = 1.0;
        self.producer = 0;
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x.set_sample_rate(sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        self.consumer += max(0.0, input[0]).to_f64();
        let d = self.consumer - self.consumer.floor();
        let consumer_i = (self.consumer - d) as usize;
        while consumer_i + 2 >= self.producer {
            let inner = self.x.tick(&Frame::default());
            for channel in 0..X::Outputs::USIZE {
                self.buffer[channel][self.producer & 0x7f] = inner[channel];
            }
            self.producer += 1;
        }
        let output: Frame<f32, Self::Outputs> = Frame::generate(|channel| {
            spline(
                self.buffer[channel][(consumer_i + 0x7f) & 0x7f],
                self.buffer[channel][consumer_i & 0x7f],
                self.buffer[channel][(consumer_i + 1) & 0x7f],
                self.buffer[channel][(consumer_i + 2) & 0x7f],
                d as f32,
            )
        });
        output
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        super::signal::Routing::Generator(0.0).route(input, self.outputs())
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.x.ping(probe, hash.hash(Self::ID))
    }

    fn allocate(&mut self) {
        self.x.allocate();
    }
}
