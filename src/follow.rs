//! Parameter smoothing filter.

use super::audionode::*;
use super::combinator::*;
use super::math::*;
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
pub struct Follow<T: Float, F: Real> {
    v3: F,
    v2: F,
    v1: F,
    coeff: F,
    // Filter coefficient that is 1 for the first sample and then assumes the value of coeff.
    coeff_now: F,
    /// Halfway response time.
    response_time: F,
    sample_rate: F,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Float, F: Real> Follow<T, F> {
    /// Create new smoothing filter. Response time (in seconds)
    /// is how long it takes for the follower to reach halfway to the new value.
    pub fn new(sample_rate: f64, response_time: F) -> Self {
        let mut node = Follow::<T, F> {
            response_time,
            ..Follow::default()
        };
        node.reset();
        node.set_sample_rate(sample_rate);
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

impl<T: Float, F: Real> AudioNode for Follow<T, F> {
    const ID: u64 = 24;
    type Sample = T;
    type Inputs = U1;
    type Outputs = U1;
    type Setting = F;

    fn set(&mut self, setting: Self::Setting) {
        self.set_response_time(setting);
    }

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
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        // Three 1-pole filters in series.
        let rcoeff = F::one() - self.coeff_now;
        self.v1 = self.coeff_now * convert(input[0]) + rcoeff * self.v1;
        self.v2 = self.coeff_now * self.v1 + rcoeff * self.v2;
        self.v3 = self.coeff_now * self.v2 + rcoeff * self.v3;
        self.coeff_now = self.coeff;
        [convert(self.v3)].into()
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        output[0] = input[0].filter(0.0, |r| {
            let c = 1.0 - self.coeff.to_f64();
            let f = frequency * TAU / self.sample_rate.to_f64();
            let z1 = Complex64::from_polar(1.0, -f);
            let pole = (1.0 - c) / (1.0 - c * z1);
            r * pole * pole * pole
        });
        output
    }
}

/// Smoothing filter with adjustable edge response times for attack and release.
/// Setting: same form as in constructor.
/// - Input 0: input signal
/// - Output 0: smoothed signal
#[derive(Default, Clone)]
pub struct AFollow<T: Float, F: Real, S: ScalarOrPair<Sample = F>> {
    v3: F,
    v2: F,
    v1: F,
    acoeff: F,
    rcoeff: F,
    acoeff_now: F,
    rcoeff_now: F,
    /// Response times.
    time: S,
    sample_rate: F,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Float, F: Real, S: ScalarOrPair<Sample = F>> AFollow<T, F, S> {
    /// Create new smoothing filter.
    /// Response time is how long it takes for the follower to reach halfway to the new value.
    pub fn new(sample_rate: f64, time: S) -> Self {
        let mut node = AFollow::<T, F, S> {
            time,
            ..AFollow::default()
        };
        node.reset();
        node.set_sample_rate(sample_rate);
        node
    }

    /// Attack time in seconds.
    pub fn attack_time(&self) -> F {
        self.time.broadcast().0
    }

    /// Release time in seconds.
    pub fn release_time(&self) -> F {
        self.time.broadcast().1
    }

    /// Set attack/release time in seconds.
    pub fn set_time(&mut self, time: S) {
        self.time = time;
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

impl<T: Float, F: Real, S: ScalarOrPair<Sample = F>> AudioNode for AFollow<T, F, S> {
    const ID: u64 = 29;
    type Sample = T;
    type Inputs = U1;
    type Outputs = U1;
    type Setting = S;

    fn set(&mut self, setting: Self::Setting) {
        self.set_time(setting);
    }

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
        self.set_time(self.time.clone());
    }

    #[inline]
    fn tick(
        &mut self,
        input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        // Three 1-pole filters in series.
        let v0: F = convert(input[0]);
        self.v1 = self
            .time
            .filter_pole(v0, self.v1, self.acoeff_now, self.rcoeff_now);
        self.v2 = self
            .time
            .filter_pole(self.v1, self.v2, self.acoeff_now, self.rcoeff_now);
        self.v3 = self
            .time
            .filter_pole(self.v2, self.v3, self.acoeff_now, self.rcoeff_now);
        self.acoeff_now = self.acoeff;
        self.rcoeff_now = self.rcoeff;
        [convert(self.v3)].into()
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut output = new_signal_frame(self.outputs());
        // The frequency response exists only in symmetric mode, as the asymmetric mode is nonlinear.
        if self.acoeff == self.rcoeff {
            output[0] = input[0].filter(0.0, |r| {
                let c = 1.0 - self.acoeff.to_f64();
                let f = frequency * TAU / self.sample_rate.to_f64();
                let z1 = Complex64::from_polar(1.0, -f);
                let pole = (1.0 - c) / (1.0 - c * z1);
                r * pole * pole * pole
            });
        } else {
            output[0] = input[0].distort(0.0);
        }
        output
    }
}
