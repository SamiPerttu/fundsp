//! Basic component tests.

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

use fundsp::audiounit::*;
use fundsp::hacker::*;
use funutd::*;

/// Check that the stereo generator given is rendered identically
/// via `process` (block processing) and `tick` (single sample processing).
/// Also check that the generator is reset properly.
fn check_wave(mut node: impl AudioUnit) {
    node.allocate();
    let wave = Wave::render(44100.0, 0.01, &mut node);
    assert!(wave.channels() == 2);
    assert!(wave.length() == 441);
    node.reset();
    for i in 0..441 {
        let (tick_x, tick_y) = node.get_stereo();
        let process_x = wave.at(0, i);
        let process_y = wave.at(1, i);
        let tolerance = 1.0e-4;
        if tick_x - tolerance > process_x || tick_x + tolerance < process_x {
            eprintln!(
                "Left channel index {} tick {} process {}",
                i, tick_x, process_x
            );
        }
        assert!(tick_x - tolerance <= process_x && tick_x + tolerance >= process_x);
        if tick_y - tolerance > process_y || tick_y + tolerance < process_y {
            eprintln!(
                "Right channel index {} tick {} process {}",
                i, tick_y, process_y
            );
        }
        assert!(tick_y - tolerance <= process_y && tick_y + tolerance >= process_y);
    }
}

/// Check that the mono generator given is rendered identically
/// via a big block adapter (block processing) and `tick` (single sample processing).
/// Also check that the generator is reset properly.
fn check_wave_big(node: Box<dyn AudioUnit>) {
    assert_eq!(node.inputs(), 0);
    assert_eq!(node.outputs(), 1);
    let mut wave = Wave::with_capacity(1, 44100.0, 441);
    wave.resize(441);
    let mut big = BigBlockAdapter::new(node);
    big.allocate();
    big.process_big(441, &[], &mut [wave.channel_mut(0)]);
    big.reset();
    for i in 0..441 {
        let tick_x = big.get_mono();
        let process_x = wave.at(0, i);
        let tolerance = 1.0e-4;
        assert!(tick_x - tolerance <= process_x && tick_x + tolerance >= process_x);
    }
}

/// Check that the stereo filter given is rendered identically
/// via `process` (block processing) and `tick` (single sample processing).
/// Also check that the filter is reset properly.
fn check_wave_filter(input: &Wave, mut node: impl AudioUnit) {
    node.allocate();
    let wave = input.filter(1.1, &mut node);
    assert!(wave.channels() == 2);
    assert!(wave.length() == 44100 + 4410);
    node.reset();
    for i in 0..44100 {
        let (tick_x, tick_y) = node.filter_stereo(input.at(0, i), input.at(1, i));
        let process_x = wave.at(0, i);
        let process_y = wave.at(1, i);
        let tolerance = 1.0e-4;
        assert!(tick_x - tolerance <= process_x && tick_x + tolerance >= process_x);
        assert!(tick_y - tolerance <= process_y && tick_y + tolerance >= process_y);
    }
}

/// Attempt to test two nodes for equality.
fn is_equal<X, Y>(rnd: &mut Rnd, x: &mut An<X>, y: &mut An<Y>) -> bool
where
    X: AudioNode,
    Y: AudioNode<Inputs = X::Inputs, Outputs = X::Outputs>,
{
    // The signature constrains the structure already, try some random inputs.
    for _ in 0..1000 {
        let input = Frame::<f32, X::Inputs>::generate(|_| (rnd.i64() % 3 - 1) as f32);
        let output_x = x.tick(&input.clone());
        let output_y = y.tick(&input.clone());
        if output_x != output_y {
            return false;
        }
    }
    true
}

/// Attempt to test two stereo filters for equality.
fn is_equal_unit<X, Y>(rnd: &mut Rnd, x: &mut X, y: &mut Y) -> bool
where
    X: AudioUnit,
    Y: AudioUnit,
{
    assert!(2 == y.inputs() && x.inputs() == 2);
    assert!(x.outputs() == 2 && 2 == y.outputs());

    for _ in 0..1000 {
        let input0 = (rnd.u64() & 0xf) as f32;
        let input1 = (rnd.u64() & 0xf) as f32;
        let output_x = x.filter_stereo(input0, input1);
        let output_y = y.filter_stereo(input0, input1);
        if output_x != output_y {
            return false;
        }
    }
    true
}

/// Check that the outputs of a node are all unique.
fn outputs_diverge<X>(rnd: &mut Rnd, x: &mut An<X>) -> bool
where
    X: AudioNode,
{
    assert!(x.outputs() <= 8);

    let mut diverged: u64 = 0;

    // Send 10 inputs. If none of them diverge, then we declare failure.
    for _ in 0..10 {
        let input = Frame::<f32, X::Inputs>::generate(|_| (rnd.i64() % 3 - 1) as f32);
        let output = x.tick(&input);
        for i in 0..x.outputs() {
            for j in 0..x.outputs() {
                if output[i] != output[j] {
                    diverged |= 1 << (i * 8 + j);
                }
            }
        }
    }

    for i in 0..x.outputs() {
        for j in 0..x.outputs() {
            if i != j && diverged & (1 << (i * 8 + j)) == 0 {
                return false;
            }
        }
    }
    true
}

#[test]
fn test_basic() {
    let mut rnd = Rnd::new();

    // Wave rendering, tick vs. process rendering, node reseting.
    check_wave(noise() >> declick() | noise() + noise());
    check_wave(noise() * noise() | busi::<U4, _, _>(|i| mls_bits(10 + i)));
    check_wave(noise() & noise() | sine_hz(440.0) & -noise());
    check_wave(
        lfo(|t| xerp(110.0, 220.0, clamp01(t))) >> sine()
            | (envelope(|t| xerp(220.0, 440.0, clamp01(t))) >> pass() >> sine()) & mls(),
    );
    check_wave(dc(1.0) >> lfo2(|t, x| t * x) | dc(1.0) >> envelope2(|t, x| t * x));
    check_wave(
        dc((1.0, 2.0)) >> lfo3(|t, x, y| t * x * y)
            | dc((1.0, 2.0)) >> envelope3(|t, x, y| t * x * y),
    );
    check_wave(dc((110.0, 220.0)) >> multipass() >> -stackf::<U2, _, _>(|f| (f - 0.5) * sine()));
    check_wave(
        dc((110.0, 220.0, 440.0, 880.0)) >> multipass() >> (sink() | -sine() | sink() | sine()),
    );
    check_wave(dc((110.0, 220.0)) >> declick_s(0.1) + pass() >> (saw() ^ dsf_square_r(0.9)));
    check_wave(
        dc((20.0, 40.0)) >> reverse() >> pass() * pass() >> (dsf_saw_r(0.999) ^ square() * 0.2),
    );
    check_wave(
        dc((880.0, 440.0)) >> pass() - pass() >> branchf::<U2, _, _>(|f| (f - 0.5) * triangle()),
    );
    check_wave(
        (noise() | dc(440.0)) >> pipei::<U3, _, _>(|_| !lowpole()) >> lowpole()
            | ((mls() | dc(880.0)) >> !butterpass() >> butterpass()),
    );
    check_wave(
        (noise() | dc(440.0)) >> pipei::<U4, _, _>(|_| !peak_q(1.0)) >> bell_q(1.0, 2.0)
            | ((mls() | dc(880.0)) >> !lowshelf_q(1.0, 0.5) >> highshelf_q(2.0, 2.0)),
    );
    check_wave(
        (noise() | dc(440.0)) >> pipei::<U4, _, _>(|_| !lowpass_q(1.0)) >> highpass_q(1.0)
            | ((mls() | dc(880.0)) >> !bandpass_q(1.0) >> notch_q(2.0)),
    );
    check_wave(
        dc((440.0, 880.0)) >> multisplit::<U2, U5>() >> sumi::<U10, _, _>(|_| saw() * 0.2)
            | noise(),
    );
    check_wave(
        dc((440.0, 880.0)) >> multisplit::<U2, U3>() >> multijoin::<U2, U3>() >> (sine() | sine()),
    );
    check_wave((noise() >> split::<U16>() >> join()) | (noise() >> split::<U11>() >> join()));
    check_wave_big(Box::new(dc((110.0, 0.5)) >> pulse() * 0.2 >> delay(0.1)));
    check_wave_big(Box::new(envelope(|t| exp(-t * 10.0))));

    let feedback1 = noise()
        >> Net::wrap(Box::new(FeedbackUnit::new(
            0.01,
            Box::new(0.5 * lowpass_hz(1000.0, 1.0)),
        )));
    let feedback2 = noise()
        >> Net::wrap(Box::new(FeedbackUnit::new(
            0.001,
            Box::new(0.5 * highpass_hz(1000.0, 1.0)),
        )));
    check_wave(feedback1 | feedback2);

    let mut sequencer = Sequencer::new(true, 2);
    sequencer.push(
        0.1,
        0.2,
        Fade::Smooth,
        0.01,
        0.0,
        Box::new(noise() | sine_hz(220.0)),
    );
    sequencer.push(
        0.3,
        0.4,
        Fade::Smooth,
        0.09,
        0.08,
        Box::new(sine_hz(110.0) | noise()),
    );
    sequencer.push(0.25, 0.5, Fade::Power, 0.0, 0.01, Box::new(mls() | noise()));
    sequencer.push(0.6, 0.7, Fade::Power, 0.02, 0.03, Box::new(noise() | mls()));
    check_wave(sequencer);

    let mut net = Net::new(0, 2);
    let id = net.push(Box::new(
        noise() >> moog_hz(1500.0, 0.8) | noise() >> moog_hz(500.0, 0.4),
    ));
    net.connect_output(id, 0, 0);
    net.connect_output(id, 1, 1);
    net.check();
    check_wave(net);

    let mut net = Net::new(0, 2);
    net.chain(Box::new(noise() | noise()));
    net.chain(Box::new(moog_hz(1500.0, 0.5) | moog_hz(1000.0, 0.6)));
    net.chain(Box::new(lowpole_hz(1000.0) | lowpole_hz(500.0)));
    net.check();
    check_wave(net);

    let mut net = Net::new(0, 2);
    net.chain(Box::new(noise()));
    net.chain(Box::new(lowpole_hz(1000.0) ^ lowpole_hz(500.0)));
    net.chain(Box::new(lowpole_hz(1000.0) | lowpole_hz(500.0)));
    net.check();
    check_wave(net);

    check_wave((noise() | envelope(|t| spline_noise(1, t * 10.0))) >> panner());
    check_wave(impulse::<U2>());

    let dc42 = Net::wrap(Box::new(dc(42.)));
    let dcs = dc42.clone() | dc42;
    let reverb = Net::wrap(Box::new(reverb_stereo(40., 5., 1.)));
    let filter = Net::wrap(Box::new(lowpass_hz(1729., 1.)));
    let filters = filter.clone() | filter;
    let net = dcs >> reverb >> filters;
    net.check();
    check_wave(net);

    // Wave filtering, tick vs. process rendering, node reseting.
    let input = Wave::render(44100.0, 1.0, &mut (noise() | noise()));
    check_wave_filter(&input, butterpass_hz(1000.0) | lowpole_hz(100.0));
    check_wave_filter(&input, allpole_delay(0.5) | highpole_hz(500.0));
    check_wave_filter(&input, pluck(60.0, 0.9, 0.8) | pluck(110.0, 0.5, 0.1));
    check_wave_filter(
        &input,
        (pass() | dc((2000.0, 5.0, 0.8))) >> morph() | morph_hz(440.0, 1.0, 0.0),
    );
    check_wave_filter(&input, lowrez_hz(440.0, 0.5) | bandrez_hz(440.0, 0.5));
    check_wave_filter(
        &input,
        resonator_hz(440.0, 110.0) | resonator_hz(880.0, 110.0),
    );

    // Constants.
    let mut d = constant(1.0);
    assert!(d.inputs() == 0 && d.outputs() == 1);
    assert!(d.get_mono() == 1.0);
    let mut d = constant((2.0, 3.0));
    assert!(d.inputs() == 0 && d.outputs() == 2);
    assert!(d.get_stereo() == (2.0, 3.0));
    assert!(d.get_mono() == 2.5);

    // Random stuff.
    let c = constant((2.0, 3.0)) * dc((2.0, 3.0));
    let e = c >> (pass() | pass());
    let mut f = e >> mul(0.5) + mul(0.5);
    assert!(f.inputs() == 0 && f.outputs() == 1);
    assert!(f.get_mono() == 6.5);

    fn inouts<X: AudioNode>(x: An<X>) -> (usize, usize) {
        (x.inputs(), x.outputs())
    }

    // Equivalent networks.
    let v = 1.0;
    let w = -2.0;
    let x = 3.0;
    let y = -4.0;
    let z = 5.0;

    // Test bus vs. branch equivalence.
    assert!(is_equal(
        &mut rnd,
        &mut ((pass() ^ mul(y)) >> add(z) + sub(x)),
        &mut (add(z) & mul(y) >> sub(x))
    ));
    assert!(is_equal(
        &mut rnd,
        &mut ((pass() ^ mul(y) ^ add(w)) >> add(z) + sub(x) + mul(y)),
        &mut (add(z) & mul(y) >> sub(x) & add(w) >> mul(y))
    ));
    assert!(is_equal(
        &mut rnd,
        &mut ((pass() ^ mul(y) ^ add(w) ^ sub(x)) >> add(z) + sub(x) + mul(y) + add(z)),
        &mut (add(z) & mul(y) >> sub(x) & add(w) >> mul(y) & sub(x) >> add(z))
    ));

    // Nodes vs. networks.
    let mut pass_through = pass() | pass();
    let mut pass_through_net = Net::new(2, 2);
    pass_through_net.pass_through(0, 0);
    pass_through_net.pass_through(1, 1);
    assert!(is_equal_unit(
        &mut rnd,
        &mut pass_through,
        &mut pass_through_net
    ));
    pass_through_net.check();

    let mut swap_through = reverse::<U2>();
    let mut swap_through_net = Net::new(2, 2);
    swap_through_net.pass_through(0, 1);
    swap_through_net.pass_through(1, 0);
    assert!(is_equal_unit(
        &mut rnd,
        &mut swap_through,
        &mut swap_through_net
    ));
    swap_through_net.check();

    let mut multiply_2_3 = mul(2.0) | mul(3.0);
    let mut multiply_net = Net::new(2, 2);
    let id0 = multiply_net.push(Box::new(mul(2.0)));
    let idd = multiply_net.push(Box::new(sink()));
    let ide = multiply_net.push(Box::new(sine()));
    let id1 = multiply_net.push(Box::new(mul(3.0)));
    multiply_net.connect_input(0, id0, 0);
    multiply_net.connect_input(1, id1, 0);
    multiply_net.connect_output(id0, 0, 0);
    multiply_net.connect_output(id1, 0, 1);
    assert!(is_equal_unit(
        &mut rnd,
        &mut multiply_2_3,
        &mut multiply_net
    ));
    multiply_net.remove(idd);
    multiply_net.remove(ide);
    multiply_net.check();
    assert!(is_equal_unit(
        &mut rnd,
        &mut multiply_2_3,
        &mut multiply_net
    ));

    let mut add_2_3 = add((2.0, 3.0));
    let mut add_net = Net::new(2, 2);
    let id0 = add_net.push(Box::new(add((2.0, 3.0))));
    let idd = add_net.push(Box::new(zero()));
    let id1 = add_net.push(Box::new(multipass::<U2>()));
    assert!(id0 != idd && id0 != id1 && idd != id1);
    add_net.remove(idd);
    add_net.pipe_input(id0);
    add_net.pipe(id0, id1);
    add_net.pipe_output(id1);
    assert!(is_equal_unit(&mut rnd, &mut add_2_3, &mut add_net));
    add_net.check();

    // Test multichannel constants vs. stacked constants.
    assert!(is_equal(
        &mut rnd,
        &mut (dc(w) | dc(x)),
        &mut (constant((w, x)))
    ));
    assert!(is_equal(
        &mut rnd,
        &mut (dc(x) | dc(y) | dc(z)),
        &mut (constant((x, y, z)))
    ));
    assert!(is_equal(
        &mut rnd,
        &mut (dc(x) | dc(y) | dc(z) | dc(w)),
        &mut (constant((x, y, z, w)))
    ));
    assert!(is_equal(
        &mut rnd,
        &mut (dc(w) | dc(v) | dc(x) | dc(y) | dc(z)),
        &mut (constant((w, v, x, y, z)))
    ));
    assert!(is_equal(
        &mut rnd,
        &mut (dc((w, x)) | dc((y, z, w))),
        &mut (constant((w, x, y, z, w)))
    ));

    // Test sinks and zeros.
    assert!(is_equal(
        &mut rnd,
        &mut (sink() | sink() | zero() | zero()),
        &mut (zero() | zero() | sink() | sink())
    ));
    assert!(is_equal(
        &mut rnd,
        &mut (sink() | zero() | sink() | zero() | zero() | sink() | zero()),
        &mut (zero() | zero() | zero() | sink() | sink() | zero() | sink())
    ));

    // Test delays.
    assert!(is_equal(
        &mut rnd,
        &mut (tick() >> tick() >> tick()),
        &mut (delay(3.0 / 44100.0))
    ));
    assert!(is_equal(
        &mut rnd,
        &mut (tick() >> tick() >> tick() >> tick() >> tick()),
        &mut (delay(5.0 / 44100.0))
    ));

    // Test pseudorandom phase: generator outputs should diverge.
    assert!(outputs_diverge(
        &mut rnd,
        &mut (noise()
            | (!zero() >> noise())
            | noise()
            | (!zero() >> noise())
            | noise()
            | noise()
            | noise())
    ));
    assert!(outputs_diverge(
        &mut rnd,
        &mut (noise()
            ^ noise()
            ^ noise() & zero()
            ^ noise()
            ^ (noise() >> pass())
            ^ noise()
            ^ noise())
    ));
    assert!(outputs_diverge(
        &mut rnd,
        &mut (mls()
            | (!zero() >> mls())
            | (!zero() >> !zero() >> mls())
            | (mls() >> pass() >> pass())
            | (mls() >> pass())
            | mls())
    ));
    assert!(outputs_diverge(
        &mut rnd,
        &mut (mls() + zero() ^ (mls() >> pass())
            | (mls() >> pass()) ^ mls()
            | mls() & zero() & zero()
            | mls())
    ));
    assert!(outputs_diverge(
        &mut rnd,
        &mut ((sine_hz(1.0) >> pass())
            | sine_hz(1.0)
            | (sine_hz(1.0) >> pass() >> pass())
            | sine_hz(1.0)
            | sine_hz(1.0))
    ));
    assert!(outputs_diverge(
        &mut rnd,
        &mut (sine_hz(1.0) ^ sine_hz(1.0) ^ sine_hz(1.0) | sine_hz(1.0) | sine_hz(1.0))
    ));
    assert!(outputs_diverge(
        &mut rnd,
        &mut (noise() | noise() & zero() | noise() & zero() | noise())
    ));
    assert!(outputs_diverge(
        &mut rnd,
        &mut (noise() ^ (!zero() >> noise()) ^ (!zero() >> noise()) ^ noise())
    ));
    assert!(outputs_diverge(
        &mut rnd,
        &mut (mls() + zero() | mls() + zero() | mls() + zero())
    ));
    assert!(outputs_diverge(&mut rnd, &mut (mls() ^ mls() ^ mls())));
    assert!(outputs_diverge(
        &mut rnd,
        &mut (sine_hz(1.0) - zero() | sine_hz(1.0) - zero())
    ));
    assert!(outputs_diverge(
        &mut rnd,
        &mut (sine_hz(1.0) ^ sine_hz(1.0))
    ));
    assert!(outputs_diverge(&mut rnd, &mut (noise() | noise())));
    assert!(outputs_diverge(&mut rnd, &mut (mls() | mls())));
    assert!(outputs_diverge(&mut rnd, &mut (saw() | saw())));
    assert!(outputs_diverge(&mut rnd, &mut (square() | square())));
    assert!(outputs_diverge(&mut rnd, &mut (triangle() | triangle())));
    assert!(outputs_diverge(&mut rnd, &mut (pulse() | pulse())));
    let net1 = Net::wrap(Box::new(noise()));
    let net2 = Net::wrap(Box::new(noise()));
    assert!(outputs_diverge(
        &mut rnd,
        &mut (unit::<U0, U2>(Box::new(net1 | net2)))
    ));

    // No-ops with sinks.
    assert_eq!(inouts(--sink() - 42.0 ^ sink() & ---sink() * 3.15), (1, 0));

    // These were converted from docs using search: ^[|] .(.*)[`].*[|] +([\d-]).+(\d-) +[|](.*)[|].*$
    // Replace with: assert_eq!(inouts($1), ($2, $3)); //$4
    assert_eq!(inouts(pass() ^ pass()), (1, 2)); // mono-to-stereo splitter
    assert_eq!(inouts(mul(0.5) + mul(0.5)), (2, 1)); // stereo-to-mono mixdown (inverse of mono-to-stereo splitter)
    assert_eq!(inouts(pass() ^ pass() ^ pass()), (1, 3)); // mono-to-trio splitter
    assert_eq!(inouts(sink() | zero()), (1, 1)); // replace signal with silence
    assert_eq!(inouts(mul(0.0)), (1, 1)); // -..-
    assert_eq!(inouts(mul(db_amp(3.0))), (1, 1)); // amplify signal by +3 dB
    assert_eq!(inouts(sink() | pass()), (2, 1)); // extract right channel
    assert_eq!(inouts(pass() | sink()), (2, 1)); // extract left channel
    assert_eq!(inouts(sink() | zero() | pass()), (2, 2)); // replace left channel with silence
    assert_eq!(inouts(mul(0.0) | pass()), (2, 2)); // -..-
    assert_eq!(inouts(mul((0.0, 1.0))), (2, 2)); // -..-
    assert_eq!(inouts(pass() | sink() | zero()), (2, 2)); // replace right channel with silence
    assert_eq!(inouts(pass() | mul(0.0)), (2, 2)); // -..-
    assert_eq!(inouts(mul((1.0, 0.0))), (2, 2)); // -..-
    assert_eq!(inouts(!butterpass() >> lowpole()), (2, 1)); // 2nd order and 1-pole lowpass filters in series (3rd order)
    assert_eq!(
        inouts(!butterpass() >> !butterpass() >> butterpass()),
        (2, 1)
    ); // triple lowpass filter in series (6th order)
    assert_eq!(inouts(!resonator() >> resonator()), (3, 1)); // double resonator in series (4th order)
    assert_eq!(inouts(sine_hz(2.0) * 2.0 * 1.0 + 2.0 >> sine()), (0, 1)); // PM (phase modulation) oscillator at `f` Hz with modulation index `m`
    assert_eq!(inouts((pass() ^ mul(2.0)) >> sine() + sine()), (1, 1)); // frequency doubled dual sine oscillator
    assert_eq!(inouts(sine() & mul(2.0) >> sine()), (1, 1)); // frequency doubled dual sine oscillator
    assert_eq!(inouts(envelope(|t| exp(-t)) * noise()), (0, 1)); // exponentially decaying white noise
    assert_eq!(inouts(feedback(delay(0.5) * 0.5)), (1, 1)); // feedback delay of 0.5 seconds
    assert_eq!(
        inouts(sine() & mul(semitone_ratio(4.0)) >> sine() & mul(semitone_ratio(7.0)) >> sine()),
        (1, 1)
    ); // major chord
    assert_eq!(
        inouts(
            dc(midi_hz(69.0)) >> sine() & dc(midi_hz(73.0)) >> sine() & dc(midi_hz(76.0)) >> sine()
        ),
        (0, 1)
    ); // A major chord generator
    assert_eq!(inouts(!zero()), (0, 0)); //  A null unit. Stacking it with a graph modifies its sound subtly, as the hash is altered.
    assert_eq!(inouts(!-!!!--!!!-!!--!zero()), (0, 0)); // Hot-rodded null unit with a custom hash. Uses more electricity.
}

#[test]
/// Test a pass-through resynthesizer.
fn test_resynth() {
    // Sanity test FFT roundtrip.
    const WINDOW: usize = 16;
    let mut rnd = Rnd::from_u64(1);
    let fft_input: [f32; WINDOW] = core::array::from_fn(|_| rnd.f32());
    let mut fft_output = [Complex32::ZERO; WINDOW];
    fft::real_fft(&fft_input, &mut fft_output[0..WINDOW / 2 + 1]);
    let mut fft_round = [Complex32::ZERO; WINDOW];
    for i in WINDOW / 2 + 1..WINDOW {
        fft_output[i] = fft_output[WINDOW - i].conj();
    }
    fft::inverse_fft(&fft_output, &mut fft_round);
    for i in 0..WINDOW {
        let tolerance = 1.0e-6;
        let norm = (fft_input[i] - fft_round[i].re).abs() + fft_round[i].im.abs();
        assert!(norm <= tolerance);
    }

    let window = 32;
    let mut synth: An<Resynth<U1, U1, _>> = resynth(window, |fft| {
        for i in 0..fft.bins() {
            fft.set(0, i, fft.at(0, i));
        }
    });

    let duration = 16.0 / DEFAULT_SR;

    let input = Wave::render(DEFAULT_SR, duration, &mut noise());
    let output = input.filter_latency(duration, &mut synth);

    // The pass-through resynthesizer should reconstruct inputs exactly after `window` samples
    // (plus another `window` samples of latency).
    // That is when the 4 overlapping windows are all active.
    for i in window..input.length() {
        let tolerance = 1.0e-6;
        assert!(
            input.at(0, i) - tolerance <= output.at(0, i)
                && input.at(0, i) + tolerance >= output.at(0, i)
        );
    }
}
