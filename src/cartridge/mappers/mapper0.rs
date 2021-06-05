use std::fs::File;

use super::Mapper;
use crate::cartridge::{MirrorMode, Rom, RomMapper};
use crate::savable::Savable;

pub struct Mapper0 {
    rom: Rom,
    ram: Vec<u8>,
}

impl Mapper0 {
    pub fn new(rom: Rom) -> Self {
        Self {
            rom,
            ram: vec![0; 0x2000],
        }
    }
}

impl RomMapper for Mapper0 {}

impl Savable for Mapper0 {
    fn save(&self, output: &File) -> bincode::Result<()> {
        self.rom.save(output)?;
        for i in 0..0x2000 {
            bincode::serialize_into(output, &self.ram[i])?;
        }
        Ok(())
    }

    fn load(&mut self, input: &File) -> bincode::Result<()> {
        self.rom.load(input)?;
        for i in 0..0x2000 {
            self.ram[i] = bincode::deserialize_from(input)?;
        }
        Ok(())
    }
}

impl Mapper for Mapper0 {
    fn read_prg(&mut self, addr: u16) -> u8 {
        if let 0x6000..=0x7FFF = addr {
            return self.ram[(addr & 0x1FFF) as usize];
        }

        let mask = if self.rom.header.prg_count() > 1 {
            0x7FFF
        } else {
            0x3FFF
        };
        self.rom.prg[(addr & mask) as usize]
    }

    fn write_prg(&mut self, addr: u16, data: u8) {
        if let 0x6000..=0x7FFF = addr {
            self.ram[(addr & 0x1FFF) as usize] = data;
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

    fn reset(&mut self) {}
}
