use registers::{Controller, Mask, Status};

mod registers;

const PALETTE_RAM_SIZE: usize = 0x20;
const VRAM_SIZE: usize = 0x800;
const OAM_SIZE: usize = 0x100;

pub trait Interface {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
}

pub struct Ppu {
    ctrl: Controller,
    mask: Mask,
    status: Status,

    bus: Box<dyn Interface>,
    pal_ram: [u8; PALETTE_RAM_SIZE],
    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],
}

impl Ppu {
    pub fn new(bus: Box<dyn Interface>) -> Self {
        Self {
            ctrl: Controller::from_bits_truncate(0),
            mask: Mask::from_bits_truncate(0),
            status: Status::from_bits_truncate(0),

            bus,
            pal_ram: [0; PALETTE_RAM_SIZE],
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],
        }
    }
}
