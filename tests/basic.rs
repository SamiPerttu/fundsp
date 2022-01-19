#![allow(
    clippy::precedence,
    clippy::type_complexity,
    clippy::float_cmp,
    clippy::len_zero,
    clippy::double_neg,
    clippy::many_single_char_names
)]
#![allow(dead_code)]

extern crate fundsp;

use fundsp::hacker::*;

// New nodes can be defined with the following return signature.
// Declaring the full arity in the signature enables use of the node
// in further combinations, as does the full type name.
// Signatures with generic number of channels can be challenging to write.
fn split_quad() -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = U4>> {
    pass() ^ pass() ^ pass() ^ pass()
}

// Attempt to test two nodes for equality.
fn is_equal<X, Y>(rnd: &mut AttoRand, x: &mut An<X>, y: &mut An<Y>) -> bool
where
    X: AudioNode,
    Y: AudioNode<Sample = X::Sample, Inputs = X::Inputs, Outputs = X::Outputs>,
{
    // The signature constrains the structure already, try some random inputs.
    for _ in 0..1000 {
        let input =
            Frame::<X::Sample, X::Inputs>::generate(|_| X::Sample::new((rnd.gen() as i64) % 3 - 1));
        let output_x = x.tick(&input.clone());
        let output_y = y.tick(&input.clone());
        if output_x != output_y {
            return false;
        }
    }
    true
}

// Check if the outputs of a node are all unique.
fn outputs_diverge<X>(rnd: &mut AttoRand, x: &mut An<X>) -> bool
where
    X: AudioNode,
{
    assert!(x.outputs() <= 8);

    let mut diverged: u64 = 0;

    // Send 10 inputs. If none of them diverge, then we declare failure.
    for _ in 0..10 {
        let input =
            Frame::<X::Sample, X::Inputs>::generate(|_| X::Sample::new((rnd.gen() as i64) % 3 - 1));
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
    let mut rnd = AttoRand::new(0);

    // Constants.
    let mut d = constant(1.0);
    assert!(d.inputs() == 0 && d.outputs() == 1);
    assert!(d.get_mono() == 1.0);
    let mut d = constant((2.0, 3.0));
    assert!(d.inputs() == 0 && d.outputs() == 2);
    assert!(d.get_stereo() == (2.0, 3.0));

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
    assert!(is_equal(&mut rnd, &mut split_quad(), &mut split_quad()));

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
        inouts(sine() & mul(semitone(4.0)) >> sine() & mul(semitone(7.0)) >> sine()),
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
