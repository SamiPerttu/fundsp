//! Audio dynamics related components.

use super::audionode::*;
use super::buffer::*;
use super::follow::*;
use super::math::*;
use super::shared::*;
use super::signal::*;
use super::*;
use core::sync::atomic::AtomicU32;
use numeric_array::typenum::*;
extern crate alloc;
use alloc::sync::Arc;
use alloc::vec::Vec;

/// Binary operation for the monoidal reducer.
pub trait Monoidal<T>: Clone {
    fn binop(&self, x: T, y: T) -> T;
}

#[derive(Default, Clone)]
pub struct Amplitude<T: Num> {
    _marker: core::marker::PhantomData<T>,
}

impl<T: Num> Amplitude<T> {
    pub fn new() -> Self {
        Amplitude::default()
    }
}

impl<T: Num> Monoidal<T> for Amplitude<T> {
    #[inline]
    fn binop(&self, x: T, y: T) -> T {
        max(abs(x), abs(y))
    }
}

#[derive(Default, Clone)]
pub struct Maximum<T: Num> {
    _marker: core::marker::PhantomData<T>,
}

impl<T: Num> Maximum<T> {
    pub fn new() -> Self {
        Maximum::default()
    }
}

impl<T: Num> Monoidal<T> for Maximum<T> {
    #[inline]
    fn binop(&self, x: T, y: T) -> T {
        max(x, y)
    }
}

/// Hierarchic reducer for a monoid.
#[derive(Clone)]
pub struct ReduceBuffer<T, B>
where
    T: Num,
    B: Monoidal<T>,
{
    // First item is unused for convenience. Buffer length is rounded up to an even number.
    buffer: Vec<T>,
    length: usize,
    leaf_offset: usize,
    binop: B,
}

impl<T, B> ReduceBuffer<T, B>
where
    T: Num,
    B: Monoidal<T>,
{
    #[inline]
    fn get_index(&self, i: usize) -> usize {
        self.leaf_offset + i
    }

    // Assumption: 0 is the zero element.
    pub fn new(length: usize, binop: B) -> Self {
        let leaf_offset = length.next_power_of_two();
        let mut buffer = Self {
            buffer: Vec::new(),
            length,
            leaf_offset,
            binop,
        };
        buffer
            .buffer
            .resize(leaf_offset + length + (length & 1), T::zero());
        buffer
    }

    #[inline]
    pub fn length(&self) -> usize {
        self.length
    }

    #[inline]
    pub fn total(&self) -> T {
        self.buffer[1]
    }

    pub fn set(&mut self, index: usize, value: T) {
        let mut i = self.get_index(index);
        self.buffer[i] = value;
        while i > 1 {
            let reduced = self.binop.binop(self.buffer[i], self.buffer[i ^ 1]);
            i >>= 1;
            self.buffer[i] = reduced;
        }
    }

    pub fn clear(&mut self) {
        for i in 0..self.buffer.len() {
            self.buffer[i] = T::zero();
        }
    }
}

/// Look-ahead limiter.
#[derive(Clone)]
pub struct Limiter<N>
where
    N: Size<f32>,
{
    lookahead: f64,
    #[allow(dead_code)]
    release: f64,
    sample_rate: f64,
    reducer: ReduceBuffer<f32, Maximum<f32>>,
    follower: AFollow<f32>,
    buffer: Vec<Frame<f32, N>>,
    index: usize,
}

impl<N> Limiter<N>
where
    N: Size<f32>,
{
    #[inline]
    fn advance(&mut self) {
        self.index += 1;
        if self.index >= self.reducer.length() {
            self.index = 0;
        }
    }

    fn buffer_length(sample_rate: f64, lookahead: f64) -> usize {
        max(1, round(sample_rate * lookahead) as usize)
    }

    fn new_buffer(sample_rate: f64, lookahead: f64) -> ReduceBuffer<f32, Maximum<f32>> {
        ReduceBuffer::new(Self::buffer_length(sample_rate, lookahead), Maximum::new())
    }

    pub fn new(sample_rate: f64, attack_time: f32, release_time: f32) -> Self {
        let mut follower = AFollow::new(attack_time * 0.4, release_time * 0.4);
        follower.set_sample_rate(sample_rate);
        Limiter {
            lookahead: attack_time as f64,
            release: release_time as f64,
            sample_rate,
            follower,
            buffer: Vec::new(),
            reducer: Self::new_buffer(sample_rate, attack_time.to_f64()),
            index: 0,
        }
    }
}

impl<N> AudioNode for Limiter<N>
where
    N: Size<f32>,
{
    const ID: u64 = 25;
    type Inputs = N;
    type Outputs = N;

    fn reset(&mut self) {
        self.set_sample_rate(self.sample_rate);
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.index = 0;
        self.sample_rate = sample_rate;
        let length = Self::buffer_length(sample_rate, self.lookahead);
        if length != self.reducer.length {
            self.reducer = Self::new_buffer(sample_rate, self.lookahead);
        }
        self.follower.set_sample_rate(sample_rate);
        self.reducer.clear();
        self.buffer.clear();
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let amplitude = input.iter().fold(0.0, |amp, &x| max(amp, abs(x)));
        self.reducer.set(self.index, amplitude);
        if self.buffer.len() < self.reducer.length() {
            // We are filling up the initial buffer.
            self.buffer.push(input.clone());
            if self.buffer.len() == self.reducer.length() {
                // When the buffer is full, start following from its total peak.
                self.follower.set_value(self.reducer.total());
            }
            self.advance();
            Frame::default()
        } else {
            let output = self.buffer[self.index].clone();
            self.buffer[self.index] = input.clone();
            // Leave some headroom.
            self.follower
                .filter_mono(max(1.0, self.reducer.total() * 1.10));
            self.advance();
            let limit = self.follower.value();
            output * Frame::splat(1.0 / limit)
        }
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = SignalFrame::new(self.outputs());
        for i in 0..N::USIZE {
            // We pretend that the limiter does not alter the frequency response.
            output.set(i, input.at(i).delay(self.reducer.length() as f64));
        }
        output
    }

    fn allocate(&mut self) {
        if self.buffer.capacity() < self.reducer.length() {
            self.buffer
                .reserve(self.reducer.length() - self.buffer.capacity());
        }
    }
}

/// Transient filter. Multiply the signal with a fade-in curve.
/// After fade-in, pass signal through.
/// - Input 0: input signal
/// - Output 0: filtered signal
#[derive(Default, Clone)]
pub struct Declick<F: Real> {
    t: F,
    duration: F,
    sample_duration: F,
    sample_rate: f64,
}

impl<F: Real> Declick<F> {
    pub fn new(duration: F) -> Self {
        let mut node = Self {
            duration,
            ..Default::default()
        };
        node.set_sample_rate(DEFAULT_SR);
        node
    }
}

impl<F: Real> AudioNode for Declick<F> {
    const ID: u64 = 23;
    type Inputs = U1;
    type Outputs = U1;

    fn reset(&mut self) {
        self.t = F::zero();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
        self.sample_duration = F::from_f64(1.0 / sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        if self.t < self.duration {
            let phase = delerp(F::zero(), self.duration, self.t);
            let value = smooth5(phase);
            self.t += self.sample_duration;
            [input[0] * value.to_f32()].into()
        } else {
            [input[0]].into()
        }
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        output.channel_mut(0)[..simd_items(size)]
            .clone_from_slice(&input.channel(0)[..simd_items(size)]);
        if self.t < self.duration {
            let mut phase = delerp(F::zero(), self.duration, self.t);
            let phase_d = self.sample_duration / self.duration;
            let end_time = self.t + F::new(size as i64) * self.sample_duration;
            let end_index = if self.duration < end_time {
                ceil((self.duration - self.t) / self.sample_duration).to_i64() as usize
            } else {
                size
            };
            for x in output.channel_f32_mut(0)[0..end_index].iter_mut() {
                *x *= smooth5(phase).to_f32();
                phase += phase_d;
            }
            self.t = end_time;
        }
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        // We pretend that the declicker does not alter the frequency response.
        input.clone()
    }
}

/// Metering modes.
#[derive(Copy, Clone)]
pub enum Meter {
    /// Latest value.
    Sample,
    /// Peak meter with smoothing timescale in seconds.
    /// Smoothing timescale is the time it takes for level estimation to move halfway to a new level.
    Peak(f64),
    /// RMS meter with smoothing timescale in seconds.
    /// Smoothing timescale is the time it takes for level estimation to move halfway to a new level.
    Rms(f64),
}

impl Meter {
    /// Whether the meter mode depends only on the latest sample.
    pub fn latest_only(&self) -> bool {
        matches!(self, Meter::Sample)
    }
}

#[derive(Clone)]
pub struct MeterState {
    /// Per-sample smoothing calculated from smoothing timescale.
    smoothing: f32,
    /// Current meter level.
    state: f32,
}

impl MeterState {
    /// Create a new MeterState for the given metering mode.
    pub fn new(meter: Meter) -> Self {
        let mut state = Self {
            smoothing: 0.0,
            state: 0.0,
        };
        state.set_sample_rate(meter, DEFAULT_SR);
        state
    }

    /// Reset meter state.
    pub fn reset(&mut self, _meter: Meter) {
        self.state = 0.0;
    }

    /// Set meter sample rate.
    pub fn set_sample_rate(&mut self, meter: Meter, sample_rate: f64) {
        let timescale = match meter {
            Meter::Sample => {
                return;
            }
            Meter::Peak(timescale) => timescale,
            Meter::Rms(timescale) => timescale,
        };
        self.smoothing = (pow(0.5f64, 1.0 / (timescale * sample_rate))).to_f32();
    }

    /// Process an input sample.
    #[inline]
    pub fn tick(&mut self, meter: Meter, value: f32) {
        match meter {
            Meter::Sample => self.state = value,
            Meter::Peak(_) => self.state = max(self.state * self.smoothing, abs(value)),
            Meter::Rms(_) => {
                self.state = self.state * self.smoothing + squared(value) * (1.0 - self.smoothing)
            }
        }
    }

    /// Current meter level.
    #[inline]
    pub fn level(&self, meter: Meter) -> f32 {
        match meter {
            Meter::Sample => self.state,
            Meter::Peak(_) => self.state,
            Meter::Rms(_) => sqrt(self.state),
        }
    }
}

/// Meters the input and outputs a summary according to the chosen metering mode.
/// - Input 0: input signal
/// - Output 0: input summary
#[derive(Clone)]
pub struct MeterNode {
    meter: Meter,
    state: MeterState,
}

impl MeterNode {
    /// Create a new metering node.
    pub fn new(meter: Meter) -> Self {
        Self {
            meter,
            state: MeterState::new(meter),
        }
    }
}

impl AudioNode for MeterNode {
    const ID: u64 = 61;
    type Inputs = U1;
    type Outputs = U1;

    fn reset(&mut self) {
        self.state.reset(self.meter);
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.state.set_sample_rate(self.meter, sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        self.state.tick(self.meter, input[0]);
        [convert(self.state.level(self.meter))].into()
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = SignalFrame::new(self.outputs());
        output.set(0, input.at(0).distort(0.0));
        output
    }
}

/// Pass through input unchanged.
/// Summary of the input signal is placed in a shared variable.
pub struct Monitor {
    meter: Meter,
    state: MeterState,
    shared: Arc<AtomicU32>,
}

impl Clone for Monitor {
    fn clone(&self) -> Self {
        Self {
            meter: self.meter,
            state: self.state.clone(),
            shared: Arc::clone(&self.shared),
        }
    }
}

impl Monitor {
    /// Create a new monitor node.
    pub fn new(shared: &Shared, meter: Meter) -> Self {
        Self {
            meter,
            state: MeterState::new(meter),
            shared: Arc::clone(shared.get_shared()),
        }
    }
}

impl AudioNode for Monitor {
    const ID: u64 = 56;
    type Inputs = U1;
    type Outputs = U1;

    fn reset(&mut self) {
        self.state.reset(self.meter);
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.state.set_sample_rate(self.meter, sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        self.state.tick(self.meter, input[0]);
        f32::store(&self.shared, self.state.level(self.meter));
        *input
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        if size == 0 {
            return;
        }
        if self.meter.latest_only() {
            self.state.tick(self.meter, input.at_f32(0, size - 1));
        } else {
            for i in 0..size {
                self.state.tick(self.meter, input.at_f32(0, i));
            }
        }
        // For efficiency, store the value only once per block.
        f32::store(&self.shared, self.state.level(self.meter));
        output.channel_mut(0)[..simd_items(size)]
            .clone_from_slice(&input.channel(0)[..simd_items(size)]);
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        input.clone()
    }
}
