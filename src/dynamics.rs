use super::audionode::*;
use super::filter::*;
use super::math::*;
use super::*;

pub trait Binop<T>: Clone {
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

impl<T: Num> Binop<T> for Amplitude<T> {
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

impl<T: Num> Binop<T> for Maximum<T> {
    fn binop(&self, x: T, y: T) -> T {
        max(x, y)
    }
}

/// Hierarchic reducer for a monoid.
#[derive(Clone)]
pub struct ReduceBuffer<T, B>
where
    T: Num,
    B: Binop<T>,
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
    B: Binop<T>,
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
pub struct Limiter<T, N>
where
    T: Float,
    N: Size<T>,
{
    lookahead: f64,
    sample_rate: f64,
    reducer: ReduceBuffer<T, Maximum<T>>,
    follower: Follower<T, f64>,
    buffer: Vec<Frame<T, N>>,
    index: usize,
}

impl<T, N> Limiter<T, N>
where
    T: Float,
    N: Size<T>,
{
    fn advance(&mut self) {
        self.index += 1;
        if self.index >= self.reducer.length() {
            self.index = 0;
        }
    }

    fn buffer_length(sample_rate: f64, lookahead: f64) -> usize {
        ceil(sample_rate * lookahead) as usize
    }

    fn new_buffer(sample_rate: f64, lookahead: f64) -> ReduceBuffer<T, Maximum<T>> {
        ReduceBuffer::new(Self::buffer_length(sample_rate, lookahead), Maximum::new())
    }

    pub fn new(sample_rate: f64, lookahead: f64) -> Self {
        Limiter {
            lookahead,
            sample_rate,
            follower: Follower::new(sample_rate, lookahead * 0.4),
            buffer: vec![],
            reducer: Self::new_buffer(sample_rate, lookahead),
            index: 0,
        }
    }
}

impl<T, N> AudioNode for Limiter<T, N>
where
    T: Float,
    N: Size<T>,
{
    const ID: u64 = 24;
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
            .filter_mono(self.reducer.total() * T::from_f64(1.15));
        if self.buffer.len() < self.reducer.length() {
            // We are filling up the initial buffer.
            self.buffer.push(input.clone());
            if self.buffer.len() == self.reducer.length() {
                // When the buffer is full, start following from its total peak.
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

    fn latency(&self) -> Option<f64> {
        Some(self.reducer.length() as f64 / self.sample_rate)
    }
}
