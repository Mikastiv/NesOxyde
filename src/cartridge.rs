use core::panic;
use std::fmt::Display;
use std::io;
use std::path::Path;

use mappers::{Mapper, Mapper0, Mapper1, Mapper2, Mapper3, Mapper7};
use rom::Rom;

mod mappers;
mod rom;

#[derive(Debug, Clone, Copy)]
pub enum MirrorMode {
    Vertical,
    Horizontal,
    OneScreenLo,
    OneScreenHi,
}

pub struct Cartridge {
    mapper: Box<dyn Mapper>,
}

impl Cartridge {
    pub fn new<P: AsRef<Path> + Display>(romfile: P) -> io::Result<Self> {
        let rom = Rom::new(romfile)?;
        let mapper: Box<dyn Mapper> = match rom.header.mapper_id() {
            0 => Box::new(Mapper0::new(rom)),
            1 => Box::new(Mapper1::new(rom)),
            2 => Box::new(Mapper2::new(rom)),
            3 => Box::new(Mapper3::new(rom)),
            7 => Box::new(Mapper7::new(rom)),
            _ => panic!("Unimplemented mapper: {}", rom.header.mapper_id()),
        };

        Ok(Self { mapper })
    }

    pub fn read_prg(&mut self, addr: u16) -> u8 {
        self.mapper.read_prg(addr)
    }

    pub fn write_prg(&mut self, addr: u16, data: u8) {
        self.mapper.write_prg(addr, data);
    }

    pub fn read_chr(&mut self, addr: u16) -> u8 {
        self.mapper.read_chr(addr)
    }

    pub fn write_chr(&mut self, addr: u16, data: u8) {
        self.mapper.write_chr(addr, data);
    }

    pub fn mirror_mode(&self) -> MirrorMode {
        self.mapper.mirror_mode()
    }

    pub fn reset(&mut self) {
        self.mapper.reset();
    }
}
