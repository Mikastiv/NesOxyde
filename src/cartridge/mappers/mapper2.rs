use std::fs::File;

use crate::cartridge::rom::PRG_PAGE_SIZE;
use crate::cartridge::{MirrorMode, Rom, RomMapper};
use crate::savable::Savable;

use super::Mapper;

pub struct Mapper2 {
    rom: Rom,
    bank: usize,
}

impl Mapper2 {
    pub fn new(rom: Rom) -> Self {
        Self { rom, bank: 0 }
    }
}

impl RomMapper for Mapper2 {}

impl Savable for Mapper2 {
    fn save(&self, output: &File) -> bincode::Result<()> {
        self.rom.save(output)?;
        bincode::serialize_into(output, &self.bank)?;
        Ok(())
    }

    fn load(&mut self, input: &File) -> bincode::Result<()> {
        self.rom.load(input)?;
        self.bank = bincode::deserialize_from(input)?;
        Ok(())
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
                let index = self.bank * PRG_PAGE_SIZE + (addr & 0x3FFF) as usize;
                self.rom.prg[index]
            }
        }
    }

    fn write_prg(&mut self, addr: u16, data: u8) {
        if let 0x8000..=0xFFFF = addr {
            self.bank = (data & 0xF) as usize;
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
        self.bank = 0;
        self.rom.chr.fill(0);
    }
}
