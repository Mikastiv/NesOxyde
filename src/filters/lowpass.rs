use std::collections::VecDeque;
use std::f32::consts::PI;

use super::Filter;

pub struct LowPass {
    a1: f32,
    a2: f32,
    a3: f32,
    b1: f32,
    b2: f32,

    in_history: VecDeque<f32>,
    out_history: VecDeque<f32>,
}

impl Filter for LowPass {
    fn filter(&mut self, input: f32) -> f32 {
        let output = self.a1 * input + self.a2 * self.in_history[0] + self.a3 * self.in_history[1]
            - self.b1 * self.out_history[0]
            - self.b2 * self.out_history[1];

        self.in_history.push_front(input);
        self.in_history.pop_back();

        self.out_history.push_front(output);
        self.out_history.pop_back();

        output
    }
}

impl LowPass {
    pub fn new(freq: f32, sample_rate: f32, resonance: f32) -> Self {
        let c = 1.0 / (PI * freq / sample_rate).tan();
        let a1 = 1.0 / (1.0 + resonance * c + c * c);
        let a2 = 2.0 * a1;
        let a3 = a1;
        let b1 = 2.0 * (1.0 - c * c) * a1;
        let b2 = (1.0 - resonance * c + c * c) * a1;

        let mut input = VecDeque::new();
        let mut output = VecDeque::new();

        for _ in 0..2 {
            input.push_back(0.0);
            output.push_back(0.0);
        }
        output.push_back(0.0);

        Self {
            a1,
            a2,
            a3,
            b1,
            b2,

            in_history: input,
            out_history: output,
        }
    }
}
