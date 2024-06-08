//! Dynamical system component.

use super::audionode::*;
use super::buffer::*;
use super::combinator::*;
use super::math::*;
use super::setting::*;
use super::signal::*;
use super::*;

/// A dynamical system is a node that has an attached update function
/// `f(t, dt, x)` where `t` is current time, `dt` is time elapsed since
/// the previous update, and `x` is the enclosed node.
#[derive(Clone)]
pub struct System<X: AudioNode, F: FnMut(f32, f32, &mut X) + Clone + Send + Sync> {
    x: X,
    f: F,
    time: f32,
    delta_time: f32,
    update_interval: f32,
    sample_rate: f32,
}

impl<X: AudioNode, F: FnMut(f32, f32, &mut X) + Clone + Send + Sync> System<X, F> {
    /// Create a new dynamical system.
    /// `dt` is the approximate target time between updates.
    pub fn new(x: An<X>, dt: f32, f: F) -> Self {
        let mut node = System {
            x: x.0,
            f,
            time: 0.0,
            delta_time: 0.0,
            update_interval: dt,
            sample_rate: DEFAULT_SR as f32,
        };
        let hash = node.ping(true, AttoHash::new(Self::ID));
        node.ping(false, hash);
        node
    }
}

impl<X: AudioNode, F: FnMut(f32, f32, &mut X) + Clone + Sync + Send> AudioNode for System<X, F> {
    const ID: u64 = 67;
    type Inputs = X::Inputs;
    type Outputs = X::Outputs;

    fn reset(&mut self) {
        self.x.reset();
        self.time = 0.0;
        self.delta_time = 0.0;
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.x.set_sample_rate(sample_rate);
        self.sample_rate = convert(sample_rate);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let delta_now = 1.0 / self.sample_rate;
        // The first update is always done at time zero.
        if self.delta_time >= self.update_interval || self.time == 0.0 {
            (self.f)(self.time, self.delta_time, &mut self.x);
            self.delta_time = 0.0;
        }
        self.time += delta_now;
        self.delta_time += delta_now;
        self.x.tick(input)
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        let delta_now = size as f32 / self.sample_rate;
        // The first update is always done at time zero.
        if self.delta_time >= self.update_interval || (self.time == 0.0 && size > 0) {
            (self.f)(self.time, self.delta_time, &mut self.x);
            self.delta_time = 0.0;
        }
        self.time += delta_now;
        self.delta_time += delta_now;
        self.x.process(size, input, output);
    }

    fn set(&mut self, setting: Setting) {
        self.x.set(setting);
    }

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        self.x.ping(probe, hash.hash(Self::ID))
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.x.route(input, frequency)
    }

    fn allocate(&mut self) {
        self.x.allocate();
    }
}
