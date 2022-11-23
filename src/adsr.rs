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

use super::audionode::Atomic;
use super::prelude::{clamp01, envelope2, lerp, shared, var, An, Envelope2};
use super::Float;

pub fn adsr_live<F: Float + Atomic>(
    attack: F,
    decay: F,
    sustain: F,
    release: F,
) -> An<Envelope2<F, F, impl Fn(F, F) -> F + Sized + Clone, F>> {
    let neg1 = F::from_f64(-1.0);
    let zero = F::from_f64(0.0);
    let a = shared(neg1);
    let b = shared(neg1);
    let attack_start = var(&a);
    let release_start = var(&b);
    envelope2(move |time, control| {
        if attack_start.value() < zero && control >= zero {
            attack_start.set_value(time);
            release_start.set_value(neg1);
        } else if release_start.value() < zero && control < zero {
            release_start.set_value(time);
            attack_start.set_value(neg1);
        }
        clamp01(if release_start.value() < zero {
            ads(attack, decay, sustain, time - attack_start.value())
        } else {
            releasing(sustain, release, time - release_start.value())
        })
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

fn releasing<F: Float>(sustain: F, release: F, release_time: F) -> F {
    if release_time > release {
        F::from_f64(0.0)
    } else {
        lerp(sustain, F::from_f64(0.0), release_time / release)
    }
}
