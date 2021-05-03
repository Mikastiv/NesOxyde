use std::time::{Duration, Instant};

use spin_sleep::SpinSleeper;

/// Helper struct for video/audio timing
pub struct Timer {
    start: Instant,
    sleeper: SpinSleeper,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            sleeper: SpinSleeper::default(),
        }
    }

    pub fn reset(&mut self) {
        self.start = Instant::now();
    }

    pub fn wait(&self, time: Duration) {
        let passed_time = Instant::now() - self.start;
        if time > passed_time {
            let wait_time = time - passed_time;
            if wait_time.as_millis() > 1 {
                self.sleeper.sleep(wait_time);
            }
        }
    }
}
