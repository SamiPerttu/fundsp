//! Reverberation related code.

use super::audionode::*;
use super::combinator::*;
use super::delay::{AllNest, Delay};
use super::math::*;
use super::signal::*;
use super::wave::*;
use super::*;
use funutd::dna::*;
use numeric_array::typenum::*;
extern crate alloc;
use alloc::format;
use alloc::vec::Vec;

/// Generate a reverb unit.
pub fn generate_reverb(dna: &mut Dna) -> An<impl AudioNode<Inputs = U2, Outputs = U2>> {
    let mut times = Vec::new();
    for i in 0..32 {
        let name = format!("Delay {}", i);
        if i < 32 {
            times.push(dna.f32_in(&name, 0.030, 0.070));
        } else {
            times.push(dna.f32_in(&name, 0.001, 0.030));
        }
    }
    super::prelude::reverb4_stereo_delays(&times, 100.0)
}

/// Attempt to measure the quality of a stereo reverb unit.
pub fn reverb_fitness(reverb: An<impl AudioNode<Inputs = U2, Outputs = U2>>) -> f32 {
    let mut response = Wave::render(
        44100.0,
        32768.0 / 44100.0,
        &mut (An(Impulse::new()) >> reverb),
    );

    let mut fitness = 0.0;

    let mut spectrum = Vec::<Complex32>::new();
    spectrum.resize(response.length() / 2 + 1, Complex32::new(0.0, 0.0));

    // Deal with left, right and center signals.
    for channel in 0..=1 {
        if channel == 0 || channel == 1 {
            // Maximize echo density.
            let echo_weight = 1.0;
            for i in 1..response.length() {
                // It is necessary to weight the initial buildup heavily to make it smooth.
                let weight = 1.0 / (i as f32);
                let r = response.at(channel, i);
                let threshold = 1.0e-9;
                if abs(r) >= threshold {
                    fitness += weight * echo_weight;
                }
            }
        }
        // Apply Hann window.
        for i in 0..response.length() {
            let w = 0.5
                + 0.5
                    * cos(
                        (i as i64 - (response.length() as i64 >> 1)) as f32 * f32::TAU
                            / response.length() as f32,
                    );
            response.set(channel, i, response.at(channel, i) * w);
        }
        let data = match channel {
            0 | 1 => response.channel(channel).clone(),
            _ => {
                let mut stereo = response.channel(0).clone();
                for i in 0..stereo.len() {
                    stereo[i] += response.at(1, i);
                }
                stereo
            }
        };
        super::fft::real_fft(&data, &mut spectrum);
        /*
        let spectral_weight = 0.001;
        let flatness_weight = 1.0;
        let mut spectral_no = 0.0;
        let mut spectral_de = 0.0;
        let mut spectral_to = 0.0;
        for i in 1..spectrum.len() {
            let f = 44100.0 / data.len() as f32 * i as f32;
            let norm2 = spectrum[i].norm_sqr();
            let aw = a_weight(f);
            //fitness -= aw * squared(norm2) * spectral_weight;
            //fitness -= norm2 * spectral_weight;
            let norm = sqrt(norm2);
            println!("Bin {} frequency {} magnitude {}", i, f, norm);
            spectral_no += aw * log(norm).max(-30.0);
            spectral_de += aw * norm;
            spectral_to += aw;
        }
        let flatness = spectral_no / spectral_to - log(spectral_de / spectral_to).max(-30.0);
        fitness += flatness * flatness_weight;
        */

        let outlier_weight = 0.1;
        let outlier_samples = 50;
        let mut sum = 0.0;

        for i in 1..=outlier_samples {
            sum += spectrum[i].norm();
        }

        for i in outlier_samples + 1..spectrum.len() {
            let norm0 = spectrum[i - outlier_samples].norm();
            let norm1 = spectrum[i].norm();
            let mean = sum / outlier_samples as f32;
            if norm1 > mean {
                fitness -= squared(norm1 - mean) * outlier_weight;
            }
            sum -= norm0;
            sum += norm1;
        }
        /*
        for i in 0..spectrum.len() {
            let norm2 = spectrum[i].norm_sqr();
            spectrum[i] = Complex32::new(norm2, 0.0);
        }
        c2r.process(&mut spectrum, &mut data).unwrap();
        let z = if data[0] > 0.0 { 1.0 / data[0] } else { 0.0 };
        //println!("data[0] = {}, data[1000..1010] = {:?}, data[10000..10010] = {:?}", data[0], &data[1000..1010], &data[10000..10010]);
        // Now `data[i] * z` is a normalized autocorrelation ranging in -1...1 for a lag of `i` samples.

        // Minimize autocorrelation.
        // Weight the frequencies by the tone response curve of the human ear.
        let auto_weight = 10.0;
        // Measure frequencies down to 5 Hz.
        for i in 1..44100 {
            let weight = 1.0; // / abs(i as f32);
            fitness -=
                a_weight(44100.0 / i as f32) * weight * squared(abs(data[i] * z)) * auto_weight;
        }
        */
    }

    fitness
}

type Schroeder = AllNest<U1, Delay>;

#[derive(Clone)]
struct ReverbBlock<F: AudioNode<Inputs = U1, Outputs = U1>> {
    allpass0: [Schroeder; 4],
    allpass1: [Schroeder; 4],
    filter0: F,
    filter1: F,
    delay: Delay,
}

/// Allpass loop based stereo reverb with user configurable loop filtering.
#[derive(Clone)]
pub struct Reverb<F: AudioNode<Inputs = U1, Outputs = U1>> {
    pre: [Schroeder; 4],
    block: Vec<ReverbBlock<F>>,
    feedback: f32,
    a: f32,
}

impl<F: AudioNode<Inputs = U1, Outputs = U1>> Reverb<F> {
    pub fn new(time: f64, diffusion: f64, filter: F) -> Self {
        let ldelays = [
            401, 421, 443, 463, 487, 503, 523, 547, 563, 587, 607, 619, 643, 661, 683, 701, 727,
            743, 761, 787, 809, 823, 839, 863, 883, 907, 929, 947, 967, 983, 1009, 1021,
        ];
        let rdelays = [
            419, 433, 457, 479, 491, 509, 541, 557, 577, 593, 613, 631, 653, 673, 691, 719, 733,
            757, 773, 797, 811, 829, 853, 877, 887, 911, 937, 953, 977, 997, 1013, 1033,
        ];
        let delays = [1087, 1091, 1093, 1097, 1103, 1109, 1117, 1123];
        let coeff = lerp(0.5, 0.9, diffusion) as f32;
        let mut block = Vec::with_capacity(8);
        for i in 0..8 {
            let allpass0 = core::array::from_fn(|j| {
                Schroeder::new(
                    coeff,
                    Delay::new((ldelays[i + j * 8] - 1) as f64 / DEFAULT_SR),
                )
            });
            let allpass1 = core::array::from_fn(|j| {
                Schroeder::new(
                    coeff,
                    Delay::new((rdelays[i + j * 8] - 1) as f64 / DEFAULT_SR),
                )
            });
            let delay = Delay::new(delays[7 - i] as f64 / DEFAULT_SR);
            block.push(ReverbBlock::<F> {
                allpass0,
                allpass1,
                filter0: filter.clone(),
                filter1: filter.clone(),
                delay,
            });
        }

        let a = pow(db_amp(-60.0), 0.035 / time) as f32;

        let predelay = [245, 367, 263, 349];

        let pre = core::array::from_fn(|i| {
            Schroeder::new(coeff, Delay::new((predelay[i] - 1) as f64 / DEFAULT_SR))
        });

        Self {
            pre,
            block,
            feedback: 0.0,
            a,
        }
    }
}

impl<F: AudioNode<Inputs = U1, Outputs = U1>> AudioNode for Reverb<F> {
    const ID: u64 = 85;
    type Inputs = U2;
    type Outputs = U2;

    fn reset(&mut self) {
        for block in self.block.iter_mut() {
            for x in block.allpass0.iter_mut() {
                x.reset();
            }
            for x in block.allpass1.iter_mut() {
                x.reset();
            }
            block.filter0.reset();
            block.filter1.reset();
            block.delay.reset();
        }
        self.feedback = 0.0;
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        for block in self.block.iter_mut() {
            for x in block.allpass0.iter_mut() {
                x.set_sample_rate(sample_rate);
            }
            for x in block.allpass1.iter_mut() {
                x.set_sample_rate(sample_rate);
            }
            block.filter0.set_sample_rate(sample_rate);
            block.filter1.set_sample_rate(sample_rate);
            block.delay.set_sample_rate(sample_rate);
        }
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let mut v0 = self.feedback;
        let mut output0 = 0.0;
        let mut output1 = 0.0;
        let input0 = self.pre[0].filter_mono(input[0] * 0.5);
        let input0 = self.pre[1].filter_mono(input0);
        let input1 = self.pre[2].filter_mono(input[1] * 0.5);
        let input1 = self.pre[3].filter_mono(input1);
        for block in self.block.iter_mut() {
            v0 = block.delay.filter_mono(v0);
            v0 = block.allpass0[0].filter_mono(self.a * v0 + input0);
            v0 = block.allpass0[1].filter_mono(v0);
            v0 = block.allpass0[2].filter_mono(v0);
            v0 = block.allpass0[3].filter_mono(v0);
            v0 = block.filter0.filter_mono(v0);
            output0 = v0;
            v0 = block.allpass1[0].filter_mono(self.a * v0 + input1);
            v0 = block.allpass1[1].filter_mono(v0);
            v0 = block.allpass1[2].filter_mono(v0);
            v0 = block.allpass1[3].filter_mono(v0);
            v0 = block.filter1.filter_mono(v0);
            output1 = v0;
        }
        self.feedback = v0;
        [output0, output1].into()
    }

    fn route(&mut self, input: &SignalFrame, _frequency: f64) -> SignalFrame {
        Routing::Arbitrary(0.0).route(input, 2)
    }
}
