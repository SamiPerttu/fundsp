//! The snoop node shares audio data with a frontend thread.

use super::audionode::*;
use super::buffer::*;
use super::signal::*;
use super::*;
use numeric_array::*;
use thingbuf::mpsc::{channel, Receiver, Sender};
extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

/// Buffer for snooped audio data.
#[derive(Clone)]
pub struct SnoopBuffer {
    data: [f32; MAX_BUFFER_SIZE],
}

impl Default for SnoopBuffer {
    fn default() -> Self {
        SnoopBuffer {
            data: [0.0; MAX_BUFFER_SIZE],
        }
    }
}

impl SnoopBuffer {
    /// Returns sample at index `i`.
    pub fn at(&self, i: usize) -> f32 {
        self.data[i]
    }
    /// Sets sample at index `i`.
    pub fn set(&mut self, i: usize, v: f32) {
        self.data[i] = v;
    }
    /// Length of the buffer.
    pub fn size(&self) -> usize {
        MAX_BUFFER_SIZE
    }
    /// Length of the buffer.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        MAX_BUFFER_SIZE
    }
}

/// Receiver for snooped audio data.
pub struct Snoop {
    receiver: Receiver<SnoopBuffer>,
    index: usize,
    total: u64,
    latest: Vec<f32>,
}

impl Snoop {
    /// Create a new snoop node. Returns a (frontend, backend) pair.
    pub fn new(capacity: usize) -> (Snoop, SnoopBackend) {
        let capacity = capacity.next_power_of_two();
        let (sender, receiver) = channel(1024);
        let snoop = Snoop {
            receiver,
            index: 0,
            total: 0,
            latest: vec![0.0; capacity],
        };
        let snoop_backend = SnoopBackend {
            index: 0,
            buffer: SnoopBuffer::default(),
            sender,
        };
        (snoop, snoop_backend)
    }

    /// Return sample where `index` is a reverse index: the latest sample is at index 0.
    pub fn at(&self, index: usize) -> f32 {
        self.latest[(self.index + self.latest.len() - index - 1) & (self.latest.len() - 1)]
    }

    /// Capacity of the latest sample buffer.
    pub fn capacity(&self) -> usize {
        self.latest.len()
    }

    /// Total number of samples received so far.
    pub fn total(&self) -> u64 {
        self.total
    }

    /// Get the next buffer of data, if available.
    /// Either this method or `update` should be polled repeatedly.
    pub fn get(&mut self) -> Option<SnoopBuffer> {
        if let Ok(buffer) = self.receiver.try_recv() {
            for i in 0..buffer.size() {
                self.latest[self.index] = buffer.at(i);
                self.index = (self.index + 1) & (self.latest.len() - 1);
                self.total += 1;
            }
            Some(buffer)
        } else {
            None
        }
    }

    /// Receive latest data.
    /// Either this method or `get` should be polled repeatedly.
    pub fn update(&mut self) {
        while let Some(_buffer) = self.get() {}
    }
}

/// The snoop backend node passes through audio data while sending it to the snoop frontend.
#[derive(Clone)]
pub struct SnoopBackend {
    index: usize,
    buffer: SnoopBuffer,
    sender: Sender<SnoopBuffer>,
}

impl AudioNode for SnoopBackend {
    const ID: u64 = 77;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;

    fn reset(&mut self) {
        self.index = 0;
    }

    #[inline]
    #[allow(clippy::needless_if)]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        self.buffer.set(self.index, input[0]);
        self.index += 1;
        if self.index == MAX_BUFFER_SIZE {
            if self.sender.try_send(self.buffer.clone()).is_ok() {}
            self.index = 0;
        }
        *input
    }

    #[allow(clippy::needless_if)]
    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        output.channel_mut(0)[..simd_items(size)]
            .clone_from_slice(&input.channel(0)[..simd_items(size)]);
        for i in 0..size {
            self.buffer.set(self.index, input.at_f32(0, i));
            self.index += 1;
            if self.index == MAX_BUFFER_SIZE {
                if self.sender.try_send(self.buffer.clone()).is_ok() {}
                self.index = 0;
            }
        }
    }
    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        input.clone()
    }
}
