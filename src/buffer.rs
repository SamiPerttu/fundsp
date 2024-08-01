//! SIMD accelerated audio buffers for block processing.

use super::audionode::*;
use super::*;
extern crate alloc;
use alloc::vec::Vec;
use numeric_array::ArrayLength;

/// Mutably borrowed audio buffer with an arbitrary number of channels
/// containing 64 (`MAX_BUFFER_SIZE`) samples per channel. Samples are stored
/// non-interleaved. Intended as a temporary borrow to feed into
/// `AudioNode::process` or `AudioUnit::process`.
pub struct BufferMut<'a>(&'a mut [F32x]);

impl<'a> BufferMut<'a> {
    /// Create new buffer from a slice. The length of the slice must be divisible by 8.
    #[inline]
    pub fn new(buffer: &'a mut [F32x]) -> Self {
        debug_assert!(buffer.len() & (SIMD_LEN - 1) == 0);
        Self(buffer)
    }

    /// Create an empty buffer with 0 channels.
    #[inline]
    pub fn empty() -> Self {
        Self(&mut [])
    }

    /// Create new buffer that is a subset of this buffer.
    #[inline]
    pub fn subset(&mut self, first_channel: usize, channels: usize) -> BufferMut {
        debug_assert!(first_channel + channels <= self.channels());
        BufferMut::new(
            &mut self.0[(first_channel << SIMD_C)..((first_channel + channels) << SIMD_C)],
        )
    }

    /// Convert this buffer into an immutable one.
    #[inline]
    pub fn buffer_ref(&mut self) -> BufferRef {
        BufferRef::new(self.0)
    }

    /// Number of channels in this buffer.
    #[inline]
    pub fn channels(&self) -> usize {
        self.0.len() >> SIMD_C
    }

    /// Get channel as a slice.
    #[inline]
    pub fn channel(&self, channel: usize) -> &[F32x] {
        debug_assert!(channel < self.channels());
        &(self.0)[(channel << SIMD_C)..(channel + 1) << SIMD_C]
    }

    /// Get channel as a mutable slice.
    #[inline]
    pub fn channel_mut(&mut self, channel: usize) -> &mut [F32x] {
        debug_assert!(channel < self.channels());
        &mut (self.0)[(channel << SIMD_C)..(channel + 1) << SIMD_C]
    }

    /// Get channel as a scalar slice.
    #[inline]
    pub fn channel_f32(&self, channel: usize) -> &'a [f32] {
        debug_assert!(channel < self.channels());
        let data = self.channel(channel).as_ptr() as *const f32;
        // Safety: we know each channel contains exactly `MAX_BUFFER_SIZE` samples.
        unsafe { core::slice::from_raw_parts(data, MAX_BUFFER_SIZE) }
    }

    /// Get channel as a mutable scalar slice.
    #[inline]
    pub fn channel_f32_mut(&mut self, channel: usize) -> &'a mut [f32] {
        debug_assert!(channel < self.channels());
        let data = self.channel_mut(channel).as_mut_ptr() as *mut f32;
        // Safety: we know each channel contains exactly `MAX_BUFFER_SIZE` samples.
        unsafe { core::slice::from_raw_parts_mut(data, MAX_BUFFER_SIZE) }
    }

    /// Set value at index `i` (0 <= `i` <= 7).
    #[inline]
    pub fn set(&mut self, channel: usize, i: usize, value: F32x) {
        debug_assert!(channel < self.channels());
        (self.0)[(channel << SIMD_C) + i] = value;
    }

    /// Set `f32` value at index `i` (0 <= `i` <= 63) of `channel`.
    #[inline]
    pub fn set_f32(&mut self, channel: usize, i: usize, value: f32) {
        debug_assert!(channel < self.channels());
        (self.0)[(channel << SIMD_C) + (i >> SIMD_S)].as_array_mut()[i & SIMD_M] = value;
        // Note. There is no difference in speed between the versions above and below.
        //self.channel_f32_mut(channel)[i] = value;
    }

    /// Get value at index `i` (0 <= `i` <= 7) of `channel`.
    #[inline]
    pub fn at(&self, channel: usize, i: usize) -> F32x {
        debug_assert!(channel < self.channels());
        (self.0)[(channel << SIMD_C) + i]
    }

    /// Get value at index `i` (0 <= `i` <= 7) of `channel`.
    #[inline]
    pub fn at_mut(&mut self, channel: usize, i: usize) -> &mut F32x {
        debug_assert!(channel < self.channels());
        &mut (self.0)[(channel << SIMD_C) + i]
    }

    /// Get `f32` value at index `i` (0 <= `i` <= 63) of `channel`.
    #[inline]
    pub fn at_f32(&self, channel: usize, i: usize) -> f32 {
        debug_assert!(channel < self.channels());
        (self.0)[(channel << SIMD_C) + (i >> SIMD_S)].as_array_ref()[i & SIMD_M]
    }

    /// Add to value at index `i` (0 <= `i` <= 7) of `channel`.
    pub fn add(&mut self, channel: usize, i: usize, value: F32x) {
        debug_assert!(channel < self.channels());
        (self.0)[(channel << SIMD_C) + i] += value;
    }
}

/// Immutably borrowed audio buffer with an arbitrary number of channels
/// containing 64 (`MAX_BUFFER_SIZE`) samples per channel. Samples are stored non-interleaved.
/// Intended as a temporary borrow to feed into `AudioNode::process` or `AudioUnit::process`.
pub struct BufferRef<'a>(&'a [F32x]);

impl<'a> BufferRef<'a> {
    /// Create new buffer from a slice. The length of the slice must be divisible by 8.
    #[inline]
    pub fn new(buffer: &'a [F32x]) -> Self {
        debug_assert!(buffer.len() & (SIMD_LEN - 1) == 0);
        Self(buffer)
    }

    /// Create an empty buffer with 0 channels.
    #[inline]
    pub fn empty() -> Self {
        Self(&[])
    }

    /// Create new buffer that is a subset of this buffer.
    #[inline]
    pub fn subset(&self, first_channel: usize, channels: usize) -> BufferRef {
        debug_assert!(first_channel + channels <= self.channels());
        BufferRef::new(&self.0[(first_channel << SIMD_C)..((first_channel + channels) << SIMD_C)])
    }

    /// Number of channels in this buffer.
    #[inline]
    pub fn channels(&self) -> usize {
        self.0.len() >> SIMD_C
    }

    /// Get channel slice.
    #[inline]
    pub fn channel(&self, channel: usize) -> &[F32x] {
        debug_assert!(channel < self.channels());
        &(self.0)[(channel << SIMD_C)..(channel + 1) << SIMD_C]
    }

    /// Get channel as a scalar slice.
    #[inline]
    pub fn channel_f32(&self, channel: usize) -> &'a [f32] {
        debug_assert!(channel < self.channels());
        let data = self.channel(channel).as_ptr() as *const f32;
        // Safety: we know each channel contains exactly `MAX_BUFFER_SIZE` samples.
        unsafe { core::slice::from_raw_parts(data, MAX_BUFFER_SIZE) }
    }

    /// Access value at index `i` (0 <= `i` <= 7).
    #[inline]
    pub fn at(&self, channel: usize, i: usize) -> F32x {
        debug_assert!(channel < self.channels());
        (self.0)[(channel << SIMD_C) + i]
    }

    /// Access `f32` value at index `i` (0 <= `i` <= 63) of `channel`.
    #[inline]
    pub fn at_f32(&self, channel: usize, i: usize) -> f32 {
        debug_assert!(channel < self.channels());
        (self.0)[(channel << SIMD_C) + (i >> SIMD_S)].as_array_ref()[i & SIMD_M]
    }
}

/// An owned buffer on the heap with an arbitrary number of channels
/// containing 64 (`MAX_BUFFER_SIZE`) samples per channel. Samples are stored non-interleaved.
#[derive(Clone, Default)]
pub struct BufferVec {
    buffer: Vec<F32x>,
}

impl BufferVec {
    /// Create new owned buffer with the given number of `channels`.
    pub fn new(channels: usize) -> Self {
        let mut buffer = Vec::with_capacity(channels << SIMD_C);
        buffer.resize(channels << SIMD_C, F32x::ZERO);
        Self { buffer }
    }

    /// Number of channels in this buffer.
    #[inline]
    pub fn channels(&self) -> usize {
        self.buffer.len() >> SIMD_C
    }

    /// Length of the buffer is 8 SIMD samples.
    #[inline]
    pub fn length(&self) -> usize {
        SIMD_LEN
    }

    /// Length of the buffer is 8 SIMD samples.
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        SIMD_LEN
    }

    /// Access value at index `i` (0 <= `i` <= 7) of `channel`.
    #[inline]
    pub fn at(&self, channel: usize, i: usize) -> F32x {
        debug_assert!(channel < self.channels());
        self.buffer[(channel << SIMD_C) + i]
    }

    /// Set `value` at index `i` (0 <= `i` <= 7) of `channel`.
    #[inline]
    pub fn set(&mut self, channel: usize, i: usize, value: F32x) {
        debug_assert!(channel < self.channels());
        self.buffer[(channel << SIMD_C) + i] = value;
    }

    /// Access `f32` value at index `i` (0 <= `i` <= 63) of `channel`.
    #[inline]
    pub fn at_f32(&self, channel: usize, i: usize) -> f32 {
        debug_assert!(channel < self.channels());
        self.buffer[(channel << SIMD_C) + (i >> SIMD_S)].as_array_ref()[i & SIMD_M]
    }

    /// Set `f32` value at index `i` (0 <= `i` <= 63) of `channel`.
    #[inline]
    pub fn set_f32(&mut self, channel: usize, i: usize, value: f32) {
        debug_assert!(channel < self.channels());
        self.buffer[(channel << SIMD_C) + (i >> SIMD_S)].as_array_mut()[i & SIMD_M] = value;
    }

    /// Get channel slice.
    #[inline]
    pub fn channel(&self, channel: usize) -> &[F32x] {
        debug_assert!(channel < self.channels());
        &self.buffer[(channel << SIMD_C)..(channel + 1) << SIMD_C]
    }

    /// Get mutable channel slice.
    #[inline]
    pub fn channel_mut(&mut self, channel: usize) -> &mut [F32x] {
        debug_assert!(channel < self.channels());
        &mut self.buffer[(channel << SIMD_C)..(channel + 1) << SIMD_C]
    }

    /// Get channel as a scalar slice.
    #[inline]
    pub fn channel_f32(&mut self, channel: usize) -> &[f32] {
        debug_assert!(channel < self.channels());
        let data = self.channel(channel).as_ptr() as *const f32;
        // Safety: we know each channel contains exactly `MAX_BUFFER_SIZE` samples.
        unsafe { core::slice::from_raw_parts(data, MAX_BUFFER_SIZE) }
    }

    /// Get channel as a mutable scalar slice.
    #[inline]
    pub fn channel_f32_mut(&mut self, channel: usize) -> &mut [f32] {
        debug_assert!(channel < self.channels());
        let data = self.channel_mut(channel).as_mut_ptr() as *mut f32;
        // Safety: we know each channel contains exactly `MAX_BUFFER_SIZE` samples.
        unsafe { core::slice::from_raw_parts_mut(data, MAX_BUFFER_SIZE) }
    }

    /// Fill all channels of buffer with zeros.
    #[inline]
    pub fn clear(&mut self) {
        self.buffer.fill(F32x::ZERO);
    }

    /// Resize the buffer.
    pub fn resize(&mut self, channels: usize) {
        self.buffer.resize(channels << SIMD_C, F32x::ZERO);
    }

    /// Get an immutably borrowed buffer.
    #[inline]
    pub fn buffer_ref(&self) -> BufferRef {
        BufferRef::new(&self.buffer)
    }

    /// Get a mutably borrowed buffer.
    #[inline]
    pub fn buffer_mut(&mut self) -> BufferMut {
        BufferMut::new(&mut self.buffer)
    }
}

/// An owned audio buffer stored inline as an array with an arbitrary number of channels
/// containing 64 (`MAX_BUFFER_SIZE`) samples per channel.
/// Samples are stored non-interleaved.
/// The number of channels must be known at compile time:
/// the size `N` is given as a type-level integer (`U0`, `U1`, ...).
#[repr(C)]
#[derive(Clone, Default)]
pub struct BufferArray<N: ArrayLength> {
    array: Frame<[F32x; SIMD_LEN], N>,
}

impl<N: ArrayLength> BufferArray<N> {
    /// Create new buffer and initialize it with zeros.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create new buffer.
    #[inline]
    pub(crate) fn uninitialized() -> Self {
        // Safety: This is undefined behavior but it seems to work fine. Zero initialization is safe but slower in benchmarks.
        #[allow(clippy::uninit_assumed_init)]
        unsafe {
            core::mem::MaybeUninit::uninit().assume_init()
        }
    }

    /// Access value at index `i` (0 <= `i` <= 7) of `channel`.
    #[inline]
    pub fn at(&self, channel: usize, i: usize) -> F32x {
        self.array[channel][i]
    }

    /// Get `f32` value at index `i` (0 <= `i` <= 63) of `channel`.
    #[inline]
    pub fn at_f32(&self, channel: usize, i: usize) -> f32 {
        debug_assert!(channel < self.channels());
        self.array[channel][i >> SIMD_S].as_array_ref()[i & SIMD_M]
    }

    /// Set value at index `i` (0 <= `i` <= 7) of `channel`.
    #[inline]
    pub fn set(&mut self, channel: usize, i: usize, value: F32x) {
        debug_assert!(channel < self.channels());
        self.array[channel][i] = value;
    }

    /// Get `f32` value at index `i` (0 <= `i` <= 63) of `channel`.
    #[inline]
    pub fn set_f32(&mut self, channel: usize, i: usize, value: f32) {
        debug_assert!(channel < self.channels());
        self.array[channel][i >> SIMD_S].as_array_mut()[i & SIMD_M] = value;
    }

    /// Number of channels in this buffer.
    #[inline]
    pub fn channels(&self) -> usize {
        N::USIZE
    }

    /// Length of the buffer is 8 SIMD samples.
    #[inline]
    pub fn length(&self) -> usize {
        SIMD_LEN
    }

    /// Length of the buffer is 8 SIMD samples.
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        SIMD_LEN
    }

    /// Fill all channels of buffer with zeros.
    #[inline]
    pub fn clear(&mut self) {
        self.array.fill([F32x::ZERO; SIMD_LEN]);
    }

    /// Get channel slice.
    #[inline]
    pub fn channel(&self, channel: usize) -> &[F32x] {
        // Safety: we know Frames are contiguous and we know the length statically.
        unsafe {
            &core::slice::from_raw_parts(self.array.as_ptr() as *const F32x, N::USIZE << SIMD_C)
                [(channel << SIMD_C)..(channel + 1) << SIMD_C]
        }
    }

    /// Get mutable channel slice.
    #[inline]
    pub fn channel_mut(&mut self, channel: usize) -> &mut [F32x] {
        // Safety: we know Frames are contiguous and we know the length statically.
        unsafe {
            &mut core::slice::from_raw_parts_mut(
                self.array.as_mut_ptr() as *mut F32x,
                N::USIZE << SIMD_C,
            )[(channel << SIMD_C)..(channel + 1) << SIMD_C]
        }
    }

    /// Get channel as a scalar slice.
    #[inline]
    pub fn channel_f32(&mut self, channel: usize) -> &[f32] {
        let data = self.channel(channel).as_ptr() as *const f32;
        // Safety: we know each channel contains exactly `MAX_BUFFER_SIZE` samples.
        unsafe { core::slice::from_raw_parts(data, MAX_BUFFER_SIZE) }
    }

    /// Get channel as a mutable scalar slice.
    #[inline]
    pub fn channel_f32_mut(&mut self, channel: usize) -> &mut [f32] {
        let data = self.channel_mut(channel).as_mut_ptr() as *mut f32;
        // Safety: we know each channel contains exactly `MAX_BUFFER_SIZE` samples.
        unsafe { core::slice::from_raw_parts_mut(data, MAX_BUFFER_SIZE) }
    }

    /// Get immutably borrowed buffer.
    #[inline]
    pub fn buffer_ref(&self) -> BufferRef {
        // Safety: we know Frames are contiguous and we know the length statically.
        let slice = unsafe {
            core::slice::from_raw_parts(self.array.as_ptr() as *const F32x, N::USIZE << SIMD_C)
        };
        BufferRef::new(slice)
    }

    /// Get mutably borrowed buffer.
    #[inline]
    pub fn buffer_mut(&mut self) -> BufferMut {
        // Safety: we know Frames are contiguous and we know the length statically.
        let data = self.array.as_mut_ptr() as *mut F32x;
        let slice = unsafe { core::slice::from_raw_parts_mut(data, N::USIZE << SIMD_C) };
        BufferMut::new(slice)
    }
}
