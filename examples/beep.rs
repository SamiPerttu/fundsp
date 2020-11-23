#![allow(clippy::precedence)]

extern crate anyhow;
extern crate cpal;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use fundsp::hacker::*;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "full"))]
fn main() {
    // Conditionally compile with jack if the feature is specified.
    #[cfg(all(
        any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"),
        feature = "jack"
    ))]
    // Manually check for flags. Can be passed through cargo with -- e.g.
    // cargo run --release --example beep --features jack -- --jack
    let host = if std::env::args()
        .collect::<String>()
        .contains(&String::from("--jack"))
    {
        cpal::host_from_id(cpal::available_hosts()
            .into_iter()
            .find(|id| *id == cpal::HostId::Jack)
            .expect(
                "make sure --features jack is specified. only works on OSes where jack is available",
            )).expect("jack host unavailable")
    } else {
        cpal::default_host()
    };

    #[cfg(any(
        not(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd")),
        not(feature = "jack")
    ))]
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

    //let c = mls();
    //let c = mls() >> lowpole_hz(400.0) >> lowpole_hz(400.0);
    //let c = (mls() | dc(500.0)) >> lowpass();
    //let c = (mls() | dc(400.0) | dc(50.0)) >> resonator();
    //let c = (((mls() | dc(800.0) | dc(50.0)) >> resonator()) | dc(800.0) | dc(50.0)) >> resonator() * 0.1;
    //let c = (((mls() | dc(400.0) | dc(50.0)) >> resonator()) | dc(400.0) | dc(50.0)) >> resonator() >> mul(0.1);
    let c = (white() & white()) * 0.5 * envelope(|t| exp(-t * 0.5) * sin_bpm(128.0, t));
    //let f = 110.0;
    //let m = 2.0;
    //let c = sine_hz(f) * f * m + f >> sine();
    let fb = feedback(lowpass_hz(1000.0) >> delay(1.0) * 0.9);
    let mut c = (c >> fb) * 0.1 >> dcblock();
    //let mut c = c * 0.1;
    c.reset(Some(sample_rate));

    //let mut next_value = move || { let v = c.get_mono(); assert!(v.is_nan() == false && abs(v) < 1.0e6); v };
    let mut next_value = move || c.get_mono() as f32;
    //let mut next_value = c.as_mono_fn();

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
    )?;
    stream.play()?;

    std::thread::sleep(std::time::Duration::from_millis(10000));

    Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let value: T = cpal::Sample::from::<f32>(&next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
