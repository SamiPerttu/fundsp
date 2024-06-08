use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fundsp::hacker32::*;

fn sine_bench(_dummy: usize) -> Wave {
    Wave::render(
        44100.0,
        1.0,
        &mut (sumi::<U100, _, _>(|i| sine_hz(100.0 * (i + 1) as f32))),
    )
}

fn resynth_bench(_dummy: usize) -> Wave {
    Wave::render(
        44100.0,
        1.0,
        &mut (noise()
            >> resynth::<U1, U1, _>(1024, |fft| {
                for i in 0..fft.bins() {
                    fft.set(0, i, fft.at(0, i));
                }
            })),
    )
}

#[allow(clippy::precedence)]
fn pass_bench(_dummy: usize) -> Wave {
    Wave::render(
        44100.0,
        1.0,
        &mut (dc((1.0, 2.0)) * 2.0 >> pass() + pass() >> pass()),
    )
}

fn wavetable_bench(_dummy: usize) -> Wave {
    Wave::render(44100.0, 1.0, &mut (saw_hz(110.0)))
}

fn envelope_bench(_dummy: usize) -> Wave {
    Wave::render(
        44100.0,
        1.0,
        &mut (noise() * envelope(|t| (-t).exp() * sin_hz(1.0, t))),
    )
}

fn oversample_bench(_dummy: usize) -> Wave {
    Wave::render(44100.0, 1.0, &mut (noise() >> oversample(pass())))
}

fn chorus_bench(_dummy: usize) -> Wave {
    Wave::render(44100.0, 1.0, &mut (noise() >> chorus(0, 0.015, 0.005, 0.5)))
}

fn equalizer_bench(_dummy: usize) -> Wave {
    Wave::render(
        44100.0,
        1.0,
        &mut (noise()
            >> pipei::<U10, _, _>(|i| bell_hz(1000.0 + 1000.0 * i as f32, 1.0, db_amp(3.0)))),
    )
}

fn reverb_bench(_dummy: usize) -> Wave {
    Wave::render(
        44100.0,
        1.0,
        &mut ((noise() | noise()) >> reverb_stereo(10.0, 1.0, 0.5)),
    )
}

fn limiter_bench(_dummy: usize) -> Wave {
    Wave::render(44100.0, 1.0, &mut (noise() >> limiter(0.1, 1.0)))
}

fn phaser_bench(_dummy: usize) -> Wave {
    Wave::render(
        44100.0,
        1.0,
        &mut (noise() >> phaser(0.5, |t| sin_hz(0.1, t) * 0.5 + 0.5)),
    )
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("sine", |b| b.iter(|| sine_bench(black_box(0))));
    c.bench_function("resynth", |b| b.iter(|| resynth_bench(black_box(0))));
    c.bench_function("pass", |b| b.iter(|| pass_bench(black_box(0))));
    c.bench_function("wavetable", |b| b.iter(|| wavetable_bench(black_box(0))));
    c.bench_function("envelope", |b| b.iter(|| envelope_bench(black_box(0))));
    c.bench_function("oversample", |b| b.iter(|| oversample_bench(black_box(0))));
    c.bench_function("chorus", |b| b.iter(|| chorus_bench(black_box(0))));
    c.bench_function("equalizer", |b| b.iter(|| equalizer_bench(black_box(0))));
    c.bench_function("reverb", |b| b.iter(|| reverb_bench(black_box(0))));
    c.bench_function("limiter", |b| b.iter(|| limiter_bench(black_box(0))));
    c.bench_function("phaser", |b| b.iter(|| phaser_bench(black_box(0))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
