//! Render a sequence and save it to disk.

#![allow(unused_must_use)]
#![allow(clippy::precedence)]

extern crate fundsp;

use fundsp::hacker::*;

fn main() {
    let sample_rate = 44100.0;
    let bassd_line = "x.....x.x.......xx....x...xx....x.....x.x.......x.......x.x....xx";
    let snare_line = "....x......xx.......x.......x.......x.......x.......x......xx.xxx";

    let bassdrum =
        || envelope(|t| 200.0 * exp(-t * 5.0)) >> sine() >> shape(Shape::Tanh(2.0)) >> pan(0.0);

    let env = || envelope(|t| exp(-t * 10.0));
    let snaredrum = || (pink() * env() | pink() * env());

    let mut sequencer = Sequencer::new(sample_rate, 2);

    let length = bassd_line.as_bytes().len();
    let bpm = 128.0 * 4.0;

    for i in 0..length {
        let t0 = i as f64 / bpm_hz(bpm);
        let t1 = (i + 6) as f64 / bpm_hz(bpm);
        if bassd_line.as_bytes()[i] == b'x' {
            sequencer.add64(t0, t1, 0.0, 0.4, Box::new(bassdrum()));
        }
        if snare_line.as_bytes()[i] == b'x' {
            sequencer.add64(t0, t1, 0.0, 0.1, Box::new(snaredrum()));
        }
    }

    let duration = length as f64 / bpm_hz(bpm) + 2.0;

    let wave = Wave64::render(sample_rate, duration, &mut sequencer);

    let mut delay_fx = (pass() | pass())
        & feedback(
            (pass()
                | lfo(|t| lerp11(0.001, 0.003, sin_hz(0.3, t)))
                | pass()
                | lfo(|t| lerp11(0.001, 0.003, sin_hz(0.4, t))))
                >> (tap(0.001, 0.003) | tap(0.001, 0.003))
                >> (shape(Shape::Tanh(1.0)) | shape(Shape::Tanh(1.0))),
        );

    let wave = wave.filter(duration, &mut delay_fx);

    let wave = wave.filter(duration, &mut (reverb_stereo(0.1, 2.0) * 3.0));

    let wave = wave.filter_latency(duration, &mut (limiter_stereo((0.1, 0.2))));

    wave.save_wav16(std::path::Path::new("sequence.wav"));
}
