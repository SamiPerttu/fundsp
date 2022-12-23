//! Make some granular noises.
#![allow(clippy::precedence)]

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use fundsp::granular::*;
use fundsp::hacker::*;

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
    }
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), anyhow::Error>
where
    T: cpal::Sample,
{
    let sample_rate = config.sample_rate.0 as f64;
    let channels = config.channels as usize;

    let c = Granular64::new(
        2,
        32,
        0.06,
        0.02,
        2.0,
        60,
        56,
        0.10,
        0.15,
        0.0,
        #[allow(unused_variables)]
        //|x, r, g, b| Box::new(sine_hz(xerp11(20.0, 2000.0, r)) * 0.02 * g >> pan(x * 0.5)),
        //|x, r, g, b| Box::new(triangle_hz(xerp11(50.0, 5000.0, r)) * 0.02 * g >> pan(x * 0.5)),
        |x, r, g, b| {
            Box::new(
                saw_hz(xerp11(20.0, 800.0, r)) * 0.1
                    >> moog_hz(xerp11(20.0, 20000.0, g), lerp11(0.1, 0.5, b))
                    >> pan(x),
            )
        },
        /*|x, r, g, b| {
            Box::new(
                (pink() | dc((xerp11(20.0, 5000.0, r), 10.0))) >> bandpass() * 0.05 * g >> pan(x * 0.5),
            )
        },*/
        /*|x, r, g, b| {
            Box::new(
                (pink() | dc((xerp11(20.0, 2000.0, r), 10.0)))
                    >> resonator() * 0.02
                    >> pan(x * 0.5),
            )
        },*/
        /*|x, r, g, b| {
            Box::new(
                sine_hz(xerp11(20.0, 2000.0, r)) >> shape(Shape::Tanh(1.0 + g * g * 10.0)) * 0.02 >> pan(b)
            )
        },*/
    );

    let mut c = Net64::wrap(Box::new(c));
    //c = c >> (multipass() & 0.1 * reverb_stereo(10.0, 2.0));

    c.reset(Some(sample_rate));

    let mut next_value = move || c.get_stereo();

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
    )?;
    stream.play()?;

    std::thread::sleep(std::time::Duration::from_millis(120000));

    Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> (f64, f64))
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let sample = next_sample();
        let left: T = cpal::Sample::from::<f32>(&(sample.0 as f32));
        let right: T = cpal::Sample::from::<f32>(&(sample.1 as f32));

        for (channel, sample) in frame.iter_mut().enumerate() {
            if channel & 1 == 0 {
                *sample = left;
            } else {
                *sample = right;
            }
        }
    }
}
