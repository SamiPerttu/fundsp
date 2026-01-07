use super::audionode::*;
use super::buffer::*;
use super::signal::*;
use super::wave::*;
use super::*;
use fft_convolver::FFTConvolver;
use std::sync::Arc;

#[derive(Clone)]
pub struct Convolver {
    convolver: FFTConvolver<f32>,
    response: Arc<Wave>,
}

impl Convolver {
    pub fn new(impulse_response: Arc<Wave>) -> Self {
        let mut convolver = FFTConvolver::<f32>::default();
        convolver
            .init(super::MAX_BUFFER_SIZE, impulse_response.channel(0))
            .unwrap();
        Self {
            convolver,
            response: impulse_response,
        }
    }
    pub fn set_response(&mut self, impulse_response: Arc<Wave>) {
        self.response = impulse_response;
        self.convolver
            .init(super::MAX_BUFFER_SIZE, self.response.channel(0))
            .unwrap();
    }
}

impl AudioNode for Convolver {
    const ID: u64 = 100;
    type Inputs = U1;
    type Outputs = U1;

    fn reset(&mut self) {
        self.convolver.reset();
    }

    #[inline]
    fn tick(&mut self, input: &Frame<f32, Self::Inputs>) -> Frame<f32, Self::Outputs> {
        let mut output: Frame<f32, U1> = Frame::default();
        self.convolver.process(&input, &mut output).unwrap();
        output
    }

    fn process(&mut self, size: usize, input: &BufferRef, output: &mut BufferMut) {
        if size > 0 {
            self.convolver
                .process(
                    &input.channel_f32(0)[0..size],
                    &mut output.channel_f32_mut(0)[0..size],
                )
                .unwrap();
        }
    }

    fn route(&mut self, _input: &SignalFrame, _frequency: f64) -> SignalFrame {
        // TODO.
        SignalFrame::new(self.outputs())
    }
}
