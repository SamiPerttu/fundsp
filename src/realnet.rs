//! Real-time friendly backend for Net.

use super::audiounit::*;
use super::buffer::*;
use super::math::*;
use super::net::*;
use super::setting::*;
use super::signal::*;
use super::*;

/// Message from frontend to backend.
#[derive(Default, Clone)]
pub(crate) enum NetMessage<T: AudioUnit + Clone> {
    #[default]
    Null,
    Net(Net<T>),
    Setting(Setting),
}

/// Message from backend to frontend.
#[derive(Default, Clone)]
pub(crate) enum NetReturn<T: AudioUnit + Clone> {
    #[default]
    Null,
    Net(Net<T>),
    Unit(T),
}

pub struct NetBackend<T: AudioUnit + Clone> {
    /// For sending versions for deallocation back to the frontend.
    sender: Option<Arc<Queue<NetReturn<T>>>>,
    /// For receiving new versions and settings from the frontend.
    receiver: Arc<Queue<NetMessage<T>>>,
    net: Net<T>,
}

impl<T: AudioUnit + Clone> Clone for NetBackend<T> {
    fn clone(&self) -> Self {
        // Allocate a dummy channel.
        let queue_return = Arc::new(Queue::<NetReturn<T>>::new_const());
        let queue_message = Arc::new(Queue::<NetMessage<T>>::new_const());
        Self {
            sender: Some(queue_return),
            receiver: queue_message,
            net: self.net.clone(),
        }
    }
}

impl<T: AudioUnit + Clone> NetBackend<T> {
    /// Create new backend.
    pub(crate) fn new(
        sender: Arc<Queue<NetReturn<T>>>,
        receiver: Arc<Queue<NetMessage<T>>>,
        net: Net<T>,
    ) -> Self {
        Self {
            sender: Some(sender),
            receiver,
            net,
        }
    }

    /// Handle changes made to the backend.
    fn handle_messages(&mut self) {
        let mut latest_net: Option<Net<T>> = None;
        #[allow(clippy::while_let_loop)]
        loop {
            match self.receiver.dequeue() {
                Some(message) => {
                    match message {
                        NetMessage::Net(net) => {
                            if let Some(mut net) = latest_net {
                                // This is not the latest network, send it back immediately for deallocation.
                                self.net.apply_foreign_edits(&mut net, &self.sender);
                                if self
                                    .sender
                                    .as_ref()
                                    .unwrap()
                                    .enqueue(NetReturn::Net(net))
                                    .is_ok()
                                {}
                            }
                            latest_net = Some(net);
                        }
                        NetMessage::Setting(setting) => {
                            self.net.set(setting);
                        }
                        NetMessage::Null => (),
                    }
                }
                _ => break,
            }
        }
        if let Some(mut net) = latest_net {
            // Migrate existing nodes to the new network.
            self.net.migrate(&mut net);
            core::mem::swap(&mut net, &mut self.net);
            self.net.apply_edits(&self.sender);
            // Send the previous network back for deallocation.
            if self
                .sender
                .as_ref()
                .unwrap()
                .enqueue(NetReturn::Net(net))
                .is_ok()
            {}
        }
    }
}

impl<T: AudioUnit + Clone> AudioUnit for NetBackend<T> {
    fn inputs(&self) -> usize {
        self.net.inputs()
    }

    fn outputs(&self) -> usize {
        self.net.outputs()
    }

    fn reset(&mut self) {
        self.net.reset();
        self.handle_messages();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.net.set_sample_rate(sample_rate);
        self.handle_messages();
    }

    fn tick(&mut self, input: &[f32], output: &mut [f32]) {
        self.handle_messages();
        self.net.tick_2(input, output, &self.sender);
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        self.handle_messages();
        self.net.process_2(size, input, output, &self.sender);
    }

    fn get_id(&self) -> u64 {
        self.net.get_id()
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.handle_messages();
        self.net.ping(probe, hash)
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.handle_messages();
        self.net.route(input, frequency)
    }

    fn footprint(&self) -> usize {
        self.net.footprint()
    }

    fn allocate(&mut self) {
        self.net.allocate();
    }
}
