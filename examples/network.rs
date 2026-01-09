//! Make real-time changes to a network while it is playing.
#![allow(clippy::precedence)]

use assert_no_alloc::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample, BufferSize};
use fundsp::prelude64::*;
use funutd::*;

#[cfg(debug_assertions)] // required when disable_release is set (default)
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

fn main() {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("Failed to find a default output device");
    let supported_config = device.default_output_config().unwrap();
    let mut config = supported_config.config();
    config.buffer_size = BufferSize::Fixed(256);

    match supported_config.sample_format() {
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

    let mut net = Net::new(0, 2);

    let id_noise = net.chain(Box::new(zero()));
    let id_pan = net.chain(Box::new(pan(0.0)));

    net.set_sample_rate(sample_rate);

    let backend = net.backend();

    let mut backend = BlockRateAdapter::new(Box::new(backend));

    // Use `assert_no_alloc` to make sure there are no allocations or deallocations in the audio thread.
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

    let mut rnd = Rnd::from_u64(1);
    let mut delay_added = false;
    let mut filter_added = false;

    loop {
        std::thread::sleep(std::time::Duration::from_millis(rnd.u64_in(200, 500)));

        if rnd.bool(0.5) {
            // Fade to brown noise.
            net.crossfade(
                id_noise,
                Fade::Smooth,
                0.2,
                Box::new(brown() * lfo(|t| 0.5 * exp(-t * 2.0))),
            );
        } else if rnd.bool(0.5) {
            // Fade to pink noise.
            net.crossfade(
                id_noise,
                Fade::Smooth,
                0.1,
                Box::new(white() * lfo(|t| 0.5 * exp(-t * 5.0))),
            );
        }
        if rnd.bool(0.5) {
            // Note: settings are always applied (or sent to the backend) immediately,
            // without waiting for the next commit.
            net.set(Setting::pan(rnd.f32_in(-1.0, 1.0)).node(id_pan));
        }
        if !delay_added && rnd.bool(0.1) {
            // Add a feedback delay.
            let id_delay = net.push(Box::new(
                pass() & feedback(delay(0.2) * db_amp(-3.0) >> pinkpass() >> highpole_hz(100.0)),
            ));
            net.pipe_all(id_noise, id_delay);
            net.pipe_all(id_delay, id_pan);
            delay_added = true;
        }

        if !filter_added && rnd.bool(0.05) {
            // We can also use the graph syntax to make changes. Connectivity can be temporarily altered,
            // as long as the network has the same number of inputs and outputs at commit time.
            net = net >> (peak_hz(1000.0, 2.0) | peak_hz(1000.0, 2.0));
            filter_added = true;
        }

        // We don't know whether we made any changes but an empty commit is harmless.
        net.commit();
    }
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> (f32, f32))
where
    T: SizedSample + FromSample<f64>,
{
    for frame in output.chunks_mut(channels) {
        let sample = next_sample();
        let left = T::from_sample(sample.0 as f64);
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
