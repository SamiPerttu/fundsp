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
    #[inline]
    pub fn filter(&self, latency: f64, filter: impl Fn(Complex64) -> Complex64) -> Signal {
        match self {
            Signal::Latency(l) => Signal::Latency(l + latency),
            Signal::Response(response, l) => Signal::Response(filter(*response), l + latency),
            _ => Signal::Unknown,
        }
    }

    /// Apply nonlinear processing to signal with extra `latency`.
    /// Nonlinear processing erases constant values and frequency responses but maintains latency.
    /// Latency is measured in samples.
    #[inline]
    pub fn distort(&self, latency: f64) -> Signal {
        match self {
            Signal::Latency(l) => Signal::Latency(l + latency),
            Signal::Response(_, l) => Signal::Latency(l + latency),
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

/// Combine signals nonlinearly with extra `latency`.
/// Nonlinear processing erases constant values and frequency responses but maintains latency.
/// Latency is measured in samples.
pub fn combine_nonlinear(x: Signal, y: Signal, latency: f64) -> Signal {
    match (x.distort(0.0), y.distort(0.0)) {
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
    x: Signal,
    y: Signal,
    latency: f64,
    value: impl Fn(f64, f64) -> f64,
    response: impl Fn(Complex64, Complex64) -> Complex64,
) -> Signal {
    match (x, y) {
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
        (Signal::Response(_, lx), Signal::Latency(ly)) => Signal::Latency(min(lx, ly) + latency),
        (Signal::Latency(lx), Signal::Response(_, ly)) => Signal::Latency(min(lx, ly) + latency),
        (Signal::Latency(lx), _) => Signal::Latency(lx + latency),
        (Signal::Response(_, lx), _) => Signal::Latency(lx + latency),
        (_, Signal::Latency(ly)) => Signal::Latency(ly + latency),
        (_, Signal::Response(_, ly)) => Signal::Latency(ly + latency),
        _ => Signal::Unknown,
    }
}
