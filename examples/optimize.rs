//! Optimize a stereo reverb. Please run me in release mode!

use fundsp::hacker32::*;
use fundsp::reverb::*;
use funutd::dna::*;
use funutd::*;
use rayon::prelude::*;

/// Evaluate reverb quality from its genotype.
fn evaluate_reverb(dna: &mut Dna) -> f32 {
    let reverb = generate_reverb(dna);
    // Prevent cases where two lines have the same length.
    let mut repeat_fitness = 0.0;
    for i in 0..dna.parameters() {
        let i_time = round(44100.0 * dna.parameter(i).value_f32().unwrap());
        for j in i + 1..dna.parameters() {
            let j_time = round(44100.0 * dna.parameter(j).value_f32().unwrap());
            let diff = abs(i_time - j_time);
            if diff < 30.0 {
                repeat_fitness -= squared(30.0 - diff);
            }
        }
    }
    repeat_fitness + reverb_fitness(reverb)
}

/// Mutate the source Dna. Return the mutated Dna.
/// The probability of mutating each parameter is `mutation_p`.
fn mutate(source: &Dna, seed: u64, mutation_p: f32) -> Dna {
    let mut rnd = Rnd::from_u64(seed);
    let mut dna = Dna::new(rnd.u64());
    let amount = xerp(1.0, (1u64 << 32) as f64, rnd.f64());
    for parameter in source.parameter_vector() {
        if rnd.f32() < mutation_p {
            if matches!(parameter.kind(), ParameterKind::Ordered) {
                let adjust = if rnd.bool(0.5) {
                    xerp(
                        1.0,
                        max(
                            1.0,
                            min(parameter.maximum() as f64 - parameter.raw() as f64, amount),
                        ),
                        rnd.f64(),
                    )
                } else {
                    -xerp(
                        1.0,
                        max(1.0, min(parameter.raw() as f64, amount)),
                        rnd.f64(),
                    )
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

    let mut dna = Dna::new(rng.u64());
    let mut fitness = evaluate_reverb(&mut dna);
    let mut rounds = 0;
    let candidates_per_round = 192;

    loop {
        rounds += 1;

        let mut seeds = vec![];
        for _ in 0..candidates_per_round {
            seeds.push((rng.u64(), rng.f32() * rng.f32()));
        }
        let min_mutation_p = 1.0 / min(30, max(1, rounds / 3)) as f32;
        let mutateds: Vec<(Dna, f32)> = seeds
            .par_iter()
            .map(|(seed, p)| {
                let mutation_p = xerp(min_mutation_p, 1.5, *p);
                let mut mutated = if mutation_p >= 1.0 {
                    Dna::new(*seed)
                } else {
                    mutate(&dna, *seed, mutation_p)
                };
                let mutated_fitness = evaluate_reverb(&mut mutated);
                (mutated, mutated_fitness)
            })
            .collect();

        let mut improved = false;
        for (mut mutated, mutated_fitness) in mutateds {
            if mutated_fitness > fitness {
                fitness = mutated_fitness;
                std::mem::swap(&mut dna, &mut mutated);
                improved = true;
            }
        }

        if improved || rounds % 1000 == 0 {
            println!(
                "Rounds {} (Candidates {}) Fitness {}",
                rounds,
                rounds * candidates_per_round,
                fitness
            );
        }
        if improved {
            let mut delays = Vec::new();
            let mut samples1 = Vec::new();
            let mut samples2 = Vec::new();
            for i in 0..dna.parameters() {
                let delay = dna.parameter(i).value_f32().unwrap();
                let samples = round(delay * 44100.0) as i32;
                if i < 8 {
                    samples1.push(samples);
                } else {
                    samples2.push(samples);
                }
                println!("{}: {} ({})", dna.parameter(i).name(), delay, samples,);
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
