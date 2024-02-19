//! Failed experiment to optimize a stereo reverb. WIP.
//! Please run me in release mode!

use fundsp::hacker32::*;
use fundsp::reverb::*;
use funutd::dna::*;
use funutd::*;
use rayon::prelude::*;

struct Specimen {
    pub dna: Dna,
    pub fitness: f32,
}

fn specimen(dna: Dna, fitness: f32) -> Specimen {
    Specimen { dna, fitness }
}

/// Evaluate reverb quality from its genotype.
fn evaluate_reverb(dna: &mut Dna) -> f32 {
    let reverb = generate_reverb(dna);
    // Prevent cases where two lines have nearly the same length.
    let repeat_weight = 0.0;
    let mut repeat_fitness = 0.0;
    for i in 0..dna.parameters() {
        let i_time = round(44100.0 * dna.parameter(i).value_f32().unwrap());
        for j in i + 1..dna.parameters() {
            let j_time = round(44100.0 * dna.parameter(j).value_f32().unwrap());
            let diff = abs(i_time - j_time);
            if diff < 20.0 {
                repeat_fitness -= repeat_weight * squared(20.0 - diff);
            }
        }
    }
    repeat_fitness + reverb_fitness(reverb)
}

/// Mutate the source Dna. Return the mutated Dna.
/// The probability of mutating each parameter is `mutation_p`.
/// Requires interactive mode.
fn mutate(source: &Dna, seed: u64, mutation_p: f32) -> Dna {
    let mut rnd = Rnd::from_u64(seed);
    let mut dna = Dna::new(rnd.u64());
    let mutate_i = rnd.u32_to(source.parameters() as u32) as usize;
    let scale = xerp(1.0, (1u64 << 32) as f64, rnd.f64());
    for i in 0..source.parameters() {
        let parameter = source.parameter(i);
        if i == mutate_i || rnd.f32() < mutation_p {
            if matches!(parameter.kind(), ParameterKind::Ordered) {
                let adjust = if rnd.bool(0.5) {
                    lerp(
                        1.0,
                        max(
                            1.0,
                            min(parameter.maximum() as f64 - parameter.raw() as f64, scale),
                        ),
                        rnd.f64(),
                    )
                } else {
                    -lerp(1.0, max(1.0, min(parameter.raw() as f64, scale)), rnd.f64())
                };
                let value = clamp(
                    0.0,
                    parameter.maximum() as f64,
                    parameter.raw() as f64 + adjust,
                );
                dna.set_value(parameter.hash(), value.round() as u32);
            }
        } else {
            dna.set_value(parameter.hash(), parameter.raw());
        }
    }
    dna
}

fn main() {
    let mut rng = Rnd::from_time();

    let mut global_dna = Dna::new(rng.u64());
    let mut global_fitness = evaluate_reverb(&mut global_dna);
    let mut global_i = 0;
    let mut rounds = 0;
    let mut accept = 0;
    let mut reject = 0;
    let population_size = 96;

    let mut population = Vec::new();
    for _ in 0..population_size {
        let mut dna = Dna::new(rng.u64());
        let fitness = evaluate_reverb(&mut dna);
        population.push(specimen(dna, fitness));
    }

    loop {
        rounds += 1;

        let temperature = 1000.0 / (rounds as f32);

        let mut seeds = vec![];
        for _ in 0..population_size {
            seeds.push((rng.u64(), squared(rng.f32())));
        }
        let mut candidates: Vec<Specimen> = seeds
            .par_iter()
            .zip(&population)
            .map(|((seed, p), spec)| {
                let mutation_p = xerp(1.0 / 32.0, 1.0, *p);
                let mut mutated = mutate(&spec.dna, *seed, mutation_p);
                let mutated_fitness = evaluate_reverb(&mut mutated);
                specimen(mutated, mutated_fitness)
            })
            .collect();

        let mut improved = false;
        for i in 0..population_size {
            if candidates[i].fitness > global_fitness {
                global_fitness = candidates[i].fitness;
                global_dna = candidates[i].dna.clone();
                global_i = i;
                improved = true;
            }
            if candidates[i].fitness > population[i].fitness
                || (temperature > 0.0
                    && rng.f32()
                        < exp((candidates[i].fitness - population[i].fitness) / temperature))
            {
                std::mem::swap(&mut candidates[i], &mut population[i]);
                accept += 1;
            } else {
                reject += 1;
            }
        }

        //population.sort_unstable_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());

        if improved || rounds % 1000 == 0 {
            println!(
                "Rounds {} (Accept {}%) Fitness {} Specimen {} Temp {}",
                rounds,
                accept as f32 * 100.0 / (accept + reject) as f32,
                global_fitness,
                global_i,
                temperature,
            );
        }
        if improved {
            let mut delays = Vec::new();
            let mut samples1 = Vec::new();
            let mut samples2 = Vec::new();
            for i in 0..global_dna.parameters() {
                let delay = global_dna.parameter(i).value_f32().unwrap();
                let samples = round(delay * 44100.0) as i32;
                if i < 16 {
                    samples1.push(samples);
                } else {
                    samples2.push(samples);
                }
                print!(
                    "  {}: {} ({})",
                    global_dna.parameter(i).name(),
                    delay,
                    samples,
                );
                if i % 2 == 1 {
                    println!();
                }
                delays.push(delay);
            }
            samples1.sort();
            samples2.sort();
            println!("{:?}", samples1);
            println!("{:?}", samples2);
            println!("{:?}", delays);
        }
    }
}
