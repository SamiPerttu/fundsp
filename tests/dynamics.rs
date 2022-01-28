#![allow(
    clippy::precedence,
    clippy::type_complexity,
    clippy::float_cmp,
    clippy::len_zero,
    clippy::double_neg,
    clippy::many_single_char_names,
    clippy::manual_range_contains
)]
#![allow(dead_code)]

extern crate fundsp;

use fundsp::dynamics::*;
use fundsp::hacker::*;

#[test]
fn test_dynamics() {
    let mut rnd = AttoRand::new(1);

    // Test ReduceBuffer.
    for _ in 0..100 {
        let length = (rnd.get() as usize & 0xff) + 1;
        let mut buffer = ReduceBuffer::<u32, _>::new(length, Maximum::new());
        let mut vector = vec![0u32; length];
        for _ in 0..1000 {
            let i = rnd.get() as usize % length;
            let value = rnd.get() >> (rnd.get() & 0x1f);
            buffer.set(i, value as u32);
            vector[i] = value as u32;
            if i % 100 == 99 {
                assert_eq!(*vector.iter().max().unwrap(), buffer.total());
            }
        }
    }

    // Test limiter.
    for _ in 0..20 {
        let samples = round(xerp(1.0, 100_000.0, rnd.get01::<f64>())) as usize;
        let sample_rate = 32768.0;
        let mut x = limiter((samples as f64 / sample_rate, samples as f64 / sample_rate));
        x.reset(Some(sample_rate));
        for _ in 0..samples {
            x.filter_mono(0.0);
        }
        // Edges are the most challenging case. Test a +100 dB edge.
        let edge = db_amp(100.0);
        for _ in 0..samples {
            assert!(x.filter_mono(edge) <= 1.0);
        }
        let value = x.filter_mono(edge);
        // The limiter leaves some headroom.
        // Check that the response is limited and has sufficient range.
        assert!(value >= 0.8 && value <= 1.0);
    }
}
