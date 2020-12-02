use super::audionode::*;
use super::combinator::*;
use super::math::*;
use super::*;
use numeric_array::*;

/// Sample a time varying function.
/// The return type can be scalar or tuple. It determines the number of output channels.
#[derive(Clone, Default)]
pub struct EnvelopeNode<T, F, E, R>
where
    T: Float,
    F: Float,
    E: Fn(F) -> R + Clone,
    R: ConstantFrame<Sample = F>,
    R::Size: Size<F>,
    R::Size: Size<T>,
{
    envelope: E,
    t: F,
    t_0: F,
    t_1: F,
    t_hash: u32,
    value_0: Frame<T, R::Size>,
    value_1: Frame<T, R::Size>,
    interval: F,
    sample_duration: F,
    hash: u32,
}

impl<T, F, E, R> EnvelopeNode<T, F, E, R>
where
    T: Float,
    F: Float,
    E: Fn(F) -> R + Clone,
    R: ConstantFrame<Sample = F>,
    R::Size: Size<F>,
    R::Size: Size<T>,
{
    pub fn new(interval: F, sample_rate: f64, envelope: E) -> Self {
        assert!(interval > F::zero());
        let mut node = EnvelopeNode::<T, F, E, R> {
            envelope,
            t: F::zero(),
            t_0: F::zero(),
            t_1: F::zero(),
            t_hash: 0,
            value_0: Frame::default(),
            value_1: Frame::default(),
            interval,
            sample_duration: convert(1.0 / sample_rate),
            hash: 0,
        };
        node.reset(Some(sample_rate));
        node
    }
}

impl<T, F, E, R> AudioNode for EnvelopeNode<T, F, E, R>
where
    T: Float,
    F: Float,
    E: Fn(F) -> R + Clone,
    R: ConstantFrame<Sample = F>,
    R::Size: Size<F>,
    R::Size: Size<T>,
{
    const ID: u64 = 14;
    type Sample = T;
    type Inputs = typenum::U0;
    type Outputs = R::Size;

    fn reset(&mut self, sample_rate: Option<f64>) {
        self.t = F::zero();
        self.t_0 = F::zero();
        self.t_1 = F::zero();
        self.t_hash = self.hash;
        let value_0: Frame<_, _> = (self.envelope)(self.t_0).convert();
        self.value_0 = Frame::generate(|i| convert(value_0[i]));
        self.value_1 = Frame::default();
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
            self.value_0 = self.value_1.clone();
            // Jitter the next sample point.
            self.t_1 = self.t_0
                + self.interval
                    * lerp(
                        F::from_f32(0.75),
                        F::from_f32(1.25),
                        convert(rnd(self.t_hash as i64)),
                    );
            let value_1: Frame<_, _> = (self.envelope)(self.t_1).convert();
            self.value_1 = Frame::generate(|i| convert(value_1[i]));
            self.t_hash = hashw(self.t_hash);
        }
        let u = delerp(self.t_0, self.t_1, self.t);
        self.t = self.t + self.sample_duration;
        Frame::generate(|i| lerp(self.value_0[i], self.value_1[i], convert(u)))
    }

    #[inline]
    fn set_hash(&mut self, hash: u32) {
        self.hash = hash;
        self.t_hash = hash;
    }
}
