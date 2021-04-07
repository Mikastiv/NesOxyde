pub trait Interface {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
}

pub struct Ppu {
    bus: Box<dyn Interface>,
}

impl Ppu {
    pub fn new(bus: Box<dyn Interface>) -> Self {
        Self { bus }
    }
}
