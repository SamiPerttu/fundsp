//! Frequency response system tests.

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

use fundsp::fft::*;
use fundsp::hacker32::*;
use num_complex::Complex32;

fn is_equal_response(x: Complex32, y: Complex32) -> bool {
    // This tolerance has been tuned to a minimum value that allows the tests to pass.
    let tolerance = 2.0e-4;
    let norm = (x - y).norm();
    norm <= max(1.0, max(x.norm(), y.norm())) * tolerance
}

fn test_response<X>(mut filter: X)
where
    X: AudioUnit,
{
    assert!(filter.inputs() == 1 && filter.outputs() == 1);

    filter.allocate();

    let length = 0x8000;
    let sample_rate = DEFAULT_SR;

    filter.reset();
    filter.set_sample_rate(sample_rate);

    let mut input = 1.0;
    let mut in_buffer = Vec::with_capacity(length);
    // Try to remove effect of DC by warming up the filter.
    for _i in 0..length / 2 {
        filter.filter_mono(0.0);
    }
    for _i in 0..length {
        let x = filter.filter_mono(input);
        in_buffer.push(x);
        input = 0.0;
    }

    let mut buffer = vec![Complex32::ZERO; length / 2 + 1];

    real_fft(&in_buffer, &mut buffer);

    let mut f = 10.0;
    while f <= 22_000.0 {
        let i = round(f * length as f64 / sample_rate) as usize;
        let f_i = i as f64 / length as f64 * sample_rate;
        let reported = filter.response(0, f_i).unwrap();
        let reported = Complex32::new(reported.re as f32, reported.im as f32);
        let response = buffer[i];
        if !is_equal_response(reported, response) {
            eprintln!(
                "{} Hz reported ({}, {}) measured ({}, {})",
                f_i,
                reported.norm(),
                reported.arg(),
                response.norm(),
                response.arg(),
            );
            panic!()
        }
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
    test_response(pinkpass() * dc(2.0));
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
        (delay(0.001) ^ delay(0.002)) >> reverse() >> (delay(0.003) | delay(0.007)) >> join(),
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
            >> stacki::<U8, _, _>(|i| {
                resonator_hz(1000.0 + 1000.0 * i as f32, 100.0 + 100.0 * i as f32)
            })
            >> join(),
    );
    test_response(branchf::<U5, _, _>(|t| resonator_hz(xerp(100.0, 20000.0, t), 10.0)) >> join());
    test_response(pipei::<U4, _, _>(|i| {
        bell_hz(
            1000.0 + 1000.0 * i as f32,
            (i + 1) as f32,
            db_amp((i + 6) as f32),
        )
    }));
    test_response(
        split() >> stacki::<U5, _, _>(|i| lowpole_hz(1000.0 + 1000.0 + i as f32)) >> join(),
    );
    test_response(busi::<U7, _, _>(|i| {
        lowpass_hz(1000.0 + 1000.0 * rnd1(i) as f32, 1.0 + 1.0 * rnd2(i) as f32)
    }));
    test_response(
        split::<U3>()
            >> multisplit::<U3, U3>()
            >> sumf::<U9, _, _>(|f| highshelf_hz(f * 10.0 + 10.0, 1.0 + f, 2.0 + f)),
    );
    test_response(1.0 - pan(0.5) >> join());
    test_response(0.5 * pan(0.0) >> join());
    test_response(pan(0.5) - 1.0 >> join());
    test_response(pan(-1.0) * 0.5 >> multijoin::<U1, U2>());
    let tmp = shared(0.0);
    test_response(fir((0.5, 0.5)) | timer(&tmp));
    test_response(fir((0.25, 0.5, 0.25)) >> monitor(&tmp, Meter::Sample));
    test_response(fir((0.4, 0.3, 0.2, 0.1)));
    test_response(morph_hz(1000.0, 1.0, 0.5));
    test_response(morph_hz(2000.0, 2.0, -0.5));
    test_response((1.0 + pass() | dc((1000.0, 0.5, 0.5))) >> morph());
    test_response((pass() | dc((500.0, 2.0, -1.0))) >> morph());
    test_response(biquad(0.0, 0.17149, 0.29287, 0.58574, 0.29287));
    test_response(biquad(0.033717, 0.171773, 1.059253, -0.035714, 0.181952));
    test_response(pass() + 1.0 >> lowpass_hz(1000.0, 1.0));
    test_response((pass() | dc(1.0)) >> rotate(0.5, 1.0) >> (pass() | sink()));
    test_response((dc(2.0) | pass()) >> rotate(-0.1, 0.5) >> (pass() | sink()));

    let mut net1 = Net::new(1, 1);
    net1.chain(Box::new(lowpole_hz(1500.0)));
    test_response(net1);

    let mut net2 = Net::new(1, 1);
    net2.chain(Box::new(lowpole_hz(500.0)));
    net2.chain(Box::new(lowpole_hz(2500.0)));
    test_response(net2);

    let mut net3 = Net::new(1, 1);
    net3.chain(Box::new(highpole_hz(1500.0)));
    let mut net4 = Net::new(1, 1);
    net4.chain(Box::new(highpole_hz(500.0)));
    test_response(net3 >> net4);

    let mut net5 = Net::new(1, 1);
    net5.chain(Box::new(highpole_hz(1500.0)));
    let mut net6 = Net::new(1, 1);
    net6.chain(Box::new(highpole_hz(500.0)));
    test_response(net5 & net6 & pass());

    let mut net7 = Net::new(1, 1);
    let id7 = net7.push(Box::new(highpass_hz(1000.0, 1.0)));
    net7.connect_input(0, id7, 0);
    net7.connect_output(id7, 0, 0);
    test_response(net7);

    let mut net8 = Net::new(1, 1);
    net8.chain(Box::new(highpole_hz(1500.0)));
    test_response(Net::wrap(Box::new(zero())) + net8);

    let mut net9 = Net::new(1, 1);
    net9.chain(Box::new(highpole_hz(2000.0)));
    test_response(Net::wrap(Box::new(dc(1.0))) - net9);

    let mut neta = Net::new(1, 1);
    neta.chain(Box::new(notch_hz(2500.0, 2.0)));
    test_response(Net::wrap(Box::new(dc(2.0))) * neta);

    let mut netb = Net::new(1, 1);
    netb.chain(Box::new(notch_hz(2500.0, 1.0)));
    test_response(netb * 2.0 >> lowpass_hz(1500.0, 1.0));

    let mut netc = Net::new(1, 1);
    netc.chain(Box::new(highpass_hz(5500.0, 1.0)));
    test_response(netc >> highpass_hz(2500.0, 1.0) + 1.0);

    let mut netd = Net::new(1, 1);
    netd.chain(Box::new(lowpass_hz(5000.0, 1.0)));
    test_response((netd ^ highpass_hz(3000.0, 1.0)) >> (pass() + pass()));

    let mut nete = Net::new(1, 1);
    nete.chain(Box::new(notch_hz(5000.0, 1.0)));
    test_response((nete.clone() ^ peak_hz(3000.0, 1.0)) >> (Net::wrap(Box::new(pass())) + pass()));

    let mut netf = Net::new(1, 1);
    netf.chain(Box::new(notch_hz(2000.0, 1.0)));
    test_response(
        (netf.clone() ^ pass() ^ peak_hz(1000.0, 1.0))
            >> (Net::wrap(Box::new(pass())) + pass() + pass()),
    );

    let mut netg = Net::new(1, 1);
    netg.chain(Box::new(notch_hz(2000.0, 1.0)));
    test_response(
        (netg ^ pass() ^ pass())
            >> (Net::wrap(Box::new(pass())) | pass() | pinkpass())
            >> (Net::wrap(Box::new(pinkpass())) + pass() + pass()),
    );
}

// Test various allpass filters for the allpass property.
#[test]
fn test_allpass() {
    let length = 0x8000;
    let mut spectrum = vec![Complex32::ZERO; length / 2 + 1];

    let allpasses: [Box<dyn AudioUnit>; 12] = [
        Box::new(pass()),
        Box::new(tick()),
        Box::new(allpole_delay(0.5)),
        Box::new(allpole_delay(0.8)),
        Box::new(delay(0.0001)),
        Box::new(delay(0.001)),
        Box::new(allpass_hz(1000.0, 1.0)),
        Box::new(allpass_hz(2000.0, 2.0)),
        Box::new(allnest_c(0.5, pass())),
        Box::new(allnest_c(0.6, tick())),
        Box::new(allnest_c(0.7, allpole_delay(0.5))),
        Box::new(allnest_c(-0.6, allpass_hz(3000.0, 3.0))),
    ];

    let impulse = Wave::render(DEFAULT_SR, 1.0 / DEFAULT_SR, &mut (impulse::<U1>()));

    for mut node in allpasses {
        let response = impulse.filter(length as f64 / DEFAULT_SR, &mut *node);
        real_fft(response.channel(0), &mut spectrum);
        // This tolerance has been tuned to a minimum value that allows the tests to pass.
        let tolerance = 1.0e-5;
        for s in &spectrum[1..] {
            let norm = s.norm();
            assert!(norm >= 1.0 - tolerance && norm <= 1.0 + tolerance);
        }
    }
}
