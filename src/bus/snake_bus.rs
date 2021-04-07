use std::usize;

use crate::cpu::Interface;

const RESET_VECTOR: u16 = 0xFFFC;

// Bus only used with the snake game
pub struct SnakeBus {
    memory: [u8; 0xFFFF],
}

impl Interface for SnakeBus {
    fn read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data
    }
}

impl SnakeBus {
    pub fn new() -> Self {
        Self {
            memory: [0; 0xFFFF],
        }
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x600..(0x600 + program.len())].copy_from_slice(&program[..]);
        self.write(RESET_VECTOR, 0x00);
        self.write(RESET_VECTOR + 1, 0x06);
    }
}
