use super::math::*;
use num_complex::Complex64;
use tinyvec::TinyVec;

/// Contents of a mono signal. Used in latency and frequency response analysis.
#[derive(Clone, Copy)]
pub enum Signal {
    /// Signal with unknown properties.
    Unknown,
    /// Constant signal with value.
    Value(f64),
    /// Signal that is connected to input(s) with latency in samples.
    Latency(f64),
    /// Signal that is connected to input(s) with complex frequency response and latency in samples.
    Response(Complex64, f64),
}

impl Default for Signal {
    fn default() -> Signal {
        Signal::Unknown
    }
}

impl Signal {
    /// Filter signal using the transfer function `filter` while adding extra `latency`.
    /// Latency is measured in samples.
    pub fn filter(&self, latency: f64, filter: impl Fn(Complex64) -> Complex64) -> Signal {
        match self {
            Signal::Latency(l) => Signal::Latency(l + latency),
            Signal::Response(response, l) => Signal::Response(filter(*response), l + latency),
            _ => Signal::Unknown,
        }
    }

    /// Apply nonlinear processing to signal with extra `latency` in samples.
    /// Nonlinear processing erases constant values and frequency responses but maintains latency.
    /// Latency is measured in samples.
    pub fn distort(&self, latency: f64) -> Signal {
        match self {
            Signal::Latency(l) => Signal::Latency(l + latency),
            Signal::Response(_, l) => Signal::Latency(l + latency),
            _ => Signal::Unknown,
        }
    }

    /// Delay signal by `latency` samples.
    pub fn delay(&self, latency: f64) -> Signal {
        match self {
            Signal::Latency(l) => Signal::Latency(l + latency),
            Signal::Response(response, l) => Signal::Response(*response, l + latency),
            x => *x,
        }
    }

    /// Scale signal by `factor`.
    pub fn scale(&self, factor: f64) -> Signal {
        match self {
            Signal::Value(x) => Signal::Value(x * factor),
            Signal::Response(response, latency) => Signal::Response(response * factor, *latency),
            x => *x,
        }
    }

    /// Combine signals nonlinearly with extra `latency`.
    /// Nonlinear processing erases constant values and frequency responses but maintains latency.
    /// Latency is measured in samples.
    pub fn combine_nonlinear(&self, other: Signal, latency: f64) -> Signal {
        match (self.distort(0.0), other.distort(0.0)) {
            (Signal::Latency(lx), Signal::Latency(ly)) => Signal::Latency(min(lx, ly) + latency),
            (Signal::Latency(lx), _) => Signal::Latency(lx + latency),
            (_, Signal::Latency(ly)) => Signal::Latency(ly + latency),
            _ => Signal::Unknown,
        }
    }

    /// Combine signals linearly with extra `latency` and `value` and `response` processing as functions.
    /// Constant signals are considered as zero responses.
    /// Latency is measured in samples.
    pub fn combine_linear(
        &self,
        other: Signal,
        latency: f64,
        value: impl Fn(f64, f64) -> f64,
        response: impl Fn(Complex64, Complex64) -> Complex64,
    ) -> Signal {
        match (*self, other) {
            (Signal::Value(vx), Signal::Value(vy)) => Signal::Value(value(vx, vy)),
            (Signal::Latency(lx), Signal::Latency(ly)) => Signal::Latency(min(lx, ly) + latency),
            (Signal::Response(rx, lx), Signal::Response(ry, ly)) => {
                Signal::Response(response(rx, ry), min(lx, ly) + latency)
            }
            (Signal::Response(rx, lx), Signal::Value(_)) => {
                Signal::Response(response(rx, Complex64::new(0.0, 0.0)), lx + latency)
            }
            (Signal::Value(_), Signal::Response(ry, ly)) => {
                Signal::Response(response(Complex64::new(0.0, 0.0), ry), ly + latency)
            }
            (Signal::Response(_, lx), Signal::Latency(ly)) => {
                Signal::Latency(min(lx, ly) + latency)
            }
            (Signal::Latency(lx), Signal::Response(_, ly)) => {
                Signal::Latency(min(lx, ly) + latency)
            }
            (Signal::Latency(lx), _) => Signal::Latency(lx + latency),
            (Signal::Response(_, lx), _) => Signal::Latency(lx + latency),
            (_, Signal::Latency(ly)) => Signal::Latency(ly + latency),
            (_, Signal::Response(_, ly)) => Signal::Latency(ly + latency),
            _ => Signal::Unknown,
        }
    }
}

/// Some components support different modes for signal flow analysis.
// TODO. This is too complex for the user. Remove this and only keep the Constant case?
#[derive(Copy, Clone)]
pub enum AnalysisMode {
    /// The component presents itself as a constant.
    Constant,
    /// The component presents itself as a bypass.
    Bypass,
    /// The component is a filter.
    Filter,
}
impl Default for AnalysisMode {
    fn default() -> Self {
        AnalysisMode::Filter
    }
}

/// Frame of input or output signals.
pub type SignalFrame = TinyVec<[Signal; 32]>;

/// Create a new signal frame with all channels marked unknown.
pub fn new_signal_frame(size: usize) -> SignalFrame {
    let mut frame = TinyVec::with_capacity(size);
    frame.resize(size, Signal::Unknown);
    frame
}

/// Create a new signal frame by copying from `source` `n` items starting from index `i`.
pub fn copy_signal_frame(source: &SignalFrame, i: usize, n: usize) -> SignalFrame {
    let mut frame = new_signal_frame(n);
    frame[0..n].copy_from_slice(&source[i..i + n]);
    frame
}

/// Signal routing information. We use this to avoid "impl AudioNode"
/// return types in preludes resulting from closures.
pub enum Routing {
    /// Conservative routing: every input influences every output nonlinearly.
    Arbitrary,
    /// Split or multisplit semantics.
    Split,
    /// Join or multijoin semantics.
    Join,
}

impl Routing {
    /// Routes signals from input to output.
    pub fn propagate(&self, input: &SignalFrame, outputs: usize) -> SignalFrame {
        let mut output = new_signal_frame(outputs);
        if input.is_empty() {
            return output;
        }
        match self {
            Routing::Arbitrary => {
                let mut combo = input[0].distort(0.0);
                for i in 1..input.len() {
                    combo = combo.combine_nonlinear(input[i], 0.0);
                }
                output.fill(combo);
            }
            Routing::Split => {
                for i in 0..outputs {
                    output[i] = input[i % input.len()];
                }
            }
            Routing::Join => {
                // How many inputs for each output.
                let bundle = input.len() / output.len();
                for i in 0..outputs {
                    let mut combo = input[i];
                    for j in 1..bundle {
                        combo = combo.combine_linear(
                            input[i + j * outputs],
                            0.0,
                            |x, y| x + y,
                            |x, y| x + y,
                        );
                    }
                    // Normalize.
                    output[i] = combo.scale(output.len() as f64 / input.len() as f64);
                }
            }
        }
        output
    }
}
