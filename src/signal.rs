use super::math::*;
use num_complex::Complex64;

/// Contents of a mono signal. Used in latency and frequency response analysis.
#[derive(Clone, Copy)]
pub enum Signal {
    /// Signal with unknown properties.
    Unknown,
    /// Constant signal.
    Value(f64),
    /// Input connected signal with latency in samples.
    Latency(f64),
    /// Input connected signal with complex frequency response and latency in samples.
    Response(Complex64, f64),
}

/// Frame of input or output signals.
/// To avoid generics, we use fixed size arrays here.
pub type SignalFrame = [Signal; 128];

/// Create a new signal frame with all outputs marked unknown.
pub fn new_signal_frame() -> SignalFrame {
    [Signal::Unknown; 128]
}

/// Create a new signal frame by copying from `source` `n` items starting from index `i`.
pub fn copy_signal_frame(source: &SignalFrame, i: usize, n: usize) -> SignalFrame {
    let mut frame = new_signal_frame();
    frame[0..n].copy_from_slice(&source[i..i + n]);
    frame
}

/// Filter signal using the transfer function `filter` while adding extra `latency`.
pub fn filter_signal(
    signal: Signal,
    latency: f64,
    filter: impl Fn(Complex64) -> Complex64,
) -> Signal {
    match signal {
        Signal::Latency(l) => Signal::Latency(l + latency),
        Signal::Response(response, l) => Signal::Response(filter(response), l + latency),
        _ => Signal::Unknown,
    }
}

/// Combine signals with given extra latency in samples and value and response processing as functions.
/// Constant signals and responses are not combined.
pub fn combine_signals(
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
        (Signal::Response(_, lx), Signal::Latency(ly)) => Signal::Latency(min(lx, ly) + latency),
        (Signal::Latency(lx), Signal::Response(_, ly)) => Signal::Latency(min(lx, ly) + latency),
        (Signal::Latency(lx), _) => Signal::Latency(lx + latency),
        (Signal::Response(_, lx), _) => Signal::Latency(lx + latency),
        (_, Signal::Latency(ly)) => Signal::Latency(ly + latency),
        (_, Signal::Response(_, ly)) => Signal::Latency(ly + latency),
        _ => Signal::Unknown,
    }
}
