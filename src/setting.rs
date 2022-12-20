//! Setting listener using MPSC from the thingbuf crate.
use super::audionode::*;
use super::combinator::*;
use super::math::*;
use super::signal::*;
pub use thingbuf::mpsc::Sender;
use thingbuf::mpsc::{channel, Receiver};

pub struct Listen<X: AudioNode> {
    x: X,
    rv: Receiver<X::Setting>,
}

impl<X: AudioNode> Clone for Listen<X> {
    fn clone(&self) -> Self {
        // Receiver cannot be cloned, so instantiate a dummy channel.
        let (_sender, receiver) = channel(1);
        Self {
            x: self.x.clone(),
            rv: receiver,
        }
    }
}

/// Instantiate setting listener for `node`. Returns pair `(sender, node)`
/// where `node` is now equipped with a setting listener and settings can be
/// sent through `sender`. The format of settings depends on the type of the node.
pub fn listen<X: AudioNode>(node: An<X>) -> (Sender<X::Setting>, An<Listen<X>>) {
    let (sender, node) = Listen::new(node.0);
    (sender, An(node))
}

impl<X: AudioNode> Listen<X> {
    pub fn new(x: X) -> (Sender<X::Setting>, Self) {
        let (sender, receiver) = channel(64);
        let mut node = Self { rv: receiver, x };
        let hash = node.ping(true, AttoRand::new(Self::ID));
        node.ping(false, hash);
        (sender, node)
    }
    fn receive_settings(&mut self) {
        while let Result::Ok(setting) = self.rv.try_recv() {
            self.set(setting);
        }
    }
}

impl<X: AudioNode> AudioNode for Listen<X> {
    const ID: u64 = 71;
    type Sample = X::Sample;
    type Inputs = X::Inputs;
    type Outputs = X::Outputs;
    type Setting = X::Setting;

    #[inline]
    fn set(&mut self, setting: Self::Setting) {
        self.x.set(setting);
    }

    #[inline]
    fn reset(&mut self, sample_rate: Option<f64>) {
        self.receive_settings();
        self.x.reset(sample_rate);
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        self.receive_settings();
        self.x.tick(input)
    }

    #[inline]
    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        self.receive_settings();
        self.x.process(size, input, output);
    }

    #[inline]
    fn ping(&mut self, probe: bool, hash: AttoRand) -> AttoRand {
        self.x.ping(probe, hash.hash(Self::ID))
    }

    #[inline]
    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.receive_settings();
        self.x.route(input, frequency)
    }
}
