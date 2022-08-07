#![allow(clippy::precedence)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fundsp::hacker32::*;

fn wavetable_bench(_dummy: usize) -> Wave32 {
    Wave32::render(44100.0, 1.0, &mut (saw_hz(110.0)))
}

fn envelope_bench(_dummy: usize) -> Wave32 {
    Wave32::render(44100.0, 1.0, &mut (noise() * envelope(|t| exp(-t))))
}

fn oversample_bench(_dummy: usize) -> Wave32 {
    Wave32::render(44100.0, 1.0, &mut (oversample(noise())))
}

fn chorus_bench(_dummy: usize) -> Wave32 {
    Wave32::render(44100.0, 1.0, &mut (noise() >> chorus(0, 0.015, 0.005, 0.5)))
}

fn equalizer_bench(_dummy: usize) -> Wave32 {
    Wave32::render(
        44100.0,
        1.0,
        &mut (noise()
            >> pipe::<U10, _, _>(|i| bell_hz(1000.0 + 1000.0 * i as f32, 1.0, db_amp(3.0)))),
    )
}

fn reverb_bench(_dummy: usize) -> Wave32 {
    Wave32::render(
        44100.0,
        1.0,
        &mut (noise()
            >> split::<U16>()
            >> fdn::<U16, _>(stack::<U16, _, _>(|i| {
                delay(0.01 * i as f32) * 0.9 >> fir((0.5, 0.5))
            }))
            >> join::<U16>()),
    )
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("wavetable", |b| b.iter(|| wavetable_bench(black_box(0))));
    c.bench_function("envelope", |b| b.iter(|| envelope_bench(black_box(0))));
    c.bench_function("oversample", |b| b.iter(|| oversample_bench(black_box(0))));
    c.bench_function("chorus", |b| b.iter(|| chorus_bench(black_box(0))));
    c.bench_function("equalizer", |b| b.iter(|| equalizer_bench(black_box(0))));
    c.bench_function("reverb", |b| b.iter(|| reverb_bench(black_box(0))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
