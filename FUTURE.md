## Future

This is a list of feature ideas for the future.

- What is the best approach to making `Granular` real-time safe.
- `AudioUnit` versions of `oversample` and `Resample` that accept an inner `AudioUnit`.
- Compressor without lookahead.
- Exponential follower (`follow` is linear).
- More physical models. Karplus-Strong exists already; figure out if it could be improved somehow.
- Dynamic bypass wrapper that bypasses a node when input and output levels drop low enough.
- Improve basic effects implemented in graph notation such as `chorus`, `flanger` and `phaser`.
- More sound generators in the `generate` module.
- Improve or replace the drum sounds in the `sound` module.
- Real-time safe sound server that uses `cpal`. It could have a static set of read/write channels for rendering audio, including hardware channels.
- Conversion of graphs into a graphical form. Format associative operator chains appropriately.
- Expand `README.md` into a book.
- Time stretching / pitch shifting algorithm.
- Make a more flexible node replacement method for `Net` where the number of inputs and outputs could be changed.
- Text-to-speech engine.
- Support feedback loops in `Net`.
- Looping in `Sequencer`.
- Delay component that crossfades between taps.
