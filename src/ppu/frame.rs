use super::Rgb;
use crate::nes::{HEIGHT, WIDTH};

pub struct Frame {
    pixels: Vec<u8>,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            pixels: vec![0; (WIDTH * HEIGHT * 3) as usize],
        }
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, pixel: Rgb) {
        let index = (y * 3 * WIDTH as usize) + (x * 3);
        self.pixels[index] = pixel.0;
        self.pixels[index + 1] = pixel.1;
        self.pixels[index + 2] = pixel.2;
    }
}
