//! Setting system.

use super::audionode::*;
use super::buffer::*;
use super::combinator::*;
use super::math::*;
use super::net::NodeId;
use super::signal::*;
pub use thingbuf::mpsc::errors::TrySendError;
use thingbuf::mpsc::{channel, Receiver, Sender};
use tinyvec::ArrayVec;

/// Parameters specify what to set and to what value.
#[derive(Default, Clone)]
pub enum Parameter {
    /// Default value.
    #[default]
    Null,
    /// Set filter center or cutoff frequency (Hz).
    Center(f32),
    /// Set filter center or cutoff frequency (Hz) and Q value.
    CenterQ(f32, f32),
    /// Set filter center or cutoff frequency (Hz), Q value and amplitude gain.
    CenterQGain(f32, f32, f32),
    /// Set filter center frequency (Hz) and bandwidth (Hz).
    CenterBandwidth(f32, f32),
    /// Set miscellaneous value.
    Value(f32),
    /// Set filter coefficient.
    Coefficient(f32),
    /// Set biquad parameters `(a1, a2, b0, b1, b2)`.
    Biquad(f32, f32, f32, f32, f32),
    /// Set delay.
    Delay(f32),
    /// Set response time.
    Time(f32),
    /// Set oscillator roughness in 0...1.
    Roughness(f32),
    /// Set sample-and-hold variability in 0...1.
    Variability(f32),
    /// Set stereo pan in -1...1.
    Pan(f32),
    /// Set attack and release times in seconds.
    AttackRelease(f32, f32),
}

/// Address specifies location to apply setting in a graph.
#[derive(Default, Clone)]
pub enum Address {
    /// Default value.
    #[default]
    Null,
    /// Take the left branch of a binary operation.
    Left,
    /// Take the right branch of a binary operation.
    Right,
    /// Specify node index.
    Index(usize),
    /// Specify node ID in `Net`.
    Node(NodeId),
}

/// Settings are node parameters with no dedicated inputs.
/// Nodes inside nodes can be accessed in the setting system by including an address
/// in the setting. Up to four levels of address are supported.
#[derive(Clone, Default)]
pub struct Setting {
    parameter: Parameter,
    address: ArrayVec<[Address; 4]>,
}

impl Setting {
    /// Create setting for a center parameter.
    pub fn center(center: f32) -> Self {
        Self {
            parameter: Parameter::Center(center),
            address: ArrayVec::new(),
        }
    }
    /// Create setting for center and Q parameters.
    pub fn center_q(center: f32, q: f32) -> Self {
        Self {
            parameter: Parameter::CenterQ(center, q),
            address: ArrayVec::new(),
        }
    }
    /// Create setting for center, Q and gain parameters.
    pub fn center_q_gain(center: f32, q: f32, gain: f32) -> Self {
        Self {
            parameter: Parameter::CenterQGain(center, q, gain),
            address: ArrayVec::new(),
        }
    }
    /// Create setting for center and bandwidth parameters.
    pub fn center_bandwidth(center: f32, bandwidth: f32) -> Self {
        Self {
            parameter: Parameter::CenterBandwidth(center, bandwidth),
            address: ArrayVec::new(),
        }
    }
    /// Create setting for constant values.
    pub fn value(value: f32) -> Self {
        Self {
            parameter: Parameter::Value(value),
            address: ArrayVec::new(),
        }
    }
    /// Create setting for biquad filter coefficients.
    pub fn biquad(a1: f32, a2: f32, b0: f32, b1: f32, b2: f32) -> Self {
        Self {
            parameter: Parameter::Biquad(a1, a2, b0, b1, b2),
            address: ArrayVec::new(),
        }
    }
    /// Create setting for delay.
    pub fn delay(delay: f32) -> Self {
        Self {
            parameter: Parameter::Delay(delay),
            address: ArrayVec::new(),
        }
    }
    /// Create setting for response time.
    pub fn time(time: f32) -> Self {
        Self {
            parameter: Parameter::Time(time),
            address: ArrayVec::new(),
        }
    }
    /// Create setting for roughness in 0...1.
    pub fn roughness(roughness: f32) -> Self {
        Self {
            parameter: Parameter::Roughness(roughness),
            address: ArrayVec::new(),
        }
    }
    /// Create setting for sample-and-hold variability in 0...1.
    pub fn variability(variability: f32) -> Self {
        Self {
            parameter: Parameter::Variability(variability),
            address: ArrayVec::new(),
        }
    }
    /// Create setting for pan in -1...1.
    pub fn pan(pan: f32) -> Self {
        Self {
            parameter: Parameter::Pan(pan),
            address: ArrayVec::new(),
        }
    }
    /// Create setting for attack and release times in seconds.
    pub fn attack_release(attack: f32, release: f32) -> Self {
        Self {
            parameter: Parameter::AttackRelease(attack, release),
            address: ArrayVec::new(),
        }
    }
    /// Add indexed address to setting.
    pub fn index(mut self, index: usize) -> Self {
        self.address.push(Address::Index(index));
        self
    }
    /// Add Net contained node address to setting.
    pub fn node(mut self, id: NodeId) -> Self {
        self.address.push(Address::Node(id));
        self
    }
    /// Add left choice address to setting.
    pub fn left(mut self) -> Self {
        self.address.push(Address::Left);
        self
    }
    /// Add right choice address to setting.
    pub fn right(mut self) -> Self {
        self.address.push(Address::Right);
        self
    }
    /// Access parameter.
    pub fn parameter(&self) -> &Parameter {
        &self.parameter
    }
    /// Get the next level of address. This is used by structural nodes.
    pub fn direction(&self) -> Address {
        if self.address.is_empty() {
            Address::Null
        } else {
            self.address[0].clone()
        }
    }
    /// Peel one address level. This is used by structural nodes.
    pub fn peel(mut self) -> Self {
        if !self.address.is_empty() {
            self.address.remove(0);
        }
        self
    }
}

#[derive(Clone)]
pub struct SettingSender {
    sender: Sender<Setting>,
}

impl SettingSender {
    pub fn new(sender: Sender<Setting>) -> Self {
        Self { sender }
    }
    pub fn try_send(&self, setting: Setting) -> Result<(), TrySendError<Setting>> {
        self.sender.try_send(setting)
    }
}

/// Setting listener using MPSC from the thingbuf crate.
pub struct SettingListener<X: AudioNode> {
    x: X,
    rv: Receiver<Setting>,
}

impl<X: AudioNode> Clone for SettingListener<X> {
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
pub fn listen<X: AudioNode>(node: An<X>) -> (SettingSender, An<SettingListener<X>>) {
    let (sender, node) = SettingListener::new(node.0);
    (sender, An(node))
}

impl<X: AudioNode> SettingListener<X> {
    pub fn new(x: X) -> (SettingSender, Self) {
        let (sender, receiver) = channel(64);
        let mut node = Self { rv: receiver, x };
        let hash = node.ping(true, AttoHash::new(Self::ID));
        node.ping(false, hash);
        let sender = SettingSender::new(sender);
        (sender, node)
    }
    fn receive_settings(&mut self) {
        while let Result::Ok(setting) = self.rv.try_recv() {
            self.set(setting);
        }
    }
}

impl<X: AudioNode> AudioNode for SettingListener<X> {
    const ID: u64 = 71;
    type Inputs = X::Inputs;
    type Outputs = X::Outputs;

    fn reset(&mut self) {
        self.receive_settings();
        self.x.reset();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.receive_settings();
        self.x.set_sample_rate(sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        self.receive_settings();
        self.x.tick(input)
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        self.receive_settings();
        self.x.process(size, input, output);
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.x.ping(probe, hash.hash(Self::ID))
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.receive_settings();
        self.x.route(input, frequency)
    }
}
