//! Attack-Decay-Sustain-Release Envelopes
//!
//! These envelopes are built upon the
//! [`envelope()`](https://docs.rs/fundsp/0.9.0/fundsp/prelude/fn.envelope.html) function to
//! control volume over time.
//!
//! When a sound begins, its volume increases from zero to one in a time interval called the
//! "Attack". It then decreases from 1.0 to the "Sustain" volume in a time interval called the
//! "Decay". It remains at the "Sustain" level for a while, after which it decreases from the
//! "Sustain" level to 0.0 in a time interval called the "Release".
//!
//! Two envelopes are available: `adsr_live()` and `adsr_fixed()`.
//! * The `adsr_live()` function does not have a predetermined "Sustain" time interval. Instead,
//! it awaits a positive value in its `releasing` parameter to indicate when to begin the release.
//! It signals completion of the release by storing a positive value in the `finished` parameter.
//! * The 'adsr_fixed()` function works similarly, except that it does have a predetermined
//! "Sustain" time interval. It also signals completion with a positive value in the `finished`
//! parameter.
//!
//! The example `live_adsr.rs` is a fully functional demonstration of `adsr_live()`. It will
//! listen to messages from the first connected MIDI input device it finds, and play the
//! corresponding pitches with the volume moderated by an `adsr_live()` envelope.

use super::audionode::{Tag, Var, Variable};
use super::prelude::{envelope, lerp, var, An, Envelope};
use super::Float;

pub const ADSR_1: Tag = 100000;
pub const ADSR_2: Tag = ADSR_1 + 1;

pub fn adsr<F: Float + Variable>(
    attack: F,
    decay: F,
    sustain: F,
    release: F,
    release_start: Option<F>,
    releasing: An<Var<F>>,
    finished: An<Var<F>>,
) -> An<Envelope<F, F, impl Fn(F) -> F + Sized + Clone, F>> {
    let release_start = var(ADSR_1, release_start.unwrap_or(F::from_f64(-1.0)));
    envelope(move |time| {
        if releasing.value() > F::from_f64(0.0) && release_start.value() < F::from_f64(0.0) {
            release_start.set_value(time);
        }
        if release_start.value() < F::from_f64(0.0) || time < release_start.value() {
            ads(attack, decay, sustain, time)
        } else {
            let release_time = time - release_start.value();
            if release_time > release {
                finished.set_value(F::from_f64(1.0));
                F::from_f64(0.0)
            } else {
                lerp(sustain, F::from_f64(0.0), release_time / release)
            }
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
