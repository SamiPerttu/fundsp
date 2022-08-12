//! Render a sequence and save it to disk.

#![allow(unused_must_use)]
#![allow(clippy::precedence)]

use fundsp::hacker::*;

fn main() {
    let sample_rate = 44100.0;
    // 'x' indicates a drum hit, while '.' is a rest.
    let bassd_line = "x.....x.x.......xx....x...xx....x.....x.x.......x.......x.x.....x";
    let snare_line = "....x.......x.......x.......x.......x.......x.......x......xx.xxx";

    // The bass drum is defined as a sine sweep dropping down from 200 Hz that is
    // waveshaped with the tanh function and then panned to the center in stereo.
    let bassdrum =
        || envelope(|t| 200.0 * exp(-t * 5.0)) >> sine() >> shape(Shape::Tanh(2.0)) >> pan(0.0);

    // The volume envelope for the snare drum is exponentially decaying.
    let env = || envelope(|t| exp(-t * 10.0));
    // The snare drum is exponentially decaying stereo pink noise.
    let snaredrum = || pink() * env() | pink() * env();

    let mut sequencer = Sequencer64::new(sample_rate, 2);

    let length = bassd_line.as_bytes().len();
    let bpm = 128.0 * 4.0;
    let duration = length as f64 / bpm_hz(bpm) + 2.0;

    for i in 0..length {
        let t0 = i as f64 / bpm_hz(bpm);
        let t1 = (i + 6) as f64 / bpm_hz(bpm);
        if bassd_line.as_bytes()[i] == b'x' {
            sequencer.add(t0, t1, 0.0, 0.4, Box::new(bassdrum()));
        }
        if snare_line.as_bytes()[i] == b'x' {
            sequencer.add(t0, t1, 0.0, 0.1, Box::new(snaredrum()));
        }
    }

    let wave = Wave64::render(sample_rate, duration, &mut sequencer);

    let wave = wave.filter(
        duration,
        &mut (0.9 * multipass() & 0.1 * reverb_stereo(10.0, 2.0)),
    );

    let wave = wave.filter_latency(duration, &mut (limiter_stereo((0.05, 0.20))));

    wave.save_wav16(std::path::Path::new("sequence.wav"));
}
