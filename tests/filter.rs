#![allow(
    clippy::precedence,
    clippy::type_complexity,
    clippy::float_cmp,
    clippy::len_zero,
    clippy::double_neg,
    clippy::clippy::many_single_char_names
)]
#![allow(dead_code)]

extern crate fundsp;
extern crate num_complex;
extern crate rustfft;

use fundsp::hacker::*;
use num_complex::Complex64;
use rustfft::algorithm::Radix4;
use rustfft::FFT;

#[test]
fn test_filter() {
    let mut rnd = AttoRand::new(1);

    // Test follow().
    for _ in 0..200 {
        // Bias testing toward smaller lengths to cut testing time shorter.
        let samples = round(xerp(1.0, 100_000.0, squared(rnd.gen_01::<f64>())));
        let sample_rate = xerp(10.0, 100_000.0, rnd.gen_01::<f64>());
        let mut x = follow(samples / sample_rate);
        x.reset(Some(sample_rate));
        let goal = lerp(-100.0, 100.0, rnd.gen_01::<f64>());
        for _ in 0..samples as usize {
            x.filter_mono(goal);
        }
        // Promise was 0.5% accuracy between 1 and 100k samples.
        let response = x.value() / goal;
        assert!(response >= 0.495 && response <= 0.505);
    }

    // Test asymmetric follow().
    for _ in 0..200 {
        // Bias testing toward smaller lengths to cut testing time shorter.
        let attack_samples = round(xerp(1.0, 100_000.0, squared(rnd.gen_01::<f64>())));
        let release_samples = round(xerp(1.0, 100_000.0, squared(rnd.gen_01::<f64>())));
        let sample_rate = xerp(10.0, 100_000.0, rnd.gen_01::<f64>());
        let goal = lerp(-100.0, 100.0, rnd.gen_01::<f64>());
        let mut x = follow((attack_samples / sample_rate, release_samples / sample_rate));
        x.reset(Some(sample_rate));
        for _ in 0..(if goal > 0.0 {
            attack_samples
        } else {
            release_samples
        }) as usize
        {
            x.filter_mono(goal);
        }
        // Promise was 0.5% accuracy between 1 and 100k samples.
        let response = x.value() / goal;
        assert!(response >= 0.495 && response <= 0.505);
    }
}

/// Complex64 with real component `x` and imaginary component zero.
fn re<T: Float>(x: T) -> Complex64 {
    Complex64::new(x.to_f64(), 0.0)
}

fn is_equal_response(x: Complex64, y: Complex64) -> bool {
    let abs_tolerance = 1.0e-9;
    let amp_tolerance = db_amp(0.1);
    let phase_tolerance = 1.0e-3 * TAU;
    let x_norm = x.norm();
    let y_norm = y.norm();
    let x_phase = x.arg();
    let y_phase = y.arg();
    x_norm / amp_tolerance - abs_tolerance <= y_norm
        && x_norm * amp_tolerance + abs_tolerance >= y_norm
        && x_phase - phase_tolerance <= y_phase
        && x_phase + phase_tolerance >= y_phase
}

fn test_response<X>(mut filter: An<X>)
where
    X: AudioNode<Sample = f64, Inputs = U1, Outputs = U1>,
{
    let length = 0x10000;
    let sample_rate = 44_100.0;

    filter.reset(Some(sample_rate));

    let mut input = 1.0;
    let mut buffer = vec![];
    for i in 0..length {
        // Apply a Hann window.
        let window = 0.5 + 0.5 * cos(i as f64 / length as f64 * PI);
        buffer.push(re(filter.filter_mono(input) * window));
        input = 0.0;
    }

    let mut spectrum = vec![re(0.0); length];
    let fft = Radix4::new(length, false);
    // Note. Output from process() appears normalized, contrary to documentation.
    fft.process(&mut buffer, &mut spectrum);

    let mut f = 10.0;
    while f <= 22_000.0 {
        let reported = filter.response(0, f).unwrap();
        let i = round(f * length as f64 / sample_rate as f64) as usize;
        let response = spectrum[i];
        /*
        println!(
            "{} Hz reported {} actual {} matches {}",
            f,
            reported.norm(),
            response.norm(),
            is_equal_response(reported, response)
        );
        */
        assert!(is_equal_response(reported, response));
        if f < 1000.0 {
            f += 10.0;
        } else {
            f += 100.0;
        }
    }
}

/// Test frequency response system.
#[test]
fn test_responses() {
    test_response(butterpass_hz(100.0));
    test_response(butterpass_hz(1000.0));
    test_response(butterpass_hz(10000.0));
    test_response(butterpass_hz(500.0) & butterpass_hz(5000.0));
    test_response(butterpass_hz(200.0) * 0.5);
    test_response(butterpass_hz(6000.0) >> butterpass_hz(600.0));
    test_response(pass() & tick());
    test_response(pass() * 0.25 & tick() * 0.5);
    test_response(pass() * 0.25 & tick() * 0.5 & tick() >> tick() * 0.25);
    test_response(tick() & butterpass_hz(20000.0));
    test_response((butterpass_hz(15000.0) ^ butterpass_hz(5000.0)) >> pass() + pass());
    test_response(
        (butterpass_hz(12000.0) ^ butterpass_hz(8000.0)) >> pass() + butterpass_hz(1200.0),
    );
}
