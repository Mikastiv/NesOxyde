use std::fs::File;

use crate::cartridge::{MirrorMode, Rom, RomMapper};
use crate::savable::Savable;

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

impl RomMapper for Mapper7 {}

impl Savable for Mapper7 {
    fn save(&self, output: &File) -> bincode::Result<()> {
        self.rom.save(output)?;
        bincode::serialize_into(output, &self.bank)?;
        bincode::serialize_into(output, &self.mirror_mode)?;
        Ok(())
    }

    fn load(&mut self, input: &File) -> bincode::Result<()> {
        self.rom.load(input)?;
        self.bank = bincode::deserialize_from(input)?;
        self.mirror_mode = bincode::deserialize_from(input)?;
        Ok(())
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
