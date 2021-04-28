use crate::cartridge::rom::PRG_PAGE_SIZE;
use crate::cartridge::{MirrorMode, Rom};

use super::Mapper;

pub struct Mapper2 {
    rom: Rom,
    page: usize,
}

impl Mapper2 {
    pub fn new(rom: Rom) -> Self {
        Self { rom, page: 0 }
    }
}

impl Mapper for Mapper2 {
    fn read_prg(&mut self, addr: u16) -> u8 {
        match addr {
            0xC000..=0xFFFF => {
                let index =
                    (self.rom.header.prg_count() - 1) * PRG_PAGE_SIZE + (addr & 0x3FFF) as usize;
                self.rom.prg[index]
            }
            _ => {
                let index = self.page * PRG_PAGE_SIZE + (addr & 0x3FFF) as usize;
                self.rom.prg[index]
            }
        }
    }

    fn write_prg(&mut self, addr: u16, data: u8) {
        if let 0x8000..=0xFFFF = addr {
            self.page = (data & 0xF) as usize;
        }
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        self.rom.chr[addr as usize]
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
        self.rom.chr.fill(0);
    }
}
