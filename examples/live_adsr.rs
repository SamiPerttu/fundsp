//! This is a monophonic synthesizer that demonstrates `adsr_live()` and `var()`. It
//! listens to messages from the first connected MIDI input device it finds and plays the
//! corresponding pitches with the volume moderated by an `adsr_live()` envelope. It uses
//! four `var()` objects to share data between threads:
//! * `pitch`: Controls the current pitch. Altered by MIDI `NoteOn` messages as they arrive.
//! * `volume`: Controls the current volume. Altered by MIDI `NoteOn` and `NoteOff` messages as they arrive.
//! * `pitch_bend`: Scales the current pitch according to MIDI `PitchBend` messages as they arrive.
//! * `control`: Signals when to start the attack, stop the sustain, and start the release.
//!    Altered by MIDI `NoteOn` and `NoteOff` messages as they arrive.
//!
//! This program's design is structured around these two threads:
//! * The `main()` thread listens for MIDI inputs and alters the `var()` objects as described above.
//! * The thread spawned by the `run_synth()` function passes the `var()` objects to `create_sound()`.
//!   It then starts playing the sound. As the `var()` objects change, the sound automatically
//!   changes accordingly.
//!
//! The MIDI input code is adapted from the
//! [`test_read_input`](https://github.com/Boddlnagg/midir/blob/master/examples/test_read_input.rs)
//! example in the [`midir` crate](https://github.com/Boddlnagg/midir).
#![allow(clippy::precedence)]

use anyhow::bail;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Sample, SampleFormat, StreamConfig};
use fundsp::hacker::{adsr_live, midi_hz, triangle, var};
use fundsp::prelude::{An, AudioUnit64, Tag, Var};
use midi_msg::{ChannelVoiceMsg, MidiMsg};
use midir::{Ignore, MidiInput, MidiInputPort};
use read_input::prelude::*;

const PITCH_TAG: Tag = 1;
const BEND_TAG: Tag = PITCH_TAG + 1;
const RELEASE_TAG: Tag = BEND_TAG + 1;
const VOLUME_TAG: Tag = RELEASE_TAG + 1;

/// The `var()` objects are created in `main()`. They are cloned when passed to `run_output()` so
/// as not to lose ownership. But as with other Rust types intended for sharing between threads
/// (e.g. `Arc`), cloning does not duplicate them - it creates an alternative reference.
fn main() -> anyhow::Result<()> {
    let mut midi_in = MidiInput::new("midir reading input")?;
    let in_port = get_midi_device(&mut midi_in)?;

    let pitch = var(PITCH_TAG, 0.0);
    let volume = var(VOLUME_TAG, 0.0);
    let pitch_bend = var(BEND_TAG, 1.0);
    let control = var(RELEASE_TAG, 0.0);

    run_output(
        pitch.clone(),
        volume.clone(),
        pitch_bend.clone(),
        control.clone(),
    );
    run_input(midi_in, in_port, pitch, volume, pitch_bend, control)
}

/// This function is where the `adsr_live()` function is employed. We have the following signal
/// chain in place:
///
/// * The `pitch_bend` value (determined by MIDI `PitchBend` messages) is multiplied by the pitch.
/// * The `triangle()` transforms the envelope output into a triangle waveform. For different
///   sounds, try out some different waveform functions here!
/// * The `adsr_live()` modulates the volume of the sound over time. Play around with the different
///   values to get a feel for the impact of different ADSR levels. The `control` `var()` is set
///   to 1.0 to start the attack and -1.0 to start the release.
/// * Finally, we modulate the volume further using the MIDI velocity.
///
fn create_sound(
    pitch: An<Var<f64>>,
    volume: An<Var<f64>>,
    pitch_bend: An<Var<f64>>,
    control: An<Var<f64>>,
) -> Box<dyn AudioUnit64> {
    Box::new(pitch_bend * pitch >> triangle() * (control >> adsr_live(0.1, 0.2, 0.4, 0.2)) * volume)
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

/// This function is where MIDI events control the values of the `var()` objects.
/// * A `NoteOn` event alters all four `var()` objects:
///   * Using `midi_hz()`, a MIDI pitch is converted to a frequency and stored.
///   * MIDI velocity values range from 0 to 127. We divide by 127 and store in `volume`.
///   * Setting `pitch_bend` to 1.0 makes the bend neutral.
///   * Setting `control` to 1.0 starts the attack.
/// * A `NoteOff` event sets `control` to -1.0 to start the release.
/// * A `PitchBend` event calls `pitch_bend_factor()` to convert the MIDI values into
///   a scaling factor for the pitch, which it stores in `pitch_bend`.
fn run_input(
    midi_in: MidiInput,
    in_port: MidiInputPort,
    pitch: An<Var<f64>>,
    volume: An<Var<f64>>,
    pitch_bend: An<Var<f64>>,
    control: An<Var<f64>>,
) -> anyhow::Result<()> {
    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(&in_port)?;
    let _conn_in = midi_in
        .connect(
            &in_port,
            "midir-read-input",
            move |_stamp, message, _| {
                let (msg, _len) = MidiMsg::from_midi(message).unwrap();
                if let MidiMsg::ChannelVoice { channel: _, msg } = msg {
                    println!("Received {msg:?}");
                    match msg {
                        ChannelVoiceMsg::NoteOn { note, velocity } => {
                            pitch.set_value(midi_hz(note as f64));
                            volume.set_value(velocity as f64 / 127.0);
                            pitch_bend.set_value(1.0);
                            control.set_value(1.0);
                        }
                        ChannelVoiceMsg::NoteOff { note, velocity: _ } => {
                            if pitch.value() == midi_hz(note as f64) {
                                control.set_value(-1.0);
                            }
                        }
                        ChannelVoiceMsg::PitchBend { bend } => {
                            pitch_bend.set_value(pitch_bend_factor(bend));
                        }
                        _ => {}
                    }
                }
            },
            (),
        )
        .unwrap();
    println!("Connection open, reading input from '{in_port_name}'");

    let _ = input::<String>().msg("(press enter to exit)...\n").get();
    println!("Closing connection");
    Ok(())
}

/// This function figures out the sample format and calls `run_synth()` accordingly.
fn run_output(
    pitch: An<Var<f64>>,
    volume: An<Var<f64>>,
    pitch_bend: An<Var<f64>>,
    control: An<Var<f64>>,
) {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config().unwrap();
    match config.sample_format() {
        SampleFormat::F32 => {
            run_synth::<f32>(pitch, volume, pitch_bend, control, device, config.into())
        }
        SampleFormat::I16 => {
            run_synth::<i16>(pitch, volume, pitch_bend, control, device, config.into())
        }
        SampleFormat::U16 => {
            run_synth::<u16>(pitch, volume, pitch_bend, control, device, config.into())
        }
    }
}

/// This function is where the sound is created and played. Once the sound is playing, it loops
/// infinitely, allowing the `var()` objects to shape the sound in response to MIDI events.
fn run_synth<T: Sample>(
    pitch: An<Var<f64>>,
    volume: An<Var<f64>>,
    pitch_bend: An<Var<f64>>,
    control: An<Var<f64>>,
    device: Device,
    config: StreamConfig,
) {
    std::thread::spawn(move || {
        let sample_rate = config.sample_rate.0 as f64;
        let mut sound = create_sound(pitch, volume, pitch_bend, control);
        sound.reset(Some(sample_rate));

        let mut next_value = move || sound.get_stereo();
        let channels = config.channels as usize;
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
        loop {}
    });
}

/// Algorithm is from here: https://sites.uci.edu/camp2014/2014/04/30/managing-midi-pitchbend-messages/
/// Converts MIDI pitch-bend message to +/- 1 semitone.
fn pitch_bend_factor(bend: u16) -> f64 {
    2.0_f64.powf(((bend as f64 - 8192.0) / 8192.0) / 12.0)
}

/// Callback function to send the current sample to the speakers.
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
