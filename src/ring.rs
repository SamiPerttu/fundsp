//! Ring buffer component.

use super::audionode::*;
use super::buffer::*;
use super::combinator::An;
use super::signal::*;
use super::typenum::*;
use super::*;

use core::sync::atomic::{AtomicUsize, Ordering};

/// Sender side of a ring buffer.
pub struct RingFront<N: Size<f32>, const M: usize> {
    /// Ring buffer.
    queue: Arc<QueueN<BufferArray<N>, M>>,
    /// Current number of items in the queue.
    items: Arc<AtomicUsize>,
}

impl<N: Size<f32>, const M: usize> RingFront<N, M> {
    /// Capacity of the ring buffer.
    #[inline]
    pub fn capacity(&self) -> usize {
        M
    }

    /// Current number of items (of type `BufferArray<N>`) in the ring buffer.
    #[inline]
    pub fn items(&self) -> usize {
        self.items.load(Ordering::Relaxed)
    }

    /// How many items (of type `BufferArray<N>`) is there space for currently in the ring buffer.
    #[inline]
    pub fn space_left(&self) -> usize {
        M - self.items()
    }

    /// Send a buffer. Returns whether we were successful.
    /// If this fails, then it means that the ring buffer is full.
    #[inline]
    pub fn send(&self, buffer: &BufferArray<N>) -> Result<(), ()> {
        let enqueue_result = self.queue.enqueue(buffer.clone());
        if enqueue_result.is_ok() {
            loop {
                let items_current = self.items.load(Ordering::Relaxed);
                let items_new = items_current.wrapping_add(1);
                let result = self.items.compare_exchange(
                    items_current,
                    items_new,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                );
                if result.is_ok() {
                    break;
                }
            }
            Ok(())
        } else {
            Err(())
        }
    }
}

/// Generator node that outputs audio data from a ring buffer sender (`RingFront`).
pub struct Ring<N: Size<f32>, const M: usize> {
    /// Ring buffer.
    queue: Arc<QueueN<BufferArray<N>, M>>,
    /// Current number of items (of type `BufferArray<N>`) in the queue.
    items: Arc<AtomicUsize>,
    /// Current buffer we are reading from.
    buffer: BufferArray<N>,
    /// Current index into buffer.
    index: usize,
}

impl<N: Size<f32>, const M: usize> Clone for Ring<N, M> {
    fn clone(&self) -> Self {
        // We cannot clone ourselves effectively. Create a dummy channel.
        let queue = Arc::new(QueueN::new_const());
        Self {
            queue,
            items: Arc::new(AtomicUsize::new(0)),
            buffer: BufferArray::new(),
            index: MAX_BUFFER_SIZE,
        }
    }
}

impl<N: Size<f32>, const M: usize> Ring<N, M> {
    /// Create new ring buffer with `N` channels and space for `M` buffers of 64 samples.
    pub fn new() -> (RingFront<N, M>, An<Ring<N, M>>) {
        let queue = Arc::new(QueueN::new_const());
        let items = Arc::new(AtomicUsize::new(0));
        (
            RingFront::<N, M> {
                queue: queue.clone(),
                items: items.clone(),
            },
            An(Self {
                queue: queue.clone(),
                items: items.clone(),
                buffer: BufferArray::new(),
                index: MAX_BUFFER_SIZE,
            }),
        )
    }
}

impl<N: Size<f32>, const M: usize> AudioNode for Ring<N, M> {
    const ID: u64 = 92;
    type Inputs = U0;
    type Outputs = N;

    #[inline]
    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        if self.index >= MAX_BUFFER_SIZE {
            self.index = 0;
            let buffer = self.queue.dequeue();
            if buffer.is_some() {
                loop {
                    let items_current = self.items.load(Ordering::Relaxed);
                    let items_new = items_current.wrapping_sub(1);
                    let result = self.items.compare_exchange(
                        items_current,
                        items_new,
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    );
                    if result.is_ok() {
                        break;
                    }
                }
            } else {
                // We output zeros in case of buffer underrun.
                self.buffer = BufferArray::default();
            }
        }
        let i = self.index;
        self.index += 1;
        Frame::generate(|channel| self.buffer.at_f32(channel, i))
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Generator(0.0).route(input, self.outputs())
    }
}
