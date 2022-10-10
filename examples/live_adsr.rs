//! This is a fully functional demonstration of `adsr_live()` and of `var()`. It will
//! listen to messages from the first connected MIDI input device it finds, and play the
//! corresponding pitches with the volume moderated by an `adsr_live()` envelope. It uses a `var()`
//! to alter the pitches as they are running in response to MIDI `pitch_bend` messages.
//!
//! This program's design is structured upon the following threads:
//! * The `main()` thread listens for MIDI inputs, and places them in a `SegQueue`.
//! * The thread started in `run_synth()` removes MIDI messages from the `SegQueue` and decides
//! what to do with them. For `Note_on` messages, it calls `start_sound()`.
//! * The `start_sound()` function creates a thread to output a particular sound.
//!
//! The `create_sound()` function is where `adsr_live()` is employed and where the pitch bend is
//! incorporated. The `var()` containing the pitch bend is set up in the `run_synth()` function.
//! It is altered whenever a pitch bend is received, with that alteration visible in the sound
//! production thread.
//!
//! The MIDI input code is adapted from the
//! [`test_read_input`](https://github.com/Boddlnagg/midir/blob/master/examples/test_read_input.rs)
//! example in the [`midir` crate](https://github.com/Boddlnagg/midir).
#![allow(clippy::precedence)]

use anyhow::bail;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Sample, SampleFormat, StreamConfig};
use crossbeam_queue::SegQueue;
use crossbeam_utils::atomic::AtomicCell;
use fundsp::adsr::SoundMsg;
use fundsp::hacker::{adsr_live, envelope2, midi_hz, triangle, var};
use fundsp::prelude::{An, AudioUnit64, Tag, Var};
use midi_msg::{ChannelVoiceMsg, MidiMsg};
use midir::{Ignore, MidiInput, MidiInputPort};
use read_input::prelude::*;
use std::collections::VecDeque;
use std::sync::Arc;

const PITCH_TAG: Tag = 1;

fn main() -> anyhow::Result<()> {
    let mut midi_in = MidiInput::new("midir reading input")?;
    let in_port = get_midi_device(&mut midi_in)?;

    let messages = Arc::new(SegQueue::new());
    run_output(messages.clone());
    run_input(messages, midi_in, in_port)
}

/// This function is where the `adsr_live()` function is employed. We have the following signal
/// chain in place:
///
/// * The `pitch_bend` value (determined by MIDI `pitch-bend` messages) feeds into an `envelope2()`.
/// * The `envelope2()` multiplies the pitch by the incoming pitch-bend value.
/// * The `triangle()` transforms the envelope output into a triangle waveform. For different
/// sounds, try out some different waveform functions!
/// * The `adsr_live()` modulates the volume of the sound over time. Play around with the different
/// values to get a feel for the impact of different ADSR levels.
/// * Finally, we modulate the volume further using the MIDI velocity. As the `triangle()` tends to
/// produce quiet sounds, we double the volume.
///
fn create_sound(
    note: u8,
    velocity: u8,
    pitch_bend: An<Var<f64>>,
    note_m: Arc<AtomicCell<SoundMsg>>,
) -> Box<dyn AudioUnit64> {
    let pitch = midi_hz(note as f64);
    let volume = velocity as f64 / 127.0;
    Box::new(
        pitch_bend
            >> envelope2(move |_t, bend| pitch * bend)
            >> triangle() * adsr_live(0.1, 0.2, 0.4, 0.2, note_m) * volume * 2.0,
    )
}

fn get_midi_device(midi_in: &mut MidiInput) -> anyhow::Result<MidiInputPort> {
    midi_in.ignore(Ignore::None);
    let in_ports = midi_in.ports();
    if in_ports.is_empty() {
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
                let (msg, _len) = MidiMsg::from_midi(message).unwrap();
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

/// This function is where `var()` is employed to create a thread-safe variable, `pitch_bend`.
/// The `pitch_bend` variable is cloned when passed along to `start_sound()`, from which it
/// eventually is passed to `create_sound()`. Each `var()` atomically stores a numerical value
/// which can be altered in a different thread.
fn run_synth<T: Sample>(
    incoming_midi: Arc<SegQueue<MidiMsg>>,
    device: Device,
    config: StreamConfig,
) {
    let sample_rate = config.sample_rate.0 as f64;
    let device = Arc::new(device);
    let config = Arc::new(config);
    std::thread::spawn(move || {
        let mut pitch_bend = var(PITCH_TAG, 1.0);
        let mut sound_thread_messages: VecDeque<Arc<AtomicCell<SoundMsg>>> = VecDeque::new();
        loop {
            if let Some(MidiMsg::ChannelVoice { channel: _, msg }) = incoming_midi.pop() {
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
                        stop_all_other_notes(&mut sound_thread_messages);
                        pitch_bend.set(PITCH_TAG, 1.0);
                        let note_m = Arc::new(AtomicCell::new(SoundMsg::Play));
                        sound_thread_messages.push_back(note_m.clone());
                        start_sound::<T>(
                            note,
                            velocity,
                            pitch_bend.clone(),
                            note_m,
                            sample_rate,
                            device.clone(),
                            config.clone(),
                        );
                    }
                    ChannelVoiceMsg::PitchBend { bend } => {
                        pitch_bend.set(PITCH_TAG, pitch_bend_factor(bend));
                    }
                    _ => {}
                }
            }
        }
    });
}

// Algorithm is from here: https://sites.uci.edu/camp2014/2014/04/30/managing-midi-pitchbend-messages/
// Converts MIDI pitch-bend message to +/- 1 semitone.
fn pitch_bend_factor(bend: u16) -> f64 {
    2.0_f64.powf(((bend as f64 - 8192.0) / 8192.0) / 12.0)
}

fn stop_all_other_notes(sound_thread_messages: &mut VecDeque<Arc<AtomicCell<SoundMsg>>>) {
    loop {
        match sound_thread_messages.pop_front() {
            None => break,
            Some(m) => m.store(SoundMsg::Finished),
        }
    }
}

fn start_sound<T: Sample>(
    note: u8,
    velocity: u8,
    pitch_bend: An<Var<f64>>,
    note_m: Arc<AtomicCell<SoundMsg>>,
    sample_rate: f64,
    device: Arc<Device>,
    config: Arc<StreamConfig>,
) {
    let mut sound = create_sound(note, velocity, pitch_bend, note_m.clone());
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
            if note_m.load() == SoundMsg::Finished {
                break;
            }
        }
    });
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
