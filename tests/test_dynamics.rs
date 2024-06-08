//! Dynamics components tests.

#![allow(clippy::manual_range_contains)]
#![allow(dead_code)]

use fundsp::dynamics::*;
use fundsp::hacker::*;
use funutd::*;

#[test]
fn test_dynamics() {
    let mut rnd = Rnd::new();

    // Test ReduceBuffer.
    for _ in 0..100 {
        let length = (rnd.u64() as usize & 0xff) + 1;
        let mut buffer = ReduceBuffer::<u32, _>::new(length, Maximum::new());
        let mut vector = vec![0u32; length];
        for _ in 0..1000 {
            let i = rnd.u64() as usize % length;
            let value = rnd.u64() >> (rnd.u64() & 0x1f);
            buffer.set(i, value as u32);
            vector[i] = value as u32;
            if i % 100 == 99 {
                assert_eq!(*vector.iter().max().unwrap(), buffer.total());
            }
        }
    }

    // Test limiter.
    for _ in 0..20 {
        let samples = round(xerp(2.0, 200_000.0, rnd.f64())) as usize;
        let sample_rate = 48000.0;
        let mut x = limiter(samples as f32 / sample_rate, samples as f32 / sample_rate);
        x.set_sample_rate(sample_rate as f64);
        for _ in 0..samples {
            x.filter_mono(0.0);
        }
        // Edges are the most challenging case. Test a +100 dB edge.
        let edge = db_amp(100.0);
        for _ in 0..samples {
            let y = x.filter_mono(edge);
            assert!(y <= 1.0);
        }
        let value = x.filter_mono(edge);
        // The limiter leaves some headroom.
        // Check that the response is limited and has sufficient range.
        assert!(value >= 0.90 && value <= 1.00);
    }

    // Test monitor and meter for consistency.
    let s1 = shared(0.0);
    let mut m1 = monitor(&s1, Meter::Sample);
    let mut m2 = meter(Meter::Sample);
    for _ in 0..10000 {
        let x = rnd.f32();
        let x1 = m1.filter_mono(x);
        let x2 = m2.filter_mono(x);
        assert_eq!(x, x1);
        assert_eq!(x, x2);
        assert_eq!(x, s1.value());
    }
    let s1 = shared(0.0);
    let mut m1 = monitor(&s1, Meter::Peak(0.1));
    let mut m2 = meter(Meter::Peak(0.1));
    for _ in 0..10000 {
        let x = rnd.f32();
        let x1 = m1.filter_mono(x);
        let x2 = m2.filter_mono(x);
        assert_eq!(x, x1);
        assert!(x2 >= 0.0);
        assert_eq!(x2, s1.value());
    }
}
