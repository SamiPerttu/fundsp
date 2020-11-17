extern crate fundsp;

pub use fundsp::prelude::*;

// New components can be defined with the following return signature.
// Declaring the full arity in the signature enables use of the component
// in further combinations, as does the full type name.
// Signatures with generic number of channels can be challenging to write.
fn split_quad() -> Ac<impl AudioComponent<Inputs = U1, Outputs = U4>> {
    pass() & pass() & pass() & pass()
}

#[test]
fn test() {

    // Constants.
    let mut d = constant(1.0);
    assert!(d.inputs() == 0 && d.outputs() == 1);
    assert!(d.get_mono() == 1.0);
    let mut d = constant((2.0, 3.0));
    assert!(d.inputs() == 0 && d.outputs() == 2);
    assert!(d.get_stereo() == (2.0, 3.0));
    assert!(d.get_mono() == 2.0);
    let mut d = constant((4.0, 5.0, 6.0));
    assert!(d.inputs() == 0 && d.outputs() == 3);
    assert!(d.get_stereo() == (4.0, 5.0));

    assert!(split_quad().filter_mono(10.0) == 10.0);

    // Random stuff.
    let c = constant((2.0, 3.0)) * dc((2.0, 3.0));
    let e = c >> (pass() | pass());
    let mut f = e >> mul(0.5) + mul(0.5);
    assert!(f.inputs() == 0 && f.outputs() == 1);
    assert!(f.get_mono() == 6.5);

    // Test a visual cascade. The notation is slightly confusing.
    let c =     (pass()            | mul(1.0)          | add(1.0)           );
    let c = c / (pass()            | pass() * add(2.0)                      );
    let c = c / (mul(5.0)          | add(2.0)          | -add(1.0)          );
    let c = c / (mul(5.0)          + mul(5.0)          + pass()             );
    let mut c = c;
    let f = | x: f48, y: f48, z: f48 | 25.0 * x + 5.0 * y * z + 15.0 * y - z + 9.0;
    assert!(c.tick(&[0.0, 0.0, 0.0].into())[0] == f(0.0, 0.0, 0.0));
    assert!(c.tick(&[1.0, 0.0, 0.0].into())[0] == f(1.0, 0.0, 0.0));
    assert!(c.tick(&[0.0, 2.0, 0.0].into())[0] == f(0.0, 2.0, 0.0));
    assert!(c.tick(&[0.0, 0.0, 3.0].into())[0] == f(0.0, 0.0, 3.0));
    assert!(c.tick(&[2.0,-1.0, 2.0].into())[0] == f(2.0,-1.0, 2.0));
    assert!(c.tick(&[0.0, 3.0,-1.0].into())[0] == f(0.0, 3.0,-1.0));

    fn inouts<X: AudioComponent>(x: Ac<X>) -> (usize, usize) { (x.inputs(), x.outputs()) }

    // No-ops with sinks.
    assert_eq!(inouts(!-!sink()-42.0^sink()&-!!--!-sink()*3.14), (1, 0));

    // These were onverted from docs using search: ^[|] .(.*)[`].*[|] +([\d-]).+(\d-) +[|](.*)[|].*$
    // Replace with: assert_eq!(inouts($1), ($2, $3)); //$4
    assert_eq!(inouts(pass() & pass()), (1, 2)); // mono-to-stereo splitter
    assert_eq!(inouts(mul(0.5) + mul(0.5)), (2, 1)); // stereo-to-mono mixdown (inverse of mono-to-stereo splitter)
    assert_eq!(inouts(pass() & pass() & pass()), (1, 3)); // mono-to-trio splitter
    assert_eq!(inouts(sink() | zero()), (1, 1)); // replace signal with silence
    assert_eq!(inouts(mul(0.0)), (1, 1)); // -..-
    assert_eq!(inouts(mul(db_gain(3.0))), (1, 1)); // amplify signal by +3 dB
    assert_eq!(inouts(sink() | pass()), (2, 1)); // extract right channel
    assert_eq!(inouts(pass() | sink()), (2, 1)); // extract left channel
    assert_eq!(inouts(sink() | zero() | pass()), (2, 2)); // replace left channel with silence
    assert_eq!(inouts(mul(0.0) | pass()), (2, 2)); // -..-
    assert_eq!(inouts(mul((0.0, 1.0))), (2, 2)); // -..-
    assert_eq!(inouts(pass() | sink() | zero()), (2, 2)); // replace right channel with silence
    assert_eq!(inouts(pass() | mul(0.0)), (2, 2)); // -..-
    assert_eq!(inouts(mul((1.0, 0.0))), (2, 2)); // -..-
    assert_eq!(inouts(lowpass() / lowpole()), (2, 1)); // 2nd order and 1-pole lowpass filters in series (3rd order)
    assert_eq!(inouts(lowpass() / lowpass() / lowpass()), (2, 1)); // triple lowpass filter in series (6th order)
    assert_eq!(inouts(resonator() / resonator()), (3, 1)); // double resonator in series (4th order)
    assert_eq!(inouts(sine_hz(2.0) * 2.0 * 1.0 + 2.0 >> sine()), (0, 1)); // PM (phase modulation) oscillator at `f` Hz with modulation index `m`
    assert_eq!(inouts((pass() & mul(2.0)) >> sine() + sine()), (1, 1)); // frequency doubled dual sine oscillator
    assert_eq!(inouts(sine() ^ (mul(2.0) >> sine())), (1, 1)); // frequency doubled dual sine oscillator
    assert_eq!(inouts(envelope(|t| exp(-t)) * noise()), (0, 1)); // exponentially decaying white noise
    assert_eq!(inouts(!feedback(delay(0.5) * 0.5)), (1, 1)); // feedback delay of 0.5 seconds
}
