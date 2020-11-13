use num_complex::Complex64;

/// Lti provides extra information on linear time-invariant (LTI) systems,
/// which are characterized by their (complex) frequency response.
/// This characterization applies to linear FIR and IIR filters with fixed parameters.
pub trait Lti {

    /// Evaluates frequency response at frequency omega.
    /// Omega is expressed as a fraction of the sample rate (0 <= omega <= 1/2).
    fn response(&self, omega: f64) -> Complex64;

    /// Evaluates magnitude response at frequency omega.
    /// Omega is expressed as a fraction of the sample rate (0 <= omega <= 1/2).
    /// Magnitude response is the amplification factor of a pure frequency component.
    fn magnitude(&self, omega: f64) -> f64 {
        assert!(omega >= 0.0 && omega <= 0.5);
        self.response(omega).norm()
    }

    /// Evaluates phase response at frequency omega.
    /// Omega is expressed as a fraction of the sample rate (0 <= omega <= 1/2).
    fn phase(&self, omega: f64) -> f64 {
        assert!(omega >= 0.0 && omega <= 0.5);
        self.response(omega).arg()
    }
}
