//! Bandlimited wavetable synthesizer.

use super::audionode::*;
use super::buffer::*;
use super::combinator::*;
use super::math::*;
use super::prelude::pass;
use super::signal::*;
use super::typenum::*;
use super::*;
use num_complex::Complex32;
extern crate alloc;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use once_cell::race::OnceBox;

/// Interpolate between `a1` and `a2` taking previous (`a0`) and next (`a3`) points into account.
/// Employs an optimal 4-point, 4th order interpolating polynomial for 4x oversampled signals.
/// The SNR of the interpolator for pink noise is 101.1 dB.
#[inline]
fn optimal4x44<T: Num>(a0: T, a1: T, a2: T, a3: T, x: T) -> T {
    // Interpolator sourced from:
    // Niemitalo, Olli, Polynomial Interpolators for High-Quality Resampling of Oversampled Audio, 2001.
    let z = x - T::from_f64(0.5);
    let even1 = a2 + a1;
    let odd1 = a2 - a1;
    let even2 = a3 + a0;
    let odd2 = a3 - a0;
    let c0 = even1 * T::from_f64(0.4656725512077848) + even2 * T::from_f64(0.03432729708429672);
    let c1 = odd1 * T::from_f64(0.5374383075356016) + odd2 * T::from_f64(0.1542946255730746);
    let c2 = even1 * T::from_f64(-0.25194210134021744) + even2 * T::from_f64(0.2519474493593906);
    let c3 = odd1 * T::from_f64(-0.46896069955075126) + odd2 * T::from_f64(0.15578800670302476);
    let c4 = even1 * T::from_f64(0.00986988334359864) + even2 * T::from_f64(-0.00989340017126506);
    (((c4 * z + c3) * z + c2) * z + c1) * z + c0
}

/// Create a single cycle wave into a power-of-two table, unnormalized.
/// Assume sample rate is at least 44.1 kHz.
/// `phase(i)` is phase in 0...1 for partial `i` (1, 2, ...).
/// `amplitude(p, i)` is amplitude for fundamental `p` Hz partial `i` (with frequency `p * i`).
pub fn make_wave<P, A>(pitch: f64, phase: &P, amplitude: &A) -> Vec<f32>
where
    P: Fn(u32) -> f64,
    A: Fn(f64, u32) -> f64,
{
    // Fade out upper harmonics starting from 20 kHz.
    const MAX_F: f64 = 22_000.0;
    const FADE_F: f64 = 20_000.0;

    let harmonics = floor(MAX_F / pitch) as usize;

    // Target at least 4x oversampling when choosing wave table length.
    let target_len = 4 * harmonics;

    let length = clamp(32, 8192, target_len.next_power_of_two());

    let mut a = vec![Complex32::new(0.0, 0.0); length];

    for i in 1..=harmonics {
        let f = pitch * i as f64;

        // Get harmonic amplitude.
        let w = amplitude(pitch, i as u32);
        // Fade out high frequencies.
        let w = w * smooth5(clamp01(delerp(MAX_F, FADE_F, f)));
        // Insert partial.
        if w > 0.0 {
            a[i] = Complex32::from_polar(w as f32, (f64::TAU * phase(i as u32)) as f32);
        }
    }

    let mut b = vec![Complex32::new(0.0, 0.0); length];
    super::fft::inverse_fft(&a, &mut b);

    let z = sqrt(length as f32);

    b.iter().map(|x| x.im * z).collect()
}

#[derive(Clone)]
pub struct Wavetable {
    /// Frequency tables arranged in order of increasing frequency.
    table: Vec<(f32, Vec<f32>)>,
}

impl Wavetable {
    /// Create new wavetable. `min_pitch` and `max_pitch` are the minimum
    /// and maximum base frequencies in Hz (for example, 20.0 and 20_000.0).
    /// `tables_per_octave` is the number of wavetables per octave
    /// (for example, 4.0). `phase(i)` is the phase of the `i`th partial.
    /// `amplitude(p, i)` is the amplitude of the `i`th partial with base frequency `p`.
    pub fn new<P, A>(
        min_pitch: f64,
        max_pitch: f64,
        tables_per_octave: f64,
        phase: &P,
        amplitude: &A,
    ) -> Wavetable
    where
        P: Fn(u32) -> f64,
        A: Fn(f64, u32) -> f64,
    {
        let mut table: Vec<(f32, Vec<f32>)> = vec![];
        let mut pitch = min_pitch;
        let p_factor = pow(2.0, 1.0 / tables_per_octave);
        let mut max_amplitude = 0.0;
        while pitch <= max_pitch {
            let wave = make_wave(pitch, phase, amplitude);
            max_amplitude = wave.iter().fold(max_amplitude, |acc, &x| max(acc, abs(x)));
            table.push((pitch as f32, wave));
            pitch *= p_factor;
        }
        if max_amplitude > 0.0 {
            let z = 1.0 / max_amplitude;
            table.iter_mut().for_each(|t| {
                t.1.iter_mut().for_each(|x| {
                    *x *= z;
                })
            });
        }
        Wavetable { table }
    }

    /// Create new wavetable from a single cycle wave. `min_pitch` and `max_pitch` are the minimum
    /// and maximum base frequencies in Hz (for example, 20.0 and 20_000.0).
    /// `tables_per_octave` is the number of wavetables per octave
    /// (for example, 4.0). The overall scale of numbers in `wave` is ignored;
    /// the wavetable is normalized to -1...1.
    pub fn from_wave(min_pitch: f64, max_pitch: f64, tables_per_octave: f64, wave: &[f32]) -> Self {
        let mut spectrum = vec![Complex32::new(0.0, 0.0); wave.len() / 2 + 1];
        super::fft::real_fft(wave, &mut spectrum);
        let phase = |i: u32| {
            if (i as usize) < spectrum.len() {
                spectrum[i as usize].arg() as f64 / f64::TAU
            } else {
                0.0
            }
        };
        let amplitude = |_p: f64, i: u32| {
            if (i as usize) < spectrum.len() {
                spectrum[i as usize].norm() as f64
            } else {
                0.0
            }
        };
        Wavetable::new(min_pitch, max_pitch, tables_per_octave, &phase, &amplitude)
    }

    /// Read wave from transposition table `i` at the given `phase` (in 0...1).
    #[inline]
    pub fn at(&self, i: usize, phase: f32) -> f32 {
        let table: &Vec<f32> = &self.table[i].1;
        let p = table.len() as f32 * phase;
        // Safety: we know phase is in 0...1.
        let i1 = unsafe { f32::to_int_unchecked::<usize>(p) };
        let w = p - i1 as f32;
        let mask = table.len() - 1;
        let i0 = i1.wrapping_sub(1) & mask;
        let i1 = i1 & mask;
        let i2 = (i1 + 1) & mask;
        let i3 = (i1 + 2) & mask;
        optimal4x44(table[i0], table[i1], table[i2], table[i3], w)
    }

    /// Read wave in parallel from transposition table `i` at the given `phase` (in 0...1).
    #[inline]
    pub fn at_simd(&self, i: usize, phase: F32x) -> F32x {
        let table: &Vec<f32> = &self.table[i].1;
        let p = F32x::splat(table.len() as f32) * phase;
        let i1 = (p - 0.5).fast_round_int();
        let w = p - F32x::new(core::array::from_fn(|j| i1.as_array_ref()[j] as f32));
        let mask = I32x::splat(table.len() as i32 - 1);
        let i0 = (i1 - 1) & mask;
        let i1 = i1 & mask;
        let i2 = (i1 + 1) & mask;
        let i3 = (i2 + 1) & mask;
        let t0 = F32x::new(core::array::from_fn(|j| {
            table[i0.as_array_ref()[j] as usize]
        }));
        let t1 = F32x::new(core::array::from_fn(|j| {
            table[i1.as_array_ref()[j] as usize]
        }));
        let t2 = F32x::new(core::array::from_fn(|j| {
            table[i2.as_array_ref()[j] as usize]
        }));
        let t3 = F32x::new(core::array::from_fn(|j| {
            table[i3.as_array_ref()[j] as usize]
        }));
        optimal4x44(t0, t1, t2, t3, w)
    }

    /// Get transposition table index, given `frequency`.
    #[inline]
    pub fn table_index(&self, table_hint: usize, frequency: f32) -> usize {
        if frequency >= self.table[table_hint].0 && frequency <= self.table[table_hint + 1].0 {
            table_hint
        } else {
            let mut i0 = 0;
            let mut i1 = self.table.len() - 3;
            while i0 < i1 {
                // i0 <= i < i1.
                let i = (i0 + i1) >> 1;
                if self.table[i].0 > frequency {
                    i1 = i;
                } else if self.table[i + 1].0 > frequency {
                    i0 = i;
                    break;
                } else {
                    i0 = i + 1;
                }
            }
            i0
        }
    }

    /// Read wavetable.
    #[inline]
    pub fn read(&self, table_hint: usize, frequency: f32, phase: f32) -> (f32, usize) {
        let table = self.table_index(table_hint, frequency);
        let w = clamp01(delerp(
            self.table[table].0,
            self.table[table + 1].0,
            frequency,
        ));
        (
            // Note the different table index. We can use `table + 1` up to its designated pitch.
            (1.0 - w) * self.at(table + 1, phase) + w * self.at(table + 2, phase),
            table,
        )
    }

    /// Read wavetable in parallel.
    #[inline]
    pub fn read_simd(&self, table_hint: usize, frequency: f32, phase: F32x) -> (F32x, usize) {
        let table = self.table_index(table_hint, frequency);
        let w = clamp01(delerp(
            self.table[table].0,
            self.table[table + 1].0,
            frequency,
        ));
        (
            // Note the different table index. We can use `table + 1` up to its designated pitch.
            (1.0 - w) * self.at_simd(table + 1, phase) + w * self.at_simd(table + 2, phase),
            table,
        )
    }
}

/// Bandlimited wavetable synthesizer with `N` outputs (1 or 2).
/// - Input 0: frequency in Hz.
/// - Output 0: audio.
/// - Output 1 (optional): phase in 0...1.
#[derive(Clone)]
pub struct WaveSynth<N>
where
    N: Size<f32>,
{
    table: Arc<Wavetable>,
    /// Phase in 0...1.
    phase: f32,
    /// Initial phase in 0...1, seeded via pseudorandom phase system.
    initial_phase: f32,
    /// Previously used transposition table.
    table_hint: usize,
    sample_rate: f32,
    sample_duration: f32,
    _marker: core::marker::PhantomData<N>,
}

impl<N> WaveSynth<N>
where
    N: Size<f32>,
{
    pub fn new(table: Arc<Wavetable>) -> Self {
        WaveSynth {
            table,
            phase: 0.0,
            initial_phase: 0.0,
            table_hint: 0,
            sample_rate: DEFAULT_SR as f32,
            sample_duration: 1.0 / DEFAULT_SR as f32,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<N> AudioNode for WaveSynth<N>
where
    N: Size<f32>,
{
    const ID: u64 = 34;
    type Inputs = numeric_array::typenum::U1;
    type Outputs = N;

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
        self.phase -= floor(self.phase);
        let (output, hint) = self.table.read(self.table_hint, abs(frequency), self.phase);
        self.table_hint = hint;
        Frame::generate(|i| {
            if i == 0 {
                convert(output)
            } else {
                convert(self.phase)
            }
        })
    }

    #[inline]
    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        let mut phase = self.phase;
        let mut table_hint = self.table_hint;
        for i in 0..full_simd_items(size) {
            let frequency = input.at(0, i).as_array_ref()[0];
            let phase_simd = F32x::new(core::array::from_fn(|j| {
                phase += input.at(0, i).as_array_ref()[j] * self.sample_duration;
                phase
            }));
            let phase_simd = phase_simd - phase_simd.floor();
            // Try to support negative frequencies as well by taking the absolute value of the input frequency.
            let (output_simd, hint) = self.table.read_simd(table_hint, abs(frequency), phase_simd);
            output.set(0, i, output_simd);
            table_hint = hint;
            if Self::Outputs::USIZE > 1 {
                output.set(1, i, phase_simd);
            }
        }
        self.phase = phase - floor(phase);
        self.table_hint = table_hint;
        self.process_remainder(size, input, output);
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Generator(0.0).route(input, self.outputs())
    }
}

/// Bandlimited wavetable synthesizer driven by a phase input.
/// - Input 0: phase in 0...1.
/// - Output 0: audio.
#[derive(Clone)]
pub struct PhaseSynth {
    table: Arc<Wavetable>,
    /// Previous phase.
    phase: f32,
    phase_ready: bool,
    table_hint: usize,
    sample_rate: f32,
}

impl PhaseSynth {
    pub fn new(table: Arc<Wavetable>) -> Self {
        PhaseSynth {
            table,
            phase: 0.0,
            phase_ready: false,
            table_hint: 0,
            sample_rate: DEFAULT_SR as f32,
        }
    }
}

impl AudioNode for PhaseSynth {
    const ID: u64 = 35;
    type Inputs = numeric_array::typenum::U1;
    type Outputs = numeric_array::typenum::U1;

    fn reset(&mut self) {
        self.phase_ready = false;
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate as f32;
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let phase = input[0].to_f32();
        let phase = phase - floor(phase);
        let delta = if self.phase_ready {
            // Interpret frequency from phase so it's always at or below Nyquist.
            // Support negative frequencies as well.
            min(
                abs(phase - self.phase),
                min(abs(phase - 1.0 - self.phase), abs(phase + 1.0 - self.phase)),
            )
        } else {
            // For the first sample, we don't have previous phase, so set frequency pessimistically to Nyquist.
            self.phase_ready = true;
            0.5
        };
        let (output, hint) = self
            .table
            .read(self.table_hint, delta * self.sample_rate, phase);
        self.table_hint = hint;
        self.phase = phase;
        Frame::generate(|i| {
            if i == 0 {
                convert(output)
            } else {
                convert(phase)
            }
        })
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Generator(0.0).route(input, self.outputs())
    }
}

/// Pulse wave oscillator.
/// - Input 0: frequency in Hz
/// - Input 1: pulse duty cycle in 0...1
/// - Output 0: pulse wave
#[derive(Clone)]
pub struct PulseWave {
    pulse: An<
        Pipe<
            Pipe<
                Stack<WaveSynth<U2>, Pass>,
                Stack<Pass, Pipe<Binop<FrameAdd<U1>, Pass, Pass>, PhaseSynth>>,
            >,
            Binop<FrameSub<U1>, Pass, Pass>,
        >,
    >,
}

#[allow(clippy::new_without_default)]
impl PulseWave {
    pub fn new() -> Self {
        Self {
            pulse: (An(WaveSynth::<U2>::new(saw_table())) | pass())
                >> (pass() | (pass() + pass()) >> An(PhaseSynth::new(saw_table())))
                >> pass() - pass(),
        }
    }
}

impl AudioNode for PulseWave {
    const ID: u64 = 44;
    type Inputs = U2;
    type Outputs = U1;

    fn reset(&mut self) {
        self.pulse.reset();
    }
    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.pulse.set_sample_rate(sample_rate);
    }
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        self.pulse.tick(input)
    }
    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        self.pulse.process(size, input, output);
    }
    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.pulse.route(input, frequency)
    }
    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.pulse.ping(probe, hash).hash(Self::ID)
    }
    fn allocate(&mut self) {
        self.pulse.allocate();
    }
}

pub fn saw_table() -> Arc<Wavetable> {
    static INSTANCE: OnceBox<Arc<Wavetable>> = OnceBox::new();
    INSTANCE
        .get_or_init(|| {
            let table = Wavetable::new(
                20.0,
                20_000.0,
                4.0,
                // To build the classic saw shape, shift even partials 180 degrees.
                &|i| if (i & 1) == 1 { 0.0 } else { 0.5 },
                &|_, i| 1.0 / i as f64,
            );
            Box::new(Arc::new(table))
        })
        .clone()
}

pub fn square_table() -> Arc<Wavetable> {
    static INSTANCE: OnceBox<Arc<Wavetable>> = OnceBox::new();
    INSTANCE
        .get_or_init(|| {
            let table = Wavetable::new(20.0, 20_000.0, 4.0, &|_| 0.0, &|_, i| {
                if (i & 1) == 1 {
                    1.0 / i as f64
                } else {
                    0.0
                }
            });
            Box::new(Arc::new(table))
        })
        .clone()
}

pub fn triangle_table() -> Arc<Wavetable> {
    static INSTANCE: OnceBox<Arc<Wavetable>> = OnceBox::new();
    INSTANCE
        .get_or_init(|| {
            let table = Wavetable::new(
                20.0,
                20_000.0,
                4.0,
                // To build the classic triangle shape, shift every other odd partial 180 degrees.
                &|i| if (i & 3) == 3 { 0.5 } else { 0.0 },
                &|_, i| {
                    if (i & 1) == 1 {
                        1.0 / (i * i) as f64
                    } else {
                        0.0
                    }
                },
            );
            Box::new(Arc::new(table))
        })
        .clone()
}

pub fn organ_table() -> Arc<Wavetable> {
    static INSTANCE: OnceBox<Arc<Wavetable>> = OnceBox::new();
    INSTANCE
        .get_or_init(|| {
            let table = Wavetable::new(
                20.0,
                20_000.0,
                4.0,
                // Set phase to enable interpolation with saw, triangle and soft saw wavetables.
                &|i| {
                    if (i & 3) == 3 {
                        0.5
                    } else if (i & 1) == 1 {
                        0.0
                    } else {
                        0.5
                    }
                },
                &|_, i| {
                    let z = i.trailing_zeros();
                    let j = i >> z;
                    1.0 / (i + j * j * j) as f64
                },
            );
            Box::new(Arc::new(table))
        })
        .clone()
}

pub fn soft_saw_table() -> Arc<Wavetable> {
    static INSTANCE: OnceBox<Arc<Wavetable>> = OnceBox::new();
    INSTANCE
        .get_or_init(|| {
            let table = Wavetable::new(
                20.0,
                20_000.0,
                4.0,
                // Set phase to enable interpolation with saw, triangle and organ wavetables.
                &|i| {
                    if (i & 3) == 3 {
                        0.5
                    } else if (i & 1) == 1 {
                        0.0
                    } else {
                        0.5
                    }
                },
                &|_, i| 1.0 / (i * i) as f64,
            );
            Box::new(Arc::new(table))
        })
        .clone()
}

pub fn hammond_table() -> Arc<Wavetable> {
    static INSTANCE: OnceBox<Arc<Wavetable>> = OnceBox::new();
    INSTANCE
        .get_or_init(|| {
            let table = Wavetable::new(20.0, 20_000.0, 4.0, &|_| 0.0, &|_, i| {
                let z = i.trailing_zeros();
                let j = i >> z;
                let f = 1.0 / ((z + 1) * (z + 1)) as f64;
                match i {
                    1 => return 1.0,
                    2 => return 1.0,
                    3 => return 1.0,
                    _ => (),
                }
                match j {
                    1 => f,
                    3 => f,
                    9 => 0.2 * f,
                    _ => 0.0,
                }
            });
            Box::new(Arc::new(table))
        })
        .clone()
}
