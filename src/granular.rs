//! Granular synthesizer.

use super::audiounit::*;
use super::math::*;
use super::sequencer::*;
use super::signal::*;
use super::*;
use duplicate::duplicate_item;
use funutd::dna::*;
use funutd::map3base::{Texture, TilingMode};
use funutd::*;

#[duplicate_item(
      f48       Thread48;
    [ f64 ]   [ Thread64 ];
    [ f32 ]   [ Thread32 ];
)]
#[derive(Clone)]
struct Thread48 {
    pub next_time: f48,
}

/// Granular synthesizer. The synthesizer works by tracing paths in 3-D space and using
/// values obtained from a 3-D procedural texture to spawn grains. The traced path forms
/// a helix (corkscrew) shape.
#[duplicate_item(
      f48      Granular48      Thread48      AudioUnit48      Sequencer48;
    [ f64 ]  [ Granular64 ]  [ Thread64 ]  [ AudioUnit64 ]  [ Sequencer64 ];
    [ f32 ]  [ Granular32 ]  [ Thread32 ]  [ AudioUnit32 ]  [ Sequencer32 ];
)]
#[derive(Clone)]
pub struct Granular48<X: Fn(f48, f48, f48, f48) -> Box<dyn AudioUnit48> + Sync + Send + Clone> {
    threads: Vec<Thread48>,
    outputs: usize,
    grain_length: f48,
    envelope_length: f48,
    beat_length: f48,
    beats_per_cycle: usize,
    texture: Box<dyn Texture>,
    jitter: f48,
    texture_origin: Vec3a,
    inner_radius: f48,
    outer_radius: f48,
    generator: X,
    sequencer: Sequencer48,
    sample_rate: f48,
    time: f48,
    rnd: Rnd,
}

#[duplicate_item(
    f48      Granular48      Thread48      AudioUnit48      Sequencer48;
  [ f64 ]  [ Granular64 ]  [ Thread64 ]  [ AudioUnit64 ]  [ Sequencer64 ];
  [ f32 ]  [ Granular32 ]  [ Thread32 ]  [ AudioUnit32 ]  [ Sequencer32 ];
)]
impl<X: Fn(f48, f48, f48, f48) -> Box<dyn AudioUnit48> + Sync + Send + Clone> Granular48<X> {
    /// Create a new granular synthesizer.
    /// - `outputs`: number of outputs.
    /// - `threads`: number of parallel threads traced along a helix. For example, 16.
    /// - `grain_length`: length of a grain in seconds. For example, 0.05 seconds.
    /// - `envelope_length`: length of a grain envelope. May not exceed grain length. For example, 0.02 seconds.
    /// - `beat_length`: length of 1 revolution along the helix in seconds. For example, 1 second.
    /// - `beats_per_cycle`: how many revolutions until the helix returns to its point of origin. For example, 8.
    /// - `texture_seed`: seed of the texture which is sampled to get data for grains.
    /// - `inner_radius`: inner radius of the helix. For example, 0.1.
    /// - `outer_radius`: outer radius of the helix. For example, 0.2.
    /// - `jitter`: amount of random jitter added to sample points on the helix. For example, 0.0.
    /// - `generator`: the generator function `f(x, r, g, b)` for grains. All the parameters are in the range -1...1.
    pub fn new(
        outputs: usize,
        threads: usize,
        grain_length: f48,
        envelope_length: f48,
        beat_length: f48,
        beats_per_cycle: usize,
        texture_seed: u64,
        inner_radius: f48,
        outer_radius: f48,
        jitter: f48,
        generator: X,
    ) -> Self {
        let thread_vector = vec![Thread48 { next_time: 0.0 }; threads];
        let mut dna = Dna::new(texture_seed);
        let texture = funutd::map3gen::genmap3palette(20.0, TilingMode::Z, &mut dna);
        //let texture = funutd::map3gen::genmap3(20.0, TilingMode::Z, &mut dna);
        let mut granular = Self {
            threads: thread_vector,
            outputs,
            grain_length,
            envelope_length,
            beat_length,
            beats_per_cycle,
            texture,
            jitter,
            texture_origin: vec3a(0.0, 0.0, 0.0),
            inner_radius,
            outer_radius,
            generator,
            sequencer: Sequencer48::new(DEFAULT_SR, outputs),
            sample_rate: DEFAULT_SR as f48,
            time: 0.0,
            rnd: Rnd::from_u64(texture_seed),
        };
        granular.reset(None);
        granular
    }

    /// Position in space at the given time for the given thread.
    fn helix_position(&mut self, thread: usize, time: f48) -> Vec3a {
        let cycle_length = self.beat_length * self.beats_per_cycle as f48;
        let cycle = (time / cycle_length).floor();
        let cycle_start = cycle * cycle_length;
        let z_depth = 1.0;
        let cycle_d = (time - cycle_start) / cycle_length;
        let z = cycle_d * z_depth;
        let beat = cycle_d * self.beats_per_cycle as f48;
        let thread_d = if self.threads.len() == 1 {
            0.5
        } else {
            thread as f48 / (self.threads.len() - 1) as f48
        };
        let r = lerp(self.inner_radius, self.outer_radius, thread_d);
        let x = cos(beat * TAU as f48) * r;
        let y = sin(beat * TAU as f48) * r;
        let random = vec3a(
            self.rnd.f32() * 2.0 - 1.0,
            self.rnd.f32() * 2.0 - 1.0,
            self.rnd.f32() * 2.0 - 1.0,
        ) * self.jitter as f32;
        self.texture_origin + vec3a(x as f32, y as f32, z as f32) + random
    }
    /// Instantiate a grain.
    fn instantiate(&mut self, thread: usize) {
        let t = self.threads[thread].next_time;
        let position = self.helix_position(thread, t);
        self.threads[thread].next_time = t + self.grain_length - self.envelope_length;
        let v = self.texture.at(position);
        let thread_d = if self.threads.len() == 1 {
            0.5
        } else {
            thread as f48 / (self.threads.len() - 1) as f48
        };
        let mut grain = (self.generator)(thread_d * 2.0 - 1.0, v.x as f48, v.y as f48, v.z as f48);
        // Use a random phase for every individual grain.
        grain.ping(false, AttoRand::new(self.rnd.u64()));
        self.sequencer.add_duration(
            t,
            self.grain_length,
            self.envelope_length,
            self.envelope_length,
            grain,
        );
    }
    /// Check all threads and instantiate grains that start before the given time.
    fn instantiate_threads(&mut self, before_time: f48) {
        for i in 0..self.threads.len() {
            while self.threads[i].next_time < before_time {
                self.instantiate(i);
            }
        }
    }
}

#[duplicate_item(
    f48      Granular48      Thread48      AudioUnit48      Sequencer48;
  [ f64 ]  [ Granular64 ]  [ Thread64 ]  [ AudioUnit64 ]  [ Sequencer64 ];
  [ f32 ]  [ Granular32 ]  [ Thread32 ]  [ AudioUnit32 ]  [ Sequencer32 ];
)]
impl<X: Fn(f48, f48, f48, f48) -> Box<dyn AudioUnit48> + Sync + Send + Clone> AudioUnit48
    for Granular48<X>
{
    fn reset(&mut self, sample_rate: Option<f64>) {
        self.sequencer.reset(sample_rate);
        self.sequencer.erase_future();
        if let Some(rate) = sample_rate {
            self.sample_rate = rate as f48;
        }
        for i in 0..self.threads.len() {
            self.threads[i].next_time = self.grain_length * i as f48 / self.threads.len() as f48;
        }
        self.time = 0.0;
    }

    fn tick(&mut self, input: &[f48], output: &mut [f48]) {
        self.time += 1.0 / self.sample_rate;
        self.instantiate_threads(self.time);
        self.sequencer.tick(input, output);
    }

    fn process(&mut self, size: usize, input: &[&[f48]], output: &mut [&mut [f48]]) {
        self.time += size as f48 / self.sample_rate;
        self.instantiate_threads(self.time);
        self.sequencer.process(size, input, output);
    }

    fn get_id(&self) -> u64 {
        const ID: u64 = 72;
        ID
    }

    fn inputs(&self) -> usize {
        0
    }
    fn outputs(&self) -> usize {
        self.outputs
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        self.sequencer.route(input, frequency)
    }

    fn footprint(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
