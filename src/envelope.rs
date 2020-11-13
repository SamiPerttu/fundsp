use super::*;
use super::prelude::*;
use numeric_array::*;
use super::audiocomponent::*;

/// EnvelopeComponent samples a time varying function.
#[derive(Clone)]
pub struct EnvelopeComponent<S: AudioFloat, F: Fn(S) -> S>
{
    f: F,
    t: f64,
    t_0: f64,
    t_1: f64,
    value_0: S,
    value_1: S,
    interval: f64,
    sample_duration: f64,
}

impl<S: AudioFloat, F: Fn(S) -> S> EnvelopeComponent<S, F>
{
    pub fn new(interval: f64, sample_rate: f64, f: F) -> Self {
        assert!(interval > 0.0);
        let mut component = EnvelopeComponent { f, t: 0.0, t_0: 0.0, t_1: 0.0, value_0: S::zero(), value_1: S::zero(), interval, sample_duration: 0.0 };
        component.reset(Some(sample_rate));
        component
    }
}

impl<S: AudioFloat, F: Fn(S) -> S> AudioComponent for EnvelopeComponent<S, F>
{
    type Sample = S;
    type Inputs = typenum::U0;
    type Outputs = typenum::U1;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.t = 0.0;
        self.t_0 = 0.0;
        self.t_1 = self.interval;
        self.value_0 = (self.f)(S::from_f64(self.t_0));
        self.value_1 = (self.f)(S::from_f64(self.t_1));
        if let Some(sr) = sample_rate { self.sample_duration = 1.0 / sr };
    }

    fn tick(&mut self, _input: &NumericArray<S, Self::Inputs>) -> NumericArray<S, Self::Outputs>
    {
        if self.t >= self.t_1 {
            self.t_0 = self.t_1;
            self.value_0 = self.value_1;
            self.t_1 = self.t_0 + self.interval;
            self.value_1 = (self.f)(S::from_f64(self.t_1));
        }
        let value = lerp(self.value_0, self.value_1, S::from_f64(delerp(self.t_0, self.t_1, self.t)));
        self.t += self.sample_duration;
        [value].into()
    }
}

/// Makes a control envelope from a time-varying function.
/// The output is linearly interpolated from samples at 2 ms intervals.
/// Synonymous with lfo.
pub fn envelope(f: impl Fn(f48) -> f48) -> Ac<impl AudioComponent<Sample=f48, Inputs = typenum::U0, Outputs = typenum::U1>> {
    // Signals containing frequencies no greater than about 20 Hz would be considered control rate.
    // Therefore, sampling at 500 Hz means these signals are fairly well represented.
    // While we represent time in double precision internally, it is often okay to use single precision
    // in envelopes, as local component time typically does not get far from origin.
    Ac(EnvelopeComponent::<f48, _>::new(0.002, DEFAULT_SR, f))
}

/// Makes a control signal from a time-varying function.
/// The output is linearly interpolated from samples at 2 ms intervals.
/// Synonymous with envelope.
pub fn lfo(f: impl Fn(f48) -> f48) -> Ac<impl AudioComponent<Sample=f48, Inputs = typenum::U0, Outputs = typenum::U1>> {
    Ac(EnvelopeComponent::<f48, _>::new(0.002, DEFAULT_SR, f))
}
