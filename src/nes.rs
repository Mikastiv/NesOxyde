use std::cell::RefCell;
use std::rc::Rc;

use crate::bus::MainBus;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;

pub struct Nes {
    cpu: Cpu,
}

impl Nes {
    pub fn new(cartridge: Cartridge) -> Self {
        let bus = MainBus::new(Rc::new(RefCell::new(cartridge)));
        Self {
            cpu: Cpu::new(Box::new(bus)),
        }
    }
}
