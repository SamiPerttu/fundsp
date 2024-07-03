//! The sequencer unit mixes together scheduled audio units with sample accurate timing.

use super::audiounit::*;
use super::buffer::*;
use super::math::*;
use super::realseq::*;
use super::shared::IdGenerator;
use super::signal::*;
use super::*;
use core::cmp::{Eq, Ord, Ordering};
extern crate alloc;
use alloc::boxed::Box;
use alloc::collections::BinaryHeap;
use alloc::vec;
use alloc::vec::Vec;
use hashbrown::HashMap;
use thingbuf::mpsc::{channel, Receiver, Sender};

/// Globally unique ID for a sequencer event.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct EventId(u64);

/// This atomic supplies globally unique IDs.
static GLOBAL_EVENT_ID: IdGenerator = IdGenerator::new();

impl EventId {
    /// Create a new, globally unique event ID.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        EventId(GLOBAL_EVENT_ID.get_id())
    }
}

/// Fade curves.
#[derive(Clone, Default)]
pub enum Fade {
    /// Equal power fade. Results in equal power mixing
    /// when fade out of one event coincides with the fade in of another.
    Power,
    /// Smooth polynomial fade.
    #[default]
    Smooth,
}

impl Fade {
    /// Evaluate fade curve at `x` (0.0 <= `x` <= 1.0).
    #[inline]
    pub fn at<T: Float>(&self, x: T) -> T {
        match self {
            Fade::Power => sine_ease(x),
            Fade::Smooth => smooth5(x),
        }
    }
}

#[derive(Clone)]
pub(crate) struct Event {
    pub unit: Box<dyn AudioUnit>,
    pub start_time: f64,
    pub end_time: f64,
    pub fade_ease: Fade,
    pub fade_in: f64,
    pub fade_out: f64,
    pub id: EventId,
}

impl Event {
    pub fn new(
        unit: Box<dyn AudioUnit>,
        start_time: f64,
        end_time: f64,
        fade_ease: Fade,
        fade_in: f64,
        fade_out: f64,
    ) -> Self {
        Self {
            unit,
            start_time,
            end_time,
            fade_ease,
            fade_in,
            fade_out,
            id: EventId::new(),
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
        Some(self.cmp(other))
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        other.start_time.total_cmp(&self.start_time)
    }
}

#[derive(Clone)]
pub(crate) struct Edit {
    pub end_time: f64,
    pub fade_out: f64,
}

#[inline]
fn fade_in(
    sample_duration: f64,
    time: f64,
    end_time: f64,
    start_index: usize,
    end_index: usize,
    ease: Fade,
    fade_duration: f64,
    fade_start_time: f64,
    output: &mut BufferMut,
) {
    let fade_end_time = fade_start_time + fade_duration;
    if fade_duration > 0.0 && fade_end_time > time {
        let fade_end_i = if fade_end_time >= end_time {
            end_index
        } else {
            round((fade_end_time - time) / sample_duration) as usize
        };
        let fade_phase = delerp(
            fade_start_time,
            fade_end_time,
            time + start_index as f64 * sample_duration,
        ) as f32;
        let fade_d = (sample_duration / fade_duration) as f32;
        match ease {
            Fade::Power => {
                for channel in 0..output.channels() {
                    let mut fade = fade_phase;
                    for x in output.channel_f32_mut(channel)[..fade_end_i].iter_mut() {
                        *x *= sine_ease(fade);
                        fade += fade_d;
                    }
                }
            }
            Fade::Smooth => {
                for channel in 0..output.channels() {
                    let mut fade = fade_phase;
                    for x in output.channel_f32_mut(channel)[..fade_end_i].iter_mut() {
                        *x *= smooth5(fade);
                        fade += fade_d;
                    }
                }
            }
        }
    }
}

#[inline]
fn fade_out(
    sample_duration: f64,
    time: f64,
    end_time: f64,
    _start_index: usize,
    end_index: usize,
    ease: Fade,
    fade_duration: f64,
    fade_end_time: f64,
    output: &mut BufferMut,
) {
    let fade_start_time = fade_end_time - fade_duration;
    if fade_duration > 0.0 && fade_start_time < end_time {
        let fade_i = if fade_start_time <= time {
            0
        } else {
            round((fade_start_time - time) / sample_duration) as usize
        };
        let fade_phase = delerp(
            fade_start_time,
            fade_end_time,
            time + fade_i as f64 * sample_duration,
        ) as f32;
        let fade_d = (sample_duration / fade_duration) as f32;
        match ease {
            Fade::Power => {
                for channel in 0..output.channels() {
                    let mut fade = fade_phase;
                    for x in output.channel_f32_mut(channel)[fade_i..end_index].iter_mut() {
                        *x *= sine_ease(1.0 - fade);
                        fade += fade_d;
                    }
                }
            }
            Fade::Smooth => {
                for channel in 0..output.channels() {
                    let mut fade = fade_phase;
                    for x in output.channel_f32_mut(channel)[fade_i..end_index].iter_mut() {
                        *x *= smooth5(1.0 - fade);
                        fade += fade_d;
                    }
                }
            }
        }
    }
}

pub struct Sequencer {
    /// Current events, unsorted.
    active: Vec<Event>,
    /// IDs of current events.
    active_map: HashMap<EventId, usize>,
    /// Events that start before the active threshold are active.
    active_threshold: f64,
    /// Future events sorted by start time.
    ready: BinaryHeap<Event>,
    /// Past events, unsorted.
    past: Vec<Event>,
    /// Map of edits to be made to events in the ready queue.
    edit_map: HashMap<EventId, Edit>,
    /// Number of output channels.
    outputs: usize,
    /// Current time. Does not apply to frontends.
    time: f64,
    /// Current sample rate.
    sample_rate: f64,
    /// Current sample duration.
    sample_duration: f64,
    /// Intermediate output buffer.
    buffer: BufferVec,
    /// Intermediate output frame.
    tick_buffer: Vec<f32>,
    /// Optional frontend.
    front: Option<(Sender<Message>, Receiver<Option<Event>>)>,
    /// Whether we replay existing events after a call to `reset`.
    replay_events: bool,
}

impl Clone for Sequencer {
    fn clone(&self) -> Self {
        if self.has_backend() {
            panic!("Frontends cannot be cloned.");
        }
        Self {
            active: self.active.clone(),
            active_map: self.active_map.clone(),
            active_threshold: self.active_threshold,
            ready: self.ready.clone(),
            past: self.past.clone(),
            edit_map: self.edit_map.clone(),
            outputs: self.outputs,
            time: self.time,
            sample_rate: self.sample_rate,
            sample_duration: self.sample_duration,
            buffer: self.buffer.clone(),
            tick_buffer: self.tick_buffer.clone(),
            front: None,
            replay_events: self.replay_events,
        }
    }
}

impl Sequencer {
    /// Create a new sequencer. The sequencer has zero inputs.
    /// The number of outputs is decided by the user.
    /// If `replay_events` is true, then past events will be retained
    /// and played back after a reset.
    /// If false, then all events will be cleared on reset.
    pub fn new(replay_events: bool, outputs: usize) -> Self {
        Self {
            active: Vec::with_capacity(16384),
            active_map: HashMap::with_capacity(16384),
            active_threshold: -f64::INFINITY,
            ready: BinaryHeap::with_capacity(16384),
            past: Vec::with_capacity(16384),
            edit_map: HashMap::with_capacity(16384),
            outputs,
            time: 0.0,
            sample_rate: DEFAULT_SR,
            sample_duration: 1.0 / DEFAULT_SR,
            buffer: BufferVec::new(outputs),
            tick_buffer: vec![0.0; outputs],
            front: None,
            replay_events,
        }
    }

    /// Current time in seconds.
    /// This method is not applicable to frontends, which do not process audio.
    pub fn time(&self) -> f64 {
        self.time
    }

    /// Add an event. All times are specified in seconds.
    /// Fade in and fade out may overlap but may not exceed the duration of the event.
    /// Returns the ID of the event.
    pub fn push(
        &mut self,
        start_time: f64,
        end_time: f64,
        fade_ease: Fade,
        fade_in_time: f64,
        fade_out_time: f64,
        mut unit: Box<dyn AudioUnit>,
    ) -> EventId {
        assert_eq!(unit.inputs(), 0);
        assert_eq!(unit.outputs(), self.outputs);
        let duration = end_time - start_time;
        assert!(fade_in_time <= duration && fade_out_time <= duration);
        // Make sure the sample rate of the unit matches ours.
        unit.set_sample_rate(self.sample_rate);
        unit.allocate();
        let event = Event::new(
            unit,
            start_time,
            end_time,
            fade_ease,
            fade_in_time,
            fade_out_time,
        );
        let id = event.id;
        self.push_event(event);
        id
    }

    /// Add event. This is an internal method.
    pub(crate) fn push_event(&mut self, event: Event) {
        if let Some((sender, receiver)) = &mut self.front {
            // Deallocate all past events.
            while receiver.try_recv().is_ok() {}
            // Send the new event over.
            if sender.try_send(Message::Push(event)).is_ok() {}
        } else if event.start_time < self.active_threshold {
            self.active_map.insert(event.id, self.active.len());
            self.active.push(event);
        } else {
            self.ready.push(event);
        }
    }

    /// Add an event. All times are specified in seconds.
    /// Start and end times are relative to current time.
    /// A start time of zero will start the event as soon as possible.
    /// Fade in and fade out may overlap but may not exceed the duration of the event.
    /// Returns the ID of the event.
    pub fn push_relative(
        &mut self,
        start_time: f64,
        end_time: f64,
        fade_ease: Fade,
        fade_in_time: f64,
        fade_out_time: f64,
        mut unit: Box<dyn AudioUnit>,
    ) -> EventId {
        assert!(unit.inputs() == 0 && unit.outputs() == self.outputs);
        let duration = end_time - start_time;
        assert!(fade_in_time <= duration && fade_out_time <= duration);
        // Make sure the sample rate of the unit matches ours.
        unit.set_sample_rate(self.sample_rate);
        unit.allocate();
        let event = Event::new(
            unit,
            start_time,
            end_time,
            fade_ease,
            fade_in_time,
            fade_out_time,
        );
        let id = event.id;
        self.push_relative_event(event);
        id
    }

    /// Add relative event. This is an internal method.
    pub(crate) fn push_relative_event(&mut self, mut event: Event) {
        if let Some((sender, receiver)) = &mut self.front {
            // Deallocate all past events.
            while receiver.try_recv().is_ok() {}
            // Send the new event over.
            if sender.try_send(Message::PushRelative(event)).is_ok() {}
        } else {
            event.start_time += self.time;
            event.end_time += self.time;
            if event.start_time < self.active_threshold {
                self.active_map.insert(event.id, self.active.len());
                self.active.push(event);
            } else {
                self.ready.push(event);
            }
        }
    }

    /// Add an event using start time and duration.
    /// Fade in and fade out may overlap but may not exceed the duration of the event.
    /// Returns the ID of the event.
    pub fn push_duration(
        &mut self,
        start_time: f64,
        duration: f64,
        fade_ease: Fade,
        fade_in_time: f64,
        fade_out_time: f64,
        unit: Box<dyn AudioUnit>,
    ) -> EventId {
        self.push(
            start_time,
            start_time + duration,
            fade_ease,
            fade_in_time,
            fade_out_time,
            unit,
        )
    }

    /// Make a change to an existing event. Only the end time and fade out time
    /// of the event may be changed. The new end time can only be used to shorten events.
    /// Edits are intended to be used with events where we do not know ahead of time
    /// how long they need to play. The original end time can be set to infinity,
    /// for example.
    pub fn edit(&mut self, id: EventId, end_time: f64, fade_out_time: f64) {
        if let Some((sender, receiver)) = &mut self.front {
            // Deallocate all past events.
            while receiver.try_recv().is_ok() {}
            // Send the new edit over.
            if sender
                .try_send(Message::Edit(
                    id,
                    Edit {
                        end_time,
                        fade_out: fade_out_time,
                    },
                ))
                .is_ok()
            {}
        } else if self.active_map.contains_key(&id) {
            // The edit applies to an active event.
            let i = self.active_map[&id];
            self.active[i].end_time = end_time;
            self.active[i].fade_out = fade_out_time;
        } else if end_time < self.active_threshold {
            // The edit is already in the past.
        } else {
            // The edit is in the future.
            self.edit_map.insert(
                id,
                Edit {
                    end_time,
                    fade_out: fade_out_time,
                },
            );
        }
    }

    /// Make a change to an existing event. Only the end time and fade out time
    /// of the event may be changed. The new end time can only be used to shorten events.
    /// The end time is relative to current time.
    /// The event starts fading out immediately if end time is equal to fade out time.
    /// Edits are intended to be used with events where we do not know ahead of time
    /// how long they need to play. The original end time can be set to infinity,
    /// for example.
    pub fn edit_relative(&mut self, id: EventId, end_time: f64, fade_out_time: f64) {
        if let Some((sender, receiver)) = &mut self.front {
            // Deallocate all past events.
            while receiver.try_recv().is_ok() {}
            // Send the new edit over.
            if sender
                .try_send(Message::EditRelative(
                    id,
                    Edit {
                        end_time,
                        fade_out: fade_out_time,
                    },
                ))
                .is_ok()
            {}
        } else if self.active_map.contains_key(&id) {
            // The edit applies to an active event.
            let i = self.active_map[&id];
            self.active[i].end_time = self.time + end_time;
            self.active[i].fade_out = fade_out_time;
        } else if self.time + end_time < self.active_threshold {
            // The edit is already in the past.
        } else {
            // The edit is in the future.
            self.edit_map.insert(
                id,
                Edit {
                    end_time: self.time + end_time,
                    fade_out: fade_out_time,
                },
            );
        }
    }

    /// Move units that start before the end time to the active set.
    fn ready_to_active(&mut self, next_end_time: f64) {
        self.active_threshold = next_end_time - self.sample_duration * 0.5;
        while let Some(ready) = self.ready.peek() {
            // Test whether start time rounded to a sample comes before the end time,
            // which always falls on a sample.
            if ready.start_time < self.active_threshold {
                if let Some(mut ready) = self.ready.pop() {
                    self.active_map.insert(ready.id, self.active.len());
                    // Check for edits to the event.
                    if self.edit_map.contains_key(&ready.id) {
                        let edit = &self.edit_map[&ready.id];
                        ready.fade_out = edit.fade_out;
                        ready.end_time = edit.end_time;
                        self.edit_map.remove(&ready.id);
                    }
                    self.active.push(ready);
                }
            } else {
                break;
            }
        }
    }

    /// Create a real-time friendly backend for this sequencer.
    /// This sequencer is then the frontend and any changes made are reflected in the backend.
    /// The backend renders audio while the frontend manages memory and
    /// communicates changes made to the backend.
    /// The backend is initialized with the current state of the sequencer.
    /// This can be called only once for a sequencer.
    pub fn backend(&mut self) -> SequencerBackend {
        assert!(!self.has_backend());
        // Create huge channel buffers to make sure we don't run out of space easily.
        let (sender_a, receiver_a) = channel(16384);
        let (sender_b, receiver_b) = channel(16384);
        let mut sequencer = self.clone();
        sequencer.allocate();
        self.front = Some((sender_a, receiver_b));
        SequencerBackend::new(sender_b, receiver_a, sequencer)
    }

    /// Returns whether this sequencer has a backend.
    pub fn has_backend(&self) -> bool {
        self.front.is_some()
    }

    /// Returns whether we retain past events and replay them after a reset.
    pub fn replay_events(&self) -> bool {
        self.replay_events
    }

    /// Get past events. This is an internal method.
    pub(crate) fn get_past_event(&mut self) -> Option<Event> {
        self.past.pop()
    }

    /// Get ready events. This is an internal method.
    pub(crate) fn get_ready_event(&mut self) -> Option<Event> {
        self.ready.pop()
    }

    /// Get active events. This is an internal method.
    pub(crate) fn get_active_event(&mut self) -> Option<Event> {
        if let Some(event) = self.active.pop() {
            self.active_map.remove(&event.id);
            return Some(event);
        }
        None
    }
}

impl AudioUnit for Sequencer {
    fn reset(&mut self) {
        if self.replay_events {
            while let Some(ready) = self.ready.pop() {
                self.active.push(ready);
            }
            while let Some(past) = self.past.pop() {
                self.active.push(past);
            }
            for i in 0..self.active.len() {
                self.active[i].unit.reset();
            }
            while let Some(active) = self.active.pop() {
                self.ready.push(active);
            }
            self.active_map.clear();
        } else {
            while let Some(_ready) = self.ready.pop() {}
            while let Some(_past) = self.past.pop() {}
            while let Some(_active) = self.active.pop() {}
            self.edit_map.clear();
            self.active_map.clear();
        }
        self.time = 0.0;
        self.active_threshold = -f64::INFINITY;
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        if self.sample_rate != sample_rate {
            self.sample_rate = sample_rate;
            self.sample_duration = 1.0 / sample_rate;
            // Move everything to the active queue, then set sample rate and move
            // everything to the ready heap.
            while let Some(ready) = self.ready.pop() {
                self.active.push(ready);
            }
            while let Some(past) = self.past.pop() {
                self.active.push(past);
            }
            for i in 0..self.active.len() {
                self.active[i].unit.set_sample_rate(sample_rate);
            }
            while let Some(active) = self.active.pop() {
                self.ready.push(active);
            }
            self.active_map.clear();
            self.active_threshold = -f64::INFINITY;
        }
    }

    #[inline]
    fn tick(&mut self, input: &[f32], output: &mut [f32]) {
        if !self.replay_events {
            while let Some(_past) = self.past.pop() {}
        }
        for channel in 0..self.outputs {
            output[channel] = 0.0;
        }
        let end_time = self.time + self.sample_duration;
        self.ready_to_active(end_time);
        let mut i = 0;
        while i < self.active.len() {
            if self.active[i].end_time <= self.time + 0.5 * self.sample_duration {
                self.active_map.remove(&self.active[i].id);
                if i + 1 < self.active.len() {
                    self.active_map
                        .insert(self.active[self.active.len() - 1].id, i);
                }
                self.past.push(self.active.swap_remove(i));
            } else {
                self.active[i].unit.tick(input, &mut self.tick_buffer);
                if self.active[i].fade_in > 0.0 {
                    let fade_in = delerp(
                        self.active[i].start_time,
                        self.active[i].start_time + self.active[i].fade_in,
                        self.time,
                    ) as f32;
                    if fade_in < 1.0 {
                        match self.active[i].fade_ease {
                            Fade::Power => {
                                for channel in 0..self.outputs {
                                    self.tick_buffer[channel] *= sine_ease(fade_in);
                                }
                            }
                            Fade::Smooth => {
                                for channel in 0..self.outputs {
                                    self.tick_buffer[channel] *= smooth5(fade_in);
                                }
                            }
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
                        match self.active[i].fade_ease {
                            Fade::Power => {
                                for channel in 0..self.outputs {
                                    self.tick_buffer[channel] *= sine_ease(1.0 - fade_out);
                                }
                            }
                            Fade::Smooth => {
                                for channel in 0..self.outputs {
                                    self.tick_buffer[channel] *= smooth5(1.0 - fade_out);
                                }
                            }
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

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        if !self.replay_events {
            while let Some(_past) = self.past.pop() {}
        }
        for channel in 0..self.outputs {
            output.channel_mut(channel)[..simd_items(size)].fill(F32x::ZERO);
        }
        let end_time = self.time + self.sample_duration * size as f64;
        self.ready_to_active(end_time);
        let mut buffer_output = self.buffer.buffer_mut();
        let mut i = 0;
        while i < self.active.len() {
            if self.active[i].end_time <= self.time + 0.5 * self.sample_duration {
                self.active_map.remove(&self.active[i].id);
                if i + 1 < self.active.len() {
                    self.active_map
                        .insert(self.active[self.active.len() - 1].id, i);
                }
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
                        .process(end_index - start_index, input, &mut buffer_output);
                    fade_in(
                        self.sample_duration,
                        self.time,
                        end_time,
                        start_index,
                        end_index,
                        self.active[i].fade_ease.clone(),
                        self.active[i].fade_in,
                        self.active[i].start_time,
                        &mut buffer_output,
                    );
                    fade_out(
                        self.sample_duration,
                        self.time,
                        end_time,
                        start_index,
                        end_index,
                        self.active[i].fade_ease.clone(),
                        self.active[i].fade_out,
                        self.active[i].end_time,
                        &mut buffer_output,
                    );
                    if start_index == 0 {
                        for channel in 0..self.outputs {
                            for j in 0..end_index >> SIMD_S {
                                output.add(channel, j, buffer_output.at(channel, j));
                            }
                            for j in end_index & !SIMD_M..end_index {
                                output.channel_f32_mut(channel)[j] +=
                                    buffer_output.channel_f32(channel)[j - start_index];
                            }
                        }
                    } else {
                        for channel in 0..self.outputs {
                            for j in start_index..end_index {
                                output.channel_f32_mut(channel)[j] +=
                                    buffer_output.channel_f32(channel)[j - start_index];
                            }
                        }
                    }
                }
                i += 1;
            }
        }
        self.time = end_time;
    }

    fn get_id(&self) -> u64 {
        const ID: u64 = 64;
        ID
    }

    fn inputs(&self) -> usize {
        0
    }
    fn outputs(&self) -> usize {
        self.outputs
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        // Treat the sequencer as a generator.
        Routing::Generator(0.0).route(input, self.outputs())
    }

    fn footprint(&self) -> usize {
        core::mem::size_of::<Self>()
    }
}
