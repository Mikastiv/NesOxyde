use std::time::{Duration, Instant};

/// Helper struct to video/audio timing
pub struct Timer {
    start: Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn reset(&mut self) {
        self.start = Instant::now();
    }

    pub fn wait(&self, time: Duration) {
        let passed_time = Instant::now() - self.start;
        let wait_time = if time > passed_time {
            time - passed_time
        } else {
            Duration::from_secs(0)
        };
        if wait_time.as_millis() > 1 {
            spin_sleep::sleep(wait_time)
        }
    }
}
