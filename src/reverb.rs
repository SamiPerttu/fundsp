//! Reverberation related code.

use super::hacker32::*;
use funutd::dna::*;
use realfft::*;

/// Generate a reverb unit.
pub fn generate_reverb(
    dna: &mut Dna,
) -> An<impl AudioNode<Sample = f32, Inputs = U2, Outputs = U2>> {
    let mut times = Vec::new();
    for i in 0..24 {
        let name = format!("Delay {}", i);
        times.push(dna.f32_in(&name, 0.020, 0.070) as f64);
    }
    reverb2_stereo_delays(&times, 3.0)
}

/// Attempt to measure the quality of a stereo reverb unit.
pub fn reverb_fitness(reverb: An<impl AudioNode<Sample = f32, Inputs = U2, Outputs = U2>>) -> f32 {
    let mut response = Wave32::render(44100.0, 65536.0 / 44100.0, &mut (impulse() >> reverb));

    // Pad the response with zeros to prevent circular convolution artifacts.
    response.resize(response.length() * 2);

    let mut fitness = 0.0;

    let mut planner = RealFftPlanner::<f32>::new();
    let r2c = planner.plan_fft_forward(response.length());
    let c2r = planner.plan_fft_inverse(response.length());
    let mut spectrum = r2c.make_output_vec();

    for channel in 0..=1 {
        let mut data = response.channel(channel).clone();
        r2c.process(&mut data, &mut spectrum).unwrap();
        for x in spectrum.iter_mut() {
            *x = Complex32::new(x.norm_sqr(), 0.0);
        }
        c2r.process(&mut spectrum, &mut data).unwrap();
        let z = if data[0] > 0.0 { 1.0 / data[0] } else { 0.0 };
        //println!("data 0 = {:?}", &data[0..100]);
        // Now `data[i] * z` is a normalized autocorrelation ranging in -1...1 for a lag of `i` samples.

        // Minimize autocorrelation.
        // Weight the frequencies by the noise response curve of the human ear.
        let auto_weight = 1.0;
        for i in 1..4410 * 4 {
            fitness -= m_weight(44100.0 / i as f32) * abs(data[i] * z) * auto_weight;
        }

        // Maximize echo density.
        let echo_weight = 1_000_000.0;
        for i in 1..response.length() {
            // It is necessary to weight the initial buildup heavily to make it smooth.
            let weight = 1.0 / squared(i as f32);
            let r = response.at(channel, i);
            let threshold = 1.0e-9;
            if abs(r) >= threshold {
                fitness += weight * echo_weight;
            }
        }
    }

    fitness
}
