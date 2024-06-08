//! Parameter smoothing filter.

use super::audionode::*;
use super::combinator::*;
use super::math::*;
use super::setting::*;
use super::signal::*;
use super::*;
use num_complex::Complex64;
use numeric_array::typenum::*;

/// Logistic sigmoid.
#[inline]
fn logistic<T: Real>(x: T) -> T {
    T::one() / (T::one() + exp(T::zero() - x))
}

fn halfway_coeff(samples: f64) -> f64 {
    // This approximation is accurate to 0.5% when 1 <= response samples <= 500_000.
    let r0 = log(max(1.0, samples)) - 0.861624594696583;
    let r1 = logistic(r0);
    let r2 = r1 * 1.13228543863477 - 0.1322853859;
    1.0 - min(0.9999999, r2)
}

/// Smoothing filter with adjustable halfway response time (in seconds).
/// Setting: response time.
/// - Input 0: input signal
/// - Output 0: smoothed signal
#[derive(Default, Clone)]
pub struct Follow<F: Real> {
    v3: F,
    v2: F,
    v1: F,
    coeff: F,
    // Filter coefficient that is 1 for the first sample and then assumes the value of coeff.
    coeff_now: F,
    /// Halfway response time.
    response_time: F,
    sample_rate: F,
}

impl<F: Real> Follow<F> {
    /// Create new smoothing filter. Response time (in seconds)
    /// is how long it takes for the follower to reach halfway to the new value.
    pub fn new(response_time: F) -> Self {
        let mut node = Self {
            response_time,
            ..Follow::default()
        };
        node.reset();
        node.set_sample_rate(DEFAULT_SR);
        node
    }

    /// Response time in seconds.
    pub fn response_time(&self) -> F {
        self.response_time
    }

    /// Set response time in seconds.
    pub fn set_response_time(&mut self, response_time: F) {
        self.response_time = response_time;
        self.coeff = F::from_f64(halfway_coeff((response_time * self.sample_rate).to_f64()));
        if self.coeff_now < F::one() {
            self.coeff_now = self.coeff;
        }
    }

    /// Current response.
    pub fn value(&self) -> F {
        self.v3
    }

    /// Jump to `x` immediately.
    pub fn set_value(&mut self, x: F) {
        self.v3 = x;
        self.v2 = x;
        self.v1 = x;
    }
}

impl<F: Real> AudioNode for Follow<F> {
    const ID: u64 = 24;
    type Inputs = U1;
    type Outputs = U1;

    fn reset(&mut self) {
        self.v3 = F::zero();
        self.v2 = F::zero();
        self.v1 = F::zero();
        self.coeff_now = F::one();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = convert(sample_rate);
        self.set_response_time(self.response_time);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        // Three 1-pole filters in series.
        let rcoeff = F::one() - self.coeff_now;
        self.v1 = self.coeff_now * convert(input[0]) + rcoeff * self.v1;
        self.v2 = self.coeff_now * self.v1 + rcoeff * self.v2;
        self.v3 = self.coeff_now * self.v2 + rcoeff * self.v3;
        self.coeff_now = self.coeff;
        [convert(self.v3)].into()
    }

    fn set(&mut self, setting: Setting) {
        if let Parameter::Time(time) = setting.parameter() {
            self.set_response_time(F::from_f32(*time));
        }
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = SignalFrame::new(self.outputs());
        output.set(
            0,
            input.at(0).filter(0.0, |r| {
                let c = 1.0 - self.coeff.to_f64();
                let f = frequency * f64::TAU / self.sample_rate.to_f64();
                let z1 = Complex64::from_polar(1.0, -f);
                let pole = (1.0 - c) / (1.0 - c * z1);
                r * pole * pole * pole
            }),
        );
        output
    }
}

/// Smoothing filter with adjustable edge response times for attack and release.
/// - Input 0: input signal
/// - Output 0: smoothed signal
#[derive(Default, Clone)]
pub struct AFollow<F: Real> {
    v3: F,
    v2: F,
    v1: F,
    acoeff: F,
    rcoeff: F,
    acoeff_now: F,
    rcoeff_now: F,
    /// Attack time.
    atime: F,
    /// Release time.
    rtime: F,
    sample_rate: F,
}

impl<F: Real> AFollow<F> {
    /// Create new smoothing filter.
    /// Response time is how long it takes for the follower to reach halfway to the new value.
    pub fn new(attack_time: F, release_time: F) -> Self {
        let mut node = Self {
            atime: attack_time,
            rtime: release_time,
            ..AFollow::default()
        };
        node.reset();
        node.set_sample_rate(DEFAULT_SR);
        node
    }

    /// Attack time in seconds.
    pub fn attack_time(&self) -> F {
        self.atime
    }

    /// Release time in seconds.
    pub fn release_time(&self) -> F {
        self.rtime
    }

    /// Set attack/release time in seconds.
    pub fn set_time(&mut self, attack_time: F, release_time: F) {
        self.atime = attack_time;
        self.rtime = release_time;
        self.acoeff = F::from_f64(halfway_coeff(
            (self.attack_time() * self.sample_rate).to_f64(),
        ));
        self.rcoeff = F::from_f64(halfway_coeff(
            (self.release_time() * self.sample_rate).to_f64(),
        ));
        if self.acoeff_now < F::one() {
            self.acoeff_now = self.acoeff;
            self.rcoeff_now = self.rcoeff;
        }
    }

    /// Current response.
    pub fn value(&self) -> F {
        self.v3
    }

    /// Jump to `x` immediately.
    pub fn set_value(&mut self, x: F) {
        self.v3 = x;
        self.v2 = x;
        self.v1 = x;
    }
}

impl<F: Real> AudioNode for AFollow<F> {
    const ID: u64 = 29;
    type Inputs = U1;
    type Outputs = U1;

    fn reset(&mut self) {
        self.v3 = F::zero();
        self.v2 = F::zero();
        self.v1 = F::zero();
        self.acoeff_now = F::one();
        self.rcoeff_now = F::one();
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = convert(sample_rate);
        // Recalculate coefficients.
        self.set_time(self.atime, self.rtime);
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        // Three 1-pole filters in series.
        let v0: F = convert(input[0]);
        self.v1 =
            (self.atime, self.rtime).filter_pole(v0, self.v1, self.acoeff_now, self.rcoeff_now);
        self.v2 = (self.atime, self.rtime).filter_pole(
            self.v1,
            self.v2,
            self.acoeff_now,
            self.rcoeff_now,
        );
        self.v3 = (self.atime, self.rtime).filter_pole(
            self.v2,
            self.v3,
            self.acoeff_now,
            self.rcoeff_now,
        );
        self.acoeff_now = self.acoeff;
        self.rcoeff_now = self.rcoeff;
        [convert(self.v3)].into()
    }

    fn set(&mut self, setting: Setting) {
        if let Parameter::AttackRelease(attack, release) = setting.parameter() {
            self.set_time(F::from_f32(*attack), F::from_f32(*release));
        }
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = SignalFrame::new(self.outputs());
        // The frequency response exists only in symmetric mode, as the asymmetric mode is nonlinear.
        if self.acoeff == self.rcoeff {
            output.set(
                0,
                input.at(0).filter(0.0, |r| {
                    let c = 1.0 - self.acoeff.to_f64();
                    let f = frequency * f64::TAU / self.sample_rate.to_f64();
                    let z1 = Complex64::from_polar(1.0, -f);
                    let pole = (1.0 - c) / (1.0 - c * z1);
                    r * pole * pole * pole
                }),
            );
        } else {
            output.set(0, input.at(0).distort(0.0));
        }
        output
    }
}
