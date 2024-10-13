## Future

This is a list of feature ideas for the future.

- What is the best approach to making `Granular` real-time safe.
- `AudioUnit` versions of `oversample` and `resample` that accept an inner `AudioUnit`.
- Compressor without lookahead.
- Exponential follower (`follow` is linear).
- More physical models. Karplus-Strong exists already; figure out if it could be improved somehow.
- Dynamic bypass wrapper that bypasses a node when input and output levels drop low enough.
- Improve basic effects implemented in graph notation such as `chorus`, `flanger` and `phaser`.
- More sound generators in the `gen` module.
- Improve or replace the drum sounds in the `sound` module.
- Real-time safe sound server that uses `cpal`. It could have a static set of read/write channels for rendering audio, including hardware channels.
- Conversion of graphs into a graphical form. Format associative operator chains appropriately.
- Expand `README.md` into a book.
- Time stretching / pitch shifting algorithm.
- FFT convolution engine and HRTF support.
- Make a more flexible node replacement method for `Net` where the number of inputs and outputs could be changed.
- Text-to-speech engine.
- Resampler with sinc interpolation.
- Support feedback loops in `Net`.
- Looping in `Sequencer`.
