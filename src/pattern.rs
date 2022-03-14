//! Musical patterns. WIP.

#[derive(Copy, Clone)]
pub enum Stage {
    Attack,
    Decay,
}

#[derive(Copy, Clone)]
pub struct Step {
    stage: Stage,
    pitch: Option<usize>,
}

impl Step {
    pub fn new(stage: Stage, pitch: Option<usize>) -> Self {
        Self { stage, pitch }
    }
    pub fn stage(&self) -> Stage {
        self.stage
    }
    pub fn pitch(&self) -> Option<usize> {
        self.pitch
    }
}

pub struct Pattern {
    step: Vec<Step>,
    voices: usize,
}

impl Pattern {
    pub fn new(voices: usize, length: usize) -> Self {
        Self {
            step: vec![Step::new(Stage::Decay, None); voices * length],
            voices,
        }
    }

    pub fn voices(&self) -> usize {
        self.voices
    }

    pub fn at(&self, voice: usize, i: usize) -> Step {
        self.step[voice + i * self.voices]
    }

    pub fn set(&mut self, voice: usize, i: usize, step: Step) {
        self.step[voice + i * self.voices] = step;
    }
}
