//! Play notes interactively on a virtual keyboard.
//! Please run me in release mode!
#![allow(clippy::precedence)]

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use eframe::egui;
use egui::*;
use fundsp::hacker::*;

#[derive(Debug, PartialEq)]
enum Waveform {
    Sine,
    Saw,
    Square,
    Triangle,
    Organ,
}

#[derive(Debug, PartialEq)]
enum Filter {
    None,
    Moog,
    Butterworth,
    Bandpass,
}

#[allow(dead_code)]
struct State {
    /// Status of keys.
    id: Vec<Option<EventId>>,
    /// Sequencer frontend.
    sequencer: Sequencer64,
    /// Network frontend.
    net: Net64,
    /// Selected waveform.
    waveform: Waveform,
    /// Selected filter.
    filter: Filter,
    /// Reverb amount.
    reverb: Shared<f64>,
    /// Left channel data for the oscilloscope.
    snoop0: Snoop<f64>,
    /// Right channel data for the oscilloscope.
    snoop1: Snoop<f64>,
}

static KEYS: [Key; 25] = [
    Key::Z,
    Key::S,
    Key::X,
    Key::D,
    Key::C,
    Key::V,
    Key::G,
    Key::B,
    Key::H,
    Key::N,
    Key::J,
    Key::M,
    Key::Q,
    Key::Num2,
    Key::W,
    Key::Num3,
    Key::E,
    Key::R,
    Key::Num5,
    Key::T,
    Key::Num6,
    Key::Y,
    Key::Num7,
    Key::U,
    Key::I,
];

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

    let mut sequencer = Sequencer64::new(false, 1);
    let sequencer_backend = sequencer.backend();

    let (snoop0, snoop_backend0) = snoop();
    let (snoop1, snoop_backend1) = snoop();

    let reverb = shared(0.2);

    let mut net = Net64::wrap(Box::new(sequencer_backend));
    net = net >> pan(0.0);
    // Smooth the reverb amount to prevent discontinuities.
    net = net
        >> ((1.0 - var(&reverb) >> follow(0.01) >> split()) * multipass()
            & (var(&reverb) >> follow(0.01) >> split()) * reverb_stereo(20.0, 2.0))
        >> (snoop_backend0 | snoop_backend1);

    net.set_sample_rate(sample_rate);

    // Use block processing for maximum efficiency.
    let mut backend = BlockRateAdapter64::new(Box::new(net.backend()));

    let mut next_value = move || backend.get_stereo();

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

    let options = eframe::NativeOptions {
        min_window_size: Some(vec2(256.0, 280.0)),
        ..eframe::NativeOptions::default()
    };

    let mut state: State = State {
        id: Vec::new(),
        sequencer,
        net,
        waveform: Waveform::Saw,
        filter: Filter::None,
        reverb,
        snoop0,
        snoop1,
    };
    state.id.resize(KEYS.len(), None);

    eframe::run_native(
        "Virtual Keyboard Example",
        options,
        Box::new(|_cc| Box::new(state)),
    )
    .unwrap();

    Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> (f64, f64))
where
    T: SizedSample + FromSample<f64>,
{
    for frame in output.chunks_mut(channels) {
        let sample = next_sample();
        let left: T = T::from_sample(sample.0);
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

impl eframe::App for State {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Virtual Keyboard Example");
            ui.separator();
            ui.end_row();

            ui.label("Waveform");
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.waveform, Waveform::Sine, "Sine");
                ui.selectable_value(&mut self.waveform, Waveform::Saw, "Saw");
                ui.selectable_value(&mut self.waveform, Waveform::Square, "Square");
                ui.selectable_value(&mut self.waveform, Waveform::Triangle, "Triangle");
                ui.selectable_value(&mut self.waveform, Waveform::Organ, "Organ");
            });
            ui.separator();
            ui.end_row();

            ui.label("Filter");
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.filter, Filter::None, "None");
                ui.selectable_value(&mut self.filter, Filter::Moog, "Moog");
                ui.selectable_value(&mut self.filter, Filter::Butterworth, "Butterworth");
                ui.selectable_value(&mut self.filter, Filter::Bandpass, "Bandpass");
            });
            ui.separator();
            ui.end_row();

            ui.label("Reverb Amount");
            let mut reverb = self.reverb.value() * 100.0;
            ui.add(egui::Slider::new(&mut reverb, 0.0..=100.0).suffix("%"));
            self.reverb.set_value(reverb * 0.01);
            ui.separator();
            ui.end_row();

            egui::containers::Frame::canvas(ui.style()).show(ui, |ui| {
                ui.ctx().request_repaint();

                self.snoop0.update();
                self.snoop1.update();

                let points = 512;
                let color0 = Color32::from_rgb(180, 200, 220);
                let color1 = Color32::from_rgb(200, 200, 200);
                let thickness: f32 = 1.0;

                let desired_size = ui.available_width() * vec2(1.0, 0.3);
                let (_id, rect) = ui.allocate_space(desired_size);

                let to_screen = emath::RectTransform::from_to(
                    Rect::from_x_y_ranges(0.0..=points as f32, -1.0..=1.0),
                    rect,
                );

                let points0: Vec<Pos2> = (0..points)
                    .map(|i| {
                        let y = self.snoop0.at(i);
                        to_screen * pos2((points - i) as f32, softsign(y * 10.0) as f32)
                    })
                    .collect();
                let line0 = epaint::Shape::line(points0, Stroke::new(thickness, color0));
                let points1: Vec<Pos2> = (0..points)
                    .map(|i| {
                        let y = self.snoop1.at(i);
                        to_screen * pos2((points - i) as f32, softsign(y * 10.0) as f32)
                    })
                    .collect();
                let line1 = epaint::Shape::line(points1, Stroke::new(thickness, color1));
                ui.painter().add(line0);
                ui.painter().add(line1);
            });

            #[allow(clippy::needless_range_loop)]
            for i in 0..KEYS.len() {
                if ctx.input(|c| !c.key_down(KEYS[i])) {
                    if let Some(id) = self.id[i] {
                        // Start fading out existing note.
                        self.sequencer.edit_relative(id, 0.2, 0.2);
                        self.id[i] = None;
                    }
                }
                if ctx.input(|c| c.key_down(KEYS[i])) && self.id[i].is_none() {
                    let pitch = midi_hz(40.0 + i as f64);
                    let waveform = match self.waveform {
                        Waveform::Sine => Net64::wrap(Box::new(sine_hz(pitch) * 0.1)),
                        Waveform::Saw => Net64::wrap(Box::new(saw_hz(pitch) * 0.5)),
                        Waveform::Square => Net64::wrap(Box::new(square_hz(pitch) * 0.5)),
                        Waveform::Triangle => Net64::wrap(Box::new(triangle_hz(pitch) * 0.5)),
                        Waveform::Organ => Net64::wrap(Box::new(organ_hz(pitch) * 0.5)),
                    };
                    let filter = match self.filter {
                        Filter::None => Net64::wrap(Box::new(pass())),
                        Filter::Moog => Net64::wrap(Box::new(
                            (pass() | lfo(move |t| (max(200.0, 10000.0 * exp(-t)), 0.6))) >> moog(),
                        )),
                        Filter::Butterworth => Net64::wrap(Box::new(
                            (pass() | lfo(move |t| max(200.0, 10000.0 * exp(-t * 5.0))))
                                >> butterpass(),
                        )),
                        Filter::Bandpass => Net64::wrap(Box::new(
                            (pass() | lfo(move |t| (xerp11(200.0, 10000.0, sin_hz(0.2, t)), 2.0)))
                                >> bandpass(),
                        )),
                    };
                    // Insert new note. We set the end time to infinity initially.
                    self.id[i] = Some(self.sequencer.push_relative(
                        0.0,
                        f64::INFINITY,
                        Fade::Smooth,
                        0.02,
                        0.2,
                        Box::new(waveform >> filter),
                    ));
                }
            }
        });
    }
}
