use super::*;
use super::audiocomponent::*;
use super::math::*;
use numeric_array::typenum::*;

/// Fixed delay.
#[derive(Clone)]
pub struct DelayComponent {
    buffer: Vec<f48>,
    i: usize,
    delay: f48,
}

impl DelayComponent {
    pub fn new(delay: f48, sample_rate: f64) -> DelayComponent {
        let mut ac = DelayComponent { buffer: vec!(), i: 0, delay };
        ac.reset(Some(sample_rate));
        ac
    }
}

impl AudioComponent for DelayComponent {
    type Inputs = U1;
    type Outputs = U1;
    
    #[inline] fn reset(&mut self, sample_rate: Option<f64>) {
        if let Some(sample_rate) = sample_rate {
            let buffer_length = ceil(self.delay as f64 * sample_rate);
            self.buffer.resize(max(1, buffer_length as usize), 0.0);
        }
        self.i = 0;
        for x in self.buffer.iter_mut() { *x = 0.0; }
    }

    #[inline] fn tick(&mut self, input: &Frame<Self::Inputs>) -> Frame<Self::Outputs>
    {
        let output = self.buffer[self.i];
        self.buffer[self.i] = input[0];
        self.i += 1;
        if self.i >= self.buffer.len() { self.i = 0; }
        [output].into()
    }
}
