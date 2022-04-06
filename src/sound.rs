//! FunDSP Sound Library.

use super::hacker::*;

/// Sound 001. Risset Glissando, stereo.
/// The direction of sound is up (true) or down (false).
pub fn risset_glissando(up: bool) -> An<impl AudioNode<Sample = f64, Inputs = U0, Outputs = U2>> {
    stack::<U40, _, _>(|i| {
        lfo(move |t| {
            let (f0, f1) = if up { (20.0, 20480.0) } else { (20480.0, 20.0) };
            let phase = (t * 0.1 + i as f64 * 10.0 / 40.0) % 10.0 / 10.0;
            let f = lerp(-1.0, 1.0, rnd(i)) + xerp(f0, f1, phase);
            let a = smooth3(sin_hz(0.5, phase));
            (a, f)
        }) >> pass() * sine()
    }) >> multijoin::<U2, U20>()
        >> (pinkpass() | pinkpass())
}
