//! Audio buffers for block processing.

use super::*;
use rsor::Slice;

/// Buffers for holding blocks of sample data.
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
    pub fn with_size(buffers: usize) -> Self {
        let mut buffer = Buffer::<T> {
            buffer: Vec::new(),
            slice: Slice::new(),
        };
        buffer.resize(buffers);
        buffer
    }

    #[inline]
    pub fn buffers(&self) -> usize {
        self.buffer.len()
    }

    #[inline]
    pub fn resize(&mut self, buffers: usize) {
        while self.buffer.len() < buffers {
            let mut v = Vec::with_capacity(MAX_BUFFER_SIZE);
            v.resize(MAX_BUFFER_SIZE, T::default());
            self.buffer.push(v);
        }
    }
    #[inline]
    pub fn get_ref(&mut self, buffers: usize) -> &[&[T]] {
        self.resize(buffers);
        self.slice.from_refs(&self.buffer)
    }
    #[inline]
    pub fn get_mut(&mut self, buffers: usize) -> &mut [&mut [T]] {
        self.resize(buffers);
        self.slice.from_muts(&mut self.buffer)
    }
    #[inline]
    pub fn at(&self, index: usize) -> &Vec<T> {
        &self.buffer[index]
    }
    #[inline]
    pub fn mut_at(&mut self, index: usize) -> &mut Vec<T> {
        &mut self.buffer[index]
    }
    #[inline]
    pub fn self_ref(&mut self) -> &[&[T]] {
        self.slice.from_refs(&self.buffer)
    }
    #[inline]
    pub fn self_mut(&mut self) -> &mut [&mut [T]] {
        self.slice.from_muts(&mut self.buffer)
    }
}
