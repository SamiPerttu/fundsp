//! FunDSP Effect Library.

use super::hacker::*;

/// Effect 001. Mono chorus, 5 voices. For stereo, stack two of these using different seed values.
/// `seed`: LFO seed.
/// `mod_frequency`: delay modulation frequency (for example, 0.2).
/// `highpass_cutoff`: highpass filter cutoff (for example, 200.0).
/// - Input 0: audio.
/// - Output 0: chorused audio, including original signal.
pub fn chorus(
    seed: i64,
    mod_frequency: f64,
    highpass_cutoff: f64,
) -> An<impl AudioNode<Sample = f64, Inputs = U1, Outputs = U1>> {
    // Minimum delay between voices.
    let base_delay = 0.012;
    // Range of delay variation.
    let delay_range = 0.005;
    pass()
        & (highpass_hz(highpass_cutoff, 1.0)
            | lfo(move |t| {
                (
                    // Delays between successive voices range from 12 to 22 ms.
                    lerp(
                        base_delay,
                        base_delay + delay_range,
                        spline_noise(seed, t * mod_frequency),
                    ),
                    lerp(
                        base_delay * 2.0 + delay_range,
                        base_delay * 2.0 + delay_range * 2.0,
                        spline_noise(hash(seed), t * (mod_frequency + 0.02)),
                    ),
                    lerp(
                        base_delay * 3.0 + delay_range * 2.0,
                        base_delay * 3.0 + delay_range * 3.0,
                        spline_noise(hash(hash(seed)), t * (mod_frequency + 0.04)),
                    ),
                    lerp(
                        base_delay * 4.0 + delay_range * 3.0,
                        base_delay * 4.0 + delay_range * 4.0,
                        spline_noise(hash(hash(hash(seed))), t * (mod_frequency + 0.06)),
                    ),
                )
            }))
            >> multitap::<U4>(base_delay, base_delay * 4.0 + delay_range * 4.0)
}
