//! Sequencer and network callbacks.

use super::net::*;
use super::sequencer::*;
use duplicate::duplicate_item;

/// Trait for user provided callbacks.
#[duplicate_item(
    f48       Sequencer48         Net48       Callback48;
    [ f64 ]   [ Sequencer64 ]   [ Net64 ]   [ Callback64 ];
    [ f32 ]   [ Sequencer32 ]   [ Net32 ]   [ Callback32 ];
)]
pub trait Callback48: Send {
    /// Update the sequencer.
    /// Current time is `t`, while `dt` is the time elapsed since the previous update.
    #[allow(unused_variables)]
    fn update_sequencer(&mut self, t: f48, dt: f48, sequencer: &mut Sequencer48) {}

    /// Update the network.
    /// Current time is `t`, while `dt` is the time elapsed since the previous update.
    #[allow(unused_variables)]
    fn update_net(&mut self, t: f48, dt: f48, net: &mut Net48) {}
}

#[duplicate_item(
    f48       Sequencer48         Net48       Callback48       CallbackContainer48;
    [ f64 ]   [ Sequencer64 ]   [ Net64 ]   [ Callback64 ]   [ CallbackContainer64 ];
    [ f32 ]   [ Sequencer32 ]   [ Net32 ]   [ Callback32 ]   [ CallbackContainer32 ];
)]
pub struct CallbackContainer48 {
    cb: Box<dyn Callback48>,
    time: f48,
    delta_time: f48,
    update_interval: f48,
}

#[duplicate_item(
    f48       Sequencer48         Net48       Callback48       CallbackContainer48;
    [ f64 ]   [ Sequencer64 ]   [ Net64 ]   [ Callback64 ]   [ CallbackContainer64 ];
    [ f32 ]   [ Sequencer32 ]   [ Net32 ]   [ Callback32 ]   [ CallbackContainer32 ];
)]
impl CallbackContainer48 {
    pub fn new(update_interval: f48, callback: Box<dyn Callback48>) -> Self {
        Self {
            cb: callback,
            time: 0.0,
            delta_time: 0.0,
            update_interval,
        }
    }
    pub fn reset(&mut self) {
        self.time = 0.0;
        self.delta_time = 0.0;
    }
    pub fn update_sequencer(&mut self, dt: f48, sequencer: &mut Sequencer48) {
        // The first update is always done at time zero.
        if self.delta_time >= self.update_interval || (self.time == 0.0 && dt > 0.0) {
            self.cb
                .update_sequencer(self.time, self.delta_time, sequencer);
            self.delta_time = 0.0;
        }
        self.delta_time += dt;
        self.time += dt;
    }
    pub fn update_net(&mut self, dt: f48, net: &mut Net48) {
        // The first update is always done at time zero.
        if self.delta_time >= self.update_interval || (self.time == 0.0 && dt > 0.0) {
            self.cb.update_net(self.time, self.delta_time, net);
            self.delta_time = 0.0;
        }
        self.delta_time += dt;
        self.time += dt;
    }
}
