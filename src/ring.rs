//! Ring buffer component.

use super::audionode::*;
use super::combinator::An;
use super::signal::*;
use super::typenum::*;
use super::*;
use thingbuf::mpsc::{channel, Receiver, Sender};

pub struct Ring<N: Size<f32>> {
    receiver: Receiver<Frame<f32, N>>,
}

impl<N: Size<f32>> Clone for Ring<N> {
    fn clone(&self) -> Self {
        // We cannot clone ourselves effectively. Create a dummy channel.
        let (_sender, receiver) = channel(1);
        Self { receiver }
    }
}

impl<N: Size<f32>> Ring<N> {
    /// Create new ring buffer with space for `capacity` frames (for example, 44100). Returns (frontend, backend).
    pub fn new(capacity: usize) -> (Sender<Frame<f32, N>>, An<Ring<N>>) {
        let (sender, receiver) = channel(capacity);
        (sender, An(Self { receiver }))
    }
}

impl<N: Size<f32>> AudioNode for Ring<N> {
    const ID: u64 = 92;
    type Inputs = U0;
    type Outputs = N;

    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        // We output zeros in case of buffer underrun.
        self.receiver.try_recv().unwrap_or_default()
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Generator(0.0).route(input, self.outputs())
    }
}
