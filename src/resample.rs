//! Cubic variable speed resampler.

use super::audionode::*;
use super::math::*;
use super::signal::*;
use super::*;
use numeric_array::typenum::*;

#[derive(Clone)]
pub struct Resampler<T, X>
where
    T: Float,
    X: AudioNode<Sample = T, Inputs = U0>,
    X::Outputs: Size<T>,
    X::Outputs: Size<Frame<T, U128>>,
{
    x: X,
    buffer: Frame<Frame<T, U128>, X::Outputs>,
    consumer: f64,
    producer: usize,
}

impl<T, X> Resampler<T, X>
where
    T: Float,
    X: AudioNode<Sample = T, Inputs = U0>,
    X::Outputs: Size<T>,
    X::Outputs: Size<Frame<T, U128>>,
{
    /// Create new resampler. Resamples enclosed generator node output(s)
    /// at speed obtained from the input, where 1 is the original speed.
    pub fn new(sample_rate: f64, mut node: X) -> Self {
        node.reset(Some(sample_rate));
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
    pub fn node(&self) -> &X {
        &self.x
    }

    // Access enclosed node.
    pub fn node_mut(&mut self) -> &mut X {
        &mut self.x
    }
}

impl<T, X> AudioNode for Resampler<T, X>
where
    T: Float,
    X: AudioNode<Sample = T, Inputs = U0>,
    X::Outputs: Size<T>,
    X::Outputs: Size<Frame<T, U128>>,
{
    const ID: u64 = 69;
    type Sample = T;
    // The input is sampling speed where 1 is the original speed.
    type Inputs = U1;
    type Outputs = X::Outputs;
    type Setting = ();

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.x.reset(sample_rate);
        // We start input at the second sample to get proper slope information.
        self.consumer = 1.0;
        self.producer = 0;
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        self.consumer += max(T::zero(), input[0]).to_f64();
        let d = self.consumer - self.consumer.floor();
        let consumer_i = (self.consumer - d) as usize;
        while consumer_i + 2 >= self.producer {
            let inner = self.x.tick(&Frame::default());
            for channel in 0..X::Outputs::USIZE {
                self.buffer[channel][self.producer & 0x7f] = inner[channel];
            }
            self.producer += 1;
        }
        let output: Frame<T, Self::Outputs> = Frame::generate(|channel| {
            spline(
                self.buffer[channel][(consumer_i + 0x7f) & 0x7f],
                self.buffer[channel][consumer_i & 0x7f],
                self.buffer[channel][(consumer_i + 1) & 0x7f],
                self.buffer[channel][(consumer_i + 2) & 0x7f],
                T::from_f64(d),
            )
        });
        output
    }

    fn route(&mut self, _input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output.fill(Signal::Latency(0.0));
        output
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.x.ping(probe, hash.hash(Self::ID))
    }
}
