use std::cell::RefCell;
use std::rc::Rc;

use super::PpuBus;
use crate::cartridge::Cartridge;
use crate::cpu::Interface;
use crate::ppu::Ppu;

const RAM_SIZE: usize = 0x800;
const RAM_MASK: u16 = 0x7FF;
const RAM_START: u16 = 0x0000;
const RAM_END: u16 = 0x1FFF;

const PPU_MASK: u16 = 0x7;
const PPU_REG_START: u16 = 0x2000;
const PPU_REG_END: u16 = 0x3FFF;

const ROM_START: u16 = 0x8000;
const ROM_END: u16 = 0xFFFF;

const JOY1: u16 = 0x4016;
const JOY2: u16 = 0x4017;

pub struct MainBus {
    ram: [u8; RAM_SIZE],
    cartridge: Rc<RefCell<Cartridge>>,
    ppu: Ppu,
}

impl Interface for MainBus {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM_START..=RAM_END => self.ram[(addr & RAM_MASK) as usize],
            PPU_REG_START..=PPU_REG_END => {
                let addr = addr & PPU_MASK;
                self.ppu.read(addr)
            }
            JOY1 => todo!(),
            JOY2 => todo!(),
            ROM_START..=ROM_END => self.cartridge.borrow_mut().read_prg(addr),
            _ => {
                println!("Ignored read at {:#04X}", addr);
                0
            }
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM_START..=RAM_END => self.ram[(addr & RAM_MASK) as usize] = data,
            PPU_REG_START..=PPU_REG_END => {
                let addr = addr & PPU_MASK;
                self.ppu.write(addr, data);
            }
            JOY1 => todo!(),
            ROM_START..=ROM_END => self.cartridge.borrow_mut().write_prg(addr, data),
            _ => println!("Ignored write at 0x{:04X}", addr),
        }
    }

    fn poll_nmi(&mut self) -> bool {
        self.ppu.poll_nmi()
    }

    fn tick(&mut self, cycles: u64) {
        for _ in 0..(cycles * 3) {
            self.ppu.clock();
        }
    }
}

impl MainBus {
    pub fn new(cartridge: Rc<RefCell<Cartridge>>) -> Self {
        let ppu_bus = PpuBus::new(Rc::clone(&cartridge));
        Self {
            ram: [0; RAM_SIZE],
            cartridge,
            ppu: Ppu::new(Box::new(ppu_bus)),
        }
    }
}
