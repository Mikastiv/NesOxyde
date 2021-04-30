use crate::cartridge::rom::CHR_PAGE_SIZE;
use crate::cartridge::{MirrorMode, Rom};

use super::Mapper;

pub struct Mapper3 {
    rom: Rom,
    page: usize,
}

impl Mapper3 {
    pub fn new(rom: Rom) -> Self {
        Self { rom, page: 0 }
    }
}

impl Mapper for Mapper3 {
    fn read_prg(&mut self, addr: u16) -> u8 {
        let mask = if self.rom.header.prg_count() > 1 {
            0x7FFF
        } else {
            0x3FFF
        };
        self.rom.prg[(addr & mask) as usize]
    }

    fn write_prg(&mut self, addr: u16, data: u8) {
        if let 0x8000..=0xFFFF = addr {
            self.page = (data & 0x3) as usize;
        }
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        if self.rom.header.chr_count() == 0 {
            return self.rom.chr[addr as usize];
        }

        let mask = self.rom.header.chr_count() * CHR_PAGE_SIZE - 1;
        let index = self.page * CHR_PAGE_SIZE + addr as usize;
        self.rom.chr[index & mask]
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        if self.rom.header.chr_count() == 0 {
            self.rom.chr[addr as usize] = data;
        }
    }

    fn mirror_mode(&self) -> MirrorMode {
        self.rom.header.mirror_mode()
    }

    fn reset(&mut self) {
        self.page = 0;
    }
}
