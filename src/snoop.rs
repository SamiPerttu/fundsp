//! The snoop node shares audio data with a frontend thread.

use super::audionode::*;
use super::signal::*;
use super::*;
use numeric_array::*;
use thingbuf::mpsc::blocking::{channel, Receiver, Sender};

/// Buffer for snooped audio data.
#[derive(Clone)]
pub struct SnoopBuffer<T: Float> {
    data: [T; MAX_BUFFER_SIZE],
}

impl<T: Float> Default for SnoopBuffer<T> {
    fn default() -> Self {
        SnoopBuffer {
            data: [T::zero(); MAX_BUFFER_SIZE],
        }
    }
}

impl<T: Float> SnoopBuffer<T> {
    /// Returns sample at index `i`.
    pub fn at(&self, i: usize) -> T {
        self.data[i]
    }
    /// Sets sample at index `i`.
    pub fn set(&mut self, i: usize, v: T) {
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
pub struct Snoop<T: Float> {
    receiver: Receiver<SnoopBuffer<T>>,
    index: usize,
    total: u64,
    latest: Vec<T>,
}

impl<T: Float> Snoop<T> {
    /// Create a new snoop node. Returns a (frontend, backend) pair.
    pub fn new(capacity: usize) -> (Snoop<T>, SnoopBackend<T>) {
        let capacity = capacity.next_power_of_two();
        let (sender, receiver) = channel(1024);
        let snoop = Snoop {
            receiver,
            index: 0,
            total: 0,
            latest: vec![T::zero(); capacity],
        };
        let snoop_backend = SnoopBackend {
            index: 0,
            buffer: SnoopBuffer::default(),
            sender,
        };
        (snoop, snoop_backend)
    }

    /// Return sample where `index` is a reverse index: the latest sample is at index 0.
    pub fn at(&self, index: usize) -> T {
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
    pub fn get(&mut self) -> Option<SnoopBuffer<T>> {
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
pub struct SnoopBackend<T: Float> {
    index: usize,
    buffer: SnoopBuffer<T>,
    sender: Sender<SnoopBuffer<T>>,
}

impl<T: Float> AudioNode for SnoopBackend<T> {
    const ID: u64 = 77;
    type Sample = T;
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;
    type Setting = ();

    fn reset(&mut self) {
        self.index = 0;
    }

    #[inline]
    #[allow(clippy::needless_if)]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        self.buffer.set(self.index, input[0]);
        self.index += 1;
        if self.index == MAX_BUFFER_SIZE {
            if self.sender.try_send(self.buffer.clone()).is_ok() {}
            self.index = 0;
        }
        *input
    }

    #[allow(clippy::needless_if)]
    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        output[0][..size].clone_from_slice(&input[0][..size]);
        for i in 0..size {
            self.buffer.set(self.index, input[0][i]);
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
