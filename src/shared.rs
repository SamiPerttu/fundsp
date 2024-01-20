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
