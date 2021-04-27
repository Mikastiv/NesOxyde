use std::usize;

use super::Mapper;
use crate::cartridge::{MirrorMode, Rom};

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
        self.rom.prg[(addr & mask) as usize]
    }

    fn write_prg(&mut self, _addr: u16, _data: u8) {}

    fn read_chr(&mut self, addr: u16) -> u8 {
        self.rom.chr[addr as usize]
    }

    fn write_chr(&mut self, _addr: u16, _data: u8) {}

    fn mirror_mode(&self) -> MirrorMode {
        self.rom.header.mirror_mode()
    }
}
