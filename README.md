# adsp

## Audio DSP Library for Rust
`
`adsp` is a high-level audio DSP library with a focus on usability.

This project is under heavy construction! But it is already useful for experimentation.
Some breakage can be expected as we experiment with best practices.

## Audio Processing Environment

The prelude defines a convenient combinator environment for audio processing.

### Combination Operators

Custom operators are available for combining audio components.

| Expression    | Meaning                 | Constraints                                                    |
| ------------- | ----------------------- | --------------------------------------------------------------
| `A >> B`      | pipe `A` to `B`         | Number of outputs in `A` must match number of inputs in `B`. |
| `A | B`       | stack `A` and `B` in parallel | - |
| `A & B`       | branch input to `A` and `B` | Number of inputs in `A` and `B` must match. |
| `A * B`       | multiply `A` with `B`   | Number of outputs in `A` and `B` must match. |
| `A + B`       | mix `A` and `B`         | Number of outputs in `A` and `B` must match. |
| `A - B`       | difference of `A` and `B` | Number of outputs in `A` and `B` must match. |
| `-A`          | negates `A` | - |

Some examples:

```rust

use adsp::prelude::*;

// Use >> operator to chain components.
// Lowpass filter some white noise with a cutoff of 400 Hz.
let a = white() >> lowpass_hz(400.0);
assert!(a.inputs() == 0 && a.outputs() == 1);

// Use | operator for parallel components.
// Here we send three distinct inputs to a constant-gain resonator:
// white noise with a center frequency of 400 Hz and a bandwidth of 200 Hz.
let b = (white() | constant(400.0) | constant(200.0)) >> resonator();
assert!(b.inputs() == 0 && b.outputs() == 1);

// Use pass() to pass through a channel. Here is the above generator as a filter.
// We also combine the constants into one stereo constant.
let c = (pass() | constant((400.0, 200.0))) >> resonator();
assert!(c.inputs() == 1 && c.outputs() == 1);

// It is easy to define new components and combinators.
// Here we define a simple mono-to-stereo splitter.
// Use & operator for branching.
// Same input is sent to both components (read A & B as "send input to A AND B").
pub fn splitter() -> Ac<impl AudioComponent> { pass() & pass() }
assert!(splitter().inputs() == 1 && splitter().outputs() == 2);

// Constants can be defined by supplying a scalar or tuple to dc() or constant().
// The two forms are synonymous.
// Here we define a stereo constant.
let d = dc((0.0, 1.0));
assert!(d.inputs() == 0 && d.outputs() == 2);

// There are many ways to extract samples from components.
// One shortcut is get_stereo(), which returns the next stereo sample pair using an all zeros input.
assert!(d.get_stereo() == (0.0, 1.0));

// Apply an exponentially decaying envelope to white noise.
// The closure argument is time in seconds.
// Time starts from zero and is reset to zero on reset().
let e = white() * envelope(|t| exp(-t));
assert!(e.inputs() == 0 && e.outputs() == 1);

```
