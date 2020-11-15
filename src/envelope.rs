use super::*;
use super::math::{lerp, delerp};
use super::audiocomponent::*;
use numeric_array::*;

/// EnvelopeComponent samples a time varying function.
#[derive(Clone)]
pub struct EnvelopeComponent<F: Fn(f48) -> f48 + Clone> where
{
    envelope: F,
    t: f64,
    t_0: f64,
    t_1: f64,
    value_0: f48,
    value_1: f48,
    interval: f64,
    sample_duration: f64,
}

impl<F: Fn(f48) -> f48 + Clone> EnvelopeComponent<F>
{
    pub fn new(interval: f64, sample_rate: f64, envelope: F) -> Self {
        assert!(interval > 0.0);
        let mut component = EnvelopeComponent { envelope, t: 0.0, t_0: 0.0, t_1: 0.0, value_0: 0.0, value_1: 0.0, interval, sample_duration: 0.0 };
        component.reset(Some(sample_rate));
        component
    }
}

impl<F: Fn(f48) -> f48 + Clone> AudioComponent for EnvelopeComponent<F>
{
    type Inputs = typenum::U0;
    type Outputs = typenum::U1;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.t = 0.0;
        self.t_0 = 0.0;
        self.t_1 = self.interval;
        self.value_0 = (self.envelope)(self.t_0 as f48);
        self.value_1 = (self.envelope)(self.t_1 as f48);
        if let Some(sr) = sample_rate { self.sample_duration = 1.0 / sr };
    }

    #[inline] fn tick(&mut self, _input: &Frame<Self::Inputs>) -> Frame<Self::Outputs>
    {
        if self.t >= self.t_1 {
            self.t_0 = self.t_1;
            self.value_0 = self.value_1;
            self.t_1 = self.t_0 + self.interval;
            self.value_1 = (self.envelope)(self.t_1 as f48);
        }
        let value = lerp(self.value_0, self.value_1, delerp(self.t_0, self.t_1, self.t) as f48);
        self.t += self.sample_duration;
        [value].into()
    }
}
