//! Ring buffer component.

use super::audionode::*;
use super::combinator::An;
use super::signal::*;
use super::typenum::*;
use super::*;

pub struct Ring<N: Size<f32>, const M: usize> {
    queue: Arc<Queue<Frame<f32, N>, M>>,
}

impl<N: Size<f32>, const M: usize> Clone for Ring<N, M> {
    fn clone(&self) -> Self {
        // We cannot clone ourselves effectively. Create a dummy channel.
        let queue = Arc::new(Queue::new_const());
        Self { queue }
    }
}

impl<N: Size<f32>, const M: usize> Ring<N, M> {
    /// Create new ring buffer with space for `capacity` frames (for example, 44100). Returns (frontend, backend).
    pub fn new() -> (Arc<Queue<Frame<f32, N>, M>>, An<Ring<N, M>>) {
        let queue = Arc::new(Queue::new_const());
        (
            queue.clone(),
            An(Self {
                queue: queue.clone(),
            }),
        )
    }
}

impl<N: Size<f32>, const M: usize> AudioNode for Ring<N, M> {
    const ID: u64 = 92;
    type Inputs = U0;
    type Outputs = N;

    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        // We output zeros in case of buffer underrun.
        self.queue.dequeue().unwrap_or_default()
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Generator(0.0).route(input, self.outputs())
    }
}
