pub struct Dmc {
    enabled: bool,
}

impl Dmc {
    pub fn new() -> Self {
        Self { enabled: false }
    }

    pub fn reset(&mut self) {
        self.enabled = false;
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.enabled = v;
    }

    pub fn write_freq(&mut self, data: u8) {}

    pub fn write_raw(&mut self, data: u8) {}

    pub fn write_start(&mut self, data: u8) {}

    pub fn write_len(&mut self, data: u8) {}

    pub fn output(&self) -> u8 {
        0
    }
}
