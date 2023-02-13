//! Sound generators.

use funutd::dna::*;

use super::audiounit::*;
use super::granular::*;
use super::hacker::*;
use super::net::*;

#[derive(Clone)]
enum Control {
    SlowSine,
    SplineNoise,
}

pub fn gen_lfo(dna: &mut Dna) -> Box<dyn AudioUnit64> {
    let control = dna.choice(
        "Control Type",
        [
            (1.0, "Slow Sine", Control::SlowSine),
            (1.0, "Spline Noise", Control::SplineNoise),
        ],
    );
    let seed = dna.u32("Control Seed");

    match control {
        Control::SlowSine => Box::new(lfo(move |t| sin_hz(0.1, t + rnd(seed as i64) / 0.1))),
        Control::SplineNoise => Box::new(lfo(move |t| {
            spline_noise(seed as i64, t + rnd(seed as i64))
        })),
    }
}

#[derive(Clone)]
enum Effect {
    Flanger,
    Phaser,
}

pub fn gen_effect(dna: &mut Dna) -> Box<dyn AudioUnit64> {
    let effect = dna.choice(
        "Effect Type",
        [
            (1.0, "Flanger", Effect::Flanger),
            (1.0, "Phaser", Effect::Phaser),
        ],
    );

    match effect {
        Effect::Flanger => Box::new(flanger(0.9, 0.005, 0.015, |t| {
            lerp11(0.005, 0.015, sin_hz(0.1, t))
        })),
        Effect::Phaser => Box::new(phaser(0.9, |t| lerp11(0.0, 1.0, sin_hz(0.1, t)))),
    }
}

#[derive(Clone)]
enum Waveform {
    Saw,
    Square,
    Triangle,
    SoftSaw,
    Organ,
}

#[derive(Clone)]
enum ChoiceX {
    Oscillator,
    PulseWave,
    /// Sine wave ring modulated with lowpassed noise.
    NoisySine,
    /// Resonator (4th order) filtered pink noise.
    Resonator,
    /// Overdriven sine wave.
    OverdriveSine,
}

#[derive(Clone)]
enum ChoiceY {
    Amplification,
    Vibrato,
    MoogFilter,
    CombFilter,
    FromX,
}

#[derive(Clone)]
enum ChoiceZ {
    Amplification,
    PeakFilter,
    BandpassFilter,
    MoogFilter,
    MoogFilterZ,
    Overdrive,
}

/// Generate a granular synthesizer. Number of output channels is 1 or 2.
/// The scale containing MIDI pitches is used if non-empty.
pub fn gen_granular(
    channels: usize,
    scale: &[f64],
    beat_length: f64,
    beats_per_cycle: usize,
    dna: &mut Dna,
) -> Box<dyn AudioUnit64> {
    assert!(channels == 1 || channels == 2);
    let scale_vec: Vec<_> = Vec::from(scale);

    let texture_seed = dna.u32("Texture Seed");

    let grain_length = dna.f32_in("Grain Length", 0.030, 0.100) as f64;
    let envelope_length = dna.f32_in("Envelope Fraction", 0.333, 0.5) as f64;

    let voices = dna.u32_in("Voices", 12, 42) as usize;

    let inner_radius = dna.f32_in("Inner Radius", 0.03, 0.10) as f64;
    let outer_radius = dna.f32_in("Outer Radius", 0.13, 0.20) as f64;
    let jitter = dna.f32_xform("Jitter", |x| xerp(0.0001, 0.0500, x)) as f64;

    let choice_x = dna.choice(
        "X Channel",
        [
            (1.5, "Oscillator", ChoiceX::Oscillator),
            (0.5, "Pulse Wave", ChoiceX::PulseWave),
            (0.5, "Noisy Sine", ChoiceX::NoisySine),
            (0.5, "Resonator", ChoiceX::Resonator),
            (0.5, "Overdrive Sine", ChoiceX::OverdriveSine),
        ],
    );

    let waveform = match choice_x {
        ChoiceX::Oscillator => dna.choice(
            "Waveform",
            [
                (1.0, "Saw", Waveform::Saw),
                (1.0, "Square", Waveform::Square),
                (1.0, "Triangle", Waveform::Triangle),
                (1.0, "Soft Saw", Waveform::SoftSaw),
                (1.5, "Organ", Waveform::Organ),
            ],
        ),
        _ => Waveform::Saw,
    };

    let choice_y = match choice_x {
        ChoiceX::Oscillator => dna.choice(
            "Y Channel",
            [
                (1.0, "Amplification", ChoiceY::Amplification),
                (1.0, "Vibrato", ChoiceY::Vibrato),
                (
                    if matches!(choice_x, ChoiceX::Resonator) {
                        0.01
                    } else {
                        1.0
                    },
                    "Moog Filter",
                    ChoiceY::MoogFilter,
                ),
                (1.0, "Comb Filter", ChoiceY::CombFilter),
            ],
        ),
        _ => ChoiceY::FromX,
    };

    let vibrato_depth = match choice_y {
        ChoiceY::Vibrato => {
            dna.f32_xform("Vibrato Depth", |x| xerp(semitone_ratio(0.2), 2.0, x * x)) as f64
        }
        _ => semitone_ratio(0.2),
    };

    let bandpass_p = match choice_x {
        ChoiceX::Oscillator => 1.0,
        ChoiceX::PulseWave => 1.0,
        ChoiceX::OverdriveSine => 0.5,
        _ => 0.01,
    };

    let choice_z = match choice_y {
        ChoiceY::MoogFilter => ChoiceZ::MoogFilter,
        _ => dna.choice(
            "Z Channel",
            [
                (
                    if matches!(choice_y, ChoiceY::Amplification) {
                        0.01
                    } else {
                        1.0
                    },
                    "Amplification",
                    ChoiceZ::Amplification,
                ),
                (2.0, "Peak Filter", ChoiceZ::PeakFilter),
                (
                    if matches!(choice_y, ChoiceY::Amplification)
                        || matches!(choice_x, ChoiceX::Resonator)
                        || matches!(choice_x, ChoiceX::NoisySine)
                    {
                        0.01
                    } else {
                        1.0
                    },
                    "Moog Filter Z",
                    ChoiceZ::MoogFilterZ,
                ),
                (
                    if matches!(choice_x, ChoiceX::OverdriveSine) {
                        0.01
                    } else {
                        1.0
                    },
                    "Overdrive",
                    ChoiceZ::Overdrive,
                ),
                (bandpass_p, "Bandpass Filter", ChoiceZ::BandpassFilter),
            ],
        ),
    };

    let bandpass_q = if matches!(choice_z, ChoiceZ::BandpassFilter) {
        dna.f32_in("Bandpass Filter Q", 1.0, 4.0) as f64
    } else {
        1.0
    };

    let peak_q = if matches!(choice_z, ChoiceZ::PeakFilter) {
        dna.f32_in("Peak Filter Q", 2.0, 5.0) as f64
    } else {
        1.0
    };

    let stereo_width = if channels == 2 {
        dna.f32_in("Stereo Width", 0.3, 0.8) as f64
    } else {
        0.0
    };

    let create_grain = move |t: f64,
                             _b: f64,
                             v: f64,
                             x: f64,
                             y: f64,
                             z: f64|
          -> (f64, f64, Box<dyn AudioUnit64>) {
        let f = if scale_vec.len() > 0 {
            let d = lerp11(0.0, scale_vec.len() as f64 - 0.01, x);
            midi_hz(scale_vec[d as usize] + 0.02 * (d - round(d)))
        } else {
            xerp11(midi_hz(36.0), midi_hz(108.0), x)
        };

        let f = match choice_y {
            ChoiceY::Vibrato => {
                let r = max(
                    1.0,
                    min(vibrato_depth, xerp11(1.0 / vibrato_depth, vibrato_depth, y)),
                );
                f * xerp11(r, 1.0 / r, sin_hz(6.0, t))
            }
            _ => f,
        };

        let mut amp = 0.1 / sqrt(voices as f64);

        let mut c = match choice_x {
            ChoiceX::Oscillator => match waveform {
                Waveform::Saw => Net64::wrap(Box::new(saw_hz(f))),
                Waveform::Square => Net64::wrap(Box::new(square_hz(f))),
                Waveform::Triangle => Net64::wrap(Box::new(triangle_hz(f))),
                Waveform::SoftSaw => Net64::wrap(Box::new(soft_saw_hz(f))),
                Waveform::Organ => Net64::wrap(Box::new(organ_hz(f))),
            },
            ChoiceX::PulseWave => {
                Net64::wrap(Box::new(dc((f, 1.0 - xerp11(0.02, 0.50, y))) >> pulse()))
            }
            ChoiceX::NoisySine => {
                let bandwidth = xerp11(1.0, 200.0, y);
                amp *= 20.0 / sqrt(bandwidth);
                Net64::wrap(Box::new(
                    (white() >> lowpass_hz(bandwidth, 1.0)) * sine_hz(f),
                ))
            }
            ChoiceX::OverdriveSine => {
                let hardness = 0.1 + xerp11(0.1, 10.0, y);
                amp /= hardness;
                Net64::wrap(Box::new(sine_hz(f) >> shape(Shape::Tanh(hardness))))
            }
            ChoiceX::Resonator => {
                let bandwidth = xerp11(2.0, 100.0, y);
                amp *= 0.5;
                Net64::wrap(Box::new(
                    pink() >> resonator_hz(f, bandwidth) >> resonator_hz(f + 0.5, bandwidth),
                ))
            }
        };

        match choice_y {
            ChoiceY::Amplification => {
                amp *= xerp11(0.1, 2.0, y);
            }
            ChoiceY::MoogFilter => {
                c = c >> moog_hz(xerp11(200.0, 20_000.0, y), lerp11(0.25, 0.65, z));
            }
            _ => (),
        }

        match choice_z {
            ChoiceZ::Amplification => {
                amp *= xerp11(0.1, 2.0, z);
            }
            ChoiceZ::BandpassFilter => {
                c = c >> bandpass_hz(xerp11(100.0, 10_000.0, z), bandpass_q);
            }
            ChoiceZ::PeakFilter => {
                c = c >> peak_hz(xerp11(100.0, 10_000.0, z), peak_q);
                amp *= 0.5;
            }
            ChoiceZ::MoogFilterZ => {
                c = c >> moog_hz(xerp11(200.0, 20_000.0, z), 0.6);
            }
            ChoiceZ::Overdrive => {
                let hardness = 0.1 + xerp11(0.1, 10.0, z);
                amp /= hardness;
                c = c >> shape(Shape::Softsign(hardness));
            }
            _ => (),
        }

        if matches!(choice_y, ChoiceY::CombFilter) {
            c = c >> feedback2(shape(Shape::Tanh(1.1)), delay(xerp11(0.001, 0.020, y)));
            amp *= 0.5;
        }

        c = c * amp;

        if channels == 2 {
            c = c >> pan(v * stereo_width);
        }

        (grain_length, grain_length * envelope_length, Box::new(c))
    };

    let granular = Granular64::new(
        channels,
        voices,
        beat_length,
        beats_per_cycle,
        texture_seed as u64,
        inner_radius,
        outer_radius,
        jitter,
        create_grain,
    );
    Box::new(granular)
}
