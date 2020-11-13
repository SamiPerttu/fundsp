# adsp

## Audio DSP Library for Rust

`adsp` is a high-level audio DSP library with a focus on usability.

**This project is under construction**! It is already useful for experimentation.
However, some breakage can be expected as we continue to experiment with best practices.


## Principia

Most filters are expected to be generic over their inner processing type.

The glue type `f48` that connects audio components and populates audio buffers is chosen statically,
as a configuration option. It defaults to `f32`. The other choice is `f64` (feature `double_precision`).

There are two parallel component systems. Both operate on audio signals synchronously as an infinite stream.

| Trait            | Dispatch             | Allocation      | Operation        | Connectivity |
| ---------------- | -------------------- | --------------- | ---------------- | ------------ |
| `AudioComponent` | static, inlined      | mostly stack    | sample by sample | input and output arity fixed at compile time |
| `AudioUnit`      | dynamic, object safe | heap            | block by block   | input and output arity fixed after construction |

The `AudioUnit` system exists in embryonic form so far. The lower level `AudioComponent`s can be lifted
to block processing mode with the object safe `AudioUnit` interface via the `AcUnit<A: AudioComponent>` wrapper.

`AudioComponent`s can be stack allocated for the most part. Some components may use the heap for audio buffers and the like.

Of the signals flowing in graphs, some contain audio while others are controls of different kinds.
With control signals and parameters in general, we prefer to use natural units like Hz and seconds.
It is useful to keep parameters independent of the sample rate, which we can then adjust as we like.


## Audio Processing Environment

The prelude defines a convenient function combinator environment for audio processing using
`AudioComponent`s via the wrapper type `Ac<A: AudioComponent>`.

In the environment, the default form for components aims to ensure there is enough precision available.
Therefore, many employ double precision internally.


### Operators

Custom operators are available for combining audio components inline.
In order of precedence, from highest to lowest:

| Expression    | Meaning                       | Notes                                                          |
| ------------- | ----------------------------- | -------------------------------------------------------------- |
| `-A`          | negates `A`                   | - |
| `A * B`       | multiply `A` with `B`         | Aka amplification, or ring modulation when both are audio signals. Number of outputs in `A` and `B` must match. |
| `A + B`       | sum `A` and `B`               | Aka mixing. Number of outputs in `A` and `B` must match. |
| `A - B`       | difference of `A` and `B`     | Number of outputs in `A` and `B` must match. |
| `A >> B`      | pipe `A` to `B`               | Aka chaining. Number of outputs in `A` must match number of inputs in `B`. |
| `A & B`       | branch input to `A` and `B` in parallel  | Number of inputs in `A` and `B` must match. |
| `A | B`       | stack `A` and `B` in parallel | - |


Arithmetic operators are applied to outputs channel-wise.

All operators are associative, except `-`.

How to read the branch operator: for instance, `A >> (B & C & D)` sends output of `A` to `B` *and* `C` *and* `D`.

The expression `A >> (B & C & D)` defines a signal processing graph. It has whatever inputs `A` has, and outputs everything from `B` and `C` and `D` in parallel. The whole structure is packed, monomorphized and inlined with the constituent components consumed.

If you want to reuse components, define them as functions or use `clone()`.

Mismatched connectivity will result in a compilation error complaining about mismatched
[`typenum`](https://crates.io/crates/typenum) [types](https://docs.rs/typenum/1.12.0/typenum/uint/struct.UInt.html).
The arrays that connect components come from the
[`numeric-array` crate](https://crates.io/crates/numeric-array).



### Free Functions

These free functions are available in the environment.


## Examples

```rust

use adsp::prelude::*;

// Use >> operator to chain components.
// Lowpass filter some white noise with a cutoff of 400 Hz.
let a = white() >> lowpass_hz(400.0);
assert!(a.inputs() == 0 && a.outputs() == 1);

// Use | operator for parallel components.
// Here we send three inputs to a constant-gain resonator.
// The output is bandlimited noise filtered with a center frequency of 400 Hz and a bandwidth of 200 Hz.
let b = (white() | dc(400.0) | dc(200.0)) >> resonator();
assert!(b.inputs() == 0 && b.outputs() == 1);

// Use pass() to pass through a channel. Here is the filter from the above generator.
// We also combine the constants into one stereo constant.
let c = (pass() | dc((400.0, 200.0))) >> resonator();
assert!(c.inputs() == 1 && c.outputs() == 1);

// It is easy to define new components and combinators with the following return type.
// Here we define a simple mono-to-quad splitter.
// Use & operator for branching.
// Same input is sent to both components (read A & B as "send input to A AND B").
pub fn split_quad() -> Ac<impl AudioComponent> { pass() & pass() & pass() & pass() }
assert!(split_quad().inputs() == 1 && split_quad().outputs() == 4);

// Constants can be defined by supplying a scalar or tuple to dc() or constant().
// The two forms are synonymous. DC is a shorthand for direct current, an electrical engineering term.
// Here we define a stereo constant.
let d = constant((0.0, 1.0));
assert!(d.inputs() == 0 && d.outputs() == 2);

// There are many ways to extract samples from components.
// One shortcut is get_stereo(), which returns the next stereo sample pair using an all zeros input.
assert_eq!(d.get_stereo(), (0.0, 1.0));
// Or get_mono() (if there is more than one output, it picks the first):
assert_eq!(d.get_mono(), 0.0);

// Apply an exponentially decaying envelope to white noise.
// The closure argument is time in seconds.
// Time starts from zero and is reset to zero on reset().
// envelope() and lfo() are equivalent forms. LFO is a shorthand for low frequency oscillator.
let e = white() * envelope(|t| exp(-t));
assert!(e.inputs() == 0 && e.outputs() == 1);
```


## License

MIT or Apache-2.0.


## Future

- Develop an equivalent combinator environment for `AudioUnit`s so the user can
  decide when and where to lift components to block processing.
- Investigate whether adding more checking at compile time is possible by introducing
  opt-in signal units/modalities for `AudioComponent` inputs and outputs.
  So if the user sends a constant marked `Hz` to an audio input, then that would fail at compile time.
- How should we initialize pseudorandom phases and other sundry where the seed is properly location dependent in a graph?
  We would like a procedure so left and right channels can sound different in a deterministic way.
  Some kind of automatic poking during combination would be possible: a location ping.
  This would be complimented with a seed argument in reset(). So, for instance, get different versions of the same drum
  hit by supplying reset() with a different seed. Or are we adding too many concerns here.
  Music is naturally random phase, so taking this into account at a fundamental level is not necessarily overstepping.
  Seeding also acknowledges imperfections and approximations in the synthesis process.
  The alternative is to embrace indeterminism.
- Add more stuff.
