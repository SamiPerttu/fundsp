//! Symphonia integration for reading audio files.

use super::wave::*;
use std::fs::File;
use std::io::Cursor;
use std::path::Path;
extern crate alloc;
use alloc::boxed::Box;
use symphonia::core::audio::{AudioBuffer, Signal};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::{Error, Result};
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub type WaveResult<T> = Result<T>;
pub type WaveError = Error;

impl Wave {
    /// Load first track of audio file from the given path.
    /// Supported formats are anything that Symphonia can read.
    pub fn load<P: AsRef<Path>>(path: P) -> WaveResult<Wave> {
        Wave::load_track(path, None)
    }

    /// Load first track of audio from the given slice.
    /// Supported formats are anything that Symphonia can read.
    pub fn load_slice(slice: &'static [u8]) -> WaveResult<Wave> {
        Wave::load_slice_track(slice, None)
    }

    /// Load audio from the given slice. Track can be optionally selected.
    /// If not selected, the first track with a known codec will be loaded.
    /// Supported formats are anything that Symphonia can read.
    pub fn load_slice_track(slice: &'static [u8], track: Option<usize>) -> WaveResult<Wave> {
        let hint = Hint::new();
        let source: Box<dyn MediaSource> = Box::new(Cursor::new(slice));
        Wave::decode(source, track, hint)
    }

    /// Load audio file from the given path. Track can be optionally selected.
    /// If not selected, the first track with a known codec will be loaded.
    /// Supported formats are anything that Symphonia can read.
    pub fn load_track<P: AsRef<Path>>(path: P, track: Option<usize>) -> WaveResult<Wave> {
        let path = path.as_ref();
        let mut hint = Hint::new();

        if let Some(extension) = path.extension() {
            if let Some(extension_str) = extension.to_str() {
                hint.with_extension(extension_str);
            }
        }

        let source: Box<dyn MediaSource> = match File::open(path) {
            Ok(file) => Box::new(file),
            Err(error) => return Err(Error::IoError(error)),
        };

        Wave::decode(source, track, hint)
    }

    /// Decode track from the given source.
    fn decode(source: Box<dyn MediaSource>, track: Option<usize>, hint: Hint) -> WaveResult<Wave> {
        let stream = MediaSourceStream::new(source, Default::default());

        let format_opts = FormatOptions {
            enable_gapless: false,
            ..Default::default()
        };

        let metadata_opts: MetadataOptions = Default::default();

        let mut wave: Option<Wave> = None;

        match symphonia::default::get_probe().format(&hint, stream, &format_opts, &metadata_opts) {
            Ok(probed) => {
                let mut reader = probed.format;

                // Select track if specified, otherwise select the first track with a known codec.
                let track = track.and_then(|t| reader.tracks().get(t)).or_else(|| {
                    reader
                        .tracks()
                        .iter()
                        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
                });

                let track_id = match track {
                    Some(track) => track.id,
                    _ => return Err(Error::DecodeError("Could not find track.")),
                };

                let track = match reader.tracks().iter().find(|track| track.id == track_id) {
                    Some(track) => track,
                    _ => return Err(Error::DecodeError("Could not find track.")),
                };

                let decode_opts = DecoderOptions::default();

                let mut decoder =
                    symphonia::default::get_codecs().make(&track.codec_params, &decode_opts)?;

                loop {
                    let packet = match reader.next_packet() {
                        Ok(packet) => packet,
                        Err(err) => {
                            if let Some(wave_output) = wave {
                                return Ok(wave_output);
                            } else {
                                return Err(err);
                            }
                        }
                    };

                    // If the packet does not belong to the selected track, skip it.
                    if packet.track_id() != track_id {
                        continue;
                    }

                    match decoder.decode(&packet) {
                        Ok(decoded) => {
                            if wave.is_none() {
                                let spec = *decoded.spec();
                                wave = Some(Wave::new(spec.channels.count(), spec.rate as f64));
                            } else {
                                // TODO: Check that audio spec hasn't changed.
                            }

                            if let Some(ref mut wave_output) = wave {
                                let mut dest = AudioBuffer::<f32>::new(
                                    decoded.capacity() as u64,
                                    *decoded.spec(),
                                );
                                dest.render_silence(Some(decoded.frames()));

                                match &decoded {
                                    symphonia::core::audio::AudioBufferRef::U8(reff) => {
                                        reff.convert(&mut dest);
                                    }
                                    symphonia::core::audio::AudioBufferRef::U16(reff) => {
                                        reff.convert(&mut dest);
                                    }
                                    symphonia::core::audio::AudioBufferRef::U24(reff) => {
                                        reff.convert(&mut dest);
                                    }
                                    symphonia::core::audio::AudioBufferRef::U32(reff) => {
                                        reff.convert(&mut dest);
                                    }
                                    symphonia::core::audio::AudioBufferRef::S8(reff) => {
                                        reff.convert(&mut dest);
                                    }
                                    symphonia::core::audio::AudioBufferRef::S16(reff) => {
                                        reff.convert(&mut dest);
                                    }
                                    symphonia::core::audio::AudioBufferRef::S24(reff) => {
                                        reff.convert(&mut dest);
                                    }
                                    symphonia::core::audio::AudioBufferRef::S32(reff) => {
                                        reff.convert(&mut dest);
                                    }
                                    symphonia::core::audio::AudioBufferRef::F32(reff) => {
                                        reff.convert(&mut dest);
                                    }
                                    symphonia::core::audio::AudioBufferRef::F64(reff) => {
                                        reff.convert(&mut dest);
                                    }
                                }

                                let buffer_len = decoded.frames();

                                for channel in 0..dest.spec().channels.count() {
                                    let x = dest.chan(channel);
                                    if channel == 0 {
                                        for _i in 0..buffer_len {
                                            wave_output.push(0.0);
                                        }
                                    }
                                    for i in 0..buffer_len {
                                        wave_output.set(
                                            channel,
                                            wave_output.len() - buffer_len + i,
                                            x[i],
                                        );
                                    }
                                }
                            }
                        }
                        Err(err) => return Err(err),
                    }
                }
            }
            Err(err) => Err(err),
        }
    }
}
