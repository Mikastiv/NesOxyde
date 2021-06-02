use std::fs::File;

use super::Mapper;
use crate::cartridge::{MirrorMode, Rom, RomMapper};
use crate::savable::Savable;

pub struct Mapper1 {
    rom: Rom,

    chr_lo: u8,
    chr_hi: u8,
    chr_8k: u8,
    prg_lo: u8,
    prg_hi: u8,
    prg_32k: u8,

    control: u8,
    count: u8,
    load: u8,

    ram: Vec<u8>,
    mirror_mode: MirrorMode,
}

impl Mapper1 {
    pub fn new(rom: Rom) -> Self {
        let prg_hi = (rom.header.prg_count() - 1) as u8;
        Self {
            rom,

            chr_lo: 0,
            chr_hi: 0,
            chr_8k: 0,
            prg_lo: 0,
            prg_hi,
            prg_32k: 0,

            control: 0x0C,
            count: 0,
            load: 0,

            ram: vec![0; 0x2000],
            mirror_mode: MirrorMode::Vertical,
        }
    }
}

impl RomMapper for Mapper1 {}

impl Savable for Mapper1 {
    fn save(&self, output: &File) -> bincode::Result<()> {
        bincode::serialize_into(output, &self.chr_lo)?;
        bincode::serialize_into(output, &self.chr_hi)?;
        bincode::serialize_into(output, &self.chr_8k)?;
        bincode::serialize_into(output, &self.prg_lo)?;
        bincode::serialize_into(output, &self.prg_hi)?;
        bincode::serialize_into(output, &self.prg_32k)?;
        bincode::serialize_into(output, &self.control)?;
        bincode::serialize_into(output, &self.count)?;
        bincode::serialize_into(output, &self.load)?;
        bincode::serialize_into(output, &self.mirror_mode)?;
        for i in 0..0x2000 {
            bincode::serialize_into(output, &self.ram[i])?;
        }
        Ok(())
    }

    fn load(&mut self, input: &File) -> bincode::Result<()> {
        self.chr_lo = bincode::deserialize_from(input)?;
        self.chr_hi = bincode::deserialize_from(input)?;
        self.chr_8k = bincode::deserialize_from(input)?;
        self.prg_lo = bincode::deserialize_from(input)?;
        self.prg_hi = bincode::deserialize_from(input)?;
        self.prg_32k = bincode::deserialize_from(input)?;
        self.control = bincode::deserialize_from(input)?;
        self.count = bincode::deserialize_from(input)?;
        self.load = bincode::deserialize_from(input)?;
        self.mirror_mode = bincode::deserialize_from(input)?;
        for i in 0..0x2000 {
            self.ram[i] = bincode::deserialize_from(input)?;
        }
        Ok(())
    }
}

impl Mapper for Mapper1 {
    fn read_prg(&mut self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.ram[(addr & 0x1FFF) as usize],
            0x8000..=0xFFFF => {
                let prg_16k_mode = self.control & 0x8 != 0;

                let index = match prg_16k_mode {
                    true => match addr {
                        0x8000..=0xBFFF => self.prg_lo as usize * 0x4000 + (addr & 0x3FFF) as usize,
                        _ => self.prg_hi as usize * 0x4000 + (addr & 0x3FFF) as usize,
                    },
                    false => self.prg_32k as usize * 0x8000 + (addr & 0x7FFF) as usize,
                };

                self.rom.prg[index]
            }
            _ => 0,
        }
    }

    fn write_prg(&mut self, addr: u16, data: u8) {
        match addr {
            0x6000..=0x7FFF => self.ram[(addr & 0x1FFF) as usize] = data,
            0x8000..=0xFFFF => match data & 0x80 != 0 {
                true => {
                    self.control |= 0x0C;
                    self.count = 0;
                    self.load = 0;
                }
                false => {
                    self.load |= (data & 0x1) << self.count;
                    self.count += 1;

                    if self.count == 5 {
                        let target = (addr >> 13) & 0x3;
                        let chr_4k_mode = self.control & 0x10 != 0;
                        match target {
                            0 => {
                                self.control = self.load & 0x1F;
                                self.mirror_mode = match self.control & 0x3 {
                                    0 => MirrorMode::OneScreenLo,
                                    1 => MirrorMode::OneScreenHi,
                                    2 => MirrorMode::Vertical,
                                    _ => MirrorMode::Horizontal,
                                };
                            }
                            1 => match chr_4k_mode {
                                true => self.chr_lo = self.load & 0x1F,
                                false => self.chr_8k = (self.load & 0x1E) >> 1,
                            },
                            2 => {
                                if chr_4k_mode {
                                    self.chr_hi = self.load & 0x1F;
                                }
                            }
                            _ => {
                                let prg_mode = (self.control >> 2) & 0x3;

                                match prg_mode {
                                    0 | 1 => self.prg_32k = (self.load & 0xE) >> 1,
                                    2 => {
                                        self.prg_lo = 0;
                                        self.prg_hi = self.load & 0xF;
                                    }
                                    _ => {
                                        self.prg_lo = self.load & 0xF;
                                        self.prg_hi = (self.rom.header.prg_count() - 1) as u8;
                                    }
                                }
                            }
                        }

                        self.count = 0;
                        self.load = 0;
                    }
                }
            },
            _ => {}
        }
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        let chr_4k_mode = self.control & 0x10 != 0;

        if self.rom.header.chr_count() == 0 {
            return self.rom.chr[addr as usize];
        }

        let index = match chr_4k_mode {
            true => match addr {
                0x0000..=0x0FFF => self.chr_lo as usize * 0x1000 + (addr & 0xFFF) as usize,
                0x1000..=0x1FFF => self.chr_hi as usize * 0x1000 + (addr & 0xFFF) as usize,
                _ => 0,
            },
            false => self.chr_8k as usize * 0x2000 + (addr & 0x1FFF) as usize,
        };
        self.rom.chr[index]
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        if self.rom.header.chr_count() == 0 {
            self.rom.chr[addr as usize] = data;
        }
    }

    fn mirror_mode(&self) -> MirrorMode {
        self.mirror_mode
    }

    fn reset(&mut self) {
        self.mirror_mode = MirrorMode::Vertical;
        self.control = 0x0C;
        self.count = 0;
        self.load = 0;
        self.prg_hi = (self.rom.header.prg_count() - 1) as u8;
    }
}
