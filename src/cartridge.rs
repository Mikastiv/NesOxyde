use core::panic;
use std::fmt::Display;
use std::fs::File;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::savable::Savable;
use mappers::{Mapper, Mapper0, Mapper1, Mapper10, Mapper2, Mapper3, Mapper4, Mapper7, Mapper9};
use rom::Rom;

mod mappers;
mod rom;

/// Mirroring modes for the VRAM
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MirrorMode {
    Vertical,
    Horizontal,
    OneScreenLo,
    OneScreenHi,
    FourScreen,
}

pub trait RomMapper: Mapper + Savable {}

/// NES ROM cartridge
pub struct Cartridge {
    mapper: Box<dyn RomMapper>,
    filename: Option<String>,
}

impl Cartridge {
    pub fn new<P: AsRef<Path> + Display>(romfile: P) -> io::Result<Self> {
        let filename = romfile
            .as_ref()
            .file_stem()
            .map(|name| name.to_string_lossy().to_string());

        let rom = Rom::new(romfile)?;
        let mapper: Box<dyn RomMapper> = match rom.header.mapper_id() {
            0 => Box::new(Mapper0::new(rom)),
            1 => Box::new(Mapper1::new(rom)),
            2 => Box::new(Mapper2::new(rom)),
            3 => Box::new(Mapper3::new(rom)),
            4 => Box::new(Mapper4::new(rom)),
            7 => Box::new(Mapper7::new(rom)),
            9 => Box::new(Mapper9::new(rom)),
            10 => Box::new(Mapper10::new(rom)),
            _ => panic!("Unimplemented mapper: {}", rom.header.mapper_id()),
        };

        Ok(Self { mapper, filename })
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

    pub fn inc_scanline(&mut self) {
        self.mapper.inc_scanline();
    }

    pub fn poll_irq(&mut self) -> bool {
        self.mapper.poll_irq()
    }

    pub fn filename(&self) -> String {
        match self.filename {
            Some(ref name) => name.clone(),
            None => "".to_string(),
        }
    }

    pub fn save(&self, output: &File) -> bincode::Result<()> {
        self.mapper.save(output)
    }

    pub fn load(&mut self, input: &File) -> bincode::Result<()> {
        self.mapper.load(input)
    }
}
