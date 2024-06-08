//! Render a sequence and save it to disk.

#![allow(unused_must_use)]
#![allow(clippy::precedence)]

use fundsp::hacker::*;
use fundsp::sound::*;
use funutd::*;

fn main() {
    let mut rng = Rnd::new();

    let bpm = 128.0;

    /*
    let wind = |seed: i64, panning| {
        (noise() | lfo(move |t| xerp11(50.0, 5000.0, fractal_noise(seed, 6, 0.5, t * 0.2))))
            >> bandpass_q(5.0)
            >> pan(panning)
    };
    */

    let sample_rate = 44100.0;
    // 'x' indicates a drum hit, while '.' is a rest.
    let bassd_line = "x.....x.x.......x.....x.xx......x.....x.x.......x.......x.x.....";
    let snare_line = "....x.......x.......x.......x.......x.......x.......x.......x...";
    let cymbl_line = "x.x.x.x.x.x.x.x.x.x.x.x.x.x.x.x.x.x.x.x.x.x.x.x.x.x.x.x.x.x.x.x.";

    /*
    let bd = |seed: i64| {
        bus::<U40, _, _>(|i| {
            let f = xerp(50.0, 2000.0, rnd(i ^ seed));
            lfo(move |t| xerp(f, f * semitone_ratio(-5.0), t))
                >> sine()
                    * lfo(move |t| {
                        xerp(1.0, 0.02, dexerp(50.0, 2000.0, f)) * exp(-t * f * f * 0.002)
                    })
                >> pan(0.0)
        })
    };

    let bd2 = || {
        let sweep = (lfo(|t| xerp(100.0, 50.0, t)) >> saw() | lfo(|t| xerp(3000.0, 3.0, t)))
            >> !lowpass_q(2.0)
            >> lowpass_q(1.0);
        sweep >> pinkpass() >> shape(Shape::Tanh(2.0)) >> pan(0.0)
    };
    */
    /*
    let stab = move || {
        let bps = bpm / 60.0;
        fundsp::sound::pebbles(14.0, 200)
            * lfo(move |t| {
                if t * bps - round(t * bps) > 0.0 && round(t * bps) < 32.0 {
                    0.1
                } else {
                    0.0
                }
            })
            >> highpass_hz(3200.0, 1.0)
            >> phaser(0.85, |t| sin_hz(0.1, t) * 0.5 + 0.5)
            >> pan(0.0)
    };
    */

    let mut sequencer = Sequencer::new(true, 2);
    sequencer.set_sample_rate(sample_rate);

    //sequencer.push(0.0, 60.0, Fade::Smooth, 0.0, 0.0, Box::new(stab() * 0.4));

    let length = bassd_line.as_bytes().len();
    let duration = length as f64 / bpm_hz(bpm) / 4.0 * 2.0 + 2.0;

    for i in 0..length * 2 {
        let t0 = i as f64 / bpm_hz(bpm) / 4.0;
        let t1 = t0 + 1.0;
        if bassd_line.as_bytes()[i % length] == b'x' {
            sequencer.push(
                t0 + 0.001 * rng.f64(),
                t1,
                Fade::Smooth,
                0.0,
                0.25,
                Box::new(bassdrum(0.2 + rng.f32() * 0.02, 180.0, 60.0) >> pan(0.0)),
            );
        }
        if snare_line.as_bytes()[i % length] == b'x' {
            sequencer.push(
                t0 + 0.001 * rng.f64(),
                t1,
                Fade::Smooth,
                0.0,
                0.25,
                Box::new(snaredrum(rng.i64(), 0.4 + rng.f32() * 0.02) * 1.5 >> pan(0.0)),
            );
        }
        if cymbl_line.as_bytes()[i % length] == b'x' {
            sequencer.push(
                t0 + 0.001 * rng.f64(),
                t1,
                Fade::Smooth,
                0.0,
                0.25,
                Box::new(cymbal(rng.i64()) * 0.05 >> pan(0.0)),
            );
        }
    }

    let wave = Wave::render(sample_rate, duration, &mut sequencer);

    //let wave = wave.filter(
    //    duration,
    //    &mut (chorus(0, 0.0, 0.01, 0.5) | chorus(1, 0.0, 0.01, 0.5)),
    //);

    let wave = wave.filter(
        duration,
        &mut (multipass()
            & 0.2 * reverb2_stereo(10.0, 1.0, 0.5, 1.0, highshelf_hz(6000.0, 1.0, db_amp(-3.0)))),
    );

    let wave = wave.filter_latency(duration, &mut (limiter_stereo(5.0, 5.0)));

    wave.save_wav16(std::path::Path::new("sequence.wav"))
        .expect("Could not save sequence.wav");
    println!("sequence.wav written.");
}
