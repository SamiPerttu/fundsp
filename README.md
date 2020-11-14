# adsp

## Audio DSP Library for Rust

`adsp` is a high-level audio DSP library with a focus on usability.

**This project is under construction**! It is already useful for experimentation.
However, some breakage can be expected as we continue to experiment with best practices.


## Principia

Most filters are expected to be generic over their inner processing type.

The ubiquitous glue type `f48` that connects audio components and populates audio buffers
is chosen globally, as a configuration option.
It defaults to `f32`. The other choice is `f64` (feature `double_precision`).

There are two parallel component systems: the static `AudioComponent` and the dynamic `AudioUnit`.
Both systems operate on audio signals synchronously as an infinite stream.

---

| Trait            | Dispatch             | Allocation      | Operation        | Connectivity |
| ---------------- | -------------------- | --------------- | ---------------- | ------------ |
| `AudioComponent` | static, inlined      | mostly stack    | sample by sample | input and output arity fixed at compile time |
| `AudioUnit`      | dynamic, object safe | heap            | block by block   | input and output arity fixed after construction |

---

The `AudioUnit` system exists in embryonic form so far. The lower level `AudioComponent`s can be lifted
to block processing mode with the object safe `AudioUnit` interface via the `AcUnit<A: AudioComponent>` wrapper.
Block processing aims to maximize efficiency in dynamic situations.

`AudioComponent`s can be stack allocated for the most part.
Some components may use the heap for audio buffers and the like.

Of the signals flowing in graphs, some contain audio while others are controls of different kinds.

With control signals and parameters in general, we prefer to use natural units like Hz and seconds.
It is useful to keep parameters independent of the sample rate, which we can then adjust as we like.

In both systems, a component `A` can be reinitialized with a new sample rate: `A.reset(Some(sr))`.


## Audio Processing Environment

The prelude defines a convenient combinator environment for audio processing.
It operates on `AudioComponent`s via the wrapper type `Ac<A: AudioComponent>`.

In the environment, the default form for components aims to ensure there is enough precision available.
Therefore, many employ double precision internally.


## Operators

Custom operators are available for combining audio components inline.
In order of precedence, from highest to lowest:

---

| Expression     | Meaning                       | Inputs  | Outputs | Notes                                       |
| -------------- | ----------------------------- |:-------:|:-------:| ------------------------------------------- |
| `-A`           | negate `A`                    | `a`     | `a`     | Negates any number of outputs, even zero. |
| `A * B`        | multiply `A` with `B`         | `a + b` | `a = b` | Aka amplification, or ring modulation when both are audio signals. Number of outputs in `A` and `B` must match. |
| `A`&#160;`*`&#160;`constant` | multiply `A`    | `a`     | `a`     | Broadcasts constant. Same applies to `constant * A`. |
| `A / B`        | cascade `A` and `B` in series | `a = b`  | `b`     | Pipes `A` to `B`, supplying missing `B` inputs from matching `A` inputs. Number of inputs in `A` and `B` must match. |
| `A + B`        | sum `A` and `B`               | `a + b` | `a = b` | Aka mixing. Number of outputs in `A` and `B` must match. |
| `A`&#160;`+`&#160;`constant` | add to `A`      | `a`     | `a`     | Broadcasts constant. Same applies to `constant + A`. |
| `A - B`        | difference of `A` and `B`     | `a + b` | `a = b` | Number of outputs in `A` and `B` must match. |
| `A`&#160;`-`&#160;`constant` | subtract from `A`             | `a`     | `a`     | Broadcasts constant. Same applies to `constant - A`. |
| `A >> B`       | pipe `A` to `B`               | `a`     | `b`     | Aka chaining. Number of outputs in `A` must match number of inputs in `B`. |
| `A & B`        | branch input to `A` and `B` in parallel | `a = b` | `a + b` | Number of inputs in `A` and `B` must match. |
| `A | B`        | stack `A` and `B` in parallel | `a + b` | `a + b` | - |

---

In the table, `constant` denotes an `f48` value.

All operators are associative, except the left associative `-` and `/`.

Arithmetic operators are applied to outputs channel-wise.
Arithmetic between two components never broadcasts channels.

Direct arithmetic with `f48` values, however, broadcasts to an arbitrary number of channels.
The negation operator broadcasts also: `-A` is equivalent with `(0.0 - A)`.

For example, `A * constant(2.0)` and `A >> mul(2.0)` are equivalent and expect `A` to have one output.
On the other hand, `A * 2.0` works with any `A`, even *sinks*.

Sinks are components with no outputs. Direct arithmetic on a sink translates to a no-op.
In the prelude, `sink()` returns a mono sink.

Of special interest among operators are the four custom combinators:
*cascade* ( `/` ), *pipe* ( `>>` ), *branch* ( `&` ) and *stack* ( `|` ).
Each come with their own connectivity rules.

### Cascade

The idea of the cascade is of a processing chain where some channels are threaded through and some are bypassed.
Signals that are threaded throughout - typically audio - are placed in the first channels.

The number of reused inputs depends on the number of preceding outputs.
The number of inputs remains the same throughout a cascade.

Due to associativity, a chain of cascade ( `/` ) operators defines a *far* cascade.
In a far cascade, all bypasses are sourced from the leftmost input.

For instance, in `A / B / C / D`, missing inputs to `C` and `D` are sourced from inputs to `A`, not `B` or `C`.

To get a *close* cascade, write it right associatively: `A / (B / (C / D))`.
In a close cascade, bypasses are sourced locally, resulting in *iterative* modulation.

Cascading is equivalent to piping when no bypasses are needed.
Chains can be notated as modulator-filter pairs; for example,
`resonator() / mul((1.0, 1.0, 2.0)) / resonator() / mul((1.0, 1.0, 4.0)) / resonator()`.

In the preceding cascade, bandwidth is doubled and quadrupled for the second and third resonator stages, respectively,
while audio in the first channel is threaded through.

If we convert it to right associative form, then modulation becomes left-to-right iterative:
both modulator expressions are now applied to the third stage, doubling its bandwidth, while the second
stage remains the same.

### Branch

Where the arithmetic operators are reducing in nature, the branch ( `&` ) operator splits a signal.

A nice mnemonic for reading the branch operator is to think of it as sending a signal to a conjunction of components:
`A >> (B & C & D)` is a triple branch that sends from `A` the same output to `B` *and* `C` *and* `D`.

### Expressions Are Graphs

The expression `A >> (B & C & D)` defines a signal processing graph.
It has whatever inputs `A` has, and outputs everything from `B` and `C` and `D` in parallel.

The whole structure is packed, monomorphized and inlined with the constituent components consumed.
If you want to reuse components, define them as functions or clone them. See the prelude for examples of the former.

Connectivity is checked during compilation.
Mismatched connectivity will result in a compilation error complaining about mismatched
[`typenum`](https://crates.io/crates/typenum) [types](https://docs.rs/typenum/1.12.0/typenum/uint/struct.UInt.html).
The arrays `Frame<Size>` that connect components come from the
[`numeric-array` crate](https://crates.io/crates/numeric-array).


## Free Functions

These free functions are available in the environment.

---

| Function               | Inputs | Outputs  | Explanation                                    |
| ---------------------- |:------:|:--------:| ---------------------------------------------- |
| `add(x)`               |    x   |    x     | Adds constant `x` to signal. |
| `constant(x)`          |    0   |    x     | Outputs constant `x`. Synonymous with `dc(x)`. |
| `dc(x)`                |    0   |    x     | Outputs constant `x`. Synonymous with `constant(x)`. |
| `envelope(Fn(f48)`&#160;`->`&#160;`f48)` | 0  |    1     | Time-varying control, e.g., `|t| exp(-t)`. Synonymous with `lfo(f)`. |
| `lfo(Fn(f48)`&#160;`->`&#160;`f48)`  |    0   |    1     | Time-varying control, e.g., `|t| exp(-t)`. Synonymous with `envelope(f)`. |
| `lowpass()`            | 2 (audio, cutoff) | 1 | Butterworth lowpass filter (2nd order). |
| `lowpass_hz(c)`        |    1   |    1     | Butterworth lowpass filter (2nd order) with fixed cutoff frequency `c` Hz. |
| `mul(x)`               |    x   |    x     | Multiplies signal with constant `x`. |
| `noise()`              |    0   |    1     | White noise source. |
| `pass()`               |    1   |    1     | Mono pass-through. |
| `resonator()`          | 3 (audio, center, bandwidth) | 1 | Constant-gain bandpass resonator (2nd order). |
| `sine()`               | 1 (pitch) | 1     | Sine oscillator. |
| `sine_hz(f)`           |    0   |    1     | Sine oscillator at fixed frequency `f` Hz. |
| `sink()`               |    1   |    0     | Consumes a channel. |
| `zero()`               |    0   |    1     | A zero signal. |

---

## Examples

For the practice of *graph fu*, some examples of graph expressions.

---

| Expression                               | Inputs | Outputs | Meaning                                       |
| ---------------------------------------- |:------:|:-------:| --------------------------------------------- |
| `pass() & pass()`                        |   1    |    2    | mono-to-stereo splitter                       |
| `pass() & pass() & pass()`               |   1    |    3    | mono-to-trio splitter                         |
| `sink() | pass()`                        |   2    |    1    | extract right channel                         |
| `pass() | sink()`                        |   2    |    1    | extract left channel                          |
| `sink() | zero() | pass()`               |   2    |    2    | replace left channel with silence             |
| `pass() | sink() | zero()`               |   2    |    2    | replace right channel with silence            |
| `lowpass() / lowpass() / lowpass()`      |   2    |    1    | triple lowpass filter in series (6th order)   |
| `resonator() / resonator()`              |   3    |    1    | double resonator in series (4th order)        |
| `sine_hz(f) * f * m + f >> sine()`       |   0    |    1    | PM (phase modulation) oscillator at `f` Hz with modulation index `m` |
| `(pass() & mul(2.0)) >> sine() + sine()` |   1    |    1    | frequency doubled dual sine oscillator        |
| `envelope(|t| exp(-t)) * noise()`        |   0    |    1    | exponentially decaying white noise            |

---

Note that besides associativity, `sink() | zero()` and `zero() | sink()` are equivalent -
both replace a signal with zeros.
This is because `sink()` only adds an input, while `zero()` only adds an output.

Native operator precedences are well suited for audio work, except for the sky high `/`.

```rust

use adsp::prelude::*;

// Use the ">>" operator to chain components.
// Lowpass filter some white noise with a cutoff of 400 Hz.
let a = white() >> lowpass_hz(400.0);
assert!(a.inputs() == 0 && a.outputs() == 1);

// Use the "|" operator to stack components in parallel.
// Here we send three inputs to a constant-gain resonator.
// The output is bandlimited noise filtered with a center frequency of 400 Hz and a bandwidth of 200 Hz.
let b = (white() | dc(400.0) | dc(200.0)) >> resonator();
assert!(b.inputs() == 0 && b.outputs() == 1);

// Use pass() to pass through a channel. Here is the filter from the above generator.
// We also combine the constants into one stereo constant.
let c = (pass() | dc((400.0, 200.0))) >> resonator();
assert!(c.inputs() == 1 && c.outputs() == 1);

// New components can be defined with the following return type.
// Declaring the full arity in the signature enables use of the component in further combinations,
// as does the full type name. Signatures with generic number of channels can be challenging to write.
// Here we define a mono-to-quad splitter.
// Use the "&" operator for branching.
// Same input is sent to both components (read A & B as "send input to A AND B").
pub fn split_quad() -> Ac<impl AudioComponent<Inputs = U1, Outputs = U4>> {
  pass() & pass() & pass() & pass()
}

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
let e: Ac<impl AudioComponent<Inputs = U0, Outputs = U1>> = white() * envelope(|t| exp(-t));

```


## License

MIT or Apache-2.0.


## Future

- Develop an equivalent combinator environment for `AudioUnit`s so the user can
  decide when and where to lift components to block processing. `AudioUnit` combinators would operate
  at runtime and thus connectivity checks would be deferred to runtime, too.
  Ideally, we would like SIMD optimization to happen automatically during lifting, but this is a lofty goal.
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
- Add support for `AudioComponent` graph parameters using a broadcast/gather mechanism.
- Add support for some fixed-point type for `f48`.
- Examine and optimize performance.
- Add tests.
- Add more stuff.
