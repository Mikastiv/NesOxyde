use crate::cpu::Interface;

pub struct MainBus {}

impl Interface for MainBus {
    fn read(&self, addr: u16) -> u8 {
        todo!()
    }

    fn write(&mut self, addr: u16, data: u8) {
        todo!()
    }
}

impl MainBus {
    pub fn new() -> Self {
        Self {}
    }
}
