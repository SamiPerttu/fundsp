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
//! it awaits a `SoundMsg::Release` message by way of its `note_m`
//! [`AtomicCell`](https://docs.rs/crossbeam/latest/crossbeam/atomic/struct.AtomicCell.html)
//! parameter indicating when to begin the release. It signals completion of the release by placing
//! a `SoundMsg::Finished` message in the `note_m` parameter.
//! * The 'adsr_fixed()` function works similarly, except that it does have a predetermined
//! "Sustain" time interval. It will release early if it receives a `SoundMsg::Release` message.
//! It also signals completion with a `SoundMsg::Finished` message.
//!
//! The example `live_adsr.rs` is a fully functional demonstration of `adsr_live()`. It will
//! listen to messages from the first connected MIDI input device it finds, and play the
//! corresponding pitches with the volume moderated by an `adsr_live()` envelope.

use super::prelude::{envelope, lerp, An, Envelope};
use super::Float;
use crossbeam::atomic::AtomicCell;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SoundMsg {
    Play,
    Release,
    Finished,
}

pub fn adsr<F: Float>(
    attack: F,
    decay: F,
    sustain: F,
    release: F,
    release_start: Option<F>,
    note_m: Arc<AtomicCell<SoundMsg>>,
) -> An<Envelope<F, F, impl Fn(F) -> F + Sized + Clone, F>> {
    let adsr = Arc::new(AtomicCell::new(Adsr {
        attack,
        decay,
        sustain,
        release,
        release_start,
    }));
    envelope(move |t| {
        if note_m.load() == SoundMsg::Release {
            let mut adsr_inner = adsr.load();
            adsr_inner.release(t);
            adsr.store(adsr_inner);
            note_m.store(SoundMsg::Play);
        }
        match adsr.load().volume(t) {
            Some(v) => v,
            None => {
                note_m.store(SoundMsg::Finished);
                F::from_f64(0.0)
            }
        }
    })
}

#[derive(Copy, Clone, Debug)]
struct Adsr<F: Float> {
    attack: F,
    decay: F,
    sustain: F,
    release: F,
    release_start: Option<F>,
}

impl<F: Float> Adsr<F> {
    fn release(&mut self, time_now: F) {
        self.release_start = Some(time_now);
    }

    fn volume(&self, time: F) -> Option<F> {
        match self.release_start {
            None => Some(self.ads(time)),
            Some(release_start) => {
                if time < release_start {
                    Some(self.ads(time))
                } else {
                    let release_time = time - release_start;
                    if release_time > self.release {
                        None
                    } else {
                        Some(lerp(
                            self.sustain,
                            F::from_f64(0.0),
                            release_time / self.release,
                        ))
                    }
                }
            }
        }
    }

    fn ads(&self, time: F) -> F {
        if time < self.attack {
            lerp(F::from_f64(0.0), F::from_f64(1.0), time / self.attack)
        } else if time - self.attack < self.decay {
            lerp(
                F::from_f64(1.0),
                self.sustain,
                (time - self.attack) / self.decay,
            )
        } else {
            self.sustain
        }
    }
}
