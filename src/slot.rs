//! Slot for an audio unit that can be updated with a crossfade in real time.

use super::audiounit::*;
use super::buffer::*;
use super::math::*;
use super::sequencer::*;
use super::signal::*;
use super::*;
use thingbuf::mpsc::{channel, Receiver, Sender};
extern crate alloc;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

#[derive(Clone, Default)]
enum SlotMessage {
    #[default]
    Nothing,
    /// Update unit using fade shape and fade time (in seconds).
    Update(Fade, f64, Box<dyn AudioUnit>),
    /// Return a unit for deallocation.
    #[allow(dead_code)]
    Return(Box<dyn AudioUnit>),
}

/// Frontend for an updatable unit slot.
pub struct Slot {
    inputs: usize,
    outputs: usize,
    receiver: Receiver<SlotMessage>,
    sender: Sender<SlotMessage>,
}

impl Slot {
    /// Create a new slot. The number of inputs and outputs will be taken from the initial unit.
    /// Returns (frontend, backend) pair.
    pub fn new(mut initial_unit: Box<dyn AudioUnit>) -> (Slot, SlotBackend) {
        let (sender_a, receiver_a) = channel(1024);
        let (sender_b, receiver_b) = channel(1024);
        let inputs = initial_unit.inputs();
        let outputs = initial_unit.outputs();
        let slot = Slot {
            inputs,
            outputs,
            receiver: receiver_a,
            sender: sender_b,
        };
        initial_unit.set_sample_rate(DEFAULT_SR);
        #[allow(clippy::unnecessary_cast)]
        let backend = SlotBackend {
            inputs,
            outputs,
            sample_rate: DEFAULT_SR,
            current: initial_unit,
            next: None,
            fade: Fade::Smooth,
            fade_time: 0.0,
            fade_phase: 0.0,
            latest: None,
            latest_fade: Fade::Smooth,
            latest_fade_time: 0.0,
            receiver: receiver_b,
            sender: sender_a,
            buffer: BufferVec::new(outputs),
            tick: vec![0.0; outputs],
        };
        (slot, backend)
    }

    /// Set the unit. The current unit will be faded out and the new unit will be faded in
    /// simultaneously.
    pub fn set(&mut self, fade: Fade, fade_time: f64, unit: Box<dyn AudioUnit>) {
        assert_eq!(self.inputs, unit.inputs());
        assert_eq!(self.outputs, unit.outputs());
        // Deallocate units that were sent back.
        while self.receiver.try_recv().is_ok() {}
        let message = SlotMessage::Update(fade, fade_time, unit);
        if self.sender.try_send(message).is_ok() {}
    }

    /// Number of inputs.
    pub fn inputs(&self) -> usize {
        self.inputs
    }

    /// Number of outputs.
    pub fn outputs(&self) -> usize {
        self.outputs
    }
}

pub struct SlotBackend {
    inputs: usize,
    outputs: usize,
    sample_rate: f64,
    current: Box<dyn AudioUnit>,
    next: Option<Box<dyn AudioUnit>>,
    fade: Fade,
    fade_time: f64,
    fade_phase: f64,
    latest: Option<Box<dyn AudioUnit>>,
    latest_fade: Fade,
    latest_fade_time: f64,
    receiver: Receiver<SlotMessage>,
    sender: Sender<SlotMessage>,
    buffer: BufferVec,
    tick: Vec<f32>,
}

impl Clone for SlotBackend {
    fn clone(&self) -> Self {
        // Backends cannot be cloned effectively. Allocate a dummy channel.
        let (sender, receiver) = channel(1);
        Self {
            inputs: self.inputs,
            outputs: self.outputs,
            sample_rate: self.sample_rate,
            current: self.current.clone(),
            next: self.next.clone(),
            fade: self.fade.clone(),
            fade_time: self.fade_time,
            fade_phase: self.fade_phase,
            latest: self.latest.clone(),
            latest_fade: self.latest_fade.clone(),
            latest_fade_time: self.latest_fade_time,
            receiver,
            sender,
            buffer: BufferVec::new(self.outputs),
            tick: self.tick.clone(),
        }
    }
}

impl SlotBackend {
    /// Handle updates.
    fn handle_messages(&mut self) {
        while let Ok(message) = self.receiver.try_recv() {
            if let SlotMessage::Update(fade, fade_time, unit) = message {
                if self.next.is_none() {
                    self.next = Some(unit);
                    self.fade_phase = 0.0;
                    self.fade_time = fade_time;
                    self.fade = fade;
                } else {
                    if let Some(latest) = self.latest.take() {
                        if self.sender.try_send(SlotMessage::Return(latest)).is_ok() {}
                    }
                    self.latest = Some(unit);
                    self.latest_fade = fade;
                    self.latest_fade_time = fade_time;
                }
            }
        }
    }
    /// We have faded to the next unit, now start fading to the latest unit, if any.
    #[allow(clippy::needless_if)]
    fn next_phase(&mut self) {
        let mut next = self.next.take().unwrap();
        core::mem::swap(&mut self.current, &mut next);
        if self.sender.try_send(SlotMessage::Return(next)).is_ok() {}
        self.fade = self.latest_fade.clone();
        self.fade_phase = 0.0;
        self.fade_time = self.latest_fade_time;
        core::mem::swap(&mut self.next, &mut self.latest);
    }
}

impl AudioUnit for SlotBackend {
    #[allow(clippy::needless_if)]
    fn reset(&mut self) {
        // Adopt the latest configuration and reset the unit.
        if let Some(mut latest) = self.latest.take() {
            core::mem::swap(&mut self.current, &mut latest);
            if self.sender.try_send(SlotMessage::Return(latest)).is_ok() {}
            if let Some(next) = self.next.take() {
                if self.sender.try_send(SlotMessage::Return(next)).is_ok() {}
            }
        } else if let Some(mut next) = self.next.take() {
            core::mem::swap(&mut self.current, &mut next);
            if self.sender.try_send(SlotMessage::Return(next)).is_ok() {}
        }
        self.current.reset();
    }

    #[allow(clippy::unnecessary_cast)]
    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
        self.current.set_sample_rate(sample_rate);
        if let Some(next) = self.next.as_deref_mut() {
            next.set_sample_rate(sample_rate);
        }
        if let Some(latest) = self.latest.as_deref_mut() {
            latest.set_sample_rate(sample_rate);
        }
    }

    fn tick(&mut self, input: &[f32], output: &mut [f32]) {
        self.handle_messages();
        self.current.tick(input, output);
        if let Some(next) = self.next.as_deref_mut() {
            let f = self.fade.at(1.0 - self.fade_phase) as f32;
            for x in output.iter_mut() {
                *x *= f;
            }
            next.tick(input, &mut self.tick);
            let f = self.fade.at(self.fade_phase) as f32;
            for (x, y) in output.iter_mut().zip(self.tick.iter()) {
                *x += *y * f;
            }
            self.fade_phase += 1.0 / (self.fade_time * self.sample_rate);
            if self.fade_phase >= 1.0 {
                self.next_phase();
            }
        }
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        self.handle_messages();
        self.current.process(size, input, output);
        if let Some(next) = self.next.as_deref_mut() {
            let phase_left = ((1.0 - self.fade_phase) * self.fade_time * self.sample_rate) as usize;
            let n = min(size, phase_left);
            let fade_d = 1.0 / (self.fade_time * self.sample_rate) as f32;
            for channel in 0..self.outputs {
                let mut fade = self.fade_phase as f32;
                match self.fade {
                    Fade::Power => {
                        for x in output.channel_f32_mut(channel)[..n].iter_mut() {
                            *x *= sine_ease(1.0 - fade);
                            fade += fade_d;
                        }
                    }
                    Fade::Smooth => {
                        for x in output.channel_f32_mut(channel)[..n].iter_mut() {
                            *x *= smooth5(1.0 - fade);
                            fade += fade_d;
                        }
                    }
                }
            }
            next.process(size, input, &mut self.buffer.buffer_mut());
            for channel in 0..self.outputs {
                let mut fade = self.fade_phase as f32;
                match self.fade {
                    Fade::Power => {
                        for (x, y) in output.channel_f32_mut(channel)[..n]
                            .iter_mut()
                            .zip(self.buffer.channel_f32(channel)[..n].iter())
                        {
                            *x += *y * sine_ease(fade);
                            fade += fade_d;
                        }
                    }
                    Fade::Smooth => {
                        for (x, y) in output.channel_f32_mut(channel)[..n]
                            .iter_mut()
                            .zip(self.buffer.channel_f32(channel)[..n].iter())
                        {
                            *x += *y * smooth5(fade);
                            fade += fade_d;
                        }
                    }
                }
                for (x, y) in output.channel_f32_mut(channel)[n..size]
                    .iter_mut()
                    .zip(self.buffer.channel_f32(channel)[n..size].iter())
                {
                    *x = *y;
                }
            }
            self.fade_phase += n as f64 / (self.fade_time * self.sample_rate);
            if phase_left <= size {
                // We don't start fading in the latest unit until the next block.
                self.next_phase();
            }
        }
    }

    fn inputs(&self) -> usize {
        self.inputs
    }

    fn outputs(&self) -> usize {
        self.outputs
    }

    fn get_id(&self) -> u64 {
        const ID: u64 = 78;
        ID
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        if !probe {
            self.current.set_hash(hash.state());
            if let Some(next) = self.next.as_deref_mut() {
                next.set_hash(hash.state());
            }
            if let Some(latest) = self.latest.as_deref_mut() {
                latest.set_hash(hash.state());
            }
        }
        hash.hash(self.get_id())
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.current.route(input, frequency)
    }

    fn footprint(&self) -> usize {
        core::mem::size_of::<SlotBackend>()
    }

    fn allocate(&mut self) {
        self.current.allocate();
        if let Some(next) = self.next.as_deref_mut() {
            next.allocate();
        }
        if let Some(latest) = self.latest.as_deref_mut() {
            latest.allocate();
        }
    }
}
