//! Render a sequence and save it to disk.

#![allow(unused_must_use)]
#![allow(clippy::precedence)]

extern crate fundsp;

use fundsp::hacker::*;

fn main() {
    let sample_rate = 44100.0;
    let bassd_line = "x..xx...x..x.x..x..xx...x...xx..";
    let snare_line = "..x...x...x...x...x...x...x...xx";

    let bassdrum = || envelope(|t| 200.0 * exp(-t * 5.0)) >> sine() >> shape(Shape::Tanh(2.0)) >> split::<U2>();

    let snaredrum = || pink() * envelope(|t| exp(-t * 10.0)) >> split::<U2>();

    let mut sequencer = Sequencer::new(sample_rate, 2);

    let length = bassd_line.as_bytes().len();
    let bpm = 128.0 * 2.0;

    for i in 0..length {
        let t0 = i as f64 / bpm_hz(bpm);
        let t1 = (i + 2) as f64 / bpm_hz(bpm);
        if bassd_line.as_bytes()[i] == b'x' {
            sequencer.add64(t0, t1, Box::new(bassdrum()));
        }
        if snare_line.as_bytes()[i] == b'x' {
            sequencer.add64(t0, t1, Box::new(snaredrum()));
        }
    }

    let duration = length as f64 / bpm_hz(bpm);

    let wave = Wave64::render(sample_rate, duration + 1.0, &mut sequencer);

    let wave = wave.filter(duration + 1.0, &mut (reverb_stereo(0.1, 3.0) * 3.0));

    let wave = wave.filter(duration + 1.0, &mut (limiter_stereo((0.05, 0.2))));

    wave.save_wav16(std::path::Path::new("sequence.wav"));
}
