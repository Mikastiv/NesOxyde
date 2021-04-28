use core::panic;
use std::fmt::Display;
use std::io;
use std::path::Path;

use mappers::{Mapper, Mapper0};
use rom::Rom;

mod mappers;
mod rom;

#[derive(Debug)]
pub enum MirrorMode {
    Vertical,
    Horizontal,
}

pub struct Cartridge {
    mapper: Box<dyn Mapper>,
}

impl Cartridge {
    pub fn new<P: AsRef<Path> + Display>(romfile: P) -> io::Result<Self> {
        let rom = Rom::new(romfile)?;
        let mapper = match rom.header.mapper_id() {
            0 => Mapper0::new(rom),
            _ => panic!("Unimplemented mapper: {}", rom.header.mapper_id()),
        };

        Ok(Self {
            mapper: Box::new(mapper),
        })
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
}
