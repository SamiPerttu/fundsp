//! Shared atomic controls.

use super::audionode::*;
use super::buffer::*;
use super::combinator::*;
use super::signal::*;
use super::*;
use core::sync::atomic::{AtomicU32, AtomicU64};
use numeric_array::typenum::*;
extern crate alloc;
use alloc::sync::Arc;
use alloc::vec::Vec;

/// A variable floating point number to use as a control.
pub trait Atomic: Float {
    type Storage: Send + Sync;

    fn storage(t: Self) -> Self::Storage;
    fn store(stored: &Self::Storage, t: Self);
    fn get_stored(stored: &Self::Storage) -> Self;
}

impl Atomic for f32 {
    type Storage = AtomicU32;

    fn storage(t: Self) -> Self::Storage {
        AtomicU32::from(t.to_bits())
    }

    #[inline]
    fn store(stored: &Self::Storage, t: Self) {
        stored.store(t.to_bits(), core::sync::atomic::Ordering::Relaxed);
    }

    #[inline]
    fn get_stored(stored: &Self::Storage) -> Self {
        let u = stored.load(core::sync::atomic::Ordering::Relaxed);
        f32::from_bits(u)
    }
}

impl Atomic for f64 {
    type Storage = AtomicU64;

    fn storage(t: Self) -> Self::Storage {
        AtomicU64::from(t.to_bits())
    }

    #[inline]
    fn store(stored: &Self::Storage, t: Self) {
        stored.store(t.to_bits(), core::sync::atomic::Ordering::Relaxed);
    }

    #[inline]
    fn get_stored(stored: &Self::Storage) -> Self {
        let u = stored.load(core::sync::atomic::Ordering::Relaxed);
        f64::from_bits(u)
    }
}

/// A shared float variable that can be accessed from multiple threads.
#[derive(Default, Clone)]
pub struct Shared {
    value: Arc<AtomicU32>,
}

impl Shared {
    pub fn new(value: f32) -> Self {
        Self {
            value: Arc::new(f32::storage(value)),
        }
    }

    /// Get reference to underlying atomic.
    #[inline]
    pub fn get_shared(&self) -> &Arc<AtomicU32> {
        &self.value
    }

    /// Set the value of this variable. Synonymous with `set`.
    #[inline]
    pub fn set_value(&self, value: f32) {
        f32::store(&self.value, value)
    }

    /// Set the value of this variable. Synonymous with `set_value`.
    #[inline]
    pub fn set(&self, value: f32) {
        f32::store(&self.value, value)
    }

    /// Get the value of this variable.
    #[inline]
    pub fn value(&self) -> f32 {
        f32::get_stored(&self.value)
    }
}

/// Outputs the value of a shared variable.
#[derive(Default, Clone)]
pub struct Var {
    value: Arc<AtomicU32>,
}

impl Var {
    pub fn new(shared: &Shared) -> Self {
        Self {
            value: Arc::clone(shared.get_shared()),
        }
    }

    /// Set the value of this variable.
    pub fn set_value(&self, value: f32) {
        f32::store(&self.value, value)
    }

    /// Get the value of this variable.
    pub fn value(&self) -> f32 {
        f32::get_stored(&self.value)
    }
}

impl AudioNode for Var {
    const ID: u64 = 68;

    type Inputs = U0;
    type Outputs = U1;

    #[inline]
    fn tick(&mut self, _: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let sample = self.value();
        [sample].into()
    }

    fn process(&mut self, size: usize, _input: &BufferRef, output: &mut BufferMut) {
        let sample = self.value();
        output.channel_mut(0)[..simd_items(size)].fill(F32x::splat(sample.to_f32()));
    }

    fn route(&mut self, _input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut signal = SignalFrame::new(self.outputs());
        signal.set(0, Signal::Value(self.value().to_f64()));
        signal
    }
}

/// Outputs the value of a shared variable mapped through a function.
#[derive(Default, Clone)]
pub struct VarFn<F, R>
where
    F: Clone + Fn(f32) -> R + Send + Sync,
    R: ConstantFrame<Sample = f32>,
    R::Size: Size<f32>,
{
    value: Arc<AtomicU32>,
    f: F,
}

impl<F, R> VarFn<F, R>
where
    F: Clone + Fn(f32) -> R + Send + Sync,
    R: ConstantFrame<Sample = f32>,
    R::Size: Size<f32>,
{
    pub fn new(shared: &Shared, f: F) -> Self {
        Self {
            value: Arc::clone(shared.get_shared()),
            f,
        }
    }
}

impl<F, R> AudioNode for VarFn<F, R>
where
    F: Clone + Fn(f32) -> R + Send + Sync,
    R: ConstantFrame<Sample = f32>,
    R::Size: Size<f32>,
{
    const ID: u64 = 70;

    type Inputs = U0;
    type Outputs = R::Size;

    #[inline]
    fn tick(&mut self, _input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        (self.f)(f32::get_stored(&self.value)).frame()
    }

    fn process(&mut self, size: usize, _input: &BufferRef, output: &mut BufferMut) {
        let frame = (self.f)(f32::get_stored(&self.value)).frame();
        for channel in 0..self.outputs() {
            output.channel_mut(channel)[..simd_items(size)]
                .fill(F32x::splat(frame[channel].to_f32()));
        }
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        // TODO. Should we cache the latest function value and use it as a constant?
        super::signal::Routing::Generator(0.0).route(input, self.outputs())
    }
}

/// Store present stream time to a shared variable.
#[derive(Clone)]
pub struct Timer {
    shared: Shared,
    time: f64,
    sample_duration: f64,
}

impl Timer {
    /// Create a new timer node. Current time can be read from the shared variable.
    pub fn new(shared: &Shared) -> Self {
        shared.set_value(0.0);
        Self {
            shared: shared.clone(),
            time: 0.0,
            sample_duration: 1.0 / DEFAULT_SR,
        }
    }
}

impl AudioNode for Timer {
    const ID: u64 = 57;
    type Inputs = U0;
    type Outputs = U0;

    fn reset(&mut self) {
        self.time = 0.0;
        self.shared.set_value(0.0);
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_duration = 1.0 / sample_rate;
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        self.time += self.sample_duration;
        self.shared.set_value(self.time as f32);
        *input
    }

    fn process(&mut self, size: usize, _input: &BufferRef, _output: &mut BufferMut) {
        self.time += size as f64 * self.sample_duration;
        self.shared.set_value(self.time as f32);
    }

    fn route(&mut self, _input: &SignalFrame, _frequency: f64) -> SignalFrame {
        SignalFrame::new(self.outputs())
    }
}

/// Atomic wavetable that can be modified on the fly.
pub struct AtomicTable {
    table: Vec<AtomicU32>,
}

impl AtomicTable {
    /// Create new table from a slice. The length of the slice must be a power of two.
    pub fn new(wave: &[f32]) -> Self {
        assert!(wave.len().is_power_of_two());
        let mut table = Vec::with_capacity(wave.len());
        for x in wave {
            table.push(f32::storage(*x));
        }
        Self { table }
    }
    /// Length of the table.
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.table.len()
    }
    /// Read sample at index `i`.
    #[inline]
    pub fn at(&self, i: usize) -> f32 {
        f32::get_stored(&self.table[i])
    }
    /// Set sample at index `i`.
    #[inline]
    pub fn set(&self, i: usize, value: f32) {
        f32::store(&self.table[i], value);
    }
    /// Read cubic interpolated value at the given `phase` (in 0...1).
    #[inline]
    pub fn read_cubic(&self, phase: f32) -> f32 {
        let p = self.table.len() as f32 * phase;
        // Safety: we know phase is in 0...1.
        let i1 = unsafe { f32::to_int_unchecked::<usize>(p) };
        let w = p - i1 as f32;
        let mask = self.table.len() - 1;
        let i0 = i1.wrapping_sub(1) & mask;
        let i1 = i1 & mask;
        let i2 = (i1 + 1) & mask;
        let i3 = (i1 + 2) & mask;
        super::math::spline(self.at(i0), self.at(i1), self.at(i2), self.at(i3), w)
    }
    /// Read linear interpolated value at the given `phase` (in 0...1).
    #[inline]
    pub fn read_linear(&self, phase: f32) -> f32 {
        let p = self.table.len() as f32 * phase;
        // Safety: we know phase is in 0...1.
        let i0 = unsafe { f32::to_int_unchecked::<usize>(p) };
        let w = p - i0 as f32;
        let mask = self.table.len() - 1;
        let i0 = i0 & mask;
        let i1 = (i0 + 1) & mask;
        super::math::lerp(self.at(i0), self.at(i1), w)
    }
    /// Read nearest value at the given `phase` (in 0...1).
    #[inline]
    pub fn read_nearest(&self, phase: f32) -> f32 {
        let p = self.table.len() as f32 * phase;
        // Safety: we know phase is in 0...1.
        let i = unsafe { f32::to_int_unchecked::<usize>(p) };
        let mask = self.table.len() - 1;
        self.at(i & mask)
    }
}

/// Wavetable oscillator with cubic interpolation that reads from an atomic wavetable.
#[derive(Clone)]
pub struct AtomicSynth<T: Float> {
    table: Arc<AtomicTable>,
    /// Phase in 0...1.
    phase: f32,
    /// Initial phase in 0...1, seeded via pseudorandom phase system.
    initial_phase: f32,
    sample_rate: f32,
    sample_duration: f32,
    _marker: core::marker::PhantomData<T>,
}

impl<T: Float> AtomicSynth<T> {
    pub fn new(table: Arc<AtomicTable>) -> Self {
        Self {
            table,
            phase: 0.0,
            initial_phase: 0.0,
            sample_rate: DEFAULT_SR as f32,
            sample_duration: 1.0 / DEFAULT_SR as f32,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<T: Float> AudioNode for AtomicSynth<T> {
    const ID: u64 = 86;
    type Inputs = numeric_array::typenum::U1;
    type Outputs = numeric_array::typenum::U1;

    fn reset(&mut self) {
        self.phase = self.initial_phase;
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate as f32;
        self.sample_duration = 1.0 / sample_rate as f32;
    }

    fn set_hash(&mut self, hash: u64) {
        self.initial_phase = super::math::rnd1(hash) as f32;
        self.phase = self.initial_phase;
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let frequency = input[0];
        let delta = frequency * self.sample_duration;
        self.phase += delta;
        self.phase -= self.phase.floor();
        let output = self.table.read_nearest(self.phase);
        Frame::splat(convert(output))
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        super::signal::Routing::Generator(0.0).route(input, self.outputs())
    }
}
