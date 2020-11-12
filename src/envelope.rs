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
        let mut component = EnvelopeComponent { f, t: 0.0, t_0: 0.0, t_1: 0.0, value_0: S::zero(), value_1: S::zero(), interval, sample_duration: 1.0 / sample_rate };
        component.reset(None);
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
        self.value_0 = (self.f)(S::new_f64(self.t_0));
        self.value_1 = (self.f)(S::new_f64(self.t_1));
        if let Some(sr) = sample_rate { self.sample_duration = 1.0 / sr };
    }

    fn tick(&mut self, _input: &NumericArray<S, Self::Inputs>) -> NumericArray<S, Self::Outputs>
    {
        if self.t >= self.t_1 {
            self.t_0 = self.t_1;
            self.value_0 = self.value_1;
            self.t_1 = self.t_0 + self.interval;
            self.value_1 = (self.f)(S::new_f64(self.t_1));
        }
        let value = lerp(self.value_0, self.value_1, S::new_f64(delerp(self.t_0, self.t_1, self.t)));
        self.t += self.sample_duration;
        [value].into()
    }
}

/// Makes an envelope from a time-varying function.
/// The output is linearly interpolated from samples at 1 ms intervals.
/// Synonymous with lfo.
pub fn envelope(f: impl Fn(F32) -> F32) -> Ac<impl AudioComponent> { Ac(EnvelopeComponent::new(0.001, DEFAULT_SR, f)) }

/// Makes an envelope from a time-varying function.
/// The output is linearly interpolated from samples at 1 ms intervals.
/// Synonymous with envelope.
pub fn lfo(f: impl Fn(F32) -> F32) -> Ac<impl AudioComponent> { Ac(EnvelopeComponent::new(0.001, DEFAULT_SR, f)) }
