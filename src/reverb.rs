use std::collections::VecDeque;

pub struct Reverb {
    delay: usize,
    decay: f32,
    buf: VecDeque<f32>,
}

impl Reverb {
    pub fn new(delay_ms: usize, sample_rate: usize, decay: f32) -> Self {
        let mut v = VecDeque::new();
        v.make_contiguous();

        Self {
            delay: delay_ms * sample_rate / 1000,
            decay,
            buf: v,
        }
    }

    pub fn apply(&mut self, samples: &mut [f32]) {
        self.buf.extend(samples.iter());

        if self.buf.len() > self.delay {
            let count = std::cmp::min(self.buf.len() - self.delay, samples.len());
            for s in samples.iter_mut().take(count) {
                if let Some(sample) = self.buf.pop_front() {
                    *s += sample * self.decay;
                } else {
                    return;
                }
            }
        }
    }
}
