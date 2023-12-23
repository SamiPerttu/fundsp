//! Audio buffers for block processing.

use super::*;
use rsor::Slice;

/// Multichannel buffer for holding blocks of sample data.
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

impl<T: Float> Clone for Buffer<T> {
    fn clone(&self) -> Self {
        let mut buffer = Buffer::new();
        buffer.resize(self.channels());
        buffer
    }
}

impl<T: Float> Buffer<T> {
    /// Create an empty buffer. No allocations are made.
    pub fn new() -> Self {
        Buffer::<T> {
            buffer: Vec::new(),
            slice: Slice::new(),
        }
    }

    /// Create a buffer with the specified number of `channels`.
    pub fn with_channels(channels: usize) -> Self {
        let mut buffer = Buffer::<T> {
            buffer: Vec::new(),
            slice: Slice::new(),
        };
        buffer.resize(channels);
        buffer
    }

    /// Number of channels presently in the buffer.
    #[inline]
    pub fn channels(&self) -> usize {
        self.buffer.len()
    }

    /// Set the number of `channels`.
    #[inline]
    pub fn resize(&mut self, channels: usize) {
        if self.buffer.len() != channels {
            if self.buffer.len() > channels {
                self.buffer.truncate(channels);
            } else {
                self.slice.fill(|mut v| {
                    v.reserve_exact(channels);
                    v
                });
                while self.buffer.len() < channels {
                    let mut v = Vec::with_capacity(MAX_BUFFER_SIZE);
                    v.resize(MAX_BUFFER_SIZE, T::default());
                    self.buffer.push(v);
                }
            }
        }
    }

    /// Get reference to a slice of slices with the given number of `channels`.
    /// The buffer is resized if necessary.
    #[inline]
    pub fn get_ref(&mut self, channels: usize) -> &[&[T]] {
        self.resize(channels);
        self.slice.from_refs(&self.buffer)
    }

    /// Get reference to a mutable slice of slices with the given number of `channels`.
    /// The buffer is resized if necessary.
    #[inline]
    pub fn get_mut(&mut self, channels: usize) -> &mut [&mut [T]] {
        self.resize(channels);
        self.slice.from_muts(&mut self.buffer)
    }

    /// Return reference to `channel` vector.
    #[inline]
    pub fn at(&self, channel: usize) -> &Vec<T> {
        &self.buffer[channel]
    }

    /// Return mutable reference to `channel` vector.
    #[inline]
    pub fn mut_at(&mut self, channel: usize) -> &mut Vec<T> {
        &mut self.buffer[channel]
    }

    /// Get reference to a slice of slices.
    #[inline]
    pub fn self_ref(&mut self) -> &[&[T]] {
        self.slice.from_refs(&self.buffer)
    }

    /// Get reference to a mutable slice of slices.
    #[inline]
    pub fn self_mut(&mut self) -> &mut [&mut [T]] {
        self.slice.from_muts(&mut self.buffer)
    }

    /// Get reference to the vector of vectors.
    #[inline]
    pub fn vec(&self) -> &Vec<Vec<T>> {
        &self.buffer
    }

    /// Get mutable reference to the vector of vectors.
    #[inline]
    pub fn vec_mut(&mut self) -> &mut Vec<Vec<T>> {
        &mut self.buffer
    }
}
