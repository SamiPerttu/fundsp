## Changes

### Version 0.11

-Composable setting system for making it easier to control basic settings in real time.

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
