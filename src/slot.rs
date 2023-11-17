//! Slot for an audio unit that can be updated with a crossfade in real time.

use super::audiounit::*;
use super::buffer::*;
use super::math::*;
use super::sequencer::*;
use super::signal::*;
use super::*;
use duplicate::duplicate_item;
use thingbuf::mpsc::blocking::{channel, Receiver, Sender};

#[duplicate_item(
    f48       Slot48       SlotMessage48       SlotBackend48       AudioUnit48;
    [ f64 ]   [ Slot64 ]   [ SlotMessage64 ]   [ SlotBackend64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Slot32 ]   [ SlotMessage32 ]   [ SlotBackend32 ]   [ AudioUnit32 ];
)]
#[derive(Clone, Default)]
enum SlotMessage48 {
    #[default]
    Nothing,
    /// Update unit using fade shape and fade time (in seconds).
    Update(Fade, f48, Box<dyn AudioUnit48>),
    /// Return a unit for deallocation.
    Return(Box<dyn AudioUnit48>),
}

#[duplicate_item(
    f48       Slot48       SlotMessage48       SlotBackend48       AudioUnit48;
    [ f64 ]   [ Slot64 ]   [ SlotMessage64 ]   [ SlotBackend64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Slot32 ]   [ SlotMessage32 ]   [ SlotBackend32 ]   [ AudioUnit32 ];
)]
/// Frontend for an updatable unit slot.
pub struct Slot48 {
    inputs: usize,
    outputs: usize,
    receiver: Receiver<SlotMessage48>,
    sender: Sender<SlotMessage48>,
}

#[duplicate_item(
    f48       Slot48       SlotMessage48       SlotBackend48       AudioUnit48;
    [ f64 ]   [ Slot64 ]   [ SlotMessage64 ]   [ SlotBackend64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Slot32 ]   [ SlotMessage32 ]   [ SlotBackend32 ]   [ AudioUnit32 ];
)]
impl Slot48 {
    /// Create a new slot. The number of inputs and outputs will be taken from the initial unit.
    /// Returns (frontend, backend) pair.
    pub fn new(mut initial_unit: Box<dyn AudioUnit48>) -> (Slot48, SlotBackend48) {
        let (sender_a, receiver_a) = channel(1024);
        let (sender_b, receiver_b) = channel(1024);
        let inputs = initial_unit.inputs();
        let outputs = initial_unit.outputs();
        let slot = Slot48 {
            inputs,
            outputs,
            receiver: receiver_a,
            sender: sender_b,
        };
        initial_unit.set_sample_rate(DEFAULT_SR);
        #[allow(clippy::unnecessary_cast)]
        let backend = SlotBackend48 {
            inputs,
            outputs,
            sample_rate: DEFAULT_SR as f48,
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
            buffer: Buffer::with_channels(outputs),
            tick: vec![0.0; outputs],
        };
        (slot, backend)
    }

    /// Set the unit. The current unit will be faded out and the new unit will be faded in
    /// simultaneously.
    pub fn set(&mut self, fade: Fade, fade_time: f48, unit: Box<dyn AudioUnit48>) {
        assert_eq!(self.inputs, unit.inputs());
        assert_eq!(self.outputs, unit.outputs());
        // Deallocate units that were sent back.
        while self.receiver.try_recv().is_ok() {}
        let message = SlotMessage48::Update(fade, fade_time, unit);
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

#[duplicate_item(
    f48       Slot48       SlotMessage48       SlotBackend48       AudioUnit48;
    [ f64 ]   [ Slot64 ]   [ SlotMessage64 ]   [ SlotBackend64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Slot32 ]   [ SlotMessage32 ]   [ SlotBackend32 ]   [ AudioUnit32 ];
)]
pub struct SlotBackend48 {
    inputs: usize,
    outputs: usize,
    sample_rate: f48,
    current: Box<dyn AudioUnit48>,
    next: Option<Box<dyn AudioUnit48>>,
    fade: Fade,
    fade_time: f48,
    fade_phase: f48,
    latest: Option<Box<dyn AudioUnit48>>,
    latest_fade: Fade,
    latest_fade_time: f48,
    receiver: Receiver<SlotMessage48>,
    sender: Sender<SlotMessage48>,
    buffer: Buffer<f48>,
    tick: Vec<f48>,
}

#[duplicate_item(
    f48       Slot48       SlotMessage48       SlotBackend48       AudioUnit48;
    [ f64 ]   [ Slot64 ]   [ SlotMessage64 ]   [ SlotBackend64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Slot32 ]   [ SlotMessage32 ]   [ SlotBackend32 ]   [ AudioUnit32 ];
)]
impl Clone for SlotBackend48 {
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
            buffer: Buffer::with_channels(self.outputs),
            tick: self.tick.clone(),
        }
    }
}

#[duplicate_item(
    f48       Slot48       SlotMessage48       SlotBackend48       AudioUnit48;
    [ f64 ]   [ Slot64 ]   [ SlotMessage64 ]   [ SlotBackend64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Slot32 ]   [ SlotMessage32 ]   [ SlotBackend32 ]   [ AudioUnit32 ];
)]
impl SlotBackend48 {
    /// Handle updates.
    fn handle_messages(&mut self) {
        while let Ok(message) = self.receiver.try_recv() {
            if let SlotMessage48::Update(fade, fade_time, unit) = message {
                if self.next.is_none() {
                    self.next = Some(unit);
                    self.fade_phase = 0.0;
                    self.fade_time = fade_time;
                    self.fade = fade;
                } else {
                    if let Some(latest) = self.latest.take() {
                        if self.sender.try_send(SlotMessage48::Return(latest)).is_ok() {}
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
        std::mem::swap(&mut self.current, &mut next);
        if self.sender.try_send(SlotMessage48::Return(next)).is_ok() {}
        self.fade = self.latest_fade.clone();
        self.fade_phase = 0.0;
        self.fade_time = self.latest_fade_time;
        std::mem::swap(&mut self.next, &mut self.latest);
    }
}

#[duplicate_item(
    f48       Slot48       SlotMessage48       SlotBackend48       AudioUnit48;
    [ f64 ]   [ Slot64 ]   [ SlotMessage64 ]   [ SlotBackend64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Slot32 ]   [ SlotMessage32 ]   [ SlotBackend32 ]   [ AudioUnit32 ];
)]
impl AudioUnit48 for SlotBackend48 {
    #[allow(clippy::needless_if)]
    fn reset(&mut self) {
        // Adopt the latest configuration and reset the unit.
        if let Some(mut latest) = self.latest.take() {
            std::mem::swap(&mut self.current, &mut latest);
            if self.sender.try_send(SlotMessage48::Return(latest)).is_ok() {}
            if let Some(next) = self.next.take() {
                if self.sender.try_send(SlotMessage48::Return(next)).is_ok() {}
            }
        } else if let Some(mut next) = self.next.take() {
            std::mem::swap(&mut self.current, &mut next);
            if self.sender.try_send(SlotMessage48::Return(next)).is_ok() {}
        }
        self.current.reset();
    }

    #[allow(clippy::unnecessary_cast)]
    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate as f48;
        self.current.set_sample_rate(sample_rate);
        if let Some(next) = self.next.as_deref_mut() {
            next.set_sample_rate(sample_rate);
        }
        if let Some(latest) = self.latest.as_deref_mut() {
            latest.set_sample_rate(sample_rate);
        }
    }

    fn tick(&mut self, input: &[f48], output: &mut [f48]) {
        self.handle_messages();
        self.current.tick(input, output);
        if let Some(next) = self.next.as_deref_mut() {
            let f = self.fade.at(1.0 - self.fade_phase);
            for x in output.iter_mut() {
                *x *= f;
            }
            next.tick(input, &mut self.tick);
            let f = self.fade.at(self.fade_phase);
            for (x, y) in output.iter_mut().zip(self.tick.iter()) {
                *x += *y * f;
            }
            self.fade_phase += 1.0 / (self.fade_time * self.sample_rate);
            if self.fade_phase >= 1.0 {
                self.next_phase();
            }
        }
    }

    fn process(&mut self, size: usize, input: &[&[f48]], output: &mut [&mut [f48]]) {
        self.handle_messages();
        self.current.process(size, input, output);
        if let Some(next) = self.next.as_deref_mut() {
            let phase_left = ((1.0 - self.fade_phase) * self.fade_time * self.sample_rate) as usize;
            let n = min(size, phase_left);
            for i in 0..self.outputs {
                let mut fade = self.fade_phase;
                match self.fade {
                    Fade::Power => {
                        for x in output[i][..n].iter_mut() {
                            *x *= sine_ease(1.0 - fade);
                            fade += 1.0 / (self.fade_time * self.sample_rate);
                        }
                    }
                    Fade::Smooth => {
                        for x in output[i][..n].iter_mut() {
                            *x *= smooth5(1.0 - fade);
                            fade += 1.0 / (self.fade_time * self.sample_rate);
                        }
                    }
                }
            }
            next.process(size, input, self.buffer.self_mut());
            for i in 0..self.outputs {
                let mut fade = self.fade_phase;
                match self.fade {
                    Fade::Power => {
                        for (x, y) in output[i][..n]
                            .iter_mut()
                            .zip(self.buffer.mut_at(i)[..n].iter())
                        {
                            *x += *y * sine_ease(fade);
                            fade += 1.0 / (self.fade_time * self.sample_rate);
                        }
                    }
                    Fade::Smooth => {
                        for (x, y) in output[i][..n]
                            .iter_mut()
                            .zip(self.buffer.mut_at(i)[..n].iter())
                        {
                            *x += *y * smooth5(fade);
                            fade += 1.0 / (self.fade_time * self.sample_rate);
                        }
                    }
                }
                for (x, y) in output[i][n..size]
                    .iter_mut()
                    .zip(self.buffer.mut_at(i)[n..size].iter())
                {
                    *x = *y;
                }
            }
            self.fade_phase += n as f48 / (self.fade_time * self.sample_rate);
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
        std::mem::size_of::<SlotBackend48>()
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
