//! Realtime safe backend for Sequencer.

use super::audiounit::*;
use super::buffer::*;
use super::math::*;
use super::sequencer::*;
use super::signal::*;
use super::*;
use tinyvec::TinyVec;

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

#[derive(Default)]
pub(crate) struct SequencerMessage {
    pub edits: Vec<Message>,
}

#[derive(Default)]
pub(crate) struct SequencerReturn {
    pub msg: Box<TinyVec<[Option<SequencerMessage>; 256]>>,
    pub vec: Box<TinyVec<[Option<Event>; 256]>>,
}

pub struct SequencerBackend {
    /// For sending events for deallocation back to the frontend.
    pub(crate) sender: Arc<Queue<SequencerReturn>>,
    /// Return message that is being filled.
    pub(crate) fill_message: SequencerReturn,
    /// For receiving new events from the frontend.
    receiver: Arc<Queue<SequencerMessage>>,
    /// The backend sequencer.
    sequencer: Sequencer,
}

impl Clone for SequencerBackend {
    fn clone(&self) -> Self {
        // Allocate a dummy channel.
        let queue_event = Arc::new(Queue::<SequencerReturn>::new_const());
        let queue_message = Arc::new(Queue::<SequencerMessage>::new_const());
        SequencerBackend {
            sender: queue_event,
            receiver: queue_message,
            fill_message: SequencerReturn::default(),
            sequencer: self.sequencer.clone(),
        }
    }
}

impl SequencerBackend {
    /// Create new backend.
    pub(crate) fn new(
        sender: Arc<Queue<SequencerReturn>>,
        receiver: Arc<Queue<SequencerMessage>>,
        sequencer: Sequencer,
    ) -> Self {
        Self {
            sender,
            receiver,
            fill_message: SequencerReturn::default(),
            sequencer,
        }
    }

    /// Handle changes made to the backend.
    fn handle_messages(&mut self) {
        while let Some(mut message) = self.receiver.dequeue() {
            while let Some(msg) = message.edits.pop() {
                match msg {
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
            self.fill_message.msg.push(Some(message));
            if self.fill_message.msg.len() == self.fill_message.msg.capacity() {
                let mut return_msg = SequencerReturn::default();
                core::mem::swap(&mut self.fill_message, &mut return_msg);
                if self.sender.enqueue(return_msg).is_ok() {}
            }
        }
    }

    #[inline]
    fn send_back_past(&mut self) {
        while let Some(event) = self.sequencer.get_past_event() {
            self.return_event(event);
        }
    }

    #[inline]
    fn return_event(&mut self, event: Event) {
        self.fill_message.vec.push(Some(event));
        if self.fill_message.vec.len() == self.fill_message.vec.capacity() {
            let mut msg = SequencerReturn::default();
            core::mem::swap(&mut self.fill_message, &mut msg);
            if self.sender.enqueue(msg).is_ok() {}
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
                    self.return_event(event);
                }
                while let Some(event) = self.sequencer.get_ready_event() {
                    self.return_event(event);
                }
                while let Some(event) = self.sequencer.get_active_event() {
                    self.return_event(event);
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
