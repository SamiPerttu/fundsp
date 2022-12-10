//! Bandlimited wavetable synthesizer.

use super::audionode::*;
use super::math::*;
use super::signal::*;
use super::*;
use num_complex::Complex32;
use rustfft::algorithm::Radix4;
use rustfft::Fft;
use rustfft::FftDirection;

/// Interpolate between `a1` and `a2` taking previous (`a0`) and next (`a3`) points into account.
/// Employs an optimal 4-point, 4th order interpolating polynomial for 4x oversampled signals.
/// The SNR of the interpolator for pink noise is 101.1 dB.
#[inline]
fn optimal4x44<T: Float>(a0: T, a1: T, a2: T, a3: T, x: T) -> T {
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
    let c4 = even1 * T::from_f64(0.00986988334359864) + even2 * -T::from_f64(0.00989340017126506);
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
            a[i] = Complex32::from_polar(w as f32, (TAU * phase(i as u32)) as f32);
        }
    }

    let fft = Radix4::new(length, FftDirection::Inverse);
    fft.process(&mut a);

    let z = 1.0 / sqrt(length as f32);
    a.iter().map(|x| x.re * z).collect()
}

#[derive(Clone)]
pub struct Wavetable {
    /// Frequency tables arranged in order of increasing frequency.
    table: Vec<(f32, Vec<f32>)>,
}

impl Wavetable {
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
        let mut p = min_pitch;
        let p_factor = pow(2.0, 1.0 / tables_per_octave);
        let mut max_amplitude = 0.0;
        //let mut total_size = 0;
        while p <= max_pitch {
            let wave = make_wave(p, phase, amplitude);
            max_amplitude = wave.iter().fold(max_amplitude, |acc, &x| max(acc, abs(x)));
            //total_size += wave.len();
            table.push((p as f32, wave));
            p *= p_factor;
        }
        if max_amplitude > 0.0 {
            let z = 1.0 / max_amplitude;
            table.iter_mut().for_each(|t| {
                t.1.iter_mut().for_each(|x| {
                    *x *= z;
                })
            });
        }
        //println!(
        //    "Wavetable transpositions {} max amplitude {} total bytes {}",
        //    table.len(),
        //    max_amplitude,
        //    total_size * 4
        //);
        Wavetable { table }
    }

    /// Read wave at the given phase (in 0...1).
    #[inline]
    pub fn at(&self, i: usize, phase: f32) -> f32 {
        let table: &Vec<f32> = &self.table[i].1;
        let p = table.len() as f32 * phase;
        let i1 = p as usize;
        let w = p - i1 as f32;
        let i0 = i1.wrapping_sub(1) & (table.len() - 1);
        let i1 = i1 & (table.len() - 1);
        let i2 = (i1 + 1) & (table.len() - 1);
        let i3 = (i1 + 2) & (table.len() - 1);
        optimal4x44(table[i0], table[i1], table[i2], table[i3], w)
    }

    /// Read wavetable.
    #[inline]
    pub fn read(&self, table_hint: usize, frequency: f32, phase: f32) -> (f32, usize) {
        let table =
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
            };
        let w = delerp(self.table[table].0, self.table[table + 1].0, frequency) as f32;
        (
            // Note the different table index. We can use `table + 1` up to its designated pitch.
            (1.0 - w) * self.at(table + 1, phase) + w * self.at(table + 2, phase),
            table,
        )
    }
}

/// Bandlimited wavetable synthesizer with `N` outputs (1 or 2).
/// - Input 0: frequency in Hz.
/// - Output 0: audio.
/// - Output 1 (optional): phase in 0...1.
#[derive(Clone)]
pub struct WaveSynth<'a, T, N>
where
    T: Float,
    N: Size<T>,
{
    table: &'a Wavetable,
    /// Phase in 0...1.
    phase: f32,
    /// Initial phase in 0...1, seeded via pseudorandom phase system.
    initial_phase: f32,
    /// Previously used transposition table.
    table_hint: usize,
    sample_rate: f32,
    _marker: std::marker::PhantomData<(T, N)>,
}

impl<'a, T, N> WaveSynth<'a, T, N>
where
    T: Float,
    N: Size<T>,
{
    pub fn new(sample_rate: f64, table: &'a Wavetable) -> Self {
        WaveSynth {
            table,
            phase: 0.0,
            initial_phase: 0.0,
            table_hint: 0,
            sample_rate: sample_rate as f32,
            _marker: std::marker::PhantomData::default(),
        }
    }
}

impl<'a, T, N> AudioNode for WaveSynth<'a, T, N>
where
    T: Float,
    N: Size<T>,
{
    const ID: u64 = 34;
    type Sample = T;
    type Inputs = numeric_array::typenum::U1;
    type Outputs = N;
    type Setting = ();

    #[inline]
    fn reset(&mut self, sample_rate: Option<f64>) {
        if let Some(sample_rate) = sample_rate {
            self.sample_rate = sample_rate as f32;
        }
        self.phase = self.initial_phase;
    }

    #[inline]
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
        let delta = frequency / self.sample_rate;
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

    fn route(&self, _input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        for i in 0..N::USIZE {
            output[i] = Signal::Latency(0.0);
        }
        output
    }
}

/// Bandlimited wavetable synthesizer driven by a phase input.
/// - Input 0: phase in 0...1.
/// - Output 0: audio.
#[derive(Clone)]
pub struct PhaseSynth<'a, T>
where
    T: Float,
{
    table: &'a Wavetable,
    /// Previous phase.
    phase: f32,
    phase_ready: bool,
    table_hint: usize,
    sample_rate: f32,
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T> PhaseSynth<'a, T>
where
    T: Float,
{
    pub fn new(sample_rate: f64, table: &'a Wavetable) -> Self {
        PhaseSynth {
            table,
            phase: 0.0,
            phase_ready: false,
            table_hint: 0,
            sample_rate: sample_rate as f32,
            _marker: std::marker::PhantomData::default(),
        }
    }
}

impl<'a, T> AudioNode for PhaseSynth<'a, T>
where
    T: Float,
{
    const ID: u64 = 35;
    type Sample = T;
    type Inputs = numeric_array::typenum::U1;
    type Outputs = numeric_array::typenum::U1;
    type Setting = ();

    #[inline]
    fn reset(&mut self, sample_rate: Option<f64>) {
        if let Some(sample_rate) = sample_rate {
            self.sample_rate = sample_rate as f32;
        }
        self.phase_ready = false;
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
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

    fn route(&self, _input: &SignalFrame, _frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = Signal::Latency(0.0);
        output
    }
}

lazy_static! {
    /// Saw wavetable.
    pub static ref SAW_TABLE: Wavetable = Wavetable::new(
        20.0,
        20_000.0,
        4.0,
        // To build the classic saw shape, shift even partials 180 degrees.
        &|i| if (i & 1) == 1 { 0.0 } else { 0.5 },
        &|_, i| { 1.0 / i as f64 }
    );
}

lazy_static! {
    /// Square wavetable.
    pub static ref SQUARE_TABLE: Wavetable =
        Wavetable::new(20.0, 20_000.0, 4.0, &|_| 0.0, &|_, i| if (i & 1) == 1 {
            1.0 / i as f64
        } else {
            0.0
        });
}

lazy_static! {
    /// Triangle wavetable.
    pub static ref TRIANGLE_TABLE: Wavetable = Wavetable::new(
        20.0,
        20_000.0,
        4.0,
        // To build the classic triangle shape, shift every other odd partial 180 degrees.
        &|i| if (i & 3) == 3 { 0.5 } else { 0.0 },
        &|_, i| if (i & 1) == 1 {
            1.0 / (i * i) as f64
        } else {
            0.0
        }
    );
}

lazy_static! {
    // Organ wavetable. Emphasizes octave partials.
    pub static ref ORGAN_TABLE: Wavetable = Wavetable::new(
        20.0, 20_000.0, 4.0,
        // Set phase to enable interpolation with saw, triangle and soft saw wavetables.
        &|i| if (i & 3) == 3 { 0.5 } else if (i & 1) == 1 { 0.0 } else { 0.5 },
        &|_, i| {
            let z = i.trailing_zeros();
            let j = i >> z;
            1.0 / (i + j * j * j) as f64
        }
    );
}

lazy_static! {
    // Soft saw wavetable. Falls off like a triangle wave, contains all partials.
    pub static ref SOFT_SAW_TABLE: Wavetable = Wavetable::new(
        20.0, 20_000.0, 4.0,
        // Set phase to enable interpolation with saw, triangle and organ wavetables.
        &|i| if (i & 3) == 3 { 0.5 } else if (i & 1) == 1 { 0.0 } else { 0.5 },
        &|_, i| {
            1.0 / (i * i) as f64
        }
    );
}
