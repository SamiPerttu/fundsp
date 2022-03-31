//! FunDSP Sound Library.

use super::hacker::*;

/// Sound 001. Risset Glissando.
pub fn risset_glissando() -> An<impl AudioNode<Sample = f64, Inputs = U0, Outputs = U2>> {
    stack::<U40, _, _>(|i| {
        lfo(move |t| {
            let f = lerp(-1.0, 1.0, rnd(i))
                + xerp(20.0, 20480.0, (t * 0.1 + i as f64 / 40.0) % 10.0 / 10.0);
            let a = smooth3(sin_hz(0.05, (t * 0.1 + i as f64 * 0.5) % 10.0));
            (a, f)
        }) >> pass() * sine()
    }) >> multijoin::<U2, U20>()
        >> (pinkpass() | pinkpass())
}
