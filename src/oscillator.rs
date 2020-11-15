use super::*;
use super::audiocomponent::*;
use numeric_array::*;
use super::math::*;

/// Sine oscillator.
#[derive(Clone)]
pub struct SineComponent {
    phase: f64,
    sample_duration: f64,
}

impl SineComponent {
    pub fn new() -> SineComponent { SineComponent { phase: 0.0, sample_duration: 1.0 / DEFAULT_SR } }
}

impl AudioComponent for SineComponent {
    type Inputs = typenum::U1;
    type Outputs = typenum::U1;

    fn reset(&mut self, sample_rate: Option<f64>)
    {
        self.phase = 0.0;
        if let Some(sr) = sample_rate { self.sample_duration = 1.0 / sr };
    }

    #[inline] fn tick(&mut self, input: &Frame<Self::Inputs>) -> Frame<Self::Outputs>
    {
        let frequency = input[0] as f64;
        self.phase += frequency * self.sample_duration;
        [into_f48(sin(self.phase * TAU))].into()
    }
}
