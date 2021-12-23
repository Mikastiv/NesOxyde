use std::fs::File;
use std::io::{BufReader, BufWriter};

use crate::cartridge::rom::CHR_PAGE_SIZE;
use crate::cartridge::{MirrorMode, Rom, RomMapper};
use crate::savable::Savable;

use super::Mapper;

pub struct Mapper3 {
    rom: Rom,
    bank: usize,
}

impl Mapper3 {
    pub fn new(rom: Rom) -> Self {
        Self { rom, bank: 0 }
    }
}

impl RomMapper for Mapper3 {}

impl Savable for Mapper3 {
    fn save(&self, output: &mut BufWriter<File>) -> bincode::Result<()> {
        self.rom.save(output)?;
        bincode::serialize_into(output, &self.bank)?;
        Ok(())
    }

    fn load(&mut self, input: &mut BufReader<File>) -> bincode::Result<()> {
        self.rom.load(input)?;
        self.bank = bincode::deserialize_from(input)?;
        Ok(())
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
            self.bank = (data & 0x3) as usize;
        }
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        if self.rom.header.chr_count() == 0 {
            return self.rom.chr[addr as usize];
        }

        let mask = self.rom.header.chr_count() * CHR_PAGE_SIZE - 1;
        let index = self.bank * CHR_PAGE_SIZE + addr as usize;
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
        self.bank = 0;
    }
}
