//! User provided callbacks.

use duplicate::duplicate_item;

#[duplicate_item(
    f48       Callback48;
    [ f64 ]   [ Callback64 ];
    [ f32 ]   [ Callback32 ];
)]
pub struct Callback48<T> {
    callback: Box<dyn FnMut(f48, f48, &mut T) + Send>,
    update_interval: f48,
    time: f48,
    delta_time: f48,
}

#[duplicate_item(
    f48       Callback48;
    [ f64 ]   [ Callback64 ];
    [ f32 ]   [ Callback32 ];
)]
impl<T> Callback48<T> {
    /// Create a new callback from a closure. Time between updates is indicated by `update_interval`,
    /// which is in seconds.
    pub fn new(update_interval: f48, callback: Box<dyn FnMut(f48, f48, &mut T) + Send>) -> Self {
        Self {
            callback,
            update_interval,
            time: 0.0,
            delta_time: 0.0,
        }
    }

    /// Reset time to zero.
    pub fn reset(&mut self) {
        self.time = 0.0;
        self.delta_time = 0.0;
    }

    /// Indicate that time is about to elapse.
    pub fn update(&mut self, dt: f48, object: &mut T) {
        // The first update is always done at time zero.
        if self.delta_time >= self.update_interval || (self.time == 0.0 && dt > 0.0) {
            (self.callback)(self.time, self.delta_time, object);
            self.delta_time = 0.0;
        }
        self.delta_time += dt;
        self.time += dt;
    }

    /// Set current time. The next update will be issued at the new time.
    pub fn set_time(&mut self, time: f48) {
        self.time = time;
        if time > 0.0 {
            self.delta_time = self.update_interval;
        } else {
            self.delta_time = 0.0;
        }
    }
}
