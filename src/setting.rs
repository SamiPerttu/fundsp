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

/// Instantiate setting listener for `network`. Returns pair `(sender, network)`
/// where `network` is now equipped with a setting listener and settings can be
/// sent tnrough `sender`. The format of settings depends on the type of the network.
pub fn listen<X: AudioNode>(network: An<X>) -> (Sender<X::Setting>, Listen<X>) {
    Listen::new(network.0)
}

impl<X: AudioNode> Listen<X> {
    pub fn new(x: X) -> (Sender<X::Setting>, Self) {
        let (sender, receiver) = channel(64);
        let mut node = Self { rv: receiver, x };
        let hash = node.ping(true, AttoRand::new(Self::ID));
        node.ping(false, hash);
        (sender, node)
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
        while let Result::Ok(setting) = self.rv.try_recv() {
            self.set(setting);
        }
        self.x.reset(sample_rate);
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        while let Result::Ok(setting) = self.rv.try_recv() {
            self.set(setting);
        }
        self.x.tick(input)
    }

    #[inline]
    fn process(
        &mut self,
        size: usize,
        input: &[&[Self::Sample]],
        output: &mut [&mut [Self::Sample]],
    ) {
        while let Result::Ok(setting) = self.rv.try_recv() {
            self.set(setting);
        }
        self.x.process(size, input, output);
    }

    #[inline]
    fn set_hash(&mut self, hash: u64) {
        self.x.set_hash(hash);
    }

    #[inline]
    fn ping(&mut self, probe: bool, hash: AttoRand) -> AttoRand {
        self.x.ping(probe, hash.hash(Self::ID))
    }

    #[inline]
    fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.x.route(input, frequency)
    }
}
