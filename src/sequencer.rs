//! Sequencer unit.

use super::audiounit::*;
use super::buffer::*;
use super::callback::*;
use super::math::*;
use super::signal::*;
use super::*;
use duplicate::duplicate_item;
use std::cmp::Eq;
use std::cmp::Ord;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

const ID: u64 = 64;

#[duplicate_item(
    f48       Event48       AudioUnit48;
    [ f64 ]   [ Event64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Event32 ]   [ AudioUnit32 ];
)]
pub struct Event48 {
    pub unit: Box<dyn AudioUnit48>,
    pub start_time: f48,
    pub end_time: f48,
    pub fade_in: f48,
    pub fade_out: f48,
}

#[duplicate_item(
    f48       Event48       AudioUnit48;
    [ f64 ]   [ Event64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Event32 ]   [ AudioUnit32 ];
)]
impl Event48 {
    pub fn new(
        unit: Box<dyn AudioUnit48>,
        start_time: f48,
        end_time: f48,
        fade_in: f48,
        fade_out: f48,
    ) -> Self {
        Self {
            unit,
            start_time,
            end_time,
            fade_in,
            fade_out,
        }
    }
}

#[duplicate_item(
    f48       Event48       AudioUnit48;
    [ f64 ]   [ Event64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Event32 ]   [ AudioUnit32 ];
)]
impl PartialEq for Event48 {
    fn eq(&self, other: &Event48) -> bool {
        self.start_time == other.start_time
    }
}

impl Eq for Event32 {}
impl Eq for Event64 {}

#[duplicate_item(
    f48       Event48       AudioUnit48;
    [ f64 ]   [ Event64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Event32 ]   [ AudioUnit32 ];
)]
impl PartialOrd for Event48 {
    fn partial_cmp(&self, other: &Event48) -> Option<Ordering> {
        other.start_time.partial_cmp(&self.start_time)
    }
}

#[duplicate_item(
    f48       Event48       AudioUnit48;
    [ f64 ]   [ Event64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Event32 ]   [ AudioUnit32 ];
)]
impl Ord for Event48 {
    fn cmp(&self, other: &Self) -> Ordering {
        if other.start_time > self.start_time {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

#[duplicate_item(
    f48       Event48       AudioUnit48       fade_in48;
    [ f64 ]   [ Event64 ]   [ AudioUnit64 ]   [ fade_in64 ];
    [ f32 ]   [ Event32 ]   [ AudioUnit32 ]   [ fade_in32 ];
)]
fn fade_in48(
    sample_duration: f48,
    time: f48,
    end_time: f48,
    start_index: usize,
    end_index: usize,
    fade_duration: f48,
    fade_start_time: f48,
    output: &mut [&mut [f48]],
) {
    let fade_end_time = fade_start_time + fade_duration;
    if fade_duration > 0.0 && fade_end_time > time {
        let fade_end_i = if fade_end_time >= end_time {
            end_index
        } else {
            round((fade_end_time - time) / sample_duration) as usize
        };
        let fade_d = sample_duration / fade_duration;

        let fade_phase = delerp(
            fade_start_time,
            fade_end_time,
            time + start_index as f48 * sample_duration,
        );
        for channel in 0..output.len() {
            let mut fade = fade_phase;
            for i in 0..fade_end_i {
                output[channel][i] *= smooth5(fade);
                fade += fade_d;
            }
        }
    }
}

#[duplicate_item(
    f48       Event48       AudioUnit48       fade_out48;
    [ f64 ]   [ Event64 ]   [ AudioUnit64 ]   [ fade_out64 ];
    [ f32 ]   [ Event32 ]   [ AudioUnit32 ]   [ fade_out32 ];
)]
fn fade_out48(
    sample_duration: f48,
    time: f48,
    end_time: f48,
    _start_index: usize,
    end_index: usize,
    fade_duration: f48,
    fade_end_time: f48,
    output: &mut [&mut [f48]],
) {
    let fade_start_time = fade_end_time - fade_duration;
    if fade_duration > 0.0 && fade_start_time < end_time {
        let fade_i = if fade_start_time <= time {
            0
        } else {
            round((fade_start_time - time) / sample_duration) as usize
        };
        let fade_d = sample_duration / fade_duration;
        let fade_phase = delerp(
            fade_start_time,
            fade_end_time,
            time + fade_i as f48 * sample_duration,
        );
        for channel in 0..output.len() {
            let mut fade = fade_phase;
            for i in fade_i..end_index {
                output[channel][i] *= smooth5(1.0 - fade);
                fade += fade_d;
            }
        }
    }
}

#[duplicate_item(
    f48       Event48       AudioUnit48       Sequencer48         Callback48;
    [ f64 ]   [ Event64 ]   [ AudioUnit64 ]   [ Sequencer64 ]   [ Callback64 ];
    [ f32 ]   [ Event32 ]   [ AudioUnit32 ]   [ Sequencer32 ]   [ Callback32 ];
)]
pub struct Sequencer48 {
    // Unsorted.
    active: Vec<Event48>,
    // Sorted by start time.
    ready: BinaryHeap<Event48>,
    // Unsorted.
    past: Vec<Event48>,
    outputs: usize,
    time: f48,
    sample_rate: f48,
    sample_duration: f48,
    buffer: Buffer<f48>,
    tick_buffer: Vec<f48>,
    callback: Option<Callback48<Sequencer48>>,
}

#[duplicate_item(
    f48       Event48       AudioUnit48       Sequencer48         Callback48;
    [ f64 ]   [ Event64 ]   [ AudioUnit64 ]   [ Sequencer64 ]   [ Callback64 ];
    [ f32 ]   [ Event32 ]   [ AudioUnit32 ]   [ Sequencer32 ]   [ Callback32 ];
)]
impl Sequencer48 {
    /// Create a new sequencer. The sequencer has zero inputs.
    /// The number of outputs is decided by the user.
    pub fn new(sample_rate: f64, outputs: usize) -> Self {
        Self {
            active: Vec::new(),
            ready: BinaryHeap::new(),
            past: Vec::new(),
            outputs,
            time: 0.0,
            sample_rate: sample_rate as f48,
            sample_duration: 1.0 / sample_rate as f48,
            buffer: Buffer::new(),
            tick_buffer: vec![0.0; outputs],
            callback: None,
        }
    }

    /// Set the update callback.
    pub fn set_callback(
        &mut self,
        update_interval: f48,
        callback: Box<dyn FnMut(f48, f48, &mut Sequencer48) + Send>,
    ) {
        self.callback = Some(Callback48::new(update_interval, callback));
    }

    /// Current time in seconds.
    pub fn time(&self) -> f48 {
        self.time
    }

    /// Add an event.
    /// Fade in and fade out may overlap but may not exceed the duration of the event.
    pub fn add(
        &mut self,
        start_time: f48,
        end_time: f48,
        fade_in_time: f48,
        fade_out_time: f48,
        mut unit: Box<dyn AudioUnit48>,
    ) {
        assert!(unit.inputs() == 0 && unit.outputs() == self.outputs);
        let duration = end_time - start_time;
        assert!(fade_in_time <= duration && fade_out_time <= duration);
        // Make sure the sample rate of the unit matches ours.
        unit.reset(Some(self.sample_rate as f64));
        self.ready.push(Event48::new(
            unit,
            start_time,
            end_time,
            fade_in_time,
            fade_out_time,
        ));
    }

    /// Add an event using start time and duration.
    pub fn add_duration(
        &mut self,
        start_time: f48,
        duration: f48,
        fade_in_time: f48,
        fade_out_time: f48,
        unit: Box<dyn AudioUnit48>,
    ) {
        self.add(
            start_time,
            start_time + duration,
            fade_in_time,
            fade_out_time,
            unit,
        )
    }

    /// Erase past events. Events playing now are not affected.
    pub fn erase_past(&mut self) {
        self.past.clear();
    }

    /// Erase future events. Events playing now are not affected.
    pub fn erase_future(&mut self) {
        self.ready.clear();
    }

    /// Jump in time. Events playing now will be adjusted to continue seamlessly.
    /// Events that start earlier than the new time are not replayed.
    /// The next update callback will be issued at the new time.
    pub fn jump_to_time(&mut self, time: f48) {
        self.time = time;
        let time_difference = time - self.time;
        for event in self.active.iter_mut() {
            event.start_time += time_difference;
            event.end_time += time_difference;
        }
        if time_difference < 0.0 {
            let mut i = 0;
            while i < self.past.len() {
                if self.past[i].start_time >= time {
                    let event = self.past.swap_remove(i);
                    self.ready.push(event);
                } else {
                    i += 1;
                }
            }
        } else {
            while let Some(event) = self.ready.peek() {
                if event.start_time < time {
                    self.past.push(self.ready.pop().unwrap());
                }
            }
        }
        if let Some(cb) = &mut self.callback {
            cb.set_time(time);
        }
    }

    /// Move units that start before the end time to the ready heap.
    fn ready_to_active(&mut self, next_end_time: f48) {
        while let Some(ready) = self.ready.peek() {
            // Test whether start time rounded to a sample comes before the end time,
            // which always falls on a sample.
            if ready.start_time < next_end_time - self.sample_duration * 0.5 {
                if let Some(ready) = self.ready.pop() {
                    self.active.push(ready);
                }
            } else {
                break;
            }
        }
    }

    /// Indicate to callback handler that time is about to elapse.
    fn elapse(&mut self, dt: f48) {
        let mut tmp = self.callback.take();
        if let Some(cb) = &mut tmp {
            cb.update(dt, self);
        }
        self.callback = tmp.take();
    }
}

#[duplicate_item(
    f48       Event48       AudioUnit48       Sequencer48      fade_in48      fade_out48;
    [ f64 ]   [ Event64 ]   [ AudioUnit64 ]   [ Sequencer64 ]  [ fade_in64 ]  [ fade_out64 ];
    [ f32 ]   [ Event32 ]   [ AudioUnit32 ]   [ Sequencer32 ]  [ fade_in32 ]  [ fade_out32 ];
)]
impl AudioUnit48 for Sequencer48 {
    fn reset(&mut self, sample_rate: Option<f64>) {
        // Move everything to the active queue, then reset and move
        // everything to the ready heap.
        while let Some(active) = self.past.pop() {
            self.active.push(active);
        }
        if let Some(rate) = sample_rate {
            if self.sample_rate != rate as f48 {
                self.sample_rate = rate as f48;
                self.sample_duration = 1.0 / rate as f48;
                // If the sample rate changes, then we need to reset every unit.
                // Otherwise, we know that the ready units are in a reset state,
                // and don't need to be reset again.
                while let Some(ready) = self.ready.pop() {
                    self.active.push(ready);
                }
            }
        }
        for i in 0..self.active.len() {
            self.active[i].unit.reset(sample_rate);
        }
        while let Some(active) = self.active.pop() {
            self.ready.push(active);
        }
        self.time = 0.0;
        if let Some(cb) = &mut self.callback {
            cb.reset();
        }
    }

    fn tick(&mut self, input: &[f48], output: &mut [f48]) {
        self.elapse(self.sample_duration);
        for channel in 0..self.outputs {
            output[channel] = 0.0;
        }
        let end_time = self.time + self.sample_duration;
        self.ready_to_active(end_time);
        let mut i = 0;
        while i < self.active.len() {
            if self.active[i].end_time <= self.time + 0.5 * self.sample_duration {
                self.past.push(self.active.swap_remove(i));
            } else {
                self.active[i].unit.tick(input, &mut self.tick_buffer);
                if self.active[i].fade_in > 0.0 {
                    let fade_in = delerp(
                        self.active[i].start_time,
                        self.active[i].start_time + self.active[i].fade_in,
                        self.time,
                    );
                    if fade_in < 1.0 {
                        for channel in 0..self.outputs {
                            self.tick_buffer[channel] *= smooth5(fade_in);
                        }
                    }
                }
                if self.active[i].fade_out > 0.0 {
                    let fade_out = delerp(
                        self.active[i].end_time - self.active[i].fade_out,
                        self.active[i].end_time,
                        self.time,
                    );
                    if fade_out > 0.0 {
                        for channel in 0..self.outputs {
                            self.tick_buffer[channel] *= smooth5(1.0 - fade_out);
                        }
                    }
                }
                for channel in 0..self.outputs {
                    output[channel] += self.tick_buffer[channel];
                }
                i += 1;
            }
        }
        self.time = end_time;
    }

    fn process(&mut self, size: usize, input: &[&[f48]], output: &mut [&mut [f48]]) {
        self.elapse(self.sample_duration * size as f48);
        for channel in 0..self.outputs {
            for i in 0..size {
                output[channel][i] = 0.0;
            }
        }
        let end_time = self.time + self.sample_duration * size as f48;
        self.ready_to_active(end_time);
        let buffer_output = self.buffer.get_mut(self.outputs);
        let mut i = 0;
        while i < self.active.len() {
            if self.active[i].end_time <= self.time {
                self.past.push(self.active.swap_remove(i));
            } else {
                let start_index = if self.active[i].start_time <= self.time {
                    0
                } else {
                    round((self.active[i].start_time - self.time) * self.sample_rate) as usize
                };
                let end_index = if self.active[i].end_time >= end_time {
                    size
                } else {
                    round((self.active[i].end_time - self.time) * self.sample_rate) as usize
                };
                if end_index > start_index {
                    self.active[i]
                        .unit
                        .process(end_index - start_index, input, buffer_output);
                    fade_in48(
                        self.sample_duration,
                        self.time,
                        end_time,
                        start_index,
                        end_index,
                        self.active[i].fade_in,
                        self.active[i].start_time,
                        buffer_output,
                    );
                    fade_out48(
                        self.sample_duration,
                        self.time,
                        end_time,
                        start_index,
                        end_index,
                        self.active[i].fade_out,
                        self.active[i].end_time,
                        buffer_output,
                    );
                    for channel in 0..self.outputs {
                        for j in start_index..end_index {
                            output[channel][j] += buffer_output[channel][j - start_index];
                        }
                    }
                }
                i += 1;
            }
        }
        self.time = end_time;
    }

    fn get_id(&self) -> u64 {
        ID
    }

    fn inputs(&self) -> usize {
        0
    }
    fn outputs(&self) -> usize {
        self.outputs
    }

    fn route(&self, _input: &SignalFrame, _frequency: f64) -> SignalFrame {
        // Treat the sequencer as a generator.
        let mut signal = new_signal_frame(AudioUnit48::outputs(self));
        for i in 0..AudioUnit48::outputs(self) {
            signal[i] = Signal::Latency(0.0);
        }
        signal
    }
}
