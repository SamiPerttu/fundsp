//! Make some granular noises. Please run me in release mode!
#![allow(clippy::precedence)]

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

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
        2.0,
        60,
        1,
        0.05,
        0.15,
        0.0,
        #[allow(unused_variables)]
        /*|t, b, v, x, y, z| {
            let f = xerp11(60.0, 3000.0, x);
            let r = max(1.0, xerp11(0.50, 2.00, z));
            let f = f * xerp11(1.0 / r, r, sin_hz(8.0, t));
            (
                0.06,
                0.03,
                Box::new(
                    soft_saw_hz(f) * xerp11(0.005, 0.05, y) >> pan(v * 0.6),
                ),
            )
        },*/
        /*|t, b, v, x, y, z| {
            let scale = [
                44.0, 46.0, 49.0, 51.0, 53.0, 55.0, 56.0, 58.0, 61.0, 63.0, 65.0, 67.0, 68.0, 70.0,
                73.0, 75.0, 77.0, 79.0, 80.0, 82.0, 84.0, 87.0, 89.0, 91.0, 92.0,
            ];
            let d = lerp11(0.0, scale.len() as f64 - 0.01, x);
            let f = midi_hz(scale[d as usize] + 0.02 * (d - round(d)));
            (
                0.06,
                0.025,
                Box::new(
                    saw_hz(f) * 0.05
                        >> moog_hz(xerp11(20.0, 20000.0, y), lerp11(0.1, 0.65, z))
                        >> pan(v * 0.8),
                ),
            )
        },*/
        /*|t, b, v, x, y, z| {
            let f = xerp11(60.0, 3000.0, x);
            let c = xerp11(2.0, 200.0, y);
            (
                0.10,
                0.03,
                Box::new(
                    (white() >> lowpass_hz(c, 1.0)) * sine_hz(f) * (xerp11(1.0, 10.0, z) / sqrt(c))
                        >> pan(v * 0.5),
                ),
            )
        },*/
        /*|t, b, v, x, y, z| {
            let b = 2.0 * b;
            let d = b - round(b);
            (
                0.08,
                0.04,
                Box::new(
                    (pink() | dc((xerp11(50.0, 5000.0, x - 0.1 * d), xerp11(10.0, 100.0, (x - d * 2.0) * 0.5))))
                        >> !resonator() >> resonator() * 0.005
                        >> pan(v * (d + 0.5)),
                ),
            )
        },*/
        /*|t, b, v, x, y, z| {
            let scale = [
                44.0, 46.0, 49.0, 51.0, 53.0, 55.0, 56.0, 58.0, 61.0, 63.0, 65.0, 67.0, 68.0, 70.0,
                73.0, 75.0, 77.0, 79.0, 80.0, 82.0, 84.0, 87.0, 89.0, 91.0, 92.0,
            ];
            let d = lerp11(0.0, scale.len() as f64 - 0.01, x);
            let f = midi_hz(scale[d as usize] + 0.02 * (d - round(d)));
            //let f = xerp11(60.0, 3000.0, x);
            let d = 0.5 + xerp11(0.1, 10.0, y);
            (
                0.050,
                0.025,
                Box::new(
                    sine_hz(f)
                        >> shape(Shape::Tanh(d)) * (xerp11(0.002, 0.02, z) / max(0.2, a_weight(f)))
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
                0.035,
                Box::new(
                    dc((f, lerp11(0.50, 0.99, y)))
                    >> pulse() >> peak_hz(xerp11(60.0, 10000.0, z), 5.0) * (0.01 / a_weight(f))
                    >> pan(v * 0.7),
                ),
            )
        },*/
        |t, b, v, x, y, z| {
            let scale = [
                36.0, 38.0, 40.0, 43.0, 45.0, 48.0, 50.0, 52.0, 55.0, 57.0, 60.0, 62.0, 64.0, 67.0,
                69.0, 72.0, 74.0, 76.0, 79.0, 81.0, 84.0, 86.0, 88.0, 91.0, 93.0, 96.0,
            ];
            let d = lerp11(0.0, scale.len() as f64 - 0.01, x);
            let f = midi_hz(scale[d as usize] + 0.02 * (d - round(d)));
            (
                0.06,
                0.02,
                Box::new(
                    organ_hz(f) * (0.05 / a_weight(f))
                        >> moog_hz(xerp11(20.0, 20000.0, y), lerp11(0.10, 0.65, z))
                        >> pan(v * 0.7),
                ),
            )
        },
    );

    let mut c = Net64::wrap(Box::new(c));
    c = c >> (multipass() & 0.1 * reverb_stereo(20.0, 2.0)) >> (dcblock() | dcblock());

    c.reset(Some(sample_rate));

    let mut c = BlockRateAdapter64::new(Box::new(c));

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

    std::thread::sleep(std::time::Duration::from_millis(120_000));

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
