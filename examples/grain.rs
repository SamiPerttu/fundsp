//! Make some granular noises. Please run me in release mode!
#![allow(clippy::precedence)]

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use fundsp::hacker::*;
use funutd::dna::*;

fn main() {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config().unwrap();

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()).unwrap(),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()).unwrap(),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()).unwrap(),
        _ => panic!("Unsupported format"),
    }
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), anyhow::Error>
where
    T: SizedSample + FromSample<f64>,
{
    let sample_rate = config.sample_rate.0 as f64;
    let channels = config.channels as usize;

    let _c = Granular::new(
        2,
        32,
        2.0,
        32,
        20,
        0.05,
        0.15,
        0.0,
        #[allow(unused_variables)]
        |t, b, v, x, y, z| {
            let scale = [
                36.0, 38.0, 41.0, 43.0, 46.0, 48.0, 50.0, 53.0, 55.0, 58.0, 60.0, 62.0, 65.0, 67.0,
                70.0, 72.0, 74.0, 77.0, 79.0, 82.0, 84.0,
            ];
            let d = lerp11(0.0, scale.len() as f32 - 0.01, x);
            let f = midi_hz(scale[d as usize] + 0.02 * (d - round(d)));
            let r = max(1.0, xerp11(0.50, 2.00, z));
            let f = f * xerp11(1.0 / r as f64, r as f64, sin_hz(6.0, t)) as f32;
            (
                0.060,
                0.030,
                Box::new(soft_saw_hz(f) * xerp11(0.005, 0.05, y) >> pan(v * 0.6)),
            )
        },
        /*|t, b, v, x, y, z| {
            let scale = [
                44.0, 46.0, 49.0, 51.0, 53.0, 55.0, 56.0, 58.0, 61.0, 63.0, 65.0, 67.0, 68.0, 70.0,
                73.0, 75.0, 77.0, 79.0, 80.0, 82.0, 84.0, 87.0, 89.0, 91.0, 92.0,
            ];
            let d = lerp11(0.0, scale.len() as f64 - 0.01, x);
            let f = midi_hz(scale[d as usize] + 0.02 * (d - round(d)));
            (
                0.060,
                0.025,
                Box::new(
                    saw_hz(f) * 0.05
                        >> moog_hz(xerp11(20.0, 20000.0, y), lerp11(0.1, 0.65, z))
                        >> pan(v * 0.8),
                ),
            )
        },*/
        /*|t, b, v, x, y, z| {
            let scale = [
                36.0, 38.0, 41.0, 43.0, 46.0, 48.0, 50.0, 53.0, 55.0, 58.0, 60.0, 62.0, 65.0, 67.0,
                70.0, 72.0, 74.0, 77.0, 79.0, 82.0, 84.0
            ];
            let d = lerp11(0.0, scale.len() as f64 - 0.01, x);
            let f = midi_hz(scale[d as usize] + 0.02 * (d - round(d)));
            let c = xerp11(1.0, 200.0, y);
            (
                0.100,
                0.030,
                Box::new(
                    (white() >> lowpass_hz(c, 1.0)) * sine_hz(f) * (xerp11(1.0, 10.0, z) / sqrt(c) / max(0.5, a_weight(f)))
                        >> pan(v * 0.6),
                ),
            )
        },*/
        /*|t, b, v, x, y, z| {
            let scale = [
                //32.0, 34.0, 37.0, 39.0, 41.0,
                44.0, 46.0, 49.0, 51.0, 53.0, 56.0, 58.0, 61.0, 63.0, 65.0, 68.0, 70.0, 73.0, 75.0,
                77.0, 80.0, 82.0, 84.0, 87.0, 89.0, 92.0,
            ];
            let d = lerp11(0.0, scale.len() as f64 - 0.01, x);
            let f = midi_hz(scale[d as usize] + 0.02 * (d - round(d)));
            let w = xerp11(2.0, 200.0, y);
            (
                0.100,
                0.030,
                Box::new(
                    (pink() | dc((f, w)))
                        >> !resonator()
                        >> resonator() * xerp11(0.0005, 0.005, z)
                        >> pan(v * 0.7),
                ),
            )
        },*/
        /*|t, b, v, x, y, z| {
            let scale = [
                44.0, 46.0, 48.0, 49.0, 51.0, 53.0, 55.0, 56.0, 58.0, 60.0, 61.0, 63.0, 65.0, 67.0,
                68.0, 70.0, 72.0, 73.0, 75.0, 77.0, 79.0, 80.0, 82.0, 84.0, 86.0, 87.0, 89.0, 91.0,
                92.0,
            ];
            let d = lerp11(0.0, scale.len() as f64 - 0.001, y);
            let f = midi_hz(scale[d as usize] + 0.05 * (d - round(d)));
            let h = 1.0 + xerp11(0.1, 10.0, x);
            (
                0.060,
                0.030,
                Box::new(
                    sine_hz(f)
                        >> shape(Shape::Tanh(h))
                        >> bandpass_hz(xerp11(50.0, 10000.0, z), 3.0)
                            * (0.01 / max(0.2, a_weight(f)))
                        >> pan(v * 0.7),
                ),
            )
        },*/
        /*|t, b, v, x, y, z| {
            let scale = [
                36.0, 38.0, 41.0, 43.0, 46.0, 48.0, 50.0, 53.0, 55.0, 58.0, 60.0, 62.0, 65.0, 67.0,
                70.0, 72.0, 74.0, 77.0, 79.0, 82.0, 84.0,
            ];
            let d = lerp11(0.0, scale.len() as f64 - 0.01, x);
            let f = midi_hz(scale[d as usize] + 0.02 * (d - round(d)));
            (
                0.070,
                0.030,
                Box::new(
                    dc((f, lerp11(0.50, 0.99, y)))
                    >> pulse() >> peak_hz(xerp11(60.0, 10000.0, z), 5.0) * (0.01 / max(0.2, a_weight(f)))
                    >> pan(v * 0.7),
                ),
            )
        },*/
        /*|t, b, v, x, y, z| {
            let scale = [
                36.0, 38.0, 40.0, 43.0, 45.0, 48.0, 50.0, 52.0, 55.0, 57.0, 60.0, 62.0, 64.0, 67.0,
                69.0, 72.0, 74.0, 76.0, 79.0, 81.0, 84.0, 86.0, 88.0, 91.0, 93.0, 96.0,
            ];
            let d = lerp11(0.0, scale.len() as f64 - 0.01, x);
            let f = midi_hz(scale[d as usize] + 0.02 * (d - round(d)));
            (
                0.060,
                0.015,
                Box::new(
                    organ_hz(f) * (0.05 / a_weight(f))
                        >> moog_hz(xerp11(20.0, 20000.0, y), lerp11(0.10, 0.65, z))
                        >> pan(v * 0.7),
                ),
            )
        },*/
    );

    let scale = [
        36.0, 38.0, 40.0, 43.0, 45.0, 48.0, 50.0, 52.0, 55.0, 57.0, 60.0, 62.0, 64.0, 67.0, 69.0,
        72.0, 74.0, 76.0, 79.0, 81.0, 84.0, 86.0, 88.0, 91.0, 93.0, 96.0,
    ];

    //let mut dna = Dna::new(37);
    let mut dna = Dna::new(102);
    let mut c = Net::wrap(fundsp::gen::gen_granular(2, &scale, 2.4, 30, &mut dna));

    for parameter in dna.parameter_vector().iter() {
        println!("{}: {}", parameter.name(), parameter.value());
    }

    c = c
        >> (multipass()
            & 0.2 * reverb2_stereo(10.0, 4.0, 0.5, 1.0, highshelf_hz(5000.0, 1.0, db_amp(-2.0))));

    c.set_sample_rate(sample_rate);

    // Use block processing for maximum efficiency.
    let mut c = BlockRateAdapter::new(Box::new(c));

    let mut next_value = move || c.get_stereo();

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
        None,
    )?;
    stream.play()?;

    std::thread::sleep(std::time::Duration::from_millis(120_000));

    Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> (f32, f32))
where
    T: SizedSample + FromSample<f64>,
{
    for frame in output.chunks_mut(channels) {
        let sample = next_sample();
        let left: T = T::from_sample(sample.0 as f64);
        let right: T = T::from_sample(sample.1 as f64);

        for (channel, sample) in frame.iter_mut().enumerate() {
            if channel & 1 == 0 {
                *sample = left;
            } else {
                *sample = right;
            }
        }
    }
}
