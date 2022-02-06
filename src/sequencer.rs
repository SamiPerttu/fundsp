//! Sequencer unit.

use std::cmp::Eq;
use std::cmp::Ord;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

use super::audiounit::*;
use super::buffer::*;
use super::math::*;
use super::signal::*;
use super::*;

pub struct Event {
    pub unit: Au,
    pub start_time: f64,
    pub end_time: f64,
    pub fade_in: f64,
    pub fade_out: f64,
}

impl Event {
    pub fn new64(
        unit: Box<dyn AudioUnit64>,
        start_time: f64,
        end_time: f64,
        fade_in: f64,
        fade_out: f64,
    ) -> Self {
        Self {
            unit: Au::Unit64(unit),
            start_time,
            end_time,
            fade_in,
            fade_out,
        }
    }
    pub fn new32(
        unit: Box<dyn AudioUnit32>,
        start_time: f64,
        end_time: f64,
        fade_in: f64,
        fade_out: f64,
    ) -> Self {
        Self {
            unit: Au::Unit32(unit),
            start_time,
            end_time,
            fade_in,
            fade_out,
        }
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Event) -> bool {
        self.start_time == other.start_time
    }
}

impl Eq for Event {}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Event) -> Option<Ordering> {
        other.start_time.partial_cmp(&self.start_time)
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        if other.start_time > self.start_time {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

#[inline(always)]
fn do_fade_in64(
    sample_duration: f64,
    time: f64,
    end_time: f64,
    end_index: usize,
    fade_duration: f64,
    fade_start_time: f64,
    output: &mut [&mut [f64]],
) {
    let fade_end_time = fade_start_time + fade_duration;
    if fade_duration > 0.0 && fade_end_time > time {
        let fade_end_i = if fade_end_time >= end_time {
            end_index
        } else {
            round((fade_end_time - time) / sample_duration) as usize
        };
        let fade_d = sample_duration / fade_duration;
        let fade_phase = delerp(fade_start_time, fade_end_time, time);
        for channel in 0..output.len() {
            let mut fade = fade_phase;
            for i in 0..fade_end_i {
                output[channel][i] *= smooth5(fade);
                fade += fade_d;
            }
        }
    }
}

#[inline(always)]
fn do_fade_out64(
    sample_duration: f64,
    time: f64,
    end_time: f64,
    end_index: usize,
    fade_duration: f64,
    fade_end_time: f64,
    output: &mut [&mut [f64]],
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
            time + fade_i as f64 * sample_duration,
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

#[inline(always)]
fn do_fade_in32(
    sample_duration: f64,
    time: f64,
    end_time: f64,
    end_index: usize,
    fade_duration: f64,
    fade_start_time: f64,
    output: &mut [&mut [f32]],
) {
    let fade_end_time = fade_start_time + fade_duration;
    if fade_duration > 0.0 && fade_end_time > time {
        let fade_end_i = if fade_end_time >= end_time {
            end_index
        } else {
            round((fade_end_time - time) / sample_duration) as usize
        };
        let fade_d = (sample_duration / fade_duration) as f32;
        let fade_phase = delerp(fade_start_time, fade_end_time, time) as f32;
        for channel in 0..output.len() {
            let mut fade = fade_phase;
            for i in 0..fade_end_i {
                output[channel][i] *= smooth5(fade);
                fade += fade_d;
            }
        }
    }
}

#[inline(always)]
fn do_fade_out32(
    sample_duration: f64,
    time: f64,
    end_time: f64,
    end_index: usize,
    fade_duration: f64,
    fade_end_time: f64,
    output: &mut [&mut [f32]],
) {
    let fade_start_time = fade_end_time - fade_duration;
    if fade_duration > 0.0 && fade_start_time < end_time {
        let fade_i = if fade_start_time <= time {
            0
        } else {
            round((fade_start_time - time) / sample_duration) as usize
        };
        let fade_d = (sample_duration / fade_duration) as f32;
        let fade_phase = delerp(
            fade_start_time,
            fade_end_time,
            time + fade_i as f64 * sample_duration,
        ) as f32;
        for channel in 0..output.len() {
            let mut fade = fade_phase;
            for i in fade_i..end_index {
                output[channel][i] *= smooth5(1.0 - fade);
                fade += fade_d;
            }
        }
    }
}

pub struct Sequencer {
    // Unsorted.
    active: Vec<Event>,
    // Sorted by start time.
    ready: BinaryHeap<Event>,
    // Unsorted.
    past: Vec<Event>,
    outputs: usize,
    time: f64,
    sample_rate: f64,
    sample_duration: f64,
    buffer32: Buffer<f32>,
    tick32: Vec<f32>,
    buffer64: Buffer<f64>,
    tick64: Vec<f64>,
}

impl Sequencer {
    /// Create a new sequencer. The sequencer has zero inputs.
    /// The number of outputs is decided by the user.
    pub fn new(sample_rate: f64, outputs: usize) -> Self {
        Self {
            active: Vec::new(),
            ready: BinaryHeap::new(),
            past: Vec::new(),
            outputs,
            time: 0.0,
            sample_rate,
            sample_duration: 1.0 / sample_rate,
            buffer32: Buffer::new(),
            tick32: vec![0.0; outputs],
            buffer64: Buffer::new(),
            tick64: vec![0.0; outputs],
        }
    }

    /// Add a 64-bit unit.
    pub fn add64(
        &mut self,
        start_time: f64,
        end_time: f64,
        fade_in_time: f64,
        fade_out_time: f64,
        mut unit: Box<dyn AudioUnit64>,
    ) {
        assert!(unit.inputs() == 0 && unit.outputs() == self.outputs);
        // Make sure the sample rate of the unit matches ours.
        unit.reset(Some(self.sample_rate));
        self.ready.push(Event::new64(
            unit,
            start_time,
            end_time,
            fade_in_time,
            fade_out_time,
        ));
    }

    /// Add a 32-bit unit.
    pub fn add32(
        &mut self,
        start_time: f64,
        end_time: f64,
        fade_in_time: f64,
        fade_out_time: f64,
        mut unit: Box<dyn AudioUnit32>,
    ) {
        assert!(unit.inputs() == 0 && unit.outputs() == self.outputs);
        // Make sure the sample rate of the unit matches ours.
        unit.reset(Some(self.sample_rate));
        self.ready.push(Event::new32(
            unit,
            start_time,
            end_time,
            fade_in_time,
            fade_out_time,
        ));
    }

    /// Move units that start before the end time to the ready heap.
    fn ready_to_active(&mut self, next_end_time: f64) {
        while let Some(ready) = self.ready.peek() {
            if ready.start_time < next_end_time {
                if let Some(ready) = self.ready.pop() {
                    self.active.push(ready);
                }
            } else {
                break;
            }
        }
    }

    fn do_reset(&mut self, sample_rate: Option<f64>) {
        // Move everything to the active queue, then reset and move
        // everything to the ready heap.
        while let Some(active) = self.past.pop() {
            self.active.push(active);
        }
        if let Some(rate) = sample_rate {
            if self.sample_rate != rate {
                self.sample_rate = rate;
                self.sample_duration = 1.0 / rate;
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
    }

    pub fn do_route(&self, _input: &SignalFrame, _frequency: f64) -> SignalFrame {
        // Treat the sequencer as a generator.
        let mut signal = new_signal_frame(AudioUnit64::outputs(self));
        for i in 0..AudioUnit64::outputs(self) {
            signal[i] = Signal::Latency(0.0);
        }
        signal
    }
}

impl AudioUnit64 for Sequencer {
    fn reset(&mut self, sample_rate: Option<f64>) {
        self.do_reset(sample_rate);
    }

    fn tick(&mut self, input: &[f64], output: &mut [f64]) {
        for channel in 0..self.outputs {
            output[channel] = 0.0;
        }
        let end_time = self.time + self.sample_duration;
        self.ready_to_active(end_time);
        let mut i = 0;
        while i < self.active.len() {
            if self.active[i].end_time <= self.time {
                self.past.push(self.active.swap_remove(i));
            } else {
                self.active[i].unit.tick64(input, &mut self.tick64);
                if self.active[i].fade_in > 0.0 {
                    let fade_in = delerp(0.0, self.active[i].fade_in, self.time);
                    if fade_in < 1.0 {
                        for channel in 0..self.outputs {
                            self.tick64[channel] *= smooth5(fade_in);
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
                            self.tick64[channel] *= smooth5(1.0 - fade_out);
                        }
                    }
                }
                for channel in 0..self.outputs {
                    output[channel] += self.tick64[channel];
                }
                i += 1;
            }
        }
        self.time = end_time;
    }

    fn process(&mut self, size: usize, input: &[&[f64]], output: &mut [&mut [f64]]) {
        for channel in 0..self.outputs {
            for i in 0..size {
                output[channel][i] = 0.0;
            }
        }
        let end_time = self.time + self.sample_duration * size as f64;
        self.ready_to_active(end_time);
        let buffer_output = self.buffer64.get_mut(self.outputs);
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
                        .process64(end_index - start_index, input, buffer_output);
                    do_fade_in64(
                        self.sample_duration,
                        self.time,
                        end_time,
                        end_index,
                        self.active[i].fade_in,
                        self.active[i].start_time,
                        buffer_output,
                    );
                    do_fade_out64(
                        self.sample_duration,
                        self.time,
                        end_time,
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

    fn inputs(&self) -> usize {
        0
    }
    fn outputs(&self) -> usize {
        self.outputs
    }

    fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.do_route(input, frequency)
    }
}

impl AudioUnit32 for Sequencer {
    fn reset(&mut self, sample_rate: Option<f64>) {
        self.do_reset(sample_rate);
    }

    fn tick(&mut self, input: &[f32], output: &mut [f32]) {
        for channel in 0..self.outputs {
            output[channel] = 0.0;
        }
        let end_time = self.time + self.sample_duration;
        self.ready_to_active(end_time);
        let mut i = 0;
        while i < self.active.len() {
            if self.active[i].end_time <= self.time {
                self.past.push(self.active.swap_remove(i));
            } else {
                self.active[i].unit.tick32(input, &mut self.tick32);
                if self.active[i].fade_in > 0.0 {
                    let fade_in = delerp(0.0, self.active[i].fade_in, self.time) as f32;
                    if fade_in < 1.0 {
                        for channel in 0..self.outputs {
                            self.tick32[channel] *= smooth5(fade_in);
                        }
                    }
                }
                if self.active[i].fade_out > 0.0 {
                    let fade_out = delerp(
                        self.active[i].end_time - self.active[i].fade_out,
                        self.active[i].end_time,
                        self.time,
                    ) as f32;
                    if fade_out > 0.0 {
                        for channel in 0..self.outputs {
                            self.tick32[channel] *= smooth5(1.0 - fade_out);
                        }
                    }
                }
                for channel in 0..self.outputs {
                    output[channel] += self.tick32[channel];
                }
                i += 1;
            }
        }
        self.time = end_time;
    }

    fn process(&mut self, size: usize, input: &[&[f32]], output: &mut [&mut [f32]]) {
        for channel in 0..self.outputs {
            for i in 0..size {
                output[channel][i] = 0.0;
            }
        }
        let end_time = self.time + self.sample_duration * size as f64;
        self.ready_to_active(end_time);
        let buffer_output = self.buffer32.get_mut(self.outputs);
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
                        .process32(end_index - start_index, input, buffer_output);
                    do_fade_in32(
                        self.sample_duration,
                        self.time,
                        end_time,
                        end_index,
                        self.active[i].fade_in,
                        self.active[i].start_time,
                        buffer_output,
                    );
                    do_fade_out32(
                        self.sample_duration,
                        self.time,
                        end_time,
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

    fn inputs(&self) -> usize {
        0
    }
    fn outputs(&self) -> usize {
        self.outputs
    }

    fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.do_route(input, frequency)
    }
}
