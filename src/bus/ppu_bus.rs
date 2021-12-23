use std::cell::RefCell;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::rc::Rc;

use crate::cartridge::{Cartridge, MirrorMode};
use crate::ppu::{self, PpuInterface};
use crate::savable::Savable;

/// First address of the ROM memory space
const ROM_START: u16 = 0x0000;
/// Last address of the ROM memory space
const ROM_END: u16 = 0x1FFF;

/// Size of one nametable
const NTA_SIZE: u16 = 0x400;
/// Size of the VRAM (Doubled to handle some games which use 4screen mapping)
const VRAM_SIZE: usize = 0x800 * 2;
/// First address of the VRAM memory space
const VRAM_START: u16 = 0x2000;
/// Last address of the VRAM memory space
const VRAM_END: u16 = 0x3EFF;

/// Size of the palette RAM
const PALETTE_RAM_SIZE: usize = 0x20;
/// First address of the palette RAM memory space
const PALETTE_START: u16 = 0x3F00;
/// Last address of the palette RAM memory space
const PALETTE_END: u16 = 0x3FFF;

/// Memory bus of the Ppu
pub struct PpuBus {
    cartridge: Rc<RefCell<Cartridge>>,
    pal_ram: [u8; PALETTE_RAM_SIZE],
    vram: [u8; VRAM_SIZE],
}

impl PpuInterface for PpuBus {}

impl Savable for PpuBus {
    fn save(&self, output: &mut BufWriter<File>) -> bincode::Result<()> {
        for i in 0..PALETTE_RAM_SIZE {
            bincode::serialize_into::<&mut BufWriter<File>, _>(output, &self.pal_ram[i])?;
        }
        for i in 0..VRAM_SIZE {
            bincode::serialize_into::<&mut BufWriter<File>, _>(output, &self.vram[i])?;
        }
        Ok(())
    }

    fn load(&mut self, input: &mut BufReader<File>) -> bincode::Result<()> {
        for i in 0..PALETTE_RAM_SIZE {
            self.pal_ram[i] = bincode::deserialize_from::<&mut BufReader<File>, _>(input)?;
        }
        for i in 0..VRAM_SIZE {
            self.vram[i] = bincode::deserialize_from::<&mut BufReader<File>, _>(input)?;
        }
        Ok(())
    }
}

impl ppu::Interface for PpuBus {
    fn read(&self, addr: u16) -> u8 {
        // The ppu bus only maps from 0x0000 to 0x3FFF;
        let addr = addr & 0x3FFF;
        match addr {
            // ROM memory space: read from CHR ROM on the cartridge
            ROM_START..=ROM_END => self.cartridge.borrow_mut().read_chr(addr),
            // VRAM memory space: read from VRAM
            VRAM_START..=VRAM_END => {
                // Mirror the address first
                let index = self.mirrored_vaddr(addr) as usize;
                self.vram[index]
            }
            // Palette RAM memory space:
            PALETTE_START..=PALETTE_END => {
                let mut index = addr;
                // This check is because
                // 0x3F10 == 0x3F00
                // 0x3F14 == 0x3F04
                // 0x3F18 == 0x3F08
                // 0x3F1C == 0x3F0C
                if index % 4 == 0 {
                    index &= 0x0F;
                }
                // Palette mirrors every 0x20 (32)
                index &= 0x1F;
                self.pal_ram[index as usize]
            }
            _ => unreachable!("Reached impossible match arm. (Ppu bus addr) {:#04X}", addr),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        // The ppu bus only maps from 0x0000 to 0x3FFF;
        let addr = addr & 0x3FFF;
        match addr {
            // ROM memory space: read from CHR ROM on the cartridge
            ROM_START..=ROM_END => self.cartridge.borrow_mut().write_chr(addr, data),
            // VRAM memory space: read from VRAM
            VRAM_START..=VRAM_END => {
                // Mirror the address first
                let index = self.mirrored_vaddr(addr) as usize;
                self.vram[index] = data;
            }
            // Palette RAM memory space:
            PALETTE_START..=PALETTE_END => {
                let mut index = addr;
                // This check is because
                // 0x3F10 == 0x3F00
                // 0x3F14 == 0x3F04
                // 0x3F18 == 0x3F08
                // 0x3F1C == 0x3F0C
                if index % 4 == 0 {
                    index &= 0x0F;
                }
                // Palette mirrors every 0x20 (32)
                index &= 0x1F;
                self.pal_ram[index as usize] = data;
            }
            _ => unreachable!("Reached impossible match arm. (Ppu bus addr) {:#04X}", addr),
        }
    }

    // Signals a new scanline was rendered to the cartridge
    fn inc_scanline(&mut self) {
        self.cartridge.borrow_mut().inc_scanline()
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

    /// Returns the address mirrored based on the current mirroring mode
    fn mirrored_vaddr(&self, addr: u16) -> u16 {
        // Mask because 0x2000 - 0x2FFF mirrors 0x3000 - 0x3EFF
        let addr = addr & 0x2FFF;
        // Substract the memory map offset to have real memory index
        let index = addr - VRAM_START;
        // Calculate which nametable we are in
        let nta = index / NTA_SIZE;
        match self.cartridge.borrow().mirror_mode() {
            // |---------|---------|  |---------|---------|
            // |         |         |  |         |         |
            // |  0 - A  |  1 - B  |  |    0    |    1    | The hardware has space for only 2 nametables
            // |         |         |  |         |         |
            // |---------|---------|  |---------|---------|
            // |         |         |
            // |  2 - A  |  3 - B  |
            // |         |         |
            // |---------|---------|
            // Here 2 mirrors 0 and 3 mirrors 1
            // I simply substract the size of the first two nametables if we are in 2 or 3.
            // Otherwise I return the index because nametables 0 and 1 are already mapped correctly
            MirrorMode::Vertical => match nta {
                2 | 3 => index - (NTA_SIZE * 2),
                _ => index,
            },
            // |---------|---------|  |---------|---------|
            // |         |         |  |         |         |
            // |  0 - A  |  1 - A  |  |    0    |    1    | The hardware has space for only 2 nametables
            // |         |         |  |         |         |
            // |---------|---------|  |---------|---------|
            // |         |         |
            // |  2 - B  |  3 - B  |
            // |         |         |
            // |---------|---------|
            // Here 1 mirrors 0 and 3 mirrors 2.
            // I want to map nametable 0 to hardware nametable 0 and nametable 2 to hardware nametable 1.
            // Nametable 0 is already mapped
            // Because nametable 1 mirrors 0, I can simply substract the nametable size.
            // Then I want to map nametable 2 to hardware nametable 1, so I also can substract the size.
            // Finally for nametable 3, because it is a mirror of nametable 2, it should map onto hardware nametable 1.
            // So I substract twice the size of a nametable
            MirrorMode::Horizontal => match nta {
                1 | 2 => index - NTA_SIZE,
                3 => index - (NTA_SIZE * 2),
                _ => index,
            },
            // |---------|---------|  |---------|---------|
            // |         |         |  |         |         |
            // |  0 - A  |  1 - A  |  |    0    |    1    | The hardware has space for only 2 nametables
            // |         |         |  |         |         |
            // |---------|---------|  |---------|---------|
            // |         |         |
            // |  2 - A  |  3 - A  |
            // |         |         |
            // |---------|---------|
            // This setting maps everthing to hardware nametable 0
            MirrorMode::OneScreenLo => index & 0x3FF,
            // |---------|---------|  |---------|---------|
            // |         |         |  |         |         |
            // |  0 - A  |  1 - A  |  |    0    |    1    | The hardware has space for only 2 nametables
            // |         |         |  |         |         |
            // |---------|---------|  |---------|---------|
            // |         |         |
            // |  2 - A  |  3 - A  |
            // |         |         |
            // |---------|---------|
            // This setting maps everthing to hardware nametable 1.
            // I simply add the size after masking the address
            MirrorMode::OneScreenHi => (index & 0x3FF) + NTA_SIZE,
            // |---------|---------|  |---------|---------|
            // |         |         |  |         |         |
            // |  0 - A  |  1 - B  |  |    0    |    1    | The hardware has space for only 2 nametables
            // |         |         |  |         |         |
            // |---------|---------|  |---------|---------|
            // |         |         |  |         |         |
            // |  2 - C  |  3 - D  |  |    2    |    3    | The extra nametables were on the cartridge PCB
            // |         |         |  |         |         |
            // |---------|---------|  |---------|---------|
            // Real hardware would use memory on the cartridge but, I simply
            // allocated a Vec of twice the size of VRAM and use the index directly
            MirrorMode::FourScreen => index,
        }
    }
}
