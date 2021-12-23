use std::fs::File;
use std::io::{BufReader, BufWriter};

use crate::cartridge::{MirrorMode, Rom, RomMapper};
use crate::savable::Savable;

use super::Mapper;

pub struct Mapper9 {
    rom: Rom,

    latch0: bool,
    latch1: bool,

    prg_bank: usize,
    prg_fixed0: usize,
    prg_fixed1: usize,
    prg_fixed2: usize,

    chr_lo_fd: usize,
    chr_lo_fe: usize,
    chr_hi_fd: usize,
    chr_hi_fe: usize,

    mirror_mode: MirrorMode,

    ram: Vec<u8>,
}

impl Mapper9 {
    pub fn new(rom: Rom) -> Self {
        let prg_fixed0 = rom.header.prg_count() * 2 - 3;
        let prg_fixed1 = rom.header.prg_count() * 2 - 2;
        let prg_fixed2 = rom.header.prg_count() * 2 - 1;

        Self {
            rom,

            latch0: false,
            latch1: false,

            prg_bank: 0,
            prg_fixed0,
            prg_fixed1,
            prg_fixed2,

            chr_lo_fd: 0,
            chr_lo_fe: 0,
            chr_hi_fd: 0,
            chr_hi_fe: 0,

            mirror_mode: MirrorMode::Vertical,

            ram: vec![0; 0x2000],
        }
    }
}

impl RomMapper for Mapper9 {}

impl Savable for Mapper9 {
    fn save(&self, output: &mut BufWriter<File>) -> bincode::Result<()> {
        self.rom.save(output)?;
        bincode::serialize_into::<&mut BufWriter<File>, _>(output, &self.latch0)?;
        bincode::serialize_into::<&mut BufWriter<File>, _>(output, &self.latch1)?;
        bincode::serialize_into::<&mut BufWriter<File>, _>(output, &self.prg_bank)?;
        bincode::serialize_into::<&mut BufWriter<File>, _>(output, &self.prg_fixed0)?;
        bincode::serialize_into::<&mut BufWriter<File>, _>(output, &self.prg_fixed1)?;
        bincode::serialize_into::<&mut BufWriter<File>, _>(output, &self.prg_fixed2)?;
        bincode::serialize_into::<&mut BufWriter<File>, _>(output, &self.chr_lo_fd)?;
        bincode::serialize_into::<&mut BufWriter<File>, _>(output, &self.chr_lo_fe)?;
        bincode::serialize_into::<&mut BufWriter<File>, _>(output, &self.chr_hi_fd)?;
        bincode::serialize_into::<&mut BufWriter<File>, _>(output, &self.chr_hi_fe)?;
        bincode::serialize_into::<&mut BufWriter<File>, _>(output, &self.mirror_mode)?;
        for i in 0..0x2000 {
            bincode::serialize_into::<&mut BufWriter<File>, _>(output, &self.ram[i])?;
        }
        Ok(())
    }

    fn load(&mut self, input: &mut BufReader<File>) -> bincode::Result<()> {
        self.rom.load(input)?;
        self.latch0 = bincode::deserialize_from::<&mut BufReader<File>, _>(input)?;
        self.latch1 = bincode::deserialize_from::<&mut BufReader<File>, _>(input)?;
        self.prg_bank = bincode::deserialize_from::<&mut BufReader<File>, _>(input)?;
        self.prg_fixed0 = bincode::deserialize_from::<&mut BufReader<File>, _>(input)?;
        self.prg_fixed1 = bincode::deserialize_from::<&mut BufReader<File>, _>(input)?;
        self.prg_fixed2 = bincode::deserialize_from::<&mut BufReader<File>, _>(input)?;
        self.chr_lo_fd = bincode::deserialize_from::<&mut BufReader<File>, _>(input)?;
        self.chr_lo_fe = bincode::deserialize_from::<&mut BufReader<File>, _>(input)?;
        self.chr_hi_fd = bincode::deserialize_from::<&mut BufReader<File>, _>(input)?;
        self.chr_hi_fe = bincode::deserialize_from::<&mut BufReader<File>, _>(input)?;
        self.mirror_mode = bincode::deserialize_from::<&mut BufReader<File>, _>(input)?;
        for i in 0..0x2000 {
            self.ram[i] = bincode::deserialize_from::<&mut BufReader<File>, _>(input)?;
        }
        Ok(())
    }
}

impl Mapper for Mapper9 {
    fn read_prg(&mut self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.ram[(addr & 0x1FFF) as usize],
            0x8000..=0xFFFF => {
                let bank = match addr {
                    0x8000..=0x9FFF => self.prg_bank,
                    0xA000..=0xBFFF => self.prg_fixed0,
                    0xC000..=0xDFFF => self.prg_fixed1,
                    _ => self.prg_fixed2,
                };
                let index = bank * 0x2000 + (addr & 0x1FFF) as usize;
                self.rom.prg[index]
            }
            _ => 0,
        }
    }

    fn write_prg(&mut self, addr: u16, data: u8) {
        match addr {
            0x6000..=0x7FFF => self.ram[(addr & 0x1FFF) as usize] = data,
            0xA000..=0xAFFF => self.prg_bank = (data & 0xF) as usize,
            0xB000..=0xBFFF => self.chr_lo_fd = (data & 0x1F) as usize,
            0xC000..=0xCFFF => self.chr_lo_fe = (data & 0x1F) as usize,
            0xD000..=0xDFFF => self.chr_hi_fd = (data & 0x1F) as usize,
            0xE000..=0xEFFF => self.chr_hi_fe = (data & 0x1F) as usize,
            0xF000..=0xFFFF => match data & 0x1 != 0 {
                true => self.mirror_mode = MirrorMode::Horizontal,
                false => self.mirror_mode = MirrorMode::Vertical,
            },
            _ => {}
        }
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        let latch0 = self.latch0;
        let latch1 = self.latch1;

        match addr {
            0x0FD8 => self.latch0 = false,
            0x0FE8 => self.latch0 = true,
            0x1FD8..=0x1FDF => self.latch1 = false,
            0x1FE8..=0x1FEF => self.latch1 = true,
            _ => {}
        }

        let bank = match addr {
            0x0000..=0x0FFF => match latch0 {
                false => self.chr_lo_fd,
                true => self.chr_lo_fe,
            },
            0x1000..=0x1FFF => match latch1 {
                false => self.chr_hi_fd,
                true => self.chr_hi_fe,
            },
            _ => 0,
        };
        let index = bank * 0x1000 + (addr & 0xFFF) as usize;
        self.rom.chr[index]
    }

    fn write_chr(&mut self, _addr: u16, _data: u8) {}

    fn mirror_mode(&self) -> crate::cartridge::MirrorMode {
        self.mirror_mode
    }

    fn reset(&mut self) {
        self.mirror_mode = MirrorMode::Vertical;
        self.latch0 = false;
        self.latch1 = false;
    }
}
