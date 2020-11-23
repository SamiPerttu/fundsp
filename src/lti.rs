use num_complex::Complex64;

/// Lti provides extra information on linear time-invariant (LTI) systems,
/// which are characterized by their (complex) frequency response.
/// This characterization applies to linear FIR and IIR filters with fixed parameters.
pub trait Lti {
    /// Evaluate frequency response at frequency `omega`.
    /// `omega` is expressed as a fraction of the sample rate (0 <= `omega` <= 1/2).
    fn response(&self, omega: f64) -> Complex64;

    /// Evaluate magnitude response at frequency `omega`.
    /// `omega` is expressed as a fraction of the sample rate (0 <= `omega` <= 1/2).
    /// Magnitude response is the gain, or amplification factor, of a pure frequency component.
    fn gain(&self, omega: f64) -> f64 {
        assert!(omega >= 0.0 && omega <= 0.5);
        self.response(omega).norm()
    }

    /// Evaluate phase response at frequency `omega`.
    /// `omega` is expressed as a fraction of the sample rate (0 <= `omega` <= 1/2).
    fn phase(&self, omega: f64) -> f64 {
        assert!(omega >= 0.0 && omega <= 0.5);
        self.response(omega).arg()
    }
}
