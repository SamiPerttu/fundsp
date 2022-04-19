//! Equitempered scales.

use super::math::*;

pub struct Scale {
    pitch: Vec<f64>,
    priority: Vec<f64>,
    dissonance: Vec<f64>,
}

/// Calculate an overtone dissonance between
/// the first 8 partials of tones at `a` and `b` Hz.
pub fn overtone_dissonance(a: f64, b: f64) -> f64 {
    let mut d = 0.0;
    for i in 1..9 {
        for j in 1..9 {
            d += 1.0 / max(i, j) as f64 * dissonance(a * i as f64, b * j as f64);
        }
    }
    d
}

/// Calculate a scale dissonance of tones at `a` and `b` Hz
/// that is a sum of overtone dissonance and chroma dissonance.
/// Chroma dissonance is overtone dissonance where the
/// tones are brought into the same octave.
pub fn scale_dissonance(mut a: f64, mut b: f64) -> f64 {
    let dissonance = overtone_dissonance(a, b);
    while a >= b * 2.0 {
        b *= 2.0;
    }
    while b >= a * 2.0 {
        a *= 2.0;
    }
    dissonance + overtone_dissonance(a, b)
}

/// Equitempered scale.
impl Scale {
    pub fn new(lowest: f64, notes: usize, priority: &[f64]) -> Self {
        let chromas = priority.len();
        let mut pitch = Vec::with_capacity(notes);
        for i in 0..notes {
            pitch.push(lowest * exp2(i as f64 / chromas as f64));
        }
        let dissonance = vec![0.0; notes * notes];

        let mut scale = Self {
            pitch,
            priority: priority.into(),
            dissonance,
        };

        let mut max_dissonance = 0.0;
        for i in 0..notes {
            for j in i..notes {
                scale.compute_dissonance(i, j);
                max_dissonance = max(max_dissonance, scale.dissonance(i, j));
            }
        }
        for i in 0..notes {
            for j in i..notes {
                scale.set_dissonance(i, j, scale.dissonance(i, j) / max_dissonance);
            }
        }

        scale
    }

    pub fn chromas(&self) -> usize {
        self.priority.len()
    }

    pub fn chroma(&self, i: usize) -> usize {
        i % self.chromas()
    }

    pub fn notes(&self) -> usize {
        self.pitch.len()
    }

    pub fn pitch(&self, i: usize) -> f64 {
        self.pitch[i]
    }

    pub fn dissonance(&self, i: usize, j: usize) -> f64 {
        self.dissonance[i * self.notes() + j]
    }

    pub fn priority(&self, i: usize) -> f64 {
        self.priority[i]
    }

    pub fn set_dissonance(&mut self, i: usize, j: usize, dissonance: f64) {
        let n = self.notes();
        self.dissonance[i * n + j] = dissonance;
        self.dissonance[j * n + i] = dissonance;
    }

    pub fn compute_dissonance(&mut self, i: usize, j: usize) {
        if i == j {
            self.set_dissonance(i, j, 0.0);
        } else if i > j {
            self.compute_dissonance(j, i);
        } else {
            let d = scale_dissonance(self.pitch(i), self.pitch(j));
            self.set_dissonance(i, j, d);
            self.set_dissonance(j, i, d);
        }
    }
}
