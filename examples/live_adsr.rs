//! This is a fully functional demonstration of `adsr_live()`. It will
//! listen to messages from the first connected MIDI input device it finds, and play the
//! corresponding pitches with the volume moderated by an `adsr_live()` envelope.
//!
//! The `create_sound()` function at the end of this file is where `adsr_live()` is employed.
//!
//! The MIDI input code is adapted from the
//! [`test_read_input`](https://github.com/Boddlnagg/midir/blob/master/examples/test_read_input.rs)
//! example in the [`midir` crate](https://github.com/Boddlnagg/midir).

use anyhow::bail;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Sample, SampleFormat, StreamConfig};
use crossbeam::atomic::AtomicCell;
use crossbeam::queue::SegQueue;
use fundsp::adsr::SoundMsg;
use fundsp::hacker::{adsr_live, envelope, midi_hz, triangle};
use fundsp::prelude::AudioUnit64;
use midi_msg::{ChannelVoiceMsg, MidiMsg};
use midir::{Ignore, MidiInput, MidiInputPort};
use read_input::prelude::*;
use std::collections::VecDeque;
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    let mut midi_in = MidiInput::new("midir reading input")?;
    let in_port = get_midi_device(&mut midi_in)?;

    let messages = Arc::new(SegQueue::new());
    run_output(messages.clone());
    run_input(messages, midi_in, in_port)
}

fn get_midi_device(midi_in: &mut MidiInput) -> anyhow::Result<MidiInputPort> {
    midi_in.ignore(Ignore::None);
    let in_ports = midi_in.ports();
    if in_ports.len() == 0 {
        bail!("No MIDI devices attached")
    } else {
        println!(
            "Chose MIDI device {}",
            midi_in.port_name(&in_ports[0]).unwrap()
        );
        Ok(in_ports[0].clone())
    }
}

fn run_input(
    outgoing_midi: Arc<SegQueue<MidiMsg>>,
    midi_in: MidiInput,
    in_port: MidiInputPort,
) -> anyhow::Result<()> {
    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(&in_port)?;
    let _conn_in = midi_in
        .connect(
            &in_port,
            "midir-read-input",
            move |_stamp, message, _| {
                let (msg, _len) = MidiMsg::from_midi(&message).unwrap();
                outgoing_midi.push(msg);
            },
            (),
        )
        .unwrap();
    println!("Connection open, reading input from '{in_port_name}'");

    let _ = input::<String>().msg("(press enter to exit)...\n").get();
    println!("Closing connection");
    Ok(())
}

fn run_output(incoming_midi: Arc<SegQueue<MidiMsg>>) {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config().unwrap();
    match config.sample_format() {
        SampleFormat::F32 => run_synth::<f32>(incoming_midi, device, config.into()),
        SampleFormat::I16 => run_synth::<i16>(incoming_midi, device, config.into()),
        SampleFormat::U16 => run_synth::<u16>(incoming_midi, device, config.into()),
    }
}

fn run_synth<T: Sample>(
    incoming_midi: Arc<SegQueue<MidiMsg>>,
    device: Device,
    config: StreamConfig,
) {
    let sample_rate = config.sample_rate.0 as f64;
    let device = Arc::new(device);
    let config = Arc::new(config);
    std::thread::spawn(move || {
        let mut sound_thread_messages: VecDeque<Arc<AtomicCell<SoundMsg>>> = VecDeque::new();
        loop {
            if let Some(m) = incoming_midi.pop() {
                if let MidiMsg::ChannelVoice { channel: _, msg } = m {
                    println!("Received {msg:?}");
                    match msg {
                        ChannelVoiceMsg::NoteOff {
                            note: _,
                            velocity: _,
                        } => {
                            if let Some(m) = sound_thread_messages.back() {
                                m.store(SoundMsg::Release);
                            }
                        }
                        ChannelVoiceMsg::NoteOn { note, velocity } => {
                            loop {
                                match sound_thread_messages.pop_front() {
                                    None => break,
                                    Some(m) => m.store(SoundMsg::Finished),
                                }
                            }
                            let note_m = Arc::new(AtomicCell::new(SoundMsg::Play));
                            sound_thread_messages.push_back(note_m.clone());
                            start_sound::<T>(
                                note,
                                velocity,
                                note_m,
                                sample_rate,
                                device.clone(),
                                config.clone(),
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
    });
}

fn start_sound<T: Sample>(
    note: u8,
    velocity: u8,
    note_m: Arc<AtomicCell<SoundMsg>>,
    sample_rate: f64,
    device: Arc<Device>,
    config: Arc<StreamConfig>,
) {
    let mut sound = create_sound(note, velocity, note_m.clone());
    sound.reset(Some(sample_rate));
    let mut next_value = move || sound.get_stereo();
    let channels = config.channels as usize;
    std::thread::spawn(move || {
        let err_fn = |err| eprintln!("an error occurred on stream: {err}");
        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    write_data(data, channels, &mut next_value)
                },
                err_fn,
            )
            .unwrap();

        stream.play().unwrap();
        loop {
            match note_m.load() {
                SoundMsg::Finished => break,
                _ => {}
            }
        }
    });
}

fn create_sound(note: u8, velocity: u8, note_m: Arc<AtomicCell<SoundMsg>>) -> Box<dyn AudioUnit64> {
    let pitch = midi_hz(note as f64);
    let volume = velocity as f64 / 127.0;
    Box::new(
        envelope(move |_t| pitch)
            >> triangle() * adsr_live(0.2, 0.2, 0.4, 0.2, note_m) * volume * 2.0,
    )
}

fn write_data<T: Sample>(
    output: &mut [T],
    channels: usize,
    next_sample: &mut dyn FnMut() -> (f64, f64),
) {
    for frame in output.chunks_mut(channels) {
        let sample = next_sample();
        let left: T = Sample::from::<f32>(&(sample.0 as f32));
        let right: T = Sample::from::<f32>(&(sample.1 as f32));

        for (channel, sample) in frame.iter_mut().enumerate() {
            *sample = if channel & 1 == 0 { left } else { right };
        }
    }
}
