pub struct Decay {
    max_diff: f32,
    prev: f32,
}

impl Decay {
    pub fn new(max_diff: f32) -> Self {
        Self {
            max_diff,
            prev: 0.0,
        }
    }

    pub fn decay(&mut self, sample: f32) -> f32 {
        let diff = (self.prev - sample).abs();
        if sample == 0.0 && diff > self.max_diff {
            self.prev -= self.max_diff;
            if self.prev < 0.0 {
                self.prev = 0.0;
            }
            self.prev
        } else {
            self.prev = sample;
            sample
        }
    }
}
