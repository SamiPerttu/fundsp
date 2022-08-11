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

/// Sound 002. Dynamical system example.
pub fn pebbles() -> An<impl AudioNode<Sample = f64, Inputs = U0, Outputs = U1>> {
    let mut d = [0.0f64; 100];

    system(
        bus::<U100, _, _>(move |i| dc(xerp(50.0, 5000.0, rnd(i))) >> follow(0.01) >> sine()),
        0.02,
        move |t, dt, x| {
            if t == 0.0 {
                for i in 0..d.len() {
                    d[i] = x.node(i).left().left().value()[0];
                }
            }
            for i in 0..d.len() {
                for j in 0..d.len() {
                    if d[j] > d[i] {
                        continue;
                    }
                    let ratio = d[i] / d[j];
                    // Gravitate towards integer frequency ratios between partials.
                    let goal = max(1.0, round(ratio));
                    if goal - ratio < 0.0 {
                        d[i] -= d[i] * (dt * 0.01) * (0.5 + ratio - goal);
                        d[j] += d[j] * (dt * 0.01) * (0.5 + ratio - goal);
                    } else {
                        d[i] += d[i] * (dt * 0.01) * (0.5 + goal - ratio);
                        d[j] -= d[j] * (dt * 0.01) * (0.5 + goal - ratio);
                    }
                }
            }
            for i in 0..d.len() {
                x.node_mut(i)
                    .left_mut()
                    .left_mut()
                    .set_value(Frame::from([d[i]]));
            }
        },
    ) >> pinkpass()
}
