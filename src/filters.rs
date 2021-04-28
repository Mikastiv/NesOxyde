pub use highpass::HighPass;
pub use lowpass::LowPass;

mod highpass;
mod lowpass;

pub trait Filter {
    fn filter(&mut self, input: f32) -> f32;
}
