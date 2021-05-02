pub use highpass::HighPass;
pub use lowpass::LowPass;

mod highpass;
mod lowpass;

/// Audio filter
pub trait Filter {
    /// Filters an audio signal
    fn filter(&mut self, input: f32) -> f32;
}
