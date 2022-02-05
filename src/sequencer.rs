//! Sequencer unit. WIP.

use std::cmp::Eq;
use std::cmp::Ord;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

use super::audiounit::*;
use super::buffer::*;
use super::math::*;
use super::signal::*;
use super::*;

pub struct Event64 {
    pub unit: Box<dyn AudioUnit64>,
    pub start_time: f64,
    pub end_time: f64,
}

impl Event64 {
    pub fn new(unit: Box<dyn AudioUnit64>, start_time: f64, end_time: f64) -> Self {
        Self {
            unit,
            start_time,
            end_time,
        }
    }
    pub fn reset(&mut self, sample_rate: Option<f64>) {
        self.unit.reset(sample_rate);
    }
    pub fn tick64(&mut self, input: &[f64], output: &mut [f64]) {
        self.unit.tick(input, output);
    }
    pub fn process64(&mut self, size: usize, input: &[&[f64]], output: &mut [&mut [f64]]) {
        self.unit.process(size, input, output);
    }
}

impl PartialEq for Event64 {
    fn eq(&self, other: &Event64) -> bool {
        self.start_time == other.start_time
    }
}

impl Eq for Event64 {}

impl PartialOrd for Event64 {
    fn partial_cmp(&self, other: &Event64) -> Option<Ordering> {
        other.start_time.partial_cmp(&self.start_time)
    }
}

impl Ord for Event64 {
    fn cmp(&self, other: &Self) -> Ordering {
        if other.start_time > self.start_time {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

pub struct Sequencer64 {
    // Unsorted.
    active: Vec<Event64>,
    // Sorted by start time.
    ready: BinaryHeap<Event64>,
    // Unsorted.
    past: Vec<Event64>,
    outputs: usize,
    time: f64,
    sample_rate: f64,
    sample_duration: f64,
    _buffer32: Buffer<f32>,
    buffer64: Buffer<f64>,
    _tick32: Vec<f32>,
    tick64: Vec<f64>,
}

impl Sequencer64 {
    pub fn new(sample_rate: f64, outputs: usize) -> Self {
        Self {
            active: Vec::new(),
            ready: BinaryHeap::new(),
            past: Vec::new(),
            outputs,
            time: 0.0,
            sample_rate,
            sample_duration: 1.0 / sample_rate,
            _buffer32: Buffer::new(),
            buffer64: Buffer::new(),
            _tick32: vec![0.0; outputs],
            tick64: vec![0.0; outputs],
        }
    }

    pub fn add_unit(&mut self, start_time: f64, end_time: f64, mut unit: Box<dyn AudioUnit64>) {
        unit.reset(Some(self.sample_rate));
        self.ready.push(Event64 {
            unit,
            start_time,
            end_time,
        });
    }

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
            self.active[i].reset(sample_rate);
        }
        self.time = 0.0;
        while let Some(active) = self.active.pop() {
            self.ready.push(active);
        }
    }
}

impl AudioUnit64 for Sequencer64 {
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
                self.active[i].tick64(input, &mut self.tick64);
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
                    self.active[i].process64(end_index - start_index, input, buffer_output);
                    for channel in 0..self.outputs {
                        for j in start_index..end_index {
                            output[channel][j] = buffer_output[channel][j - start_index];
                        }
                    }
                }
                i += 1;
            }
        }
    }

    fn inputs(&self) -> usize {
        0
    }
    fn outputs(&self) -> usize {
        self.outputs
    }

    fn route(&self, _input: &SignalFrame, _frequency: f64) -> SignalFrame {
        // Treat the sequencer as a generator.
        let mut signal = new_signal_frame(self.outputs());
        for i in 0..self.outputs() {
            signal[i] = Signal::Latency(0.0);
        }
        signal
    }
}
