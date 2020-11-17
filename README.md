# fundsp

## Audio DSP Library for Rust

`fundsp` is a high-level audio DSP (digital dignal processing) library with a focus on usability.

It features a powerful inline graph notation that
empowers users to accomplish diverse audio processing tasks with ease and elegance.

`fundsp` comes with a function combinator environment containing
a suite of audio components, math and utility functions and procedural generation tools.

*This project is under construction*! It is already useful for experimentation.
However, some standard components are missing and breakage can be expected as we continue to experiment with best practices.


## Principia

Audio buffers and connections between audio components use single precision floating point (`f32`).

Components overall are mixed precision. Many filters are generic over their inner processing type.

### Component Systems

There are two parallel component systems: the static `AudioComponent` and the dynamic `AudioUnit`.

Both systems operate on audio signals synchronously as an infinite stream.

---

| Trait            | Dispatch             | Allocation      | Operation        | Connectivity |
| ---------------- | -------------------- | --------------- | ---------------- | ------------ |
| `AudioComponent` | static, inlined      | mostly stack    | sample by sample | input and output arity fixed at compile time |
| `AudioUnit`      | dynamic, object safe | heap            | block by block   | input and output arity fixed after construction |

---

The lower level `AudioComponent`s can be lifted
to block processing mode with the object safe `AudioUnit` interface via the `AcUnit<X: AudioComponent>` wrapper.
Block processing aims to maximize efficiency in dynamic situations.

`AudioComponent`s can be stack allocated for the most part.
Some components may use the heap for audio buffers and the like.

### Sample Rate Independence

Of the signals flowing in graphs, some contain audio while others are controls of different kinds.

With control signals and parameters in general, we prefer to use natural units like Hz and seconds.
It is useful to keep parameters independent of the sample rate, which we can then adjust as we like.

In both systems, a component `A` can be reinitialized with a new sample rate: `A.reset(Some(sr))`.


## Audio Processing Environment

The `fundsp` prelude defines a convenient combinator environment for audio processing.
It operates on `AudioComponent`s via the wrapper type `Ac<X: AudioComponent>`.

In the environment, the default form for components aims to ensure there is enough precision available.
Therefore, many employ double precision internally.

In the environment, generators are deterministic pseudorandom phase by default (to be implemented).


## Operators

Custom operators are available for combining audio components inline.
In order of precedence, from highest to lowest:

---

| Expression     | Meaning                       | Inputs  | Outputs | Notes                                       |
| -------------- | ----------------------------- |:-------:|:-------:| ------------------------------------------- |
| `-A`           | negate `A`                    | `a`     | `a`     | Negates any number of outputs, even zero.   |
| `!A`           | monitor `A`                   | `a`     | `a`     | Mixes inputs of `A` to outputs of `A`.      |
| `A * B`        | multiply `A` with `B`         | `a`&#160;`+`&#160;`b` | `a`&#160;`=`&#160;`b` | Aka amplification, or ring modulation when both are audio signals. Number of outputs in `A` and `B` must match. |
| `A`&#160;`*`&#160;`constant` | multiply `A`    | `a`     | `a`     | Broadcasts constant. Same applies to `constant * A`. |
| `A / B`        | cascade `A` and `B` in series | `a`&#160;`=`&#160;`b` | `b`     | Pipes `A` to `B`, supplying missing `B` inputs from matching `A` inputs. Number of inputs in `A` and `B` must match. |
| `A + B`        | sum `A` and `B`               | `a`&#160;`+`&#160;`b` | `a`&#160;`=`&#160;`b` | Aka mixing. Number of outputs in `A` and `B` must match. |
| `A`&#160;`+`&#160;`constant` | add to `A`      | `a`     | `a`     | Broadcasts constant. Same applies to `constant + A`. |
| `A - B`        | difference of `A` and `B`     | `a`&#160;`+`&#160;`b` | `a`&#160;`=`&#160;`b` | Number of outputs in `A` and `B` must match. |
| `A`&#160;`-`&#160;`constant` | subtract from `A` | `a`   | `a`     | Broadcasts constant. Same applies to `constant - A`. |
| `A >> B`       | pipe `A` to `B`               | `a`     | `b`     | Aka chaining. Number of outputs in `A` must match number of inputs in `B`. |
| `A & B`        | bus `A` and `B` together      | `a`&#160;`=`&#160;`b` | `a`&#160;`=`&#160;`b` | `A` and `B` must have identical connectivity. |
| `A ^ B`        | branch input to `A` and `B` in parallel | `a`&#160;`=`&#160;`b` | `a`&#160;`+`&#160;`b` | Number of inputs in `A` and `B` must match. |
| `A \| B`       | stack `A` and `B` in parallel | `a`&#160;`+`&#160;`b` | `a`&#160;`+`&#160;`b` | Concatenates `A` and `B` inputs and outputs. |

---

In the table, `constant` denotes an `f32` value.

All operators are associative, except the left associative `-` and `/`.


## Operators Diagram

![](operators.png "fundsp Graph Operators")

### Broadcasting

Arithmetic operators are applied to outputs channel-wise.
Arithmetic between two components never broadcasts channels.

Direct arithmetic with `f32` values, however, broadcasts to an arbitrary number of channels.

The negation operator broadcasts also: `-A` is equivalent with `(0.0 - A)`.

For example, `A * constant(2.0)` and `A >> mul(2.0)` are equivalent and expect `A` to have one output.
On the other hand, `A * 2.0` works with any `A`, even *sinks*.

The other unary operator, *monitor* ( `!` ), can also be applied to any component.
`!A` adds inputs of `A` to its outputs, ignoring any missing or extra channels.
`-!-A` subtracts inputs from outputs instead of adding them.

### Generators, Filters and Sinks

Components can be broadly classified into generators, filters and sinks.
*Generators* have only outputs, while *filters* have both inputs and outputs.

Sinks are components with no outputs. Direct arithmetic on a sink translates to a no-op.
In the prelude, `sink()` returns a mono sink.

### Graph Combinators

Of special interest among operators are the five custom combinators:
*cascade* ( `/` ), *pipe* ( `>>` ), *bus* ( `&` ), *branch* ( `^` ),  and *stack* ( `|` ).

Cascade and pipe are serial operators where components appear in *processing* order. Branch, stack, and
arithmetic operators are parallel operators where components appear in *channel* order.

Bus is a commutative operator where components may appear in any order.
The other operators are not commutative in general.

Each come with their own connectivity rules.
Piping, busing, branching and stacking are all fully associative operations,
while cascading is left associative.

#### Cascade

The idea of the cascade is of a processing chain where some channels are threaded through and some are bypassed.
Signals that are threaded throughout - typically audio - are placed in the first channels.

Each component in a cascade has the same number of inputs.
The number of reused inputs depends on the number of preceding outputs.

Due to associativity, a chain of cascade ( `/` ) operators defines a *far* cascade.
In a far cascade, all bypasses are sourced from the leftmost input.

For instance, in `A / B / C / D`, missing inputs to `C` and `D` are sourced from inputs to `A`, not `B` or `C`.

To get a *close* cascade, write it right associatively: `A / (B / (C / D))`.
In a close cascade, bypasses are sourced locally, resulting in *iterative* modulation.

In the default form, cascading chains similar filters.
For example, `lowpass() / lowpass()` sources the cutoff frequency for both filters
from the leftmost input while threading the audio through.

Cascading is equivalent to piping when no bypasses are needed.
Chains can be notated as modulator-filter pairs; for example,
`resonator() / mul((1.0, 1.0, 2.0)) / resonator() / mul((1.0, 1.0, 4.0)) / resonator()`.

In the preceding cascade, bandwidth is doubled and quadrupled for the second and third resonator stages, respectively,
while audio in the first channel is threaded through.

If we convert it to right associative form, then modulation becomes left-to-right iterative:
both modulator expressions are now applied to the third resonator stage, doubling its bandwidth, while the second
stage remains the same.

#### Pipe

The pipe ( `>>` ) operator builds traditional processing chains akin to composition of functions.
In `A >> B`, each output of `A` is piped to a matching input of `B`, so
the output arity of `A` must match the input arity of `B`.

It is possible to pipe a sink to a generator. This is similar to stacking.
Processing works as normal and the sink processes its inputs before the generator is run.

#### Branch

Where the arithmetic operators are reducing in nature,
the branch ( `^` ) operator splits a signal into parallel branches.

In `A ^ B`, both components receive the same input but their outputs are disjoint.
Because the components receive the same input, the number of inputs in `A` and `B` must match.
In `A ^ B`, the outputs of `A` appear first, followed with outputs of `B`.

Branching is useful for building *banks* of components such as filters.

#### Bus

The bus ( `&` ) operator can be thought of as an inline audio bus with a fixed set of input and output channels.
It builds signal buses from components with identical connectivity.

In `A & B`, the same input is sent to both `A` and `B`, and their outputs are mixed together.
Components in a bus may appear in any order.

The bus is especially useful because it does not alter connectivity:
we can always bus together any set of matching components
without touching the rest of the expression.

Both `A + B` and `A & B` are mixing operators. The difference between the two is that `A + B` is *reducing*:
`A` and `B` have their own, disjoint inputs.
In `A & B`, both components source from the same inputs, and the number of inputs must match.


#### Stack

The stack ( `|` ) operator builds composite components.
It can be applied to any two components.

As a graph operator, the stack corresponds to the *disjoint union*. In `A | B`, the inputs and outputs of `A` and `B`
are disjoint and they are processed independently, in parallel.

In stacks, components are written in channel order.
In `A | B | C`, channels of `A` come first, followed by channels of `B`, then `C`.


## Expressions Are Graphs

The expression `A >> (B ^ C ^ D)` defines a signal processing graph.
It has whatever inputs `A` has, and outputs everything from `B` and `C` and `D` in parallel.

The whole structure is packed, monomorphized and inlined with the constituent components consumed.
If you want to reuse components, define them as functions or clone them. See the prelude for examples of the former.

Connectivity is checked during compilation.
Mismatched connectivity will result in a compilation error complaining about mismatched
[`typenum`](https://crates.io/crates/typenum) [types](https://docs.rs/typenum/1.12.0/typenum/uint/struct.UInt.html).
The arrays `Frame<Size>` that connect components come from the
[`numeric-array` crate](https://crates.io/crates/numeric-array).

### Computational Structure

Graph combinators consume their arguments.
This prevents cycles and imposes an overall tree shape on the resulting computation graph.

Implicit cycle prevention means that the built structures are always computationally efficient
in the dataflow sense. All reuse of computed data takes place locally, inside combinators and components.

There are three main ways to structure the reuse of signals in `fundsp` graph notation:
*branching*, *busing* and *cascading*. All three are exposed as fundamental operators,
guiding toward efficient structuring of computation.
Dataflow concerns are thus explicated in the graph notation itself.


## Free Functions

These free functions are available in the environment.

### Component Functions

---

| Function               | Inputs | Outputs  | Explanation                                    |
| ---------------------- |:------:|:--------:| ---------------------------------------------- |
| `add(x)`               |    x   |    x     | Adds constant `x` to signal. |
| `constant(x)`          |    -   |    x     | Constant signal `x`. Synonymous with `dc`. |
| `dc(x)`                |    -   |    x     | Constant signal `x`. Synonymous with `constant`. |
| `delay(t)`             |    1   |    1     | Fixed delay of t seconds. |
| `envelope(Fn(f32)`&#160;`->`&#160;`f32)` | - | 1 | Time-varying control, e.g., `envelope(\|t\| exp(-t))`. Synonymous with `lfo`. |
| `feedback(x)`          |    x   |    x     | Encloses feedback circuit x (with equal number of inputs and outputs). |
| `lfo(Fn(f32)`&#160;`->`&#160;`f32)` | - | 1 | Time-varying control, e.g., `lfo(\|t\| exp(-t))`. Synonymous with `envelope`. |
| `lowpass()`            | 2 (audio, cutoff) | 1 | Butterworth lowpass filter (2nd order). |
| `lowpass_hz(c)`        |    1   |    1     | Butterworth lowpass filter (2nd order) with fixed cutoff frequency `c` Hz. |
| `lowpole()`            | 2 (audio, cutoff) | 1 | 1-pole lowpass filter (1st order). |
| `lowpole_hz(c)`        |    1   |    1     | 1-pole lowpass filter (1st order) with fixed cutoff frequency `c` Hz. |
| `mls()`                |    -   |    1     | White MLS noise source. |
| `mls_bits(n)`          |    -   |    1     | White MLS noise source from n-bit MLS sequence. |
| `mul(x)`               |    x   |    x     | Multiplies signal with constant `x`. |
| `noise()`              |    -   |    1     | White noise source. Synonymous with `white`. |
| `pass()`               |    1   |    1     | Passes signal through. |
| `resonator()`          | 3 (audio, center, bandwidth) | 1 | Constant-gain bandpass resonator (2nd order). |
| `resonator_hz(c, bw)`  |    1   |    1     | Constant-gain bandpass resonator (2nd order) with fixed center frequency `c` Hz and bandwidth `bw` Hz. |
| `sine()`               | 1 (pitch) | 1     | Sine oscillator. |
| `sine_hz(f)`           |    -   |    1     | Sine oscillator at fixed frequency `f` Hz. |
| `sink()`               |    1   |    -     | Consumes a channel. |
| `sub(x)`               |    x   |    x     | Subtracts constant `x` from signal. |
| `tick()`               |    1   |    1     | Single sample delay. |
| `white()`              |    -   |    1     | White noise source. Synonymous with `noise`. |
| `zero()`               |    -   |    1     | Zero signal. |

---

### Math Functions

| Function               | Explanation                                    |
| ---------------------- | ---------------------------------------------- |
| `abs(x)`               | absolute value of `x` |
| `ceil(x)`              | ceiling function |
| `clamp(min, max, x)`   | clamps `x` between `min` and `max`. |
| `clamp01(x)`           | clamps `x` between 0 and 1. |
| `clamp11(x)`           | clamps `x` between -1 and 1. |
| `cos(x)`               | cos |
| `cos_bpm(f, t)`        | cosine that oscillates at `f` BPM at time `t` seconds 
| `cos_hz(f, t)`         | cosine that oscillates at `f` Hz at time `t` seconds |
| `db_gain(x)`           | converts `x` dB to amplitude (gain amount) |
| `delerp(x0, x1, x)`    | recovers linear interpolation amount `t` from interpolated value |
| `dexerp(x0, x1, x)`    | recovers exponential interpolation amount `t` from interpolated value |
| `dissonance(f0, f1)`   | dissonance amount in 0...1 between pure tones at `f0` and `f1` Hz |
| `dissonance_max(f)`    | maximally dissonant pure frequency above `f` Hz |
| `downarc(x)`           | downward arcing quarter circle easing curve (inverse of `uparc`) |
| `exp(x)`               | exp |
| `exq(x)`               | polynomial alternative to `exp` |
| `floor(x)`             | floor function |
| `lerp(x0, x1, t)`      | linear interpolation between `x0` and `x1` |
| `log(x)`               | natural logarithm |
| `logistic(x)`          | logistic function |
| `min(x, y)`            | minimum of `x` and `y` |
| `max(x, y)`            | maximum of `x` and `y` |
| `m_weight(f)`          | M-weighted noise response amplitude at `f` Hz. |
| `pow(x, y)`            | `x` raised to the power `y` |
| `round(x)`             | rounds `x` to nearest integer |
| `signum(x)`            | sign of `x` |
| `sin(x)`               | sin |
| `sin_bpm(f, t)`        | sine that oscillates at `f` BPM at time `t` seconds |
| `sin_hz(f, t)`         | sine that oscillates at `f` Hz at time `t` seconds |
| `smooth3(x)`           | smooth cubic easing polynomial |
| `smooth5(x)`           | smooth 5th degree easing polynomial (commonly used in computer graphics) |
| `smooth7(x)`           | smooth 7th degree easing polynomial |
| `smooth9(x)`           | smooth 9th degree easing polynomial |
| `softmix(x, y, bias)`  | weighted average of `x` and `y` according to `bias`: polynomial softmin when `bias` < 0, average when `bias` = 0, polynomial softmax when `bias` > 0. |
| `softsign(x)`          | softsign function, a polynomial alternative to `tanh` |
| `spline(x0, x1, x2, x3, t)` | Catmull-Rom cubic interpolation between `x1` and `x2`, taking `x0` and `x3` into account |
| `splinem(x0, x1, x2, x3, t)` | monotonic cubic interpolation between `x1` and `x2`, taking `x0` and `x3` into account |
| `sqrt(x)`              | square root of `x` |
| `tan(x)`               | tan |
| `tanh(x)`              | hyperbolic tangent |
| `uparc(x)`             | upward arcing quarter circle easing curve (inverse of `downarc`) |
| `xerp(x0, x1, t)`      | exponential interpolation between `x0` and `x1` (`x0`, `x1` > 0) |

---

## Examples

For the practice of *graph fu*, some examples of graph expressions.

---

| Expression                               | Inputs | Outputs | Meaning                                       |
| ---------------------------------------- |:------:|:-------:| --------------------------------------------- |
| `pass() ^ pass()`                        |   1    |    2    | mono-to-stereo splitter                       |
| `mul(0.5) + mul(0.5)`                    |   2    |    1    | stereo-to-mono mixdown (inverse of mono-to-stereo splitter) |
| `pass() ^ pass() ^ pass()`               |   1    |    3    | mono-to-trio splitter                         |
| `sink() \| zero()`                       |   1    |    1    | replace signal with silence                   |
| `mul(0.0)`                               |   1    |    1    | -..-                                          |
| `mul(db_gain(3.0))`                      |   1    |    1    | amplify signal by 3 dB                        |
| `sink() \| pass()`                       |   2    |    1    | extract right channel                         |
| `pass() \| sink()`                       |   2    |    1    | extract left channel                          |
| `sink() \| zero() \| pass()`             |   2    |    2    | replace left channel with silence             |
| `mul(0.0) \| pass()`                     |   2    |    2    | -..-                                          |
| `mul((0.0, 1.0))`                        |   2    |    2    | -..-                                          |
| `pass() \| sink() \| zero()`             |   2    |    2    | replace right channel with silence            |
| `pass() \| mul(0.0)`                     |   2    |    2    | -..-                                          |
| `mul((1.0, 0.0))`                        |   2    |    2    | -..-                                          |
| `lowpass() / lowpole()`                  |   2    |    1    | 2nd order and 1-pole lowpass filters in series (3rd order) |
| `lowpass() / lowpass() / lowpass()`      |   2    |    1    | triple lowpass filter in series (6th order)   |
| `resonator() / resonator()`              |   3    |    1    | double resonator in series (4th order)        |
| `sine_hz(f) * f * m + f >> sine()`       |   -    |    1    | PM (phase modulation) oscillator at `f` Hz with modulation index `m` |
| `sine() & mul(2.0) >> sine()`            |   1    |    1    | frequency doubled dual sine oscillator        |
| `envelope(\|t\| exp(-t)) * noise()`      |   -    |    1    | exponentially decaying white noise            |
| `!feedback(delay(1.0) * db_gain(-3.0))`  |   1    |    1    | 1 second feedback delay with 3 dB attenuation |

---

### Examples From The Prelude

Many functions in the prelude itself are defined as graph expressions.

---

| Function                                 | Inputs | Outputs | Definition                                     |
| ---------------------------------------- |:------:|:-------:| ---------------------------------------------- |
| `lowpass_hz(c)`                          |   1    |    1    | `(pass() \| constant(c)) >> lowpass()`         |
| `lowpole_hz(c)`                          |   1    |    1    | `(pass() \| constant(c)) >> lowpole()`         |
| `resonator_hz(c, bw)`                    |   1    |    1    | `(pass() \| constant((c, bw))) >> resonator()` |
| `sine_hz(f)`                             |   -    |    1    | `constant(f) >> sine()`                        |
| `zero()`                                 |   -    |    1    | `constant(0.0)`                                |

---

### Equivalent Expressions ###

There are usually many ways to express a particular graph. The following expression pairs are identical.

---

| Expression                                 | Is The Same As                  | Notes |
| ------------------------------------------ | ------------------------------- | ----- |
| `(pass() ^ mul(2.0)) >> sine() + sine()`   | `sine() & mul(2.0) >> sine()`   | Busing is often more convenient than explicit branching followed with summing. |
| `!-!sink()-42.0^sink()&-!!--!-sink()*3.14` | `sink()`                        | Branching, busing, monitoring and arithmetic on sinks are no-ops. |
| `constant(0.0) \| dc(1.0)`                 | `constant((0.0, 1.0))`          | Stacking concatenates channels. |
| `sink() \| zero()`                         | `zero() \| sink()`              | The order does not matter because `sink()` only adds an input, while `zero()` only adds an output. |
| `(lowpass() ^ (sink() \| pass())) >> lowpass()` | `lowpass() / lowpass()`    | Running a manual bypass. |

---

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
- Examine and optimize performance.
- Add tests.
- Add more stuff.
