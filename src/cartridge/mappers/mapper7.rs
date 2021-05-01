use crate::cartridge::{MirrorMode, Rom};

use super::Mapper;

pub struct Mapper7 {
    rom: Rom,

    bank: usize,
    mirror_mode: MirrorMode,
}

impl Mapper7 {
    pub fn new(rom: Rom) -> Self {
        Self {
            rom,
            bank: 0,
            mirror_mode: MirrorMode::OneScreenLo,
        }
    }
}

impl Mapper for Mapper7 {
    fn read_prg(&mut self, addr: u16) -> u8 {
        let index = self.bank * 0x8000 + (addr & 0x7FFF) as usize;
        self.rom.prg[index]
    }

    fn write_prg(&mut self, addr: u16, data: u8) {
        if let 0x8000..=0xFFFF = addr {
            self.bank = (data & 0x7) as usize;
            match (data >> 4) & 0x1 != 0 {
                true => self.mirror_mode = MirrorMode::OneScreenHi,
                false => self.mirror_mode = MirrorMode::OneScreenLo,
            }
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

    fn mirror_mode(&self) -> crate::cartridge::MirrorMode {
        self.mirror_mode
    }

    fn reset(&mut self) {
        self.bank = 0;
        self.mirror_mode = MirrorMode::OneScreenLo;
    }
}
