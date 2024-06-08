//! FunDSP Sound Library. WIP.

use super::hacker::*;
use funutd::*;

/// Sound 001. Risset Glissando, stereo.
/// The direction of sound is up (true) or down (false).
pub fn risset_glissando(up: bool) -> An<impl AudioNode<Inputs = U0, Outputs = U2>> {
    stacki::<U40, _, _>(|i| {
        lfo(move |t| {
            let (f0, f1) = if up { (20.0, 20480.0) } else { (20480.0, 20.0) };
            let phase = (t * 0.1 + i as f64 * 10.0 / 40.0) % 10.0 / 10.0;
            let f = lerp(-1.0, 1.0, rnd1(i)) + xerp(f0, f1, phase);
            let a = smooth3(sin_hz(0.5, phase)) / a_weight(f);
            (a, f)
        }) >> pass() * sine()
    }) >> sumf::<U40, _, _>(|x| pan(lerp(-0.5, 0.5, x)))
}

/// Sound 002. Dynamical system example that harmonizes a chaotic set of pitches.
/// `speed` is rate of motion (for example, 1.0).
pub fn pebbles(speed: f32, seed: u64) -> An<impl AudioNode<Inputs = U0, Outputs = U1>> {
    let mut d = [0.0; 100];

    update(
        busi::<U100, _, _>(move |i| {
            dc(xerp(50.0, 5000.0, rnd1(i ^ seed) as f32)) >> follow(0.01) >> sine()
        }),
        0.01,
        move |t, dt, x| {
            // We receive 1 update at time zero.
            if t == 0.0 {
                for i in 0..d.len() {
                    d[i] = x.node(i).left().left().value()[0];
                }
            }
            // Fixed frequencies.
            d[0] = 110.0;
            // Write frequencies.
            for i in 0..d.len() {
                x.node_mut(i).left_mut().left_mut().set_scalar(d[i]);
            }
            // Compute "gravity".
            for i in 0..d.len() {
                for j in 0..d.len() {
                    if d[j] > d[i] {
                        continue;
                    }
                    let ratio = d[i] / d[j];
                    // Gravitate towards integer frequency ratios between partials.
                    let goal = max(1.0, round(ratio));
                    if goal - ratio < 0.0 {
                        d[i] -= d[i] * (dt * speed * 0.001) * (0.1 + ratio - goal);
                        d[j] += d[j] * (dt * speed * 0.001) * (0.1 + ratio - goal);
                    } else {
                        d[i] += d[i] * (dt * speed * 0.001) * (0.1 + goal - ratio);
                        d[j] -= d[j] * (dt * speed * 0.001) * (0.1 + goal - ratio);
                    }
                }
            }
        },
    ) >> pinkpass()
}

/// Sound 003. An 808 style bass drum, mono. `sharpness` in 0...1 is the sharpness of the click (for example, 0.2).
/// `pitch0` is click frequency in Hz (for example, 180.0). `pitch1` is the base pitch of the drum in Hz (for example, 60.0).
pub fn bassdrum(
    sharpness: f32,
    pitch0: f32,
    pitch1: f32,
) -> An<impl AudioNode<Inputs = U0, Outputs = U1>> {
    let sweep =
        lfo(move |t| xerp(pitch0 as f64, pitch1 as f64, clamp01(t * 50.0)) - 10.0 * t) >> sine();

    let volume = lfo(|t| exp(-t * 9.0));

    sweep * volume >> declick_s(xerp(0.002, 0.00002, sharpness))
}

/// Sound 004. A snare drum, mono. Different `seed` values produce small variations of the same sound.
/// `sharpness` in 0...1 is the sharpness of the attack (for example, 0.3).
pub fn snaredrum(seed: i64, sharpness: f32) -> An<impl AudioNode<Inputs = U0, Outputs = U1>> {
    let mut rnd = Rnd::from_u64(seed as u64);
    // Snare drum mode frequencies.
    let f0 = 180.0;
    let f1 = 330.0;
    let f2 = 275.0;
    let f3 = 320.0;
    let f4 = 400.0;
    let f5 = 430.0;
    let f6 = 509.0;
    let f7 = 550.0;
    let f8 = 616.0;

    let mut bend_sine = move |f: f64| {
        let f0 = f + 1.0 * (rnd.f64() * 2.0 - 1.0);
        let f1 = f + 3.0 * (rnd.f64() * 2.0 - 1.0);
        lfo(move |t| lerp(f0, f1, t)) >> sine()
    };

    let modes01 = bend_sine(f0) + bend_sine(f1);
    let modes28 = bend_sine(f2)
        + bend_sine(f3)
        + bend_sine(f4)
        + bend_sine(f5)
        + bend_sine(f6)
        + bend_sine(f7)
        + bend_sine(f8);

    let mix = modes01 * 0.2 * lfo(|t| exp(-t * 16.0))
        + modes28 * 0.1 * lfo(|t| exp(-t * 14.0))
        + pink() * 0.7 * lfo(|t| exp(-t * 12.0));

    (mix | lfo(|t| xerp(15000.0, 1000.0, t)))
        >> lowpass_q(1.0)
        >> declick_s(xerp(0.02, 0.002, sharpness))
}

/// Sound 005. Some kind of cymbal, mono. Different `seed` values produce small variations of the same sound.
pub fn cymbal(seed: i64) -> An<impl AudioNode<Inputs = U0, Outputs = U1>> {
    let mut rnd = Rnd::from_u64(seed as u64);
    let f1 = 1339.0586 + 5.0 * (rnd.f32() * 2.0 - 1.0);
    let f2 = 1703.2929 + 5.0 * (rnd.f32() * 2.0 - 1.0);
    let f3 = 2090.1314 + 5.0 * (rnd.f32() * 2.0 - 1.0);
    let f4 = 1425.6187 + 5.0 * (rnd.f32() * 2.0 - 1.0);
    let f5 = 1189.1727 + 5.0 * (rnd.f32() * 2.0 - 1.0);
    let f6 = 1954.3242 + 5.0 * (rnd.f32() * 2.0 - 1.0);
    let m1 = 54127.0;
    let m2 = 43480.0;
    let m3 = 56771.0;

    let complex = (square_hz(f1) * m1 + f2 >> square())
        + (square_hz(f3) * m2 + f4 >> square())
        + (square_hz(f5) * m3 + f6 >> square());

    (complex * lfo(|t| exp(-t * 8.0)) | lfo(|t| xerp(20000.0, 2000.0, clamp01(t))))
        >> lowpass_q(1.0)
        >> highpass_hz(2500.0, 1.0)
        >> declick_s(0.001)
}
