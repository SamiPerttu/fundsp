//! Musical patterns. WIP.

use super::scale::*;

#[derive(Copy, Clone)]
pub enum Stage {
    Attack,
    Decay,
    Rest,
}

#[derive(Copy, Clone)]
pub struct Step {
    stage: Stage,
    pitch: usize,
}

impl Step {
    pub fn new(stage: Stage, pitch: usize) -> Self {
        Self { stage, pitch }
    }
    pub fn stage(&self) -> Stage {
        self.stage
    }
    pub fn pitch(&self) -> usize {
        self.pitch
    }
}

pub struct Pattern {
    step: Vec<Step>,
    voices: usize,
    scale: Scale,
}

impl Pattern {
    pub fn new(voices: usize, length: usize, scale: Scale) -> Self {
        Self {
            step: vec![Step::new(Stage::Rest, 0); voices * length],
            voices,
            scale,
        }
    }

    pub fn voices(&self) -> usize {
        self.voices
    }

    pub fn scale(&self) -> &Scale {
        &self.scale
    }

    pub fn at(&self, voice: usize, i: usize) -> Step {
        self.step[voice + i * self.voices]
    }

    pub fn set(&mut self, voice: usize, i: usize, step: Step) {
        self.step[voice + i * self.voices] = step;
    }
}
