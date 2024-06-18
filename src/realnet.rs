//! Real-time friendly backend for Net.

use super::audiounit::*;
use super::buffer::*;
use super::math::*;
use super::net::*;
use super::setting::*;
use super::signal::*;
use thingbuf::mpsc::{channel, Receiver, Sender};

#[derive(Default, Clone)]
pub(crate) enum NetMessage {
    #[default]
    Null,
    Net(Net),
    Setting(Setting),
}

pub struct NetBackend {
    /// For sending versions for deallocation back to the frontend.
    sender: Sender<Net>,
    /// For receiving new versions and settings from the frontend.
    receiver: Receiver<NetMessage>,
    net: Net,
}

impl Clone for NetBackend {
    fn clone(&self) -> Self {
        // Allocate a dummy channel.
        let (sender_net, _receiver_net) = channel(1);
        let (_sender_message, receiver_message) = channel(1);
        NetBackend {
            sender: sender_net,
            receiver: receiver_message,
            net: self.net.clone(),
        }
    }
}

impl NetBackend {
    /// Create new backend.
    pub(crate) fn new(sender: Sender<Net>, receiver: Receiver<NetMessage>, net: Net) -> Self {
        Self {
            sender,
            receiver,
            net,
        }
    }

    /// Handle changes made to the backend.
    fn handle_messages(&mut self) {
        let mut latest_net: Option<Net> = None;
        #[allow(clippy::while_let_loop)]
        loop {
            match self.receiver.try_recv() {
                Ok(message) => {
                    match message {
                        NetMessage::Net(net) => {
                            if let Some(net) = latest_net {
                                // This is not the latest network, send it back immediately for deallocation.
                                if self.sender.try_send(net).is_ok() {}
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
            // Send the previous network back for deallocation.
            if self.sender.try_send(net).is_ok() {}
        }
    }
}

impl AudioUnit for NetBackend {
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
        self.net.tick(input, output);
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        self.handle_messages();
        self.net.process(size, input, output);
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
