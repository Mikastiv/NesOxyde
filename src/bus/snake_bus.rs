use std::usize;

use crate::cpu::Interface;

const RAM_SIZE: usize = 0x600;
const RESET_VECTOR: u16 = 0xFFFC;

pub struct SnakeBus {
    ram: [u8; RAM_SIZE],
    program: [u8; 0xFFFF],
}

impl Interface for SnakeBus {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x05FF => self.ram[addr as usize],
            _ => self.program[addr as usize],
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x05FF => self.ram[addr as usize] = data,
            _ => self.program[addr as usize] = data,
        }
    }
}

impl SnakeBus {
    pub fn new() -> Self {
        Self {
            ram: [0; RAM_SIZE],
            program: [0; 0xFFFF],
        }
    }

	pub fn load(&mut self, program: Vec<u8>) {
		self.program[RAM_SIZE..(RAM_SIZE + program.len())].copy_from_slice(&program[..]);
		self.write(RESET_VECTOR, 0x00);
		self.write(RESET_VECTOR + 1, 0x06);
	}
}
