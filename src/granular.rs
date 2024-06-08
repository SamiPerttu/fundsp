//! Granular synthesizer. WIP.

use super::audiounit::*;
use super::buffer::*;
use super::math::*;
use super::sequencer::*;
use super::signal::*;
use super::*;
use funutd::dna::*;
use funutd::map3base::{Texture, TilingMode};
use funutd::*;
extern crate alloc;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

#[derive(Clone)]
struct Voice {
    /// Starting time of the next grain in this voice.
    pub next_time: f64,
}

/// Granular synthesizer. The synthesizer works by tracing paths in 3-D space and using
/// values obtained from a 3-D procedural texture to spawn grains. The traced path forms
/// a helix (corkscrew) shape.
#[derive(Clone)]
pub struct Granular<
    X: Fn(f64, f32, f32, f32, f32, f32) -> (f32, f32, Box<dyn AudioUnit>) + Sync + Send + Clone,
> {
    voices: Vec<Voice>,
    outputs: usize,
    beat_length: f32,
    beats_per_cycle: usize,
    texture: Box<dyn Texture>,
    jitter: f32,
    texture_origin: Vec3a,
    inner_radius: f32,
    outer_radius: f32,
    generator: X,
    sequencer: Sequencer,
    sample_rate: f64,
    time: f64,
    rnd_seed: u64,
    rnd: Rnd,
}

impl<
        X: Fn(f64, f32, f32, f32, f32, f32) -> (f32, f32, Box<dyn AudioUnit>) + Sync + Send + Clone,
    > Granular<X>
{
    /// Create a new granular synthesizer.
    /// - `outputs`: number of outputs.
    /// - `voices`: number of parallel voices traced along a helix. For example, 16.
    /// - `beat_length`: length of 1 revolution along the helix in seconds. For example, 1 second.
    /// - `beats_per_cycle`: how many revolutions until the helix returns to its point of origin. For example, 8 or 16.
    ///    The higher this number is, the more rhythmic it will sound, due to correlations between successive revolutions.
    /// - `texture_seed`: seed of the texture which is sampled to get data for grains.
    /// - `inner_radius`: inner radius of the helix. The first voice is at the inner radius. For example, 0.1.
    /// - `outer_radius`: outer radius of the helix. The last voice is at the outer radius. For example, 0.2.
    /// - `jitter`: amount of random jitter added to sample points on the helix. For example, 0.0 or 0.01.
    /// - `generator`: the generator function `f(t, b, v, x, y, z)` for grains. `t` is time in seconds
    /// and `b` is fractional beat number starting from zero. The rest of the parameters are in the range -1...1.
    /// `v` is a voice indicator and `x`, `y` and `z` are values obtained from our texture.
    /// The generator function returns the triple (grain length, envelope length, grain graph).
    /// Lengths are in seconds.
    /// For example, `|t, b, v, x, y, z| (0.06, 0.03, Box::new(sine_hz(xerp11(20.0, 4000.0, x)) * xerp11(0.0002, 0.02, y) >> pan(v * 0.5)))`.
    pub fn new(
        outputs: usize,
        voices: usize,
        beat_length: f32,
        beats_per_cycle: usize,
        texture_seed: u64,
        inner_radius: f32,
        outer_radius: f32,
        jitter: f32,
        generator: X,
    ) -> Self {
        let voice_vector = vec![Voice { next_time: 0.0 }; voices];
        let mut dna = Dna::new(texture_seed);
        let texture = funutd::map3gen::genmap3(100.0, TilingMode::Z, &mut dna);
        let mut granular = Self {
            voices: voice_vector,
            outputs,
            beat_length,
            beats_per_cycle,
            texture,
            jitter,
            texture_origin: vec3a(0.0, 0.0, 0.0),
            inner_radius,
            outer_radius,
            generator,
            sequencer: Sequencer::new(false, outputs),
            sample_rate: DEFAULT_SR,
            time: 0.0,
            rnd_seed: texture_seed,
            rnd: Rnd::from_u64(texture_seed),
        };
        granular.reset();
        granular
    }

    /// Position in space at the given time for the given voice.
    fn helix_position(&mut self, voice: usize, time: f64) -> Vec3a {
        let cycle_length = self.beat_length as f64 * self.beats_per_cycle as f64;
        let cycle = (time / cycle_length).floor();
        let cycle_start = cycle * cycle_length;
        let z_depth = 1.0;
        let cycle_d = (time - cycle_start) / cycle_length;
        let z = cycle_d * z_depth;
        let beat = cycle_d * self.beats_per_cycle as f64;
        let voice_d = if self.voices.len() == 1 {
            0.5
        } else {
            voice as f32 / (self.voices.len() - 1) as f32
        };
        let r = lerp(self.inner_radius, self.outer_radius, voice_d);
        let x = cos(beat as f32 * f32::TAU) * r;
        let y = sin(beat as f32 * f32::TAU) * r;
        let random = vec3a(
            self.rnd.f32() * 2.0 - 1.0,
            self.rnd.f32() * 2.0 - 1.0,
            self.rnd.f32() * 2.0 - 1.0,
        ) * self.jitter;
        self.texture_origin + vec3a(x as f32, y as f32, z as f32) + random
    }

    /// Instantiate a grain.
    fn instantiate(&mut self, voice: usize) {
        let t = self.voices[voice].next_time;
        let position = self.helix_position(voice, t);
        let v = self.texture.at(position);
        let voice_d = if self.voices.len() == 1 {
            0.5
        } else {
            voice as f64 / (self.voices.len() - 1) as f64
        };
        let (grain_length, envelope_length, mut grain) = (self.generator)(
            t,
            (t / self.beat_length as f64) as f32,
            (voice_d * 2.0 - 1.0) as f32,
            v.x.clamp(-1.0, 1.0),
            v.y.clamp(-1.0, 1.0),
            v.z.clamp(-1.0, 1.0),
        );
        assert!(envelope_length >= 0.0);
        assert!(envelope_length < grain_length);
        if t == 0.0 {
            assert_eq!(voice, 0);
            // Offset voice start times based on the first grain.
            for i in 1..self.voices.len() {
                self.voices[i].next_time = (grain_length as f64 - envelope_length as f64)
                    * i as f64
                    / self.voices.len() as f64;
            }
        }
        self.voices[voice].next_time = t + grain_length as f64 - envelope_length as f64;
        // Use a random phase for each individual grain.
        grain.ping(false, AttoHash::new(self.rnd.u64()));
        self.sequencer.push_duration(
            t,
            grain_length as f64,
            Fade::Power,
            envelope_length as f64,
            envelope_length as f64,
            grain,
        );
    }

    /// Check all voices and instantiate grains that start before the given time.
    fn instantiate_voices(&mut self, before_time: f64) {
        for voice in 0..self.voices.len() {
            while self.voices[voice].next_time < before_time {
                self.instantiate(voice);
            }
        }
    }
}

impl<
        X: Fn(f64, f32, f32, f32, f32, f32) -> (f32, f32, Box<dyn AudioUnit>) + Sync + Send + Clone,
    > AudioUnit for Granular<X>
{
    fn reset(&mut self) {
        self.sequencer.reset();
        for i in 0..self.voices.len() {
            self.voices[i].next_time = 0.0;
        }
        self.time = 0.0;
        self.rnd = Rnd::from_u64(self.rnd_seed);
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
        self.sequencer.set_sample_rate(sample_rate);
    }

    fn tick(&mut self, input: &[f32], output: &mut [f32]) {
        self.time += 1.0 / self.sample_rate;
        self.instantiate_voices(self.time);
        self.sequencer.tick(input, output);
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        self.time += size as f64 / self.sample_rate;
        self.instantiate_voices(self.time);
        self.sequencer.process(size, input, output);
    }

    fn get_id(&self) -> u64 {
        const ID: u64 = 72;
        ID
    }

    fn set_hash(&mut self, hash: u64) {
        self.rnd_seed = hash;
        self.rnd = Rnd::from_u64(self.rnd_seed);
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
        core::mem::size_of::<Self>()
    }
}
