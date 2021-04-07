use crate::cartridge::Cartridge;
use crate::cpu;

const RAM_SIZE: usize = 0x800;
const RAM_MASK: u16 = 0x7FF;
const RAM_START: u16 = 0x0000;
const RAM_END: u16 = 0x1FFF;

const PPU_MASK: u16 = 0x7;
const PPU_REG_START: u16 = 0x2000;
const PPU_REG_END: u16 = 0x3FFF;

const ROM_START: u16 = 0x8000;
const ROM_END: u16 = 0xFFFF;

pub struct MainBus {
    ram: [u8; RAM_SIZE],
    cartridge: Cartridge,
}

impl cpu::Interface for MainBus {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM_START..=RAM_END => self.ram[(addr & RAM_MASK) as usize],
            PPU_REG_START..=PPU_REG_END => todo!(),
            ROM_START..=ROM_END => self.cartridge.read_prg(addr),
            _ => {
                println!("Ignored read at 0x{:04X}", addr);
                0
            }
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM_START..=RAM_END => self.ram[(addr & RAM_MASK) as usize] = data,
            PPU_REG_START..=PPU_REG_END => todo!(),
            ROM_START..=ROM_END => self.cartridge.write_prg(addr, data),
            _ => println!("Ignored read at 0x{:04X}", addr),
        }
    }
}

impl MainBus {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            ram: [0; RAM_SIZE],
            cartridge,
        }
    }
}
