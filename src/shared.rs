//! Shared atomic controls.

use super::audionode::*;
use super::combinator::*;
use super::*;
use numeric_array::typenum::*;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

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
        stored.store(t.to_bits(), std::sync::atomic::Ordering::Relaxed);
    }

    #[inline]
    fn get_stored(stored: &Self::Storage) -> Self {
        let u = stored.load(std::sync::atomic::Ordering::Relaxed);
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
        stored.store(t.to_bits(), std::sync::atomic::Ordering::Relaxed);
    }

    #[inline]
    fn get_stored(stored: &Self::Storage) -> Self {
        let u = stored.load(std::sync::atomic::Ordering::Relaxed);
        f64::from_bits(u)
    }
}

/// A shared float variable that can be accessed from multiple threads.
#[derive(Default)]
pub struct Shared<T: Atomic> {
    value: Arc<T::Storage>,
}

impl<T: Atomic> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self {
            value: Arc::clone(&self.value),
        }
    }
}

impl<T: Atomic> Shared<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Arc::new(T::storage(value)),
        }
    }

    /// Get reference to underlying atomic.
    #[inline]
    pub fn get_shared(&self) -> &Arc<T::Storage> {
        &self.value
    }

    /// Set the value of this variable. Synonymous with `set`.
    #[inline]
    pub fn set_value(&self, t: T) {
        T::store(&self.value, t)
    }

    /// Set the value of this variable. Synonymous with `set_value`.
    #[inline]
    pub fn set(&self, t: T) {
        T::store(&self.value, t)
    }

    /// Get the value of this variable.
    #[inline]
    pub fn value(&self) -> T {
        T::get_stored(&self.value)
    }
}

/// Outputs the value of a shared variable.
#[derive(Default)]
pub struct Var<T: Atomic> {
    value: Arc<T::Storage>,
}

impl<T: Atomic> Clone for Var<T> {
    fn clone(&self) -> Self {
        Self {
            value: Arc::clone(&self.value),
        }
    }
}

impl<T: Atomic> Var<T> {
    pub fn new(shared: &Shared<T>) -> Self {
        Self {
            value: Arc::clone(shared.get_shared()),
        }
    }

    /// Set the value of this variable.
    pub fn set_value(&self, t: T) {
        T::store(&self.value, t)
    }

    /// Get the value of this variable.
    pub fn value(&self) -> T {
        T::get_stored(&self.value)
    }
}

impl<T: Atomic> AudioNode for Var<T> {
    const ID: u64 = 68;

    type Sample = T;
    type Inputs = U0;
    type Outputs = U1;
    type Setting = ();

    #[inline]
    fn tick(
        &mut self,
        _: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let sample: T = self.value();
        [sample].into()
    }

    fn process(
        &mut self,
        size: usize,
        _input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        let sample = self.value();
        output[0][..size].fill(sample);
    }
}

/// Outputs the value of a shared variable mapped through a function.
#[derive(Default)]
pub struct VarFn<T, F, R>
where
    T: Atomic,
    F: Clone + Fn(T) -> R,
    R: ConstantFrame<Sample = T>,
    R::Size: Size<T>,
{
    value: Arc<T::Storage>,
    f: F,
}

impl<T, F, R> Clone for VarFn<T, F, R>
where
    T: Atomic,
    F: Clone + Fn(T) -> R,
    R: ConstantFrame<Sample = T>,
    R::Size: Size<T>,
{
    fn clone(&self) -> Self {
        Self {
            value: Arc::clone(&self.value),
            f: self.f.clone(),
        }
    }
}

impl<T, F, R> VarFn<T, F, R>
where
    T: Atomic,
    F: Clone + Fn(T) -> R,
    R: ConstantFrame<Sample = T>,
    R::Size: Size<T>,
{
    pub fn new(shared: &Shared<T>, f: F) -> Self {
        Self {
            value: Arc::clone(shared.get_shared()),
            f,
        }
    }
}

impl<T, F, R> AudioNode for VarFn<T, F, R>
where
    T: Atomic + Float,
    F: Clone + Fn(T) -> R + Send + Sync,
    R: ConstantFrame<Sample = T>,
    R::Size: Size<T>,
{
    const ID: u64 = 70;

    type Sample = T;
    type Inputs = U0;
    type Outputs = R::Size;
    type Setting = ();

    #[inline]
    fn tick(
        &mut self,
        _: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        (self.f)(T::get_stored(&self.value)).convert()
    }

    fn process(
        &mut self,
        size: usize,
        _input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        let frame = (self.f)(T::get_stored(&self.value)).convert();
        for channel in 0..self.outputs() {
            output[channel][..size].fill(frame[channel]);
        }
    }
}

/// Store present stream time to a shared variable.
#[derive(Clone)]
pub struct Timer<T: Atomic> {
    shared: Shared<T>,
    time: f64,
    sample_duration: f64,
}

impl<T: Atomic> Timer<T> {
    /// Create a new timer node. Current time can be read from the shared variable.
    pub fn new(sample_rate: f64, shared: &Shared<T>) -> Self {
        shared.set_value(T::zero());
        Self {
            shared: shared.clone(),
            time: 0.0,
            sample_duration: 1.0 / sample_rate,
        }
    }
}

impl<T: Atomic + Float> AudioNode for Timer<T> {
    const ID: u64 = 57;
    type Sample = T;
    type Inputs = U0;
    type Outputs = U0;
    type Setting = ();

    fn reset(&mut self) {
        self.time = 0.0;
        self.shared.set_value(T::zero());
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_duration = 1.0 / sample_rate;
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        self.time += self.sample_duration;
        self.shared.set_value(T::from_f64(self.time));
        *input
    }

    fn process(
        &mut self,
        size: usize,
        _input: &[&[Self::Sample]],
        _output: &mut [&mut [Self::Sample]],
    ) {
        self.time += size as f64 * self.sample_duration;
        self.shared.set_value(T::from_f64(self.time));
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
    _marker: std::marker::PhantomData<T>,
}

impl<T: Float> AtomicSynth<T> {
    pub fn new(table: Arc<AtomicTable>) -> Self {
        Self {
            table,
            phase: 0.0,
            initial_phase: 0.0,
            sample_rate: DEFAULT_SR as f32,
            sample_duration: 1.0 / DEFAULT_SR as f32,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: Float> AudioNode for AtomicSynth<T> {
    const ID: u64 = 86;
    type Sample = T;
    type Inputs = numeric_array::typenum::U1;
    type Outputs = numeric_array::typenum::U1;
    type Setting = ();

    fn reset(&mut self) {
        self.phase = self.initial_phase;
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate as f32;
        self.sample_duration = 1.0 / sample_rate as f32;
    }

    fn set_hash(&mut self, hash: u64) {
        self.initial_phase = super::hacker::rnd(hash as i64) as f32;
        self.phase = self.initial_phase;
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let frequency = input[0].to_f32();
        let delta = frequency * self.sample_duration;
        self.phase += delta;
        self.phase -= self.phase.floor();
        let output = self.table.read_nearest(self.phase);
        Frame::splat(convert(output))
    }
}
