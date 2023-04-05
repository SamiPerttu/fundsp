//! Make real-time changes to a network while it is playing.
#![allow(clippy::precedence)]

use assert_no_alloc::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use fundsp::hacker::*;
use funutd::*;

#[cfg(debug_assertions)] // required when disable_release is set (default)
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

fn main() {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("Failed to find a default output device");
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

    let mut net = Net64::new(0, 2);

    let id_noise = net.chain(Box::new(pink()));
    let id_pan = net.chain(Box::new(pan(0.0)));

    net.reset(Some(sample_rate));

    let mut backend = net.backend();

    let mut next_value = move || assert_no_alloc(|| backend.get_stereo());

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

    let mut rnd = Rnd::from_time();
    let mut delay_added = false;
    let mut filter_added = false;

    loop {
        std::thread::sleep(std::time::Duration::from_millis(rnd.u64_in(200, 500)));

        if rnd.bool(0.5) {
            net.replace(id_noise, Box::new(brown()));
        } else if rnd.bool(0.5) {
            net.replace(id_noise, Box::new(pink()));
        }
        if rnd.bool(0.5) {
            net.replace(id_pan, Box::new(pan(rnd.f64_in(-0.8, 0.8))));
        }
        if !delay_added && rnd.bool(0.1) {
            let id_delay = net.push(Box::new(pass() & feedback(delay(0.2) * db_amp(-5.0))));
            net.pipe(id_noise, id_delay);
            net.pipe(id_delay, id_pan);
            delay_added = true;
        }
        if !filter_added && rnd.bool(0.05) {
            net = net >> (peak_hz(1000.0, 2.0) | peak_hz(1000.0, 2.0));
            filter_added = true;
        }

        net.commit();
    }
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> (f64, f64))
where
    T: SizedSample + FromSample<f64>,
{
    for frame in output.chunks_mut(channels) {
        let sample = next_sample();
        let left = T::from_sample(sample.0);
        let right: T = T::from_sample(sample.1);

        for (channel, sample) in frame.iter_mut().enumerate() {
            if channel & 1 == 0 {
                *sample = left;
            } else {
                *sample = right;
            }
        }
    }
}
