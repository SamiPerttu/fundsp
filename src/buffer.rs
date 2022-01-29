//! Audio buffers for block processing.

use super::*;
use rsor::Slice;

/// Buffer for holding blocks of sample data.
/// Length of each block is `MAX_BUFFER_SIZE`.
pub struct Buffer<T: Float> {
    buffer: Vec<Vec<T>>,
    slice: Slice<[T]>,
}

impl<T: Float> Default for Buffer<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Float> Buffer<T> {
    pub fn new() -> Self {
        Buffer::<T> {
            buffer: Vec::new(),
            slice: Slice::new(),
        }
    }
    #[inline]
    pub fn get_ref(&mut self, buffers: usize) -> &[&[T]] {
        while self.buffer.len() < buffers {
            let mut v = Vec::with_capacity(MAX_BUFFER_SIZE);
            v.resize(MAX_BUFFER_SIZE, T::default());
            self.buffer.push(v);
        }
        self.slice.from_refs(&self.buffer)
    }
    #[inline]
    pub fn get_mut(&mut self, buffers: usize) -> &mut [&mut [T]] {
        while self.buffer.len() < buffers {
            let mut v = Vec::with_capacity(MAX_BUFFER_SIZE);
            v.resize(MAX_BUFFER_SIZE, T::default());
            self.buffer.push(v);
        }
        self.slice.from_muts(&mut self.buffer)
    }
    #[inline]
    pub fn at(&self, index: usize) -> &Vec<T> {
        &self.buffer[index]
    }
}
