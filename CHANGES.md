## Changes

### Version 0.20

- `Net::chain` is more robust now.
- Ring buffer component in the module `ring`.
- New method `Net::fade_in` for adding a node with a fade-in.
- `Net::pipe` is now `Net::pipe_all` and can no longer panic.
- `Net::pipe_op` is now `Net::pipe`, `Net::bus_op` is `Net::bus`, `Net::bin_op` is `Net::binary`,
  `Net::stack_op` is `Net::stack`, `Net::branch_op` is `Net::branch`, and `Net::thru_op` is `Net::thru`.
- New methods `Net::can_pipe`, `Net::can_bus`, `Net::can_binary`, `Net::can_stack`, `Net::can_branch`, `Net::can_thru`,
  `Net::can_sum` and `Net::can_product`.
- New methods `Net::sum` and `Net::product`.
- `Net::pipe_input` and `Net::pipe_output` can no longer panic.
- New method `Net::ids` for iterating over contained node IDs.
- New method `Net::contains` for checking existence of a node.
- New methods `Net::source` and `Net::output_source` for retrieving network edge information.
- New methods `Net::inputs_in` and `Net::outputs_in` for querying contained node arity.
- New methods `Net::set_source` and `Net::set_output_source` for setting network edges.
- New type `net::NetError` and method `Net::error` for detecting errors.
  Cycles no longer cause a panic, but trigger an error condition instead.

### Version 0.19.1

- Fixed `no_std` support.

### Version 0.19

- Added choice of interpolation algorithm to `AtomicSynth`.
- New opcode `sine_phase`.
- Opcodes `delay` and `tap_linear` now support zero sample delays.
- New method `Net::crossfade` for replacing a unit with a smooth crossfade.
- Clarified latency: it only applies to involuntary causal latencies.
- `AdaptiveTanh` is now generic `Adaptive` distortion with an inner shape.
  To migrate, try `Adaptive::new(timescale, Tanh(hardness))`.
- `Clip` shape now has a hardness parameter. `Clip(1.0)` to migrate.
- `SvfCoeffs` is now `SvfCoefs`.
- Implemented denormal prevention for `x86` inside feedback loops.
- The resonator now accepts a Q parameter instead of bandwidth in Hz.
  To migrate, Q = center / bandwidth.
- Feedback biquads and dirty biquads by Jatin Chowdhury.
- Sine oscillator has now generic inner state. To migrate, use
  `Sine<f32>` if speed is important or `Sine<f64>` if steady maintenance of phase is important.
- Non-bandlimited ramp node with opcodes `ramp`, `ramp_hz`, `ramp_phase` and `ramp_hz_phase`.

### Version 0.18.2

- 64-bit atomics were removed in order to support 32-bit targets.
- Fixed `no_std` support.

### Version 0.18.1

- Denormal flushing was removed for now.

### Version 0.18

- This release involves a major rewrite and many changes.
- 64-bit sample support is gone. All samples are now `f32`.
- The 64-bit prelude presents a 32-bit interface now, with 64-bit internal state.
- All types and traits with `32/64` suffix are replaced with the 32-bit version with the suffix removed.
- Explicit SIMD support in block processing via the `wide` crate.
- `no_std` support can be enabled by disabling the `std` feature.
- Buffers for the `process` method have been rewritten. Both stack and heap allocation is supported.
- Settings have been rewritten, to make them compatible with the `AudioUnit` system.
- Waveshaping is done using a trait now. Remove `Shape::` prefix from shapes to migrate,
  except for `AdaptiveTanh`, which is created using `AdaptiveTanh::new`.
- The `Atan` shape was tweaked to return values in -1...1 while retaining a slope of 1 at the origin.
- Asymmetric follow filters are now explicitly declared: `follow((a, r))` is now `afollow(a, r)`.
- Functions corresponding to operators are now available as an option, for example, `A >> B` can now be written `pipe(A, B)`.
  What used to be `pipe` etc. now has an `i` suffix, for example, `pipei`.
- `rnd` function is now `rnd1`.
- `hash` function is now `hash1`. Added new hash function `hash2`.
- Wavetable oscillator now accepts an `Arc<Wavetable>` in the constructor.
- Denormals are now flushed to zero in feedback loops.

### Version 0.17

- `Wave32/64`: `silence` is now `zero`.
- New opcode `impulse`.
- Attempted optimization of reverb delay times in the example `optimize`.
- New opcodes `node64` and `node32` for converting an `AudioUnit` into an `AudioNode`.
- New reverb opcodes `reverb2_stereo` and `reverb3_stereo`.
- New opcodes `allnest` and `allnest_c` for nested allpass filters.
- New opcodes `tap_linear` and `multitap_linear` for delay lines with linear interpolation.
- High frequency damping parameter was added to `reverb_stereo`. Damping used to be hardcoded to 1.
- New opcode `rotate` for rotating a stereo signal.

### Version 0.16

- `AudioNode` now requires `Send` and `Sync`.
- Feedback units `Feedback64` and `Feedback32`.
- `Shape::Atan` was contributed by Serdnad.
- New opcode `resynth` for frequency domain resynthesis.

### Version 0.15

- Snoop node for sharing audio data with a frontend thread.
- Meter smoothing parameters are now timescales specified in seconds.
- `Shape::SoftCrush` was tweaked.
- New adaptive distortion mode `Shape::AdaptiveTanh`.
- Oversampling now employs a minimum phase halfband filter.
- `Net64Backend` and `Net32Backend` are now called `NetBackend64` and `NetBackend32`.
- `Sequencer64Backend` and `Sequencer32Backend` are now called `SequencerBackend64` and `SequencerBackend32`.
- `Slot32/64` is a real-time updatable audio unit slot with crossfading between units.
- "Hammond" wavetable, which emphasizes the first three partials.
- Reverb time calculation was tweaked to take into account room size.

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
