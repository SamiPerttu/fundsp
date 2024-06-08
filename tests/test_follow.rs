//! Test the `follow` filter.
use fundsp::hacker::*;
use funutd::Rnd;

#[allow(clippy::manual_range_contains)]
#[test]
fn test_follow() {
    let mut rnd = Rnd::new();

    // Test follow().
    for _ in 0..200 {
        // Bias testing toward smaller lengths to cut testing time shorter.
        let samples = round(xerp(1.0, 500_000.0, squared(rnd.f32())));
        let sample_rate = xerp(10.0, 500_000.0, rnd.f32());
        let mut x = follow(samples / sample_rate);
        x.set_sample_rate(sample_rate as f64);
        x.filter_mono(0.0);
        let goal = lerp(-100.0, 100.0, rnd.f64());
        for _ in 0..samples as usize {
            x.filter_mono(goal as f32);
        }
        // Promise was 0.5% accuracy between 1 and 500k samples.
        let response = x.value() / goal;
        assert!(response >= 0.495 && response <= 0.505);
    }

    // Test asymmetric follow().
    for _ in 0..200 {
        // Bias testing toward smaller lengths to cut testing time shorter.
        let attack_samples = round(xerp(1.0, 500_000.0, squared(rnd.f32())));
        let release_samples = round(xerp(1.0, 500_000.0, squared(rnd.f32())));
        let sample_rate = xerp(10.0, 100_000.0, rnd.f32());
        let goal = lerp(-100.0, 100.0, rnd.f64());
        let mut x = afollow(attack_samples / sample_rate, release_samples / sample_rate);
        x.set_sample_rate(sample_rate as f64);
        x.filter_mono(0.0);
        for _ in 0..(if goal > 0.0 {
            attack_samples
        } else {
            release_samples
        }) as usize
        {
            x.filter_mono(goal as f32);
        }
        // Promise was 0.5% accuracy between 1 and 500k samples.
        let response = x.value() / goal;
        assert!(response >= 0.495 && response <= 0.505);
    }
}
