use crate::cartridge::{MirrorMode, Rom};

use super::Mapper;

pub struct Mapper4 {
    rom: Rom,

    target: u8,
    prg_mode: bool,
    chr_invert: bool,
    mirror_mode: MirrorMode,

    registers: [u8; 8],
    prg_banks: [usize; 4],
    chr_banks: [usize; 8],

    irq_reload: u8,
    irq_counter: u8,
    irq_enable: bool,
    pending_irq: Option<bool>,

    ram: Vec<u8>,
}

impl Mapper4 {
    pub fn new(rom: Rom) -> Self {
        Self {
            rom,

            target: 0,
            prg_mode: false,
            chr_invert: false,
            mirror_mode: MirrorMode::Horizontal,

            registers: [0; 8],
            prg_banks: [0; 4],
            chr_banks: [0; 8],

            irq_reload: 0,
            irq_counter: 0,
            irq_enable: false,
            pending_irq: None,

            ram: vec![0; 0x2000],
        }
    }
}

impl Mapper for Mapper4 {
    fn read_prg(&mut self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.ram[(addr & 0x1FFF) as usize],
            0x8000..=0xFFFF => {
                let reg_index = match addr {
                    0x8000..=0x9FFF => 0,
                    0xA000..=0xBFFF => 1,
                    0xC000..=0xDFFF => 2,
                    0xE000..=0xFFFF => 3,
                    _ => 0,
                };
                let index = self.prg_banks[reg_index] + (addr & 0x1FFF) as usize;
                self.rom.prg[index]
            }
            _ => 0,
        }
    }

    fn write_prg(&mut self, addr: u16, data: u8) {
        let even = addr & 0x1 == 0;
        match addr {
            0x6000..=0x7FFF => self.ram[(addr & 0x1FFF) as usize] = data,
            0x8000..=0x9FFF if even => {
                self.target = data & 0x7;
                self.prg_mode = data & 0x40 != 0;
                self.chr_invert = data & 0x80 != 0;
            }
            0x8000..=0x9FFF => {
                self.registers[self.target as usize] = data;

                match self.chr_invert {
                    true => {
                        self.chr_banks[0] = self.registers[2] as usize * 0x400;
                        self.chr_banks[1] = self.registers[3] as usize * 0x400;
                        self.chr_banks[2] = self.registers[4] as usize * 0x400;
                        self.chr_banks[3] = self.registers[5] as usize * 0x400;
                        self.chr_banks[4] = (self.registers[0] & 0xFE) as usize * 0x400;
                        self.chr_banks[5] = (self.registers[0] & 0xFE) as usize * 0x400 + 0x400;
                        self.chr_banks[6] = (self.registers[1] & 0xFE) as usize * 0x400;
                        self.chr_banks[7] = (self.registers[1] & 0xFE) as usize * 0x400 + 0x400;
                    }
                    false => {
                        self.chr_banks[0] = (self.registers[0] & 0xFE) as usize * 0x400;
                        self.chr_banks[1] = (self.registers[0] & 0xFE) as usize * 0x400 + 0x400;
                        self.chr_banks[2] = (self.registers[1] & 0xFE) as usize * 0x400;
                        self.chr_banks[3] = (self.registers[1] & 0xFE) as usize * 0x400 + 0x400;
                        self.chr_banks[4] = self.registers[2] as usize * 0x400;
                        self.chr_banks[5] = self.registers[3] as usize * 0x400;
                        self.chr_banks[6] = self.registers[4] as usize * 0x400;
                        self.chr_banks[7] = self.registers[5] as usize * 0x400;
                    }
                }

                match self.prg_mode {
                    true => {
                        self.prg_banks[0] = (self.rom.header.prg_count() * 2 - 2) * 0x2000;
                        self.prg_banks[2] = (self.registers[6] & 0x3F) as usize * 0x2000;
                    }
                    false => {
                        self.prg_banks[0] = (self.registers[6] & 0x3F) as usize * 0x2000;
                        self.prg_banks[2] = (self.rom.header.prg_count() * 2 - 2) * 0x2000;
                    }
                }

                self.prg_banks[1] = (self.registers[7] & 0x3F) as usize * 0x2000;
            }
            0xA000..=0xBFFF if even => match data & 0x1 != 0 {
                true => self.mirror_mode = MirrorMode::Horizontal,
                false => self.mirror_mode = MirrorMode::Vertical,
            },
            0xC000..=0xDFFF if even => self.irq_reload = data,
            0xC000..=0xDFFF => self.irq_counter = 0,
            0xE000..=0xFFFF if even => {
                self.irq_enable = false;
                self.pending_irq = None;
            }
            0xE000..=0xFFFF => self.irq_enable = true,
            _ => {}
        }
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        if self.rom.header.chr_count() == 0 {
            return self.rom.chr[addr as usize];
        }
        
        let reg_index = match addr {
            0x0000..=0x03FF => 0,
            0x0400..=0x07FF => 1,
            0x0800..=0x0BFF => 2,
            0x0C00..=0x0FFF => 3,
            0x1000..=0x13FF => 4,
            0x1400..=0x17FF => 5,
            0x1800..=0x1BFF => 6,
            0x1C00..=0x1FFF => 7,
            _ => 0,
        };
        let index = self.chr_banks[reg_index] + (addr & 0x3FF) as usize;
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
        self.target = 0;
        self.prg_mode = false;
        self.chr_invert = false;
        self.mirror_mode = MirrorMode::Horizontal;

        self.irq_reload = 0;
        self.irq_counter = 0;
        self.irq_enable = false;
        self.pending_irq = None;

        self.registers.fill(0);
        self.chr_banks.fill(0);

        self.prg_banks[0] = 0;
        self.prg_banks[1] = 0x2000;
        self.prg_banks[2] = (self.rom.header.prg_count() * 2 - 2) as usize * 0x2000;
        self.prg_banks[3] = (self.rom.header.prg_count() * 2 - 1) as usize * 0x2000;
    }

    fn inc_scanline(&mut self) {
        match self.irq_counter == 0 {
            true => self.irq_counter = self.irq_reload,
            false => self.irq_counter -= 1,
        }

        if self.irq_counter == 0 && self.irq_enable {
            self.pending_irq = Some(true);
        }
    }

    fn poll_irq(&mut self) -> bool {
        self.pending_irq.take().is_some()
    }
}
