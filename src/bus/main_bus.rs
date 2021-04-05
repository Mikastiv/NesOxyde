use crate::cpu::Interface;

const RAM_SIZE: usize = 0x800;
const RAM_MASK: usize = 0x7FF;
const RAM_START: usize = 0x0000;
const RAM_END: usize = 0x1FFF;

const PPU_MASK: usize = 0x7;
const PPU_REG_START: usize = 0x2000;
const PPU_REG_END: usize = 0x3FFF;

pub struct MainBus {
    ram: [u8; RAM_SIZE],
}

impl Interface for MainBus {
    fn read(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        match addr {
            RAM_START..=RAM_END => self.ram[addr & RAM_MASK],
            PPU_REG_START..=PPU_REG_END => todo!(),
            _ => {
                println!("Ignored read at 0x{:04X}", addr);
                0
            }
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        let addr = addr as usize;
        match addr {
            RAM_START..=RAM_END => self.ram[addr & RAM_MASK] = data,
            PPU_REG_START..=PPU_REG_END => todo!(),
            _ => println!("Ignored read at 0x{:04X}", addr),
        }
    }
}

impl MainBus {
    pub fn new() -> Self {
        Self { ram: [0; RAM_SIZE] }
    }
}
