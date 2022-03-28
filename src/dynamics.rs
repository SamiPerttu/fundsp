//! Audio dynamics related components.

use super::audionode::*;
use super::combinator::*;
use super::filter::*;
use super::math::*;
use super::signal::*;
use super::*;
use numeric_array::typenum::*;

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
    fn get_index(&self, i: usize) -> usize {
        self.leaf_offset + i
    }

    // Assumption: 0 is the zero element.
    pub fn new(length: usize, binop: B) -> Self {
        let leaf_offset = length.next_power_of_two();
        ReduceBuffer {
            buffer: vec![T::zero(); leaf_offset + length + (length & 1)],
            length,
            leaf_offset,
            binop,
        }
    }

    pub fn length(&self) -> usize {
        self.length
    }

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

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.index = 0;
        if let Some(sample_rate) = sample_rate {
            self.sample_rate = sample_rate;
            let length = Self::buffer_length(sample_rate, self.lookahead);
            if length != self.reducer.length {
                self.reducer = Self::new_buffer(sample_rate, self.lookahead);
                return;
            }
        }
        self.follower.reset(sample_rate);
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
                // TODO: follow the log value instead.
                self.follower.set_value(self.reducer.total());
            }
            self.advance();
            Frame::default()
        } else {
            let output = self.buffer[self.index].clone();
            self.buffer[self.index] = input.clone();
            // Leave some headroom.
            self.follower
                .filter_mono(max(T::one(), self.reducer.total() * T::from_f64(1.15)));
            self.advance();
            let limit = self.follower.value();
            output * Frame::splat(T::from_f64(1.0) / limit)
        }
    }

    fn route(&self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        for i in 0..N::USIZE {
            // We pretend that the limiter does not alter the frequency response.
            output[i] = input[i].delay(self.reducer.length() as f64);
        }
        output
    }
}

/// Goertzel frequency detector.
#[derive(Clone, Default)]
pub struct Detector<T: Real> {
    y1: T,
    y2: T,
    ccoeff: T,
    scoeff: T,
}
impl<T: Real> Detector<T> {
    /// Reset detector.
    pub fn reset(&mut self) {
        self.y1 = T::zero();
        self.y2 = T::zero();
    }
    /// Select the frequency (in Hz).
    pub fn set_frequency(&mut self, sample_rate: T, frequency: T) {
        let f = T::from_f64(TAU) * frequency / sample_rate;
        self.ccoeff = T::new(2) * cos(f);
        self.scoeff = sin(f);
    }
    /// Process one sample.
    pub fn tick(&mut self, x: T) {
        let y0 = x + self.ccoeff * self.y1 - self.y2;
        self.y2 = self.y1;
        self.y1 = y0;
    }
    /// Current detected power at the selected frequency.
    pub fn power(&self) -> T {
        squared(self.y2) + squared(self.y1) - self.ccoeff * self.y2 * self.y1
    }
}

/// Goertzel filter. Detects the presence of a frequency.
/// Outputs DFT power at the selected frequency.
#[derive(Default)]
pub struct Goertzel<T: Float, F: Real> {
    filter: Detector<F>,
    sample_rate: F,
    frequency: F,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Float, F: Real> Goertzel<T, F> {
    pub fn new(sample_rate: f64) -> Self {
        let mut node = Goertzel::default();
        node.reset(Some(sample_rate));
        node
    }

    /// Current detected power at the selected frequency.
    pub fn power(&self) -> F {
        self.filter.power()
    }
}

impl<T: Float, F: Real> AudioNode for Goertzel<T, F> {
    const ID: u64 = 27;
    type Sample = T;
    type Inputs = U2;
    type Outputs = U1;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.filter.reset();
        if let Some(sample_rate) = sample_rate {
            self.sample_rate = F::from_f64(sample_rate);
            self.frequency = F::zero();
        }
    }

    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        let f: F = convert(input[1]);
        if f != self.frequency {
            self.frequency = f;
            self.filter.set_frequency(self.sample_rate, f);
        }
        self.filter.tick(convert(input[0]));
        [convert(self.filter.power())].into()
    }

    fn route(&self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].combine_nonlinear(input[1], 0.0);
        output
    }
}

/// Transient filter. Multiply the signal with a fade-in curve.
/// After fade-in, pass signal through.
/// - Input 0: input signal
/// - Output 0: filtered signal
#[derive(Default)]
pub struct Declick<T: Float, F: Real> {
    _marker: std::marker::PhantomData<T>,
    t: F,
    duration: F,
    sample_duration: F,
}

impl<T: Float, F: Real> Declick<T, F> {
    pub fn new(sample_rate: f64, duration: F) -> Self {
        let mut node = Declick::<T, F> {
            duration,
            ..Default::default()
        };
        node.reset(Some(sample_rate));
        node
    }
}

impl<T: Float, F: Real> AudioNode for Declick<T, F> {
    const ID: u64 = 23;
    type Sample = T;
    type Inputs = U1;
    type Outputs = U1;

    fn reset(&mut self, sample_rate: Option<f64>) {
        if let Some(sample_rate) = sample_rate {
            self.sample_duration = F::from_f64(1.0 / sample_rate);
        }
        self.t = F::zero();
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
            for i in 0..end_index {
                output[0][i] *= convert(smooth5(phase));
                phase += phase_d;
            }
            self.t = end_time;
        }
    }

    fn route(&self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        // We pretend that the declicker does not alter frequency response.
        output[0] = input[0];
        output
    }
}

/// Metering modes.
#[derive(Copy, Clone)]
pub enum Meter {
    /// Latest value.
    Sample,
    /// Peak meter with per-sample smoothing in 0...1.
    Peak(f64),
    /// RMS meter with per-sample smoothing in 0...1.
    Rms(f64),
    /// Frequency detector with frequency in Hz.
    Detect(f64),
}

impl Meter {
    /// Whether the meter mode depends only on the latest sample.
    pub fn latest_only(&self) -> bool {
        matches!(self, Meter::Sample)
    }
}

pub struct MeterState<T: Real> {
    detector: Detector<T>,
    state: T,
}

impl<T: Real> MeterState<T> {
    /// Creates a new MeterState for the given metering mode.
    pub fn new(meter: Meter, sample_rate: f64) -> Self {
        let mut state = Self {
            detector: Detector::default(),
            state: T::zero(),
        };
        if let Meter::Detect(frequency) = meter {
            state
                .detector
                .set_frequency(T::from_f64(sample_rate), T::from_f64(frequency))
        }
        state
    }

    /// Resets meter state.
    pub fn reset(&mut self, meter: Meter, sample_rate: Option<f64>) {
        self.state = T::zero();
        if let Some(sr) = sample_rate {
            if let Meter::Detect(frequency) = meter {
                self.detector.set_frequency(convert(sr), convert(frequency));
            }
        }
    }

    /// Process an input sample.
    pub fn tick(&mut self, meter: Meter, value: T) {
        match meter {
            Meter::Sample => self.state = value,
            Meter::Peak(smoothing) => self.state = max(self.state * convert(smoothing), abs(value)),
            Meter::Rms(smoothing) => {
                self.state =
                    self.state * convert(smoothing) + value * value * convert(1.0 - smoothing)
            }
            Meter::Detect(_frequency) => self.detector.tick(value),
        }
    }

    /// Current meter value.
    pub fn value(&self, meter: Meter) -> T {
        match meter {
            Meter::Sample => self.state,
            Meter::Peak(_) => self.state,
            Meter::Rms(_) => sqrt(self.state),
            Meter::Detect(_) => self.detector.power(),
        }
    }
}

/// Meters the input and outputs a summary according to the chosen metering mode.
pub struct MeterNode<T: Real> {
    meter: Meter,
    state: MeterState<T>,
    sample_rate: f64,
}

impl<T: Real> MeterNode<T> {
    /// Create a new monitor node.
    pub fn new(sample_rate: f64, meter: Meter) -> Self {
        Self {
            meter,
            state: MeterState::new(meter, sample_rate),
            sample_rate,
        }
    }
}

impl<T: Real> AudioNode for MeterNode<T> {
    const ID: u64 = 61;
    type Sample = T;
    type Inputs = U1;
    type Outputs = U1;

    fn reset(&mut self, sample_rate: Option<f64>) {
        if let Some(sr) = sample_rate {
            self.sample_rate = sr;
        }
        self.state.reset(self.meter, sample_rate);
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        self.state.tick(self.meter, input[0]);
        [convert(self.state.value(self.meter))].into()
    }

    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        for i in 0..size {
            self.state.tick(self.meter, input[0][i]);
            output[0][i] = convert(self.state.value(self.meter));
        }
    }

    fn route(&self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].distort(0.0);
        output
    }
}

/// Pass through input unchanged.
/// Latest input value can be queried as a read-only parameter.
pub struct Monitor<T: Real> {
    tag: Tag,
    meter: Meter,
    state: MeterState<T>,
    sample_rate: f64,
}

impl<T: Real> Monitor<T> {
    /// Create a new monitor node.
    pub fn new(tag: Tag, sample_rate: f64, meter: Meter) -> Self {
        Self {
            tag,
            meter,
            state: MeterState::new(meter, sample_rate),
            sample_rate,
        }
    }
}

impl<T: Real> AudioNode for Monitor<T> {
    const ID: u64 = 56;
    type Sample = T;
    type Inputs = U1;
    type Outputs = U1;

    fn reset(&mut self, sample_rate: Option<f64>) {
        if let Some(sr) = sample_rate {
            self.sample_rate = sr;
        }
        self.state.reset(self.meter, sample_rate);
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        self.state.tick(self.meter, input[0]);
        *input
    }

    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        if self.meter.latest_only() {
            self.state.tick(self.meter, input[0][size - 1]);
        } else {
            for i in 0..size {
                self.state.tick(self.meter, input[0][i]);
            }
        }
        output[0][..size].clone_from_slice(&input[0][..size]);
    }

    fn route(&self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        input.clone()
    }

    fn get(&self, parameter: Tag) -> Option<f64> {
        if self.tag == parameter {
            Some(self.state.value(self.meter).to_f64())
        } else {
            None
        }
    }
}
