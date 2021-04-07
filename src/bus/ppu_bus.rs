use std::cell::RefCell;
use std::rc::Rc;

use crate::cartridge::Cartridge;
use crate::ppu::Interface;

const ROM_START: u16 = 0x0000;
const ROM_END: u16 = 0x1FFF;

const VRAM_START: u16 = 0x2000;
const VRAM_END: u16 = 0x3EFF;

const PALETTE_START: u16 = 0x3F00;
const PALETTE_END: u16 = 0x3FFF;

pub struct PpuBus {
    cartridge: Rc<RefCell<Cartridge>>,
}

impl Interface for PpuBus {
    fn read(&self, addr: u16) -> u8 {
        let addr = addr & 0x3FFF;
        match addr {
            ROM_START..=ROM_END => self.cartridge.borrow_mut().read_chr(addr),
            VRAM_START..=VRAM_END => todo!(),
            PALETTE_START..=PALETTE_END => todo!(),
            _ => unreachable!("Reached impossible match arm. (Ppu bus addr) {:#04X}", addr),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        let addr = addr & 0x3FFF;
        match addr {
            ROM_START..=ROM_END => self.cartridge.borrow_mut().write_chr(addr, data),
            VRAM_START..=VRAM_END => todo!(),
            PALETTE_START..=PALETTE_END => todo!(),
            _ => unreachable!("Reached impossible match arm. (Ppu bus addr) {:#04X}", addr),
        }
    }
}

impl PpuBus {
    pub fn new(cartridge: Rc<RefCell<Cartridge>>) -> Self {
        Self { cartridge }
    }
}
