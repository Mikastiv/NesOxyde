use std::usize;

use super::super::Rom;
use super::Mapper;

pub struct Mapper0 {
    rom: Rom,
}

impl Mapper0 {
    pub fn new(rom: Rom) -> Self {
        Self { rom }
    }
}

impl Mapper for Mapper0 {
    fn read_prg(&mut self, addr: u16) -> u8 {
        let mask = if self.rom.header.prg_count() > 1 {
            0x7FFF
        } else {
            0x3FFF
        };
        let index = addr & mask;
        self.rom.prg[index as usize]
    }

    fn write_prg(&mut self, addr: u16, data: u8) {
        todo!()
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        todo!()
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        todo!()
    }
}
