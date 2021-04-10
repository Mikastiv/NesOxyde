use super::Pixel;

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 240;

pub struct Frame {
    pixels: Vec<u8>,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            pixels: vec![0; WIDTH * HEIGHT * 3],
        }
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, pixel: Pixel) {
        let index = (y * 3) * WIDTH + (x * 3);
        assert!(index + 2 < self.pixels.len());
        self.pixels[index] = pixel.0;
        self.pixels[index + 1] = pixel.1;
        self.pixels[index + 2] = pixel.2;
    }
}
