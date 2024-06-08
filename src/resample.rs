//! Cubic variable speed resampler.

use super::audionode::*;
use super::math::*;
use super::signal::*;
use super::*;
use numeric_array::typenum::*;

#[derive(Clone)]
pub struct Resampler<X>
where
    X: AudioNode<Inputs = U0>,
    X::Outputs: Size<f32> + Size<Frame<f32, U128>>,
{
    x: X,
    buffer: Frame<Frame<f32, U128>, X::Outputs>,
    consumer: f64,
    producer: usize,
}

impl<X> Resampler<X>
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
}

impl<X> AudioNode for Resampler<X>
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
