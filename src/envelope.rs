use super::audionode::*;
use super::math::*;
use super::*;
use numeric_array::*;

/// Sample a time varying function.
#[derive(Clone, Default)]
pub struct EnvelopeNode<T: Float, F: Real, E: Fn(F) -> F + Clone> {
    envelope: E,
    t: F,
    t_0: F,
    t_1: F,
    t_hash: u32,
    value_0: T,
    value_1: T,
    interval: F,
    sample_duration: F,
    hash: u32,
}

impl<T: Float, F: Real, E: Fn(F) -> F + Clone> EnvelopeNode<T, F, E> {
    pub fn new(interval: F, sample_rate: f64, envelope: E) -> Self {
        assert!(interval > F::zero());
        let mut node = EnvelopeNode::<T, F, E> {
            envelope,
            t: F::zero(),
            t_0: F::zero(),
            t_1: F::zero(),
            t_hash: 0,
            value_0: T::zero(),
            value_1: T::zero(),
            interval,
            sample_duration: convert(1.0 / sample_rate),
            hash: 0,
        };
        node.reset(Some(sample_rate));
        node
    }
}

impl<T: Float, F: Real, E: Fn(F) -> F + Clone> AudioNode for EnvelopeNode<T, F, E> {
    const ID: u64 = 14;
    type Sample = T;
    type Inputs = typenum::U0;
    type Outputs = typenum::U1;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.t = F::zero();
        self.t_0 = F::zero();
        self.t_1 = F::zero();
        self.t_hash = self.hash;
        self.value_0 = convert((self.envelope)(self.t_0));
        self.value_1 = T::zero();
        if let Some(sr) = sample_rate {
            self.sample_duration = convert(1.0 / sr)
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
            // Jitter the next sample point.
            self.t_1 = self.t_0
                + self.interval
                    * lerp(
                        F::from_f32(0.75),
                        F::from_f32(1.25),
                        convert(rnd(self.t_hash as i64)),
                    );
            self.value_1 = convert((self.envelope)(self.t_1));
            self.t_hash = hashw(self.t_hash);
        }
        let value = lerp(
            self.value_0,
            self.value_1,
            convert(delerp(self.t_0, self.t_1, self.t)),
        );
        self.t = self.t + self.sample_duration;
        [value].into()
    }

    #[inline]
    fn set_hash(&mut self, hash: u32) {
        self.hash = hash;
        self.t_hash = hash;
    }
}
