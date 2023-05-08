//! Real-time friendly backend for the sequencer unit.

use super::audiounit::*;
use super::math::*;
use super::sequencer::*;
use super::signal::*;
use duplicate::duplicate_item;
use thingbuf::mpsc::blocking::{channel, Receiver, Sender};

#[duplicate_item(
    f48       Sequencer48       Message48       Sequencer48Backend       Event48       AudioUnit48       Edit48;
    [ f64 ]   [ Sequencer64 ]   [ Message64 ]   [ Sequencer64Backend ]   [ Event64 ]   [ AudioUnit64 ]   [ Edit64 ];
    [ f32 ]   [ Sequencer32 ]   [ Message32 ]   [ Sequencer32Backend ]   [ Event32 ]   [ AudioUnit32 ]   [ Edit32 ];
)]
#[derive(Default)]
pub enum Message48 {
    /// Nothing.
    #[default]
    Null,
    /// Add new event in absolute time.
    Push(Event48),
    /// Add new event in relative time.
    PushRelative(Event48),
    /// Edit event.
    Edit(EventId, Edit48),
    /// Edit event in relative time.
    EditRelative(EventId, Edit48),
}

#[duplicate_item(
    f48       Sequencer48       Message48       Sequencer48Backend       Event48       AudioUnit48;
    [ f64 ]   [ Sequencer64 ]   [ Message64 ]   [ Sequencer64Backend ]   [ Event64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Sequencer32 ]   [ Message32 ]   [ Sequencer32Backend ]   [ Event32 ]   [ AudioUnit32 ];
)]
impl Clone for Message48 {
    fn clone(&self) -> Self {
        Message48::Null
    }
}

#[duplicate_item(
    f48       Sequencer48       Message48       Sequencer48Backend       Event48       AudioUnit48;
    [ f64 ]   [ Sequencer64 ]   [ Message64 ]   [ Sequencer64Backend ]   [ Event64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Sequencer32 ]   [ Message32 ]   [ Sequencer32Backend ]   [ Event32 ]   [ AudioUnit32 ];
)]
pub struct Sequencer48Backend {
    /// For sending events for deallocation back to the frontend.
    pub sender: Sender<Option<Event48>>,
    /// For receiving new events from the frontend.
    receiver: Receiver<Message48>,
    sequencer: Sequencer48,
}

#[duplicate_item(
    f48       Sequencer48       Message48       Sequencer48Backend       Event48       AudioUnit48;
    [ f64 ]   [ Sequencer64 ]   [ Message64 ]   [ Sequencer64Backend ]   [ Event64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Sequencer32 ]   [ Message32 ]   [ Sequencer32Backend ]   [ Event32 ]   [ AudioUnit32 ];
)]
impl Clone for Sequencer48Backend {
    fn clone(&self) -> Self {
        // Allocate a dummy channel.
        let (sender_1, _receiver_1) = channel(1);
        let (_sender_2, receiver_2) = channel(1);
        Sequencer48Backend {
            sender: sender_1,
            receiver: receiver_2,
            sequencer: self.sequencer.clone(),
        }
    }
}

#[duplicate_item(
    f48       Sequencer48       Message48       Sequencer48Backend       Event48       AudioUnit48;
    [ f64 ]   [ Sequencer64 ]   [ Message64 ]   [ Sequencer64Backend ]   [ Event64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Sequencer32 ]   [ Message32 ]   [ Sequencer32Backend ]   [ Event32 ]   [ AudioUnit32 ];
)]
impl Sequencer48Backend {
    /// Create new backend.
    pub fn new(
        sender: Sender<Option<Event48>>,
        receiver: Receiver<Message48>,
        sequencer: Sequencer48,
    ) -> Self {
        Self {
            sender,
            receiver,
            sequencer,
        }
    }

    /// Handle changes made to the backend.
    fn handle_messages(&mut self) {
        while let Ok(message) = self.receiver.try_recv() {
            match message {
                Message48::Push(event) => {
                    self.sequencer.push_event(event);
                }
                Message48::PushRelative(event) => {
                    self.sequencer.push_relative_event(event);
                }
                Message48::Edit(id, edit) => {
                    self.sequencer.edit(id, edit.end_time, edit.fade_out);
                }
                Message48::EditRelative(id, edit) => {
                    self.sequencer
                        .edit_relative(id, edit.end_time, edit.fade_out);
                }
                Message48::Null => {}
            }
        }
    }

    #[inline]
    fn send_back_past(&mut self) {
        while let Some(event) = self.sequencer.get_past_event() {
            if self.sender.try_send(Some(event)).is_ok() {}
        }
    }
}

#[duplicate_item(
    f48       Sequencer48       Sequencer48Backend       Event48       AudioUnit48;
    [ f64 ]   [ Sequencer64 ]   [ Sequencer64Backend ]   [ Event64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Sequencer32 ]   [ Sequencer32Backend ]   [ Event32 ]   [ AudioUnit32 ];
)]
impl AudioUnit48 for Sequencer48Backend {
    fn inputs(&self) -> usize {
        0
    }

    fn outputs(&self) -> usize {
        self.sequencer.outputs()
    }

    fn reset(&mut self) {
        self.handle_messages();
        while let Some(event) = self.sequencer.get_past_event() {
            if self.sender.try_send(Some(event)).is_ok() {}
        }
        while let Some(event) = self.sequencer.get_ready_event() {
            if self.sender.try_send(Some(event)).is_ok() {}
        }
        while let Some(event) = self.sequencer.get_active_event() {
            if self.sender.try_send(Some(event)).is_ok() {}
        }
        self.sequencer.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.handle_messages();
        self.sequencer.set_sample_rate(sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &[f48], output: &mut [f48]) {
        self.handle_messages();
        self.sequencer.tick(input, output);
        // Tick and process are the only places where events may be pushed to the past vector.
        if !self.sequencer.replay_events() {
            self.send_back_past();
        }
    }

    fn process(&mut self, size: usize, input: &[&[f48]], output: &mut [&mut [f48]]) {
        self.handle_messages();
        self.sequencer.process(size, input, output);
        // Tick and process are the only places where events may be pushed to the past vector.
        if !self.sequencer.replay_events() {
            self.send_back_past();
        }
    }

    fn get_id(&self) -> u64 {
        self.sequencer.get_id()
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.handle_messages();
        self.sequencer.ping(probe, hash)
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.handle_messages();
        self.sequencer.route(input, frequency)
    }

    fn footprint(&self) -> usize {
        self.sequencer.footprint()
    }

    fn allocate(&mut self) {
        self.sequencer.allocate();
    }
}
