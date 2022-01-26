use super::audionode::*;
use super::combinator::*;
use super::filter::*;
use super::math::*;
use super::signal::*;
use super::*;
use numeric_array::typenum::*;

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
#[derive(Clone)]
pub struct Limiter<T, N, S>
where
    T: Float,
    N: Size<T>,
    S: ScalarOrPair<Sample = f64>,
{
    lookahead: f64,
    #[allow(dead_code)]
    release: f64,
    sample_rate: f64,
    reducer: ReduceBuffer<T, Maximum<T>>,
    follower: AFollow<T, f64, S>,
    buffer: Vec<Frame<T, N>>,
    index: usize,
}

impl<T, N, S> Limiter<T, N, S>
where
    T: Float,
    N: Size<T>,
    S: ScalarOrPair<Sample = f64>,
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
            lookahead,
            release,
            sample_rate,
            follower: AFollow::new(sample_rate, S::construct(lookahead * 0.4, release * 0.4)),
            buffer: vec![],
            reducer: Self::new_buffer(sample_rate, lookahead),
            index: 0,
        }
    }
}

impl<T, N, S> AudioNode for Limiter<T, N, S>
where
    T: Float,
    N: Size<T>,
    S: ScalarOrPair<Sample = f64>,
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
        // Leave some headroom.
        self.follower
            .filter(self.reducer.total() * T::from_f64(1.15));
        if self.buffer.len() < self.reducer.length() {
            // We are filling up the initial buffer.
            self.buffer.push(input.clone());
            if self.buffer.len() == self.reducer.length() {
                // When the buffer is full, start following from its total peak.
                // TODO: follow the log value instead.
                self.follower.set_value(self.reducer.total().to_f64());
            }
            self.advance();
            Frame::default()
        } else {
            let output = self.buffer[self.index].clone();
            self.buffer[self.index] = input.clone();
            self.advance();
            let limit = max(1.0, self.follower.value());
            output * Frame::splat(T::from_f64(1.0 / limit))
        }
    }

    fn propagate(&self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        for i in 0..N::USIZE {
            output[i] = input[i].distort(self.reducer.length() as f64);
        }
        output
    }
}

#[derive(Clone, Default)]
pub struct Detector<F: Real> {
    y1: F,
    y2: F,
    ccoeff: F,
    scoeff: F,
}
impl<F: Real> Detector<F> {
    pub fn reset(&mut self) {
        self.y1 = F::zero();
        self.y2 = F::zero();
    }
    pub fn set_frequency(&mut self, sample_rate: F, frequency: F) {
        let f = F::from_f64(TAU) * frequency / sample_rate;
        self.ccoeff = F::new(2) * cos(f);
        self.scoeff = sin(f);
    }
    pub fn tick(&mut self, x: F) {
        let y0 = x + self.ccoeff * self.y1 - self.y2;
        self.y2 = self.y1;
        self.y1 = y0;
    }
    pub fn power(&self) -> F {
        squared(self.y2) + squared(self.y1) - self.ccoeff * self.y2 * self.y1
    }
}

/// Goertzel filter. Detects the presence of a frequency. Outputs DFT power at the selected frequency.
#[derive(Clone, Default)]
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
            // TODO: Remain in a valid state?
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

    fn propagate(&self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = combine_nonlinear(input[0], input[1], 0.0);
        output
    }
}

/// Transient filter. Multiply the signal with a fade-in curve.
/// After fade-in, pass signal through.
/// - Input 0: input signal
/// - Output 0: filtered signal
#[derive(Copy, Clone, Default)]
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

    fn propagate(&self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].distort(0.0);
        output
    }
}
