//! Make some granular noises. Please run me in release mode!
#![allow(clippy::precedence)]

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use fundsp::generate::*;
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
    let sample_rate = config.sample_rate as f64;
    let channels = config.channels as usize;

    let scale = [
        36.0, 38.0, 40.0, 43.0, 45.0, 48.0, 50.0, 52.0, 55.0, 57.0, 60.0, 62.0, 64.0, 67.0, 69.0,
        72.0, 74.0, 76.0, 79.0, 81.0, 84.0, 86.0, 88.0, 91.0, 93.0, 96.0,
    ];

    // We sample this granular synthesizer as source material for another granular synthesizer.
    let mut dna = Dna::new(3);
    let mut c = gen_granular(1, &scale, 2.0, 30, &mut dna);

    for parameter in dna.parameter_vector().iter() {
        println!("{}: {}", parameter.name(), parameter.value());
    }

    let mut dna2 = Dna::new(4);
    let mut fx = gen_effect(&mut dna2);
    for parameter in dna2.parameter_vector().iter() {
        println!("{}: {}", parameter.name(), parameter.value());
    }

    println!("Rendering...");
    let wave = Wave::render(sample_rate, 10.0, &mut *c);
    let mut wave2 = wave.filter(10.0, &mut *fx);
    wave2.normalize();
    let wave_arc = std::sync::Arc::new(wave2);
    println!("OK.");

    let granular = Granular::new(
        2,
        24,
        1.0,
        60,
        162,
        0.075,
        0.150,
        0.0,
        #[allow(unused_variables)]
        move |t, b, v, x, y, z| {
            let start = lerp11(4410.0, (wave_arc.len() - 4410) as f32, x);
            let start_i = round(start) as usize;
            let duration = 0.05;
            (
                duration,
                duration * 0.5,
                Box::new(
                    wavech_at(&wave_arc, 0, start_i, wave_arc.len(), None) * xerp11(0.01, 0.1, y)
                        >> peak_hz(xerp11(50.0, 3000.0, z), 3.0)
                        >> pan(v * 0.6),
                ),
            )
        },
    );

    let mut c = Net::wrap(Box::new(granular));

    c = c
        >> (multipass()
            & 0.2 * reverb2_stereo(10.0, 4.0, 0.5, 1.0, highshelf_hz(5000.0, 1.0, db_amp(-2.0))));

    c.set_sample_rate(sample_rate);

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
