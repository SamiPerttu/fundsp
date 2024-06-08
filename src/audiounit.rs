//! The dynamical `AudioUnit` abstraction and utilities.

use super::audionode::*;
use super::buffer::*;
use super::combinator::*;
use super::math::*;
use super::setting::*;
use super::signal::*;
use super::*;
use core::marker::PhantomData;
use dyn_clone::DynClone;
use num_complex::Complex64;
extern crate alloc;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;

/// An audio processor with an object safe interface.
/// Once constructed, it has a fixed number of inputs and outputs.
pub trait AudioUnit: Send + Sync + DynClone {
    /// Reset the input state of the unit to an initial state where it has not processed any data.
    /// In other words, reset time to zero.
    fn reset(&mut self);

    /// Set the sample rate of the unit.
    /// The default sample rate is 44100 Hz.
    /// The unit is allowed to reset itself here in response to sample rate changes.
    /// If the sample rate stays unchanged, then the goal is to maintain current state.
    fn set_sample_rate(&mut self, sample_rate: f64);

    /// Process one sample.
    /// The length of `input` and `output` must be equal to `inputs` and `outputs`, respectively.
    fn tick(&mut self, input: &[f32], output: &mut [f32]);

    /// Process up to 64 (MAX_BUFFER_SIZE) samples.
    /// If `size` is zero then this is a no-op, which is permitted.
    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut);

    /// Set a parameter. What formats are recognized depends on the component.
    #[allow(unused_variables)]
    fn set(&mut self, setting: Setting) {}

    /// Number of inputs to this unit.
    /// Equals size of the input argument in `tick` and `process`.
    /// This should be fixed after construction.
    fn inputs(&self) -> usize;

    /// Number of outputs from this unit.
    /// Equals size of the output argument in `tick` and `process`.
    /// This should be fixed after construction.
    fn outputs(&self) -> usize;

    /// Route constants, latencies and frequency responses at `frequency` Hz
    /// from inputs to outputs. Return output signal.
    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame;

    /// Return an ID code for this type of unit.
    fn get_id(&self) -> u64;

    /// Set unit pseudorandom phase hash. Override this to use the hash.
    /// This is called from `ping` (only). It should not be called by users.
    #[allow(unused_variables)]
    fn set_hash(&mut self, hash: u64) {
        // The default implementation does nothing.
    }

    /// Ping contained `AudioUnit`s and `AudioNode`s to obtain
    /// a deterministic pseudorandom hash. The local hash includes children, too.
    /// Leaf nodes should not need to override this.
    /// If `probe` is true, then this is a probe for computing the network hash
    /// and `set_hash` should not be called yet.
    /// To set a custom hash for a graph, call this method with `ping`
    /// set to false and `hash` initialized with the custom hash.
    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        if !probe {
            self.set_hash(hash.state());
        }
        hash.hash(self.get_id())
    }

    /// Memory footprint of this unit in bytes, without counting buffers and other allocations.
    fn footprint(&self) -> usize;

    /// Preallocate all needed memory, including buffers for block processing.
    fn allocate(&mut self) {
        // The default implementation does nothing.
    }

    // End of interface. There is no need to override the following.

    /// Retrieve the next mono sample from a generator.
    /// The node must have no inputs and 1 or 2 outputs.
    /// If there are two outputs, average them.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// assert_eq!(dc(2.0).get_mono(), 2.0);
    /// assert_eq!(dc((3.0, 4.0)).get_mono(), 3.5);
    /// ```
    #[inline]
    fn get_mono(&mut self) -> f32 {
        debug_assert!(self.inputs() == 0);
        match self.outputs() {
            1 => {
                let mut output = [0.0];
                self.tick(&[], &mut output);
                output[0]
            }
            2 => {
                let mut output = [0.0, 0.0];
                self.tick(&[], &mut output);
                (output[0] + output[1]) * 0.5
            }
            _ => panic!("AudioUnit::get_mono(): Unit must have 1 or 2 outputs"),
        }
    }

    /// Retrieve the next stereo sample (left, right) from a generator.
    /// The node must have no inputs and 1 or 2 outputs.
    /// If there is just one output, duplicate it.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// assert_eq!(dc((5.0, 6.0)).get_stereo(), (5.0, 6.0));
    /// assert_eq!(dc(7.0).get_stereo(), (7.0, 7.0));
    /// ```
    #[inline]
    fn get_stereo(&mut self) -> (f32, f32) {
        debug_assert!(self.inputs() == 0);
        match self.outputs() {
            1 => {
                let mut output = [0.0];
                self.tick(&[], &mut output);
                (output[0], output[0])
            }
            2 => {
                let mut output = [0.0, 0.0];
                self.tick(&[], &mut output);
                (output[0], output[1])
            }
            _ => panic!("AudioUnit::get_stereo(): Unit must have 1 or 2 outputs"),
        }
    }

    /// Filter the next mono sample `x`.
    /// The node must have exactly 1 input and 1 output.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// assert_eq!(add(4.0).filter_mono(5.0), 9.0);
    /// ```
    #[inline]
    fn filter_mono(&mut self, x: f32) -> f32 {
        debug_assert!(self.inputs() == 1 && self.outputs() == 1);
        let mut output = [0.0];
        self.tick(&[x], &mut output);
        output[0]
    }

    /// Filter the next stereo sample `(x, y)`.
    /// The node must have exactly 2 inputs and 2 outputs.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// assert_eq!(add((2.0, 3.0)).filter_stereo(4.0, 5.0), (6.0, 8.0));
    /// ```
    #[inline]
    fn filter_stereo(&mut self, x: f32, y: f32) -> (f32, f32) {
        debug_assert!(self.inputs() == 2 && self.outputs() == 2);
        let mut output = [0.0, 0.0];
        self.tick(&[x, y], &mut output);
        (output[0], output[1])
    }

    /// Evaluate frequency response of `output` at `frequency` Hz.
    /// Any linear response can be composed.
    /// Return `None` if there is no response or it could not be calculated.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// use num_complex::Complex64;
    /// assert_eq!(pass().response(0, 440.0), Some(Complex64::new(1.0, 0.0)));
    /// ```
    fn response(&mut self, output: usize, frequency: f64) -> Option<Complex64> {
        assert!(output < self.outputs());
        let mut input = SignalFrame::new(self.inputs());
        for i in 0..self.inputs() {
            input.set(i, Signal::Response(Complex64::new(1.0, 0.0), 0.0));
        }
        let response = self.route(&input, frequency);
        match response.at(output) {
            Signal::Response(rx, _) => Some(rx),
            _ => None,
        }
    }

    /// Evaluate frequency response of `output` in dB at `frequency` Hz.
    /// Any linear response can be composed.
    /// Return `None` if there is no response or it could not be calculated.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// let db = pass().response_db(0, 440.0).unwrap();
    /// assert!(db < 1.0e-7 && db > -1.0e-7);
    /// ```
    fn response_db(&mut self, output: usize, frequency: f64) -> Option<f64> {
        assert!(output < self.outputs());
        self.response(output, frequency).map(|r| amp_db(r.norm()))
    }

    /// Causal latency in (fractional) samples.
    /// After a reset, we can discard this many samples from the output to avoid incurring a pre-delay.
    /// The latency may depend on the sample rate.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// assert_eq!(pass().latency(), Some(0.0));
    /// assert_eq!(tick().latency(), Some(1.0));
    /// assert_eq!(sink().latency(), None);
    /// assert_eq!(lowpass_hz(440.0, 1.0).latency(), Some(0.0));
    /// ```
    fn latency(&mut self) -> Option<f64> {
        if self.outputs() == 0 {
            return None;
        }
        let mut input = SignalFrame::new(self.inputs());
        for i in 0..self.inputs() {
            input.set(i, Signal::Latency(0.0));
        }
        // The frequency argument can be anything as there are no responses to propagate,
        // only latencies. Latencies are never promoted to responses during signal routing.
        let response = self.route(&input, 1.0);
        // Return the minimum latency.
        let mut result: Option<f64> = None;
        for output in 0..self.outputs() {
            match (result, response.at(output)) {
                (None, Signal::Latency(x)) => result = Some(x),
                (Some(r), Signal::Latency(x)) => result = Some(r.min(x)),
                _ => (),
            }
        }
        result
    }

    /// Print information about this unit into a string.
    fn display(&mut self) -> String {
        let mut string = String::new();

        if self.inputs() > 0 && self.outputs() > 0 && self.response(0, 440.0).is_some() {
            let scope = [
                b"------------------------------------------------",
                b"                                                ",
                b"------------------------------------------------",
                b"                                                ",
                b"------------------------------------------------",
                b"                                                ",
                b"------------------------------------------------",
                b"                                                ",
                b"------------------------------------------------",
                b"                                                ",
                b"------------------------------------------------",
                b"                                                ",
                b"------------------------------------------------",
            ];

            let mut scope: Vec<_> = scope.iter().map(|x| x.to_vec()).collect();

            let f: [f64; 48] = [
                10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0, 120.0, 140.0, 160.0,
                180.0, 200.0, 250.0, 300.0, 350.0, 400.0, 450.0, 500.0, 600.0, 700.0, 800.0, 900.0,
                1000.0, 1200.0, 1400.0, 1600.0, 1800.0, 2000.0, 2500.0, 3000.0, 3500.0, 4000.0,
                4500.0, 5000.0, 6000.0, 7000.0, 8000.0, 9000.0, 10000.0, 12000.0, 14000.0, 16000.0,
                18000.0, 20000.0, 22000.0,
            ];

            let r: Vec<_> = f
                .iter()
                .map(|&f| (self.response_db(0, f).unwrap(), f))
                .collect();

            let epsilon_db = 1.0e-2;
            let max_r = r.iter().fold((-f64::INFINITY, None), {
                |acc, &x| {
                    if abs(acc.0 - x.0) <= epsilon_db {
                        (max(acc.0, x.0), None)
                    } else if acc.0 > x.0 {
                        acc
                    } else {
                        (x.0, Some(x.1))
                    }
                }
            });
            let max_db = ceil(max_r.0 / 10.0) * 10.0;

            for i in 0..f.len() {
                let row = (max_db - r[i].0) / 5.0;
                let mut j = ceil(row) as usize;
                let mut c = if row - floor(row) <= 0.5 { b'*' } else { b'.' };
                while j < scope.len() {
                    scope[j][i] = c;
                    j += 1;
                    c = b'*';
                }
            }

            for (row, ascii_line) in scope.into_iter().enumerate() {
                let line = String::from_utf8(ascii_line).unwrap();
                if row & 1 == 0 {
                    let db = round(max_db - row as f64 * 5.0) as i64;
                    writeln!(&mut string, "{:3} dB {} {:3} dB", db, line, db).unwrap();
                } else {
                    writeln!(&mut string, "       {}", line).unwrap();
                }
            }

            writeln!(
                &mut string,
                "       |   |    |    |     |    |    |     |    |    |"
            )
            .unwrap();
            writeln!(
                &mut string,
                "       10  50   100  200   500  1k   2k    5k   10k  20k Hz\n"
            )
            .unwrap();

            write!(&mut string, "Peak Magnitude : {:.2} dB", max_r.0).unwrap();

            match max_r.1 {
                Some(frequency) => {
                    writeln!(&mut string, " ({} Hz)", frequency as i64).unwrap();
                }
                _ => {
                    string.push('\n');
                }
            }
        }

        writeln!(&mut string, "Inputs         : {}", self.inputs()).unwrap();
        writeln!(&mut string, "Outputs        : {}", self.outputs()).unwrap();
        writeln!(
            &mut string,
            "Latency        : {:.1} samples",
            self.latency().unwrap_or(0.0)
        )
        .unwrap();
        writeln!(&mut string, "Footprint      : {} bytes", self.footprint()).unwrap();

        string
    }
}

dyn_clone::clone_trait_object!(AudioUnit);

impl<X: AudioNode + Sync + Send> AudioUnit for An<X>
where
    X::Inputs: Size<f32>,
    X::Outputs: Size<f32>,
{
    fn reset(&mut self) {
        self.0.reset();
    }
    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.0.set_sample_rate(sample_rate);
    }
    #[inline]
    fn tick(&mut self, input: &[f32], output: &mut [f32]) {
        debug_assert!(input.len() == self.inputs());
        debug_assert!(output.len() == self.outputs());
        output.copy_from_slice(self.0.tick(Frame::from_slice(input)).as_slice());
    }
    #[inline]
    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        self.0.process(size, input, output);
    }
    #[inline]
    fn set(&mut self, setting: Setting) {
        self.0.set(setting);
    }
    #[inline]
    fn inputs(&self) -> usize {
        self.0.inputs()
    }
    #[inline]
    fn outputs(&self) -> usize {
        self.0.outputs()
    }
    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.0.route(input, frequency)
    }
    #[inline]
    fn get_id(&self) -> u64 {
        X::ID
    }
    fn set_hash(&mut self, hash: u64) {
        self.0.set_hash(hash);
    }
    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.0.ping(probe, hash)
    }
    fn footprint(&self) -> usize {
        core::mem::size_of::<X>()
    }
    fn allocate(&mut self) {
        self.0.allocate();
    }
}

/// Converts an AudioUnit into an AudioNode with `I` inputs and `O` outputs.
#[derive(Clone)]
pub struct Unit<I: Size<f32>, O: Size<f32>> {
    _marker: PhantomData<(I, O)>,
    unit: Box<dyn AudioUnit>,
}

impl<I: Size<f32>, O: Size<f32>> Unit<I, O> {
    pub fn new(unit: Box<dyn AudioUnit>) -> Self {
        assert!(I::USIZE == unit.inputs());
        assert!(O::USIZE == unit.outputs());
        Self {
            _marker: PhantomData,
            unit,
        }
    }
}

impl<I: Size<f32>, O: Size<f32>> AudioNode for Unit<I, O> {
    const ID: u64 = 82;
    type Inputs = I;
    type Outputs = O;

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.unit.set_sample_rate(sample_rate);
    }

    fn reset(&mut self) {
        self.unit.reset();
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let mut output = Frame::default();
        self.unit.tick(input, &mut output);
        output
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        self.unit.process(size, input, output);
    }

    fn set(&mut self, setting: Setting) {
        self.unit.set(setting);
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.unit.ping(probe, hash)
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.unit.route(input, frequency)
    }

    fn allocate(&mut self) {
        self.unit.allocate();
    }
}

/// A big block adapter.
/// The adapter enables calls to `process_big` with arbitrary buffer sizes.
#[derive(Clone)]
pub struct BigBlockAdapter {
    source: Box<dyn AudioUnit>,
    input: BufferVec,
    output: BufferVec,
}

impl BigBlockAdapter {
    /// Create a new big block adapter.
    pub fn new(source: Box<dyn AudioUnit>) -> Self {
        let input = BufferVec::new(source.inputs());
        let output = BufferVec::new(source.outputs());
        Self {
            source,
            input,
            output,
        }
    }

    /// Process a block of any size.
    pub fn process_big(&mut self, size: usize, input: &[&[f32]], output: &mut [&mut [f32]]) {
        let mut i = 0;
        while i < size {
            let n = min(size - i, MAX_BUFFER_SIZE);
            for input_i in 0..self.input.channels() {
                for j in 0..n {
                    self.input.set_f32(input_i, j, input[input_i][i + j]);
                }
            }
            self.source
                .process(n, &self.input.buffer_ref(), &mut self.output.buffer_mut());
            for output_i in 0..self.output.channels() {
                for j in 0..n {
                    output[output_i][i + j] = self.output.at_f32(output_i, j);
                }
            }
            i += n;
        }
    }
}

impl AudioUnit for BigBlockAdapter {
    fn reset(&mut self) {
        self.source.reset();
    }
    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.source.set_sample_rate(sample_rate);
    }
    fn tick(&mut self, input: &[f32], output: &mut [f32]) {
        self.source.tick(input, output);
    }
    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        self.source.process(size, input, output);
    }
    fn set(&mut self, setting: Setting) {
        self.source.set(setting);
    }
    fn inputs(&self) -> usize {
        self.source.inputs()
    }
    fn outputs(&self) -> usize {
        self.source.outputs()
    }
    fn get_id(&self) -> u64 {
        self.source.get_id()
    }
    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.source.ping(probe, hash)
    }
    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.source.route(input, frequency)
    }
    fn footprint(&self) -> usize {
        self.source.footprint()
    }
    fn allocate(&mut self) {
        self.source.allocate();
    }
}

/// Block rate adapter converts all processing calls to maximum length block processing.
/// Maximizes performance at the expense of latency.
/// The unit to be adapted must have no inputs.
#[derive(Clone)]
pub struct BlockRateAdapter {
    unit: Box<dyn AudioUnit>,
    channels: usize,
    buffer: BufferVec,
    index: usize,
}

impl BlockRateAdapter {
    /// Create new block rate adapter for the unit.
    pub fn new(unit: Box<dyn AudioUnit>) -> Self {
        assert_eq!(unit.inputs(), 0);
        let channels = unit.outputs();
        Self {
            unit,
            channels,
            buffer: BufferVec::new(channels),
            index: MAX_BUFFER_SIZE,
        }
    }
}

impl AudioUnit for BlockRateAdapter {
    fn reset(&mut self) {
        self.unit.reset();
        self.index = MAX_BUFFER_SIZE;
    }
    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.unit.set_sample_rate(sample_rate);
    }
    fn tick(&mut self, _input: &[f32], output: &mut [f32]) {
        if self.index == MAX_BUFFER_SIZE {
            self.unit.process(
                MAX_BUFFER_SIZE,
                &BufferRef::empty(),
                &mut self.buffer.buffer_mut(),
            );
            self.index = 0;
        }
        for channel in 0..self.channels {
            output[channel] = self.buffer.at_f32(channel, self.index);
        }
        self.index += 1;
    }
    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        let mut i = 0;
        while i < size {
            if self.index == MAX_BUFFER_SIZE {
                self.unit
                    .process(MAX_BUFFER_SIZE, input, &mut self.buffer.buffer_mut());
                self.index = 0;
            }
            let n = min(size - i, MAX_BUFFER_SIZE - self.index);
            for channel in 0..self.channels {
                output.channel_f32_mut(channel)[i..i + n].clone_from_slice(
                    &self.buffer.channel_f32(channel)[self.index..self.index + n],
                );
            }
            i += n;
            self.index += n;
        }
    }
    fn set(&mut self, setting: Setting) {
        self.unit.set(setting);
    }
    fn inputs(&self) -> usize {
        0
    }
    fn outputs(&self) -> usize {
        self.channels
    }
    fn get_id(&self) -> u64 {
        self.unit.get_id()
    }
    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.unit.ping(probe, hash)
    }
    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.unit.route(input, frequency)
    }
    fn footprint(&self) -> usize {
        self.unit.footprint()
    }
    fn allocate(&mut self) {
        self.unit.allocate();
    }
}
