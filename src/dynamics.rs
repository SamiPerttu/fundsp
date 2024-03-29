//! Audio dynamics related components.

use super::audionode::*;
use super::combinator::*;
use super::follow::*;
use super::math::*;
use super::shared::*;
use super::signal::*;
use super::*;
use numeric_array::typenum::*;
use std::sync::Arc;

/// Binary operation for the monoidal reducer.
pub trait Monoidal<T>: Clone {
    fn binop(&self, x: T, y: T) -> T;
}

#[derive(Default, Clone)]
pub struct Amplitude<T: Num> {
    _marker: std::marker::PhantomData<T>,
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
    _marker: std::marker::PhantomData<T>,
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
            buffer: vec![],
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
pub struct Limiter<T, N, S>
where
    T: Real,
    N: Size<T>,
    S: ScalarOrPair<Sample = T>,
{
    lookahead: f64,
    #[allow(dead_code)]
    release: f64,
    sample_rate: f64,
    reducer: ReduceBuffer<T, Maximum<T>>,
    follower: AFollow<T, T, S>,
    buffer: Vec<Frame<T, N>>,
    index: usize,
}

impl<T, N, S> Limiter<T, N, S>
where
    T: Real,
    N: Size<T>,
    S: ScalarOrPair<Sample = T>,
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

    fn new_buffer(sample_rate: f64, lookahead: f64) -> ReduceBuffer<T, Maximum<T>> {
        ReduceBuffer::new(Self::buffer_length(sample_rate, lookahead), Maximum::new())
    }

    pub fn new(sample_rate: f64, time: S) -> Self {
        let (lookahead, release) = time.broadcast();
        Limiter {
            lookahead: lookahead.to_f64(),
            release: release.to_f64(),
            sample_rate,
            follower: AFollow::new(
                sample_rate,
                S::construct(
                    lookahead * convert::<f64, T>(0.4),
                    release * convert::<f64, T>(0.4),
                ),
            ),
            buffer: vec![],
            reducer: Self::new_buffer(sample_rate, lookahead.to_f64()),
            index: 0,
        }
    }
}

impl<T, N, S> AudioNode for Limiter<T, N, S>
where
    T: Real,
    N: Size<T>,
    S: ScalarOrPair<Sample = T>,
{
    const ID: u64 = 25;
    type Sample = T;
    type Inputs = N;
    type Outputs = N;
    type Setting = ();

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
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let amplitude = input.iter().fold(T::zero(), |amp, &x| max(amp, abs(x)));
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
                .filter_mono(max(T::one(), self.reducer.total() * T::from_f64(1.10)));
            self.advance();
            let limit = self.follower.value();
            output * Frame::splat(T::from_f64(1.0) / limit)
        }
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        for i in 0..N::USIZE {
            // We pretend that the limiter does not alter the frequency response.
            output[i] = input[i].delay(self.reducer.length() as f64);
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
pub struct Declick<T: Float, F: Real> {
    _marker: std::marker::PhantomData<T>,
    t: F,
    duration: F,
    sample_duration: F,
    sample_rate: f64,
}

impl<T: Float, F: Real> Declick<T, F> {
    pub fn new(sample_rate: f64, duration: F) -> Self {
        let mut node = Declick::<T, F> {
            duration,
            ..Default::default()
        };
        node.set_sample_rate(sample_rate);
        node
    }
}

impl<T: Float, F: Real> AudioNode for Declick<T, F> {
    const ID: u64 = 23;
    type Sample = T;
    type Inputs = U1;
    type Outputs = U1;
    type Setting = ();

    fn reset(&mut self) {
        self.t = F::zero();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
        self.sample_duration = F::from_f64(1.0 / sample_rate);
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        if self.t < self.duration {
            let phase = delerp(F::zero(), self.duration, self.t);
            let value = smooth5(phase);
            self.t += self.sample_duration;
            [input[0] * convert(value)].into()
        } else {
            [input[0]].into()
        }
    }

    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        output[0][..size].clone_from_slice(&input[0][..size]);
        if self.t < self.duration {
            let mut phase = delerp(F::zero(), self.duration, self.t);
            let phase_d = self.sample_duration / self.duration;
            let end_time = self.t + F::new(size as i64) * self.sample_duration;
            let end_index = if self.duration < end_time {
                ceil((self.duration - self.t) / self.sample_duration).to_i64() as usize
            } else {
                size
            };
            for x in output[0][0..end_index].iter_mut() {
                *x *= convert(smooth5(phase));
                phase += phase_d;
            }
            self.t = end_time;
        }
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        // We pretend that the declicker does not alter the frequency response.
        output[0] = input[0];
        output
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
pub struct MeterState<T: Real> {
    /// Per-sample smoothing calculated from smoothing timescale.
    smoothing: T,
    /// Current meter level.
    state: T,
}

impl<T: Real> MeterState<T> {
    /// Create a new MeterState for the given metering mode.
    pub fn new(meter: Meter) -> Self {
        let mut state = Self {
            smoothing: T::zero(),
            state: T::zero(),
        };
        state.set_sample_rate(meter, DEFAULT_SR);
        state
    }

    /// Reset meter state.
    pub fn reset(&mut self, _meter: Meter) {
        self.state = T::zero();
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
        self.smoothing = T::from_f64(pow(0.5, 1.0 / (timescale * sample_rate)));
    }

    /// Process an input sample.
    #[inline]
    pub fn tick(&mut self, meter: Meter, value: T) {
        match meter {
            Meter::Sample => self.state = value,
            Meter::Peak(_) => self.state = max(self.state * self.smoothing, abs(value)),
            Meter::Rms(_) => {
                self.state =
                    self.state * self.smoothing + squared(value) * (T::one() - self.smoothing)
            }
        }
    }

    /// Current meter level.
    #[inline]
    pub fn level(&self, meter: Meter) -> T {
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
pub struct MeterNode<T: Real> {
    meter: Meter,
    state: MeterState<T>,
}

impl<T: Real> MeterNode<T> {
    /// Create a new metering node.
    pub fn new(meter: Meter) -> Self {
        Self {
            meter,
            state: MeterState::new(meter),
        }
    }
}

impl<T: Real> AudioNode for MeterNode<T> {
    const ID: u64 = 61;
    type Sample = T;
    type Inputs = U1;
    type Outputs = U1;
    type Setting = ();

    fn reset(&mut self) {
        self.state.reset(self.meter);
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.state.set_sample_rate(self.meter, sample_rate);
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        self.state.tick(self.meter, input[0]);
        [convert(self.state.level(self.meter))].into()
    }

    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        for i in 0..size {
            self.state.tick(self.meter, input[0][i]);
            output[0][i] = convert(self.state.level(self.meter));
        }
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].distort(0.0);
        output
    }
}

/// Pass through input unchanged.
/// Summary of the input signal is placed in a shared variable.
pub struct Monitor<T: Real + Atomic> {
    meter: Meter,
    state: MeterState<T>,
    shared: Arc<T::Storage>,
}

impl<T: Real + Atomic> Clone for Monitor<T> {
    fn clone(&self) -> Self {
        Self {
            meter: self.meter,
            state: self.state.clone(),
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<T: Real + Atomic> Monitor<T> {
    /// Create a new monitor node.
    pub fn new(shared: &Shared<T>, meter: Meter) -> Self {
        Self {
            meter,
            state: MeterState::new(meter),
            shared: Arc::clone(shared.get_shared()),
        }
    }
}

impl<T: Real + Atomic> AudioNode for Monitor<T> {
    const ID: u64 = 56;
    type Sample = T;
    type Inputs = U1;
    type Outputs = U1;
    type Setting = ();

    fn reset(&mut self) {
        self.state.reset(self.meter);
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.state.set_sample_rate(self.meter, sample_rate);
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        self.state.tick(self.meter, input[0]);
        T::store(&self.shared, self.state.level(self.meter));
        *input
    }

    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        if size == 0 {
            return;
        }
        if self.meter.latest_only() {
            self.state.tick(self.meter, input[0][size - 1]);
        } else {
            for i in 0..size {
                self.state.tick(self.meter, input[0][i]);
            }
        }
        // For efficiency, store the value only once per block.
        T::store(&self.shared, self.state.level(self.meter));
        output[0][..size].clone_from_slice(&input[0][..size]);
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        input.clone()
    }
}
