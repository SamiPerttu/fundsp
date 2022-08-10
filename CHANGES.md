## Changes

### Version 0.8

- `semitone` is now `semitone_ratio`.
- New opcodes `feedback2` and `fdn2`, which include extra feedback loop processing. The extra processing is not applied
in the feedforward path.
- Wetness argument was removed from `reverb_stereo`. Room size argument was added. An average room size is 10 meters. To migrate, replace `reverb_stereo(wet, time)` with `wet * reverb_stereo(10.0, time) & (1.0 - wet) * multipass()`.
- `goertzel` is now `detector`.
