//! Realtime safe backend for Sequencer.

use super::audiounit::*;
use super::buffer::*;
use super::math::*;
use super::sequencer::*;
use super::signal::*;
use super::*;

#[derive(Default, Clone)]
pub(crate) enum Message {
    /// Reset the sequencer.
    #[default]
    Reset,
    /// Add new event in absolute time.
    Push(Event),
    /// Add new event in relative time.
    PushRelative(Event),
    /// Edit event.
    Edit(EventId, Edit),
    /// Edit event in relative time.
    EditRelative(EventId, Edit),
}

pub struct SequencerBackend {
    /// For sending events for deallocation back to the frontend.
    pub(crate) sender: Arc<Queue<Event, 256>>,
    /// For receiving new events from the frontend.
    receiver: Arc<Queue<Message, 256>>,
    /// The backend sequencer.
    sequencer: Sequencer,
}

impl Clone for SequencerBackend {
    fn clone(&self) -> Self {
        // Allocate a dummy channel.
        let queue_event = Arc::new(Queue::<Event, 256>::new_const());
        let queue_message = Arc::new(Queue::<Message, 256>::new_const());
        SequencerBackend {
            sender: queue_event,
            receiver: queue_message,
            sequencer: self.sequencer.clone(),
        }
    }
}

impl SequencerBackend {
    /// Create new backend.
    pub(crate) fn new(
        sender: Arc<Queue<Event, 256>>,
        receiver: Arc<Queue<Message, 256>>,
        sequencer: Sequencer,
    ) -> Self {
        Self {
            sender,
            receiver,
            sequencer,
        }
    }

    /// Handle changes made to the backend.
    fn handle_messages(&mut self) {
        while let Some(message) = self.receiver.dequeue() {
            match message {
                Message::Reset => {
                    self.reset();
                }
                Message::Push(event) => {
                    self.sequencer.push_event(event);
                }
                Message::PushRelative(event) => {
                    self.sequencer.push_relative_event(event);
                }
                Message::Edit(id, edit) => {
                    self.sequencer.edit(id, edit.end_time, edit.fade_out);
                }
                Message::EditRelative(id, edit) => {
                    self.sequencer
                        .edit_relative(id, edit.end_time, edit.fade_out);
                }
            }
        }
    }

    #[inline]
    fn send_back_past(&mut self) {
        while let Some(event) = self.sequencer.get_past_event() {
            if self.sender.enqueue(event).is_ok() {}
        }
    }
}

impl AudioUnit for SequencerBackend {
    fn inputs(&self) -> usize {
        0
    }

    fn outputs(&self) -> usize {
        self.sequencer.outputs()
    }

    fn reset(&mut self) {
        self.handle_messages();
        match self.sequencer.replay_mode() {
            ReplayMode::None => {
                while let Some(event) = self.sequencer.get_past_event() {
                    if self.sender.enqueue(event).is_ok() {}
                }
                while let Some(event) = self.sequencer.get_ready_event() {
                    if self.sender.enqueue(event).is_ok() {}
                }
                while let Some(event) = self.sequencer.get_active_event() {
                    if self.sender.enqueue(event).is_ok() {}
                }
            }
            _ => (),
        }
        self.sequencer.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.handle_messages();
        self.sequencer.set_sample_rate(sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &[f32], output: &mut [f32]) {
        self.handle_messages();
        self.sequencer.tick(input, output);
        // Tick and process are the only places where events may be pushed to the past vector.
        match self.sequencer.replay_mode() {
            ReplayMode::None => {
                self.send_back_past();
            }
            _ => (),
        }
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        self.handle_messages();
        self.sequencer.process(size, input, output);
        // Tick and process are the only places where events may be pushed to the past vector.
        match self.sequencer.replay_mode() {
            ReplayMode::None => {
                self.send_back_past();
            }
            _ => (),
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
