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
        let i_time = dna.parameter(i).value_f32().unwrap();
        for j in i + 1..dna.parameters() {
            let j_time = dna.parameter(j).value_f32().unwrap();
            if round(44100.0 * i_time) == round(44100.0 * j_time) {
                repeat_fitness -= 100.0;
            }
        }
    }
    repeat_fitness + reverb_fitness(reverb)
}

fn main() {
    let mut rng = Rnd::from_time();

    let mut dna = Dna::new(rng.u64());
    let mut fitness = evaluate_reverb(&mut dna);
    let mut rounds = 0;

    loop {
        rounds += 1;

        let mut seeds = vec![];
        for _ in 0..360 {
            seeds.push((rng.u64(), rng.f32() * rng.f32()));
        }
        let mutateds: Vec<(Dna, f32)> = seeds
            .par_iter()
            .map(|(seed, p)| {
                let mutation_p = xerp(1.0 / 30.0, 1.5, *p).min(1.0);
                let mut mutated = Dna::mutate(&dna, *seed, mutation_p);
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
                rounds * 360,
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
