#![allow(
    dead_code,
    clippy::precedence,
    clippy::type_complexity,
    clippy::float_cmp,
    clippy::len_zero,
    clippy::double_neg,
    clippy::many_single_char_names,
    clippy::manual_range_contains
)]

use fundsp::hacker::*;
use num_complex::Complex64;
use rustfft::algorithm::Radix4;
use rustfft::Fft;
use rustfft::FftDirection;

#[test]
fn test_filter() {
    let mut rnd = AttoRand::new(1);

    // Test follow().
    for _ in 0..200 {
        // Bias testing toward smaller lengths to cut testing time shorter.
        let samples = round(xerp(1.0, 500_000.0, squared(rnd.get01::<f64>())));
        let sample_rate = xerp(10.0, 500_000.0, rnd.get01::<f64>());
        let mut x = follow(samples / sample_rate);
        x.reset(Some(sample_rate));
        let goal = lerp(-100.0, 100.0, rnd.get01::<f64>());
        for _ in 0..samples as usize {
            x.filter_mono(goal);
        }
        // Promise was 0.5% accuracy between 1 and 500k samples.
        let response = x.value() / goal;
        assert!(response >= 0.495 && response <= 0.505);
    }

    // Test asymmetric follow().
    for _ in 0..200 {
        // Bias testing toward smaller lengths to cut testing time shorter.
        let attack_samples = round(xerp(1.0, 500_000.0, squared(rnd.get01::<f64>())));
        let release_samples = round(xerp(1.0, 500_000.0, squared(rnd.get01::<f64>())));
        let sample_rate = xerp(10.0, 100_000.0, rnd.get01::<f64>());
        let goal = lerp(-100.0, 100.0, rnd.get01::<f64>());
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
        // Promise was 0.5% accuracy between 1 and 500k samples.
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
    let amp_tolerance = db_amp(0.05);
    let phase_tolerance = 5.0e-4 * TAU;
    let x_norm = x.norm();
    let y_norm = y.norm();
    let x_phase = x.arg();
    let y_phase = y.arg();
    x_norm / amp_tolerance - abs_tolerance <= y_norm
        && x_norm * amp_tolerance + abs_tolerance >= y_norm
        && min(
            abs(x_phase - y_phase),
            min(abs(x_phase - y_phase + TAU), abs(x_phase - y_phase - TAU)),
        ) <= phase_tolerance
}

fn test_response<X>(mut filter: X)
where
    X: AudioUnit64,
{
    assert!(filter.inputs() == 1 && filter.outputs() == 1);

    let length = 0x10000;
    let sample_rate = 44_100.0;

    filter.reset(Some(sample_rate));

    let mut input = 1.0;
    let mut buffer = Vec::with_capacity(length);
    for i in 0..length {
        // Apply a Hann window.
        let window = 0.5 + 0.5 * cos(i as f64 / length as f64 * PI);
        buffer.push(re(filter.filter_mono(input) * window));
        input = 0.0;
    }

    let fft = Radix4::new(length, FftDirection::Forward);
    // Note. Output from process() appears normalized, contrary to documentation.
    fft.process(&mut buffer);

    let mut f = 10.0;
    while f <= 22_000.0 {
        let i = round(f * length as f64 / sample_rate) as usize;
        let f_i = i as f64 / length as f64 * sample_rate;
        let reported = filter.response(0, f_i).unwrap();
        let response = buffer[i];
        /*
        println!(
            "{} Hz reported ({}, {}) actual ({}, {}) matches {}",
            f,
            reported.norm(),
            reported.arg(),
            response.norm(),
            response.arg(),
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

#[test]
fn test_misc() {
    let epsilon = 1.0e-9;
    assert!((pass() & tick()).response(0, 22050.0).unwrap().norm() < epsilon);
    assert!(
        (0.5 * pass() & tick() & 0.5 * tick() >> tick())
            .response(0, 22050.0)
            .unwrap()
            .norm()
            < epsilon
    );
    assert!(
        (pass() & tick() & tick() >> tick())
            .response(0, 22050.0)
            .unwrap()
            .norm()
            > 0.1
    );
}

/// Test frequency response system.
#[test]
fn test_responses() {
    test_response(bell_hz(500.0, 1.0, 2.0) * 0.5);
    test_response(lowshelf_hz(2000.0, 10.0, 5.0));
    test_response(highshelf_hz(2000.0, 10.0, 5.0));
    test_response(peak_hz(5000.0, 1.0));
    test_response(allpass_hz(500.0, 5.0));
    test_response(notch_hz(1000.0, 1.0));
    test_response(lowpass_hz(50.0, 1.0));
    test_response(highpass_hz(5000.0, 1.0));
    test_response(bandpass_hz(100.0, 1.0));
    test_response(highpass_hz(500.0, 1.0) & bandpass_hz(500.0, 2.0));
    test_response(pinkpass());
    test_response(follow(0.0002));
    test_response(follow(0.01));
    test_response(delay(0.0001));
    test_response(delay(0.0001) >> delay(0.0002));
    test_response(dcblock());
    test_response(dcblock_hz(100.0) & follow(0.001));
    test_response(lowpole_hz(1000.0));
    test_response(split() >> (lowpole_hz(100.0) + lowpole_hz(190.0)));
    test_response(lowpole_hz(10000.0));
    test_response(resonator_hz(300.0, 20.0));
    test_response(butterpass_hz(200.0));
    test_response(butterpass_hz(1000.0));
    test_response(butterpass_hz(500.0) & bell_hz(2000.0, 10.0, 5.0));
    test_response(butterpass_hz(6000.0) >> lowpass_hz(500.0, 3.0));
    test_response(pass() & tick());
    test_response(pass() * 0.25 & tick() * 0.5 & tick() >> tick() * 0.25);
    test_response(tick() & lowshelf_hz(500.0, 2.0, 0.1));
    test_response(allpole_delay(0.5) & allpole_delay(1.3) & allpole_delay(0.1));
    test_response(highpole_hz(5000.0) & highpole_hz(500.0) & highpole_hz(2000.0));
    test_response(
        (delay(0.001) ^ delay(0.002)) >> swap() >> (delay(0.003) | delay(0.007)) >> join(),
    );
    test_response(
        (butterpass_hz(15000.0) ^ allpass_hz(10000.0, 10.0)) >> lowpole_hz(500.0) + pass(),
    );
    test_response(
        (resonator_hz(12000.0, 500.0) ^ lowpass_hz(3000.0, 0.5))
            >> pass() + highshelf_hz(3000.0, 0.5, 4.0),
    );
    test_response(split() >> multipass::<U32>() >> join());
    test_response(
        split()
            >> stack::<U8, _, _>(|i| {
                resonator_hz(1000.0 + 1000.0 * i as f64, 100.0 + 100.0 * i as f64)
            })
            >> join(),
    );
    test_response(branchf::<U5, _, _>(|t| resonator_hz(xerp(100.0, 20000.0, t), 10.0)) >> join());
    test_response(pipe::<U4, _, _>(|i| {
        bell_hz(
            1000.0 + 1000.0 * i as f64,
            (i + 1) as f64,
            db_amp((i + 6) as f64),
        )
    }));
    test_response(
        split() >> stack::<U5, _, _>(|i| lowpole_hz(1000.0 + 1000.0 + i as f64)) >> join(),
    );
    test_response(bus::<U7, _, _>(|i| {
        lowpass_hz(1000.0 + 1000.0 * rnd(i), 1.0 + 1.0 * rnd(i << 1))
    }));
    test_response(
        split::<U3>()
            >> multisplit::<U3, U3>()
            >> sumf::<U9, _, _>(|f| highshelf_hz(f, 1.0 + f, 2.0 + f)),
    );
    test_response(pan(0.5) >> join());
    test_response(pan(0.0) >> join());
    test_response(pan(-1.0) >> multijoin::<U1, U2>());
    test_response(fir((0.5, 0.5)) >> monitor(Meter::Sample, 0));
    test_response(fir((0.25, 0.5, 0.25)) | timer(0));
    test_response(fir((0.4, 0.3, 0.2, 0.1)));
    test_response(morph_hz(1000.0, 1.0, 0.5));
    test_response(morph_hz(2000.0, 2.0, -0.5));
    test_response((pass() | dc((1000.0, 0.5, 0.5))) >> morph());
    test_response((pass() | dc((500.0, 2.0, -1.0))) >> morph());
    test_response(biquad(0.0, 0.17149959, 0.29287490, 0.58574979, 0.29287490));
    test_response(biquad(
        0.03371705,
        0.17177385,
        1.05925373,
        -0.03571491,
        0.18195209,
    ));

    let mut net1 = Net64::new(1, 1);
    net1.chain(Box::new(lowpole_hz(1500.0)));
    test_response(net1);

    let mut net2 = Net64::new(1, 1);
    net2.chain(Box::new(lowpole_hz(500.0)));
    net2.chain(Box::new(lowpole_hz(2500.0)));
    test_response(net2);

    let mut net3 = Net64::new(1, 1);
    net3.chain(Box::new(highpole_hz(1500.0)));
    let mut net4 = Net64::new(1, 1);
    net4.chain(Box::new(highpole_hz(500.0)));
    test_response(net3 >> net4);

    let mut net5 = Net64::new(1, 1);
    net5.chain(Box::new(highpole_hz(1500.0)));
    let mut net6 = Net64::new(1, 1);
    net6.chain(Box::new(highpole_hz(500.0)));
    test_response(net5 & net6);

    let mut net7 = Net64::new(1, 1);
    net7.chain(Box::new(highpass_hz(1000.0, 1.0)));
    test_response(Net64::enclose(Box::new(timer(0))) | net7);

    let mut net8 = Net64::new(1, 1);
    net8.chain(Box::new(highpole_hz(1500.0)));
    test_response(Net64::enclose(Box::new(zero())) + net8);

    let mut net9 = Net64::new(1, 1);
    net9.chain(Box::new(highpole_hz(2000.0)));
    test_response(Net64::enclose(Box::new(dc(0.0))) - net9);

    let mut neta = Net64::new(1, 1);
    neta.chain(Box::new(notch_hz(2500.0, 2.0)));
    test_response(Net64::enclose(Box::new(dc(1.0))) * neta);

    let mut netb = Net64::new(1, 1);
    netb.chain(Box::new(notch_hz(2500.0, 1.0)));
    test_response(netb >> lowpass_hz(1500.0, 1.0));
}
