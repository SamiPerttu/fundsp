use super::audionode::*;
use super::math::{delerp, lerp};
use super::*;
use numeric_array::*;

/// EnvelopeNode samples a time varying function.
#[derive(Clone)]
pub struct EnvelopeNode<T: Float, F: Fn(f64) -> f64 + Clone> {
    envelope: F,
    t: f64,
    t_0: f64,
    t_1: f64,
    value_0: T,
    value_1: T,
    interval: f64,
    sample_duration: f64,
}

impl<T: Float, F: Fn(f64) -> f64 + Clone> EnvelopeNode<T, F> {
    pub fn new(interval: f64, sample_rate: f64, envelope: F) -> Self {
        assert!(interval > 0.0);
        let mut component = EnvelopeNode {
            envelope,
            t: 0.0,
            t_0: 0.0,
            t_1: 0.0,
            value_0: T::zero(),
            value_1: T::zero(),
            interval,
            sample_duration: 0.0,
        };
        component.reset(Some(sample_rate));
        component
    }
}

impl<T: Float, F: Fn(f64) -> f64 + Clone> AudioNode for EnvelopeNode<T, F> {
    type Sample = T;
    type Inputs = typenum::U0;
    type Outputs = typenum::U1;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.t = 0.0;
        self.t_0 = 0.0;
        self.t_1 = self.interval;
        self.value_0 = T::from_f64((self.envelope)(self.t_0));
        self.value_1 = T::from_f64((self.envelope)(self.t_1));
        if let Some(sr) = sample_rate {
            self.sample_duration = 1.0 / sr
        };
    }

    #[inline]
    fn tick(
        &mut self,
        _input: &Frame<Self::Sample, Self::Inputs>,
    ) -> Frame<Self::Sample, Self::Outputs> {
        if self.t >= self.t_1 {
            self.t_0 = self.t_1;
            self.value_0 = self.value_1;
            self.t_1 = self.t_0 + self.interval;
            self.value_1 = T::from_f64((self.envelope)(self.t_1));
        }
        let value = lerp(
            self.value_0,
            self.value_1,
            convert(delerp(self.t_0, self.t_1, self.t)),
        );
        self.t += self.sample_duration;
        [value].into()
    }
}
