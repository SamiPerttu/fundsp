//! Real-time friendly backend for Net64 and Net32.

use super::audiounit::*;
use super::math::*;
use super::net::*;
use super::signal::*;
use duplicate::duplicate_item;
use thingbuf::mpsc::blocking::{channel, Receiver, Sender};

#[duplicate_item(
    f48       Net48       NetBackend48       Vertex48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ NetBackend64 ]   [ Vertex64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ NetBackend32 ]   [ Vertex32 ]   [ AudioUnit32 ];
)]
pub struct NetBackend48 {
    /// For sending versions for deallocation back to the frontend.
    sender: Sender<Net48>,
    /// For receiving new versions from the frontend.
    receiver: Receiver<Net48>,
    net: Net48,
}

#[duplicate_item(
    f48       Net48       NetBackend48       Vertex48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ NetBackend64 ]   [ Vertex64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ NetBackend32 ]   [ Vertex32 ]   [ AudioUnit32 ];
)]
impl Clone for NetBackend48 {
    fn clone(&self) -> Self {
        // Allocate a dummy channel.
        let (sender, receiver) = channel(1);
        NetBackend48 {
            sender,
            receiver,
            net: self.net.clone(),
        }
    }
}

#[duplicate_item(
    f48       Net48       NetBackend48       Vertex48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ NetBackend64 ]   [ Vertex64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ NetBackend32 ]   [ Vertex32 ]   [ AudioUnit32 ];
)]
impl NetBackend48 {
    /// Create new backend.
    pub fn new(sender: Sender<Net48>, receiver: Receiver<Net48>, net: Net48) -> Self {
        Self {
            sender,
            receiver,
            net,
        }
    }

    /// Handle changes made to the backend.
    fn handle_messages(&mut self) {
        let mut latest_net: Option<Net48> = None;
        #[allow(clippy::while_let_loop)]
        loop {
            match self.receiver.try_recv() {
                Ok(net) => {
                    if let Some(net) = latest_net {
                        // This is not the latest network, send it back immediately for deallocation.
                        if self.sender.try_send(net).is_ok() {}
                    }
                    latest_net = Some(net)
                }
                _ => break,
            }
        }
        if let Some(mut net) = latest_net {
            // Migrate existing nodes to the new network.
            self.net.migrate(&mut net);
            std::mem::swap(&mut net, &mut self.net);
            // Send the previous network back for deallocation.
            if self.sender.try_send(net).is_ok() {}
        }
    }
}

#[duplicate_item(
    f48       Net48       NetBackend48       Vertex48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ NetBackend64 ]   [ Vertex64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ NetBackend32 ]   [ Vertex32 ]   [ AudioUnit32 ];
)]
impl AudioUnit48 for NetBackend48 {
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

    fn tick(&mut self, input: &[f48], output: &mut [f48]) {
        self.handle_messages();
        self.net.tick(input, output);
    }

    fn process(&mut self, size: usize, input: &[&[f48]], output: &mut [&mut [f48]]) {
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
