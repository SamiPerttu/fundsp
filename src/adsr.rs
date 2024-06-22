//! Attack-Decay-Sustain-Release Envelope
//!
//! This envelope is built upon the
//! [`envelope2()`](https://docs.rs/fundsp/0.9.0/fundsp/prelude/fn.envelope2.html) function to
//! control volume over time.
//!
//! When a sound begins, its volume increases from zero to one in a time interval called the
//! "Attack". It then decreases from 1.0 to the "Sustain" volume in a time interval called the
//! "Decay". It remains at the "Sustain" level until an external input indicates that the note
//! is finished, after which it decreases from the
//! "Sustain" level to 0.0 in a time interval called the "Release".
//!
//! The example [`live_adsr.rs`](https://github.com/SamiPerttu/fundsp/blob/master/examples/live_adsr.rs)
//! is a fully functional demonstration of `adsr_live()`. It will listen to messages from the first
//! connected MIDI input device it finds, and play the corresponding pitches with the volume moderated by
//! an `adsr_live()` envelope.

use super::prelude::{clamp01, delerp, envelope2, lerp, shared, var, An, EnvelopeIn, Frame, U1};
use super::Float;

pub fn adsr_live(
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
) -> An<EnvelopeIn<f32, impl FnMut(f32, &Frame<f32, U1>) -> f32 + Clone, U1, f32>> {
    let neg1 = -1.0;
    let zero = 0.0;
    let a = shared(zero);
    let b = shared(neg1);
    let attack_start = var(&a);
    let release_start = var(&b);
    envelope2(move |time, control| {
        if release_start.value() >= zero && control > zero {
            attack_start.set_value(time);
            release_start.set_value(neg1);
        } else if release_start.value() < zero && control <= zero {
            release_start.set_value(time);
        }
        let ads_value = ads(attack, decay, sustain, time - attack_start.value());
        if release_start.value() < zero {
            ads_value
        } else {
            ads_value
                * clamp01(delerp(
                    release_start.value() + release,
                    release_start.value(),
                    time,
                ))
        }
    })
}

fn ads<F: Float>(attack: F, decay: F, sustain: F, time: F) -> F {
    if time < attack {
        lerp(F::from_f64(0.0), F::from_f64(1.0), time / attack)
    } else {
        let decay_time = time - attack;
        if decay_time < decay {
            lerp(F::from_f64(1.0), sustain, decay_time / decay)
        } else {
            sustain
        }
    }
}
