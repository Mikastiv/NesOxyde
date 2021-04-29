use std::cell::RefCell;
use std::rc::Rc;

use crate::cartridge::{Cartridge, MirrorMode};
use crate::ppu::Interface;

const ROM_START: u16 = 0x0000;
const ROM_END: u16 = 0x1FFF;

const NTA_SIZE: u16 = 0x400;
const VRAM_SIZE: usize = 0x800;
const VRAM_START: u16 = 0x2000;
const VRAM_END: u16 = 0x3EFF;

const PALETTE_RAM_SIZE: usize = 0x20;
const PALETTE_START: u16 = 0x3F00;
const PALETTE_END: u16 = 0x3FFF;

pub struct PpuBus {
    cartridge: Rc<RefCell<Cartridge>>,
    pal_ram: [u8; PALETTE_RAM_SIZE],
    vram: [u8; VRAM_SIZE],
}

impl Interface for PpuBus {
    fn read(&self, addr: u16) -> u8 {
        let addr = addr & 0x3FFF;
        match addr {
            ROM_START..=ROM_END => self.cartridge.borrow_mut().read_chr(addr),
            VRAM_START..=VRAM_END => {
                let index = self.mirrored_vaddr(addr) as usize;
                self.vram[index]
            }
            PALETTE_START..=PALETTE_END => {
                let mut index = addr;
                if index % 4 == 0 {
                    index &= 0x0F;
                }
                index &= 0x1F;
                self.pal_ram[index as usize]
            }
            _ => unreachable!("Reached impossible match arm. (Ppu bus addr) {:#04X}", addr),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        let addr = addr & 0x3FFF;
        match addr {
            ROM_START..=ROM_END => self.cartridge.borrow_mut().write_chr(addr, data),
            VRAM_START..=VRAM_END => {
                let index = self.mirrored_vaddr(addr) as usize;
                self.vram[index] = data;
            }
            PALETTE_START..=PALETTE_END => {
                let mut index = addr;
                if index % 4 == 0 {
                    index &= 0x0F;
                }
                index &= 0x1F;
                self.pal_ram[index as usize] = data;
            }
            _ => unreachable!("Reached impossible match arm. (Ppu bus addr) {:#04X}", addr),
        }
    }
}

impl PpuBus {
    pub fn new(cartridge: Rc<RefCell<Cartridge>>) -> Self {
        Self {
            cartridge,
            pal_ram: [0; PALETTE_RAM_SIZE],
            vram: [0; VRAM_SIZE],
        }
    }

    fn mirrored_vaddr(&self, addr: u16) -> u16 {
        let addr = addr & 0x2FFF;
        let index = addr - VRAM_START;
        let nta = index / NTA_SIZE;
        match self.cartridge.borrow().mirror_mode() {
            MirrorMode::Vertical => match nta {
                2 | 3 => index - (NTA_SIZE * 2),
                0 | 1 => index,
                _ => unreachable!(
                    "Reached impossible match arm. Nametable (id: {}, addr: {})",
                    nta, addr
                ),
            },
            MirrorMode::Horizontal => match nta {
                1 | 2 => index - NTA_SIZE,
                3 => index - (NTA_SIZE * 2),
                0 => index,
                _ => unreachable!(
                    "Reached impossible match arm. Nametable (id: {}, addr: {})",
                    nta, addr
                ),
            },
            MirrorMode::OneScreenLo => index & 0x3FF,
            MirrorMode::OneScreenHi => (index & 0x3FF) + NTA_SIZE,
        }
    }
}
