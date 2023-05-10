## Changes

### Version 0.15 (Next Version)

- Snoop node for sharing audio data with a frontend thread.
- Meter smoothing parameters are now timescales specified in seconds.
- `Shape::SoftCrush` was tweaked.
- New adaptive distortion mode `Shape::AdaptiveTanh`.

### Version 0.14

- New math functions `sqr_hz` and `tri_hz` for non-bandlimited square and triangle waves.
- Lorenz and Rössler chaotic system oscillators as opcodes `lorenz` and `rossler`.
- `swap_stereo` is now generic `reverse`, which reverses channel order.
- Resonant two-pole filter by Paul Kellett as `bandrez` and `lowrez`.
- Sample-and-hold opcodes `hold` and `hold_hz`.
- Fixed inbuilt waveform phases.
- Reduced number of all-pass stages in `phaser` to 10.
- Sequencer `add` and `add_duration` are now `push` and `push_duration`.
- Reseting a node and setting its sample rate are now two distinct operations.
- Sequencer can now have a real-time safe backend.

### Version 0.13

- Fade curves for Sequencer events. To migrate, use the curve `Fade::Smooth`.
- `Net32/64` now operates on stable node IDs of type `NodeId`. Other improvements.
- `swap` opcode is now `swap_stereo`, to avoid possible conflicts with `std::mem::swap`.
- New method `AudioNode::allocate` for preallocating everything.
- Identity function has been renamed from `id` to `identity`, to match the standard library.
- Setting system MPSC channels were async by mistake; they are now blocking.
- `Net32/64` can now have a real-time safe backend.

### Version 0.12

- More settings implemented.
- Signal routing is now an exclusive operation, for practical reasons.
- Granular synthesizer. FunUTD is now a dependency.
- Block rate adapter that converts calls to block processing.
- New random function `rnd2` from Krull64 output stage.

### Version 0.11

- Composable setting system for making it easier to control basic settings in real time.
- `system` opcode was renamed `update`.
- New waveforms, organ and soft saw.
- New envelope opcodes with any number of inputs: `envelope_in` and `lfo_in`.

### Version 0.10

- New opcode `resample` for variable speed cubic resampling.
- `Wave32/64` improvements. Symphonia integration for reading audio files.
- Tagged constants were removed. They were not scaleable.
- Callbacks were removed from `Sequencer32/64` and `Net32/64`. Will reimplement if requested.
- The `follow` filter now jumps immediately to the very first input value.

### Version 0.9

- `detector` was removed (it did not work).
- New opcode `biquad` for an arbitrary biquad filter.
- `Net32/64` method `add` was renamed `push` (it conflicted with the operator implementation).
- Graph syntax was implemented for `Net32/64`. They can also be combined with components from the preludes.

### Version 0.8

- `semitone` is now `semitone_ratio`.
- New opcodes `feedback2` and `fdn2`, which include extra feedback loop processing. The extra processing is not applied
in the feedforward path.
- Wetness argument was removed from `reverb_stereo`. Room size argument was added. An average room size is 10 meters. To migrate, replace `reverb_stereo(wet, time)` with `wet * reverb_stereo(10.0, time) & (1.0 - wet) * multipass()`.
- `goertzel` is now `detector`.
- `chorus` gain was adjusted.
- `flanger` was tweaked.
- New opcode `system` with a user provided callback. See the `sound` module for an example.
- Ability to replace nodes in `Net32` and `Net64`. Improved chaining method.
- `Sequencer` has been replaced with `Sequencer32` and `Sequencer64`.
- `Au` has been removed.
- Callback functionality was added to the sequencer and network components.
- Ability to jump in time was added to the sequencer.
