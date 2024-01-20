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
        times.push(dna.f32_in(&name, 0.020, 0.050) as f64);
    }
    reverb2_stereo_delays(&times, 3.0)
}

/// Attempt to measure the quality of a stereo reverb unit.
pub fn reverb_fitness(reverb: An<impl AudioNode<Sample = f32, Inputs = U2, Outputs = U2>>) -> f32 {
    let mut response = Wave32::render(44100.0, 2.0 * 65536.0 / 44100.0, &mut (impulse() >> reverb));

    // Pad the response with zeros to prevent circular convolution artifacts.
    //response.resize(response.length() * 2);

    let mut fitness = 0.0;

    let mut planner = RealFftPlanner::<f32>::new();
    let r2c = planner.plan_fft_forward(response.length());
    //let c2r = planner.plan_fft_inverse(response.length());
    let mut spectrum = r2c.make_output_vec();

    // Deal with left, right and center signals.
    for channel in 0..=1 {
        if channel == 0 || channel == 1 {
            // Maximize echo density.
            let echo_weight = 0.0;
            for i in 1..response.length() / 2 {
                // It is necessary to weight the initial buildup heavily to make it smooth.
                let weight = 1.0 / squared(i as f32);
                let r = response.at(channel, i);
                let threshold = 1.0e-6;
                if abs(r) >= threshold {
                    fitness += weight * echo_weight;
                }
            }
        }
        for i in 0..response.length() {
            let w = 0.5
                + 0.5
                    * cos(
                        (i as i64 - (response.length() as i64 >> 1)) as f32 * TAU as f32
                            / response.length() as f32,
                    );
            response.set(channel, i, response.at(channel, i) * w);
        }
        let mut data = match channel {
            0 | 1 => response.channel(channel).clone(),
            _ => {
                let mut stereo = response.channel(0).clone();
                for i in 0..stereo.len() {
                    stereo[i] += response.at(1, i);
                }
                stereo
            }
        };
        let spectral_weight = 0.001;
        r2c.process(&mut data, &mut spectrum).unwrap();
        //let mut spectral_no = 0.0;
        //let mut spectral_de = 0.0;
        for i in 1..spectrum.len() {
            let f = 44100.0 / data.len() as f32 * i as f32;
            let norm2 = spectrum[i].norm_sqr();
            //*x = Complex32::new(norm2, 0.0);
            fitness -= a_weight(f) * squared(norm2 * spectral_weight);
            //fitness -= norm2 * spectral_weight;
            //spectral_no += log(norm2).max(-30.0);
            //spectral_de += norm2;
        }
        //println!("flatness {}", spectral_no - log(spectral_de).max(-30.0));
        //fitness += exp(spectral_no) / spectral_de;
        //fitness += spectral_no / spectrum.len() as f32 - log(spectral_de / (spectrum.len() as f32));
        //continue;
        /*
        c2r.process(&mut spectrum, &mut data).unwrap();
        let z = if data[0] > 0.0 { 1.0 / data[0] } else { 0.0 };
        //println!("data[0] = {}, data[1000..1010] = {:?}", data[0], &data[1000..1010]);
        // Now `data[i] * z` is a normalized autocorrelation ranging in -1...1 for a lag of `i` samples.

        // Minimize autocorrelation.
        // Weight the frequencies by the tone response curve of the human ear.
        let auto_weight = 1.0;
        // Measure frequencies down to 5 Hz.
        for i in 1..44100 {
            let weight = 1.0; // / abs(i as f32);
            fitness -= a_weight(44100.0 / i as f32) * weight * abs(data[i] * z) * auto_weight;
        }
        */
    }

    fitness
}
