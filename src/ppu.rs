use registers::{Controller, Loopy, Mask, Status};

mod registers;

struct Pixel(u8, u8, u8);

#[rustfmt::skip]
static NES_PALETTE: [Pixel; 0x40] = [
    Pixel(84, 84, 84),    Pixel(0, 30, 116),    Pixel(8, 16, 144),    Pixel(48, 0, 136),    Pixel(68, 0, 100),    Pixel(92, 0, 48),     Pixel(84, 4, 0),      Pixel(60, 24, 0),
    Pixel(32, 42, 0),     Pixel(8, 58, 0),      Pixel(0, 64, 0),      Pixel(0, 60, 0),      Pixel(0, 50, 60),     Pixel(0, 0, 0),       Pixel(0, 0, 0),       Pixel(0, 0, 0),

    Pixel(152, 150, 152), Pixel(8, 76, 196),    Pixel(48, 50, 236),   Pixel(92, 30, 228),   Pixel(136, 20, 176),  Pixel(160, 20, 100),  Pixel(152, 34, 32),   Pixel(120, 60, 0),
    Pixel(84, 90, 0),     Pixel(40, 114, 0),    Pixel(8, 124, 0),     Pixel(0, 118, 40),    Pixel(0, 102, 120),   Pixel(0, 0, 0),       Pixel(0, 0, 0),       Pixel(0, 0, 0),

    Pixel(236, 238, 236), Pixel(76, 154, 236),  Pixel(120, 124, 236), Pixel(176, 98, 236),  Pixel(228, 84, 236),  Pixel(236, 88, 180),  Pixel(236, 106, 100), Pixel(212, 136, 32),
    Pixel(160, 170, 0),   Pixel(116, 196, 0),   Pixel(76, 208, 32),   Pixel(56, 204, 108),  Pixel(56, 180, 204),  Pixel(60, 60, 60),    Pixel(0, 0, 0),       Pixel(0, 0, 0),

    Pixel(236, 238, 236), Pixel(168, 204, 236), Pixel(188, 188, 236), Pixel(212, 178, 236), Pixel(236, 174, 236), Pixel(236, 174, 212), Pixel(236, 180, 176), Pixel(228, 196, 144),
    Pixel(204, 210, 120), Pixel(180, 222, 120), Pixel(168, 226, 144), Pixel(152, 226, 180), Pixel(160, 214, 228), Pixel(160, 162, 160), Pixel(0, 0, 0),       Pixel(0, 0, 0),
];

const PPU_CTRL: u16 = 0x0;
const PPU_MASK: u16 = 0x1;
const PPU_STATUS: u16 = 0x2;
const OAM_ADDR: u16 = 0x3;
const OAM_DATA: u16 = 0x4;
const PPU_SCROLL: u16 = 0x5;
const PPU_ADDR: u16 = 0x6;
const PPU_DATA: u16 = 0x7;

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
    pending_nmi: bool,
    open_bus: u8,
    oam: [u8; OAM_SIZE],

    addr_toggle: bool,
    read_buffer: u8,
    xfine: u8,
    v_addr: Loopy,
    scroll: Loopy,
}

impl Ppu {
    pub fn new(bus: Box<dyn Interface>) -> Self {
        Self {
            ctrl: Controller::from_bits_truncate(0),
            mask: Mask::from_bits_truncate(0),
            status: Status::from_bits_truncate(0),

            bus,
            pending_nmi: false,
            open_bus: 0,
            oam: [0; OAM_SIZE],

            addr_toggle: false,
            read_buffer: 0,
            xfine: 0,
            v_addr: Loopy::new(),
            scroll: Loopy::new(),
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        let mut data = self.open_bus;
        match addr {
            PPU_CTRL => {}
            PPU_MASK => {}
            PPU_STATUS => {
                data = self.status.bits() | (self.open_bus & 0x1F);
                self.status.remove(Status::IN_VBLANK);
                self.addr_toggle = false;
            }
            OAM_ADDR => {}
            OAM_DATA => {}
            PPU_SCROLL => {}
            PPU_ADDR => {}
            PPU_DATA => {
                data = self.read_buffer;
                self.read_buffer = self.mem_read(self.v_addr.raw());
                if (self.v_addr.raw() & 0x3F00) == 0x3F00 {
                    data = (self.open_bus & 0xC0) | (self.read_buffer & 0x3F);
                }
                self.open_bus = data;
                self.increment_addr();
            }
            _ => {}
        }
        data
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.open_bus = data;
        match addr {
            PPU_CTRL => {
                self.ctrl.set_raw(data);
            }
            PPU_MASK => {
                self.mask.set_raw(data);
            }
            PPU_STATUS => {}
            OAM_ADDR => {}
            OAM_DATA => {}
            PPU_SCROLL => {
                match self.addr_toggle {
                    true => {
                        self.scroll.set_yfine(data & 0x3);
                        self.scroll.set_ycoarse(data >> 3);
                    }
                    false => {
                        self.xfine = data & 0x3;
                        self.scroll.set_xcoarse(data >> 3);
                    }
                }
                self.addr_toggle = !self.addr_toggle;
            }
            PPU_ADDR => {
                match self.addr_toggle {
                    true => {
                        self.scroll.set_addr_lo(data);
                        self.v_addr = self.scroll;
                    }
                    false => self.scroll.set_addr_hi(data & 0x3F),
                }
                self.addr_toggle = !self.addr_toggle;
            }
            PPU_DATA => {
                self.mem_write(self.v_addr.raw(), data);
                self.increment_addr();
            }
            _ => {}
        }
    }

    pub fn poll_nmi(&mut self) -> bool {
        let nmi = self.pending_nmi;
        self.pending_nmi = false;
        nmi
    }

    pub fn clock(&mut self) {}

    fn increment_addr(&mut self) {
        let new_addr = self.v_addr.raw().wrapping_add(self.ctrl.increment());
        self.v_addr.set_raw(new_addr);
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        self.bus.read(addr)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.write(addr, data);
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::bus::PpuBus;
    use crate::cartridge::Cartridge;

    use super::*;

    fn get_ppu() -> Ppu {
        let cart = Cartridge::new("roms/nestest.nes").unwrap();
        let bus = PpuBus::new(Rc::new(RefCell::new(cart)));
        Ppu::new(Box::new(bus))
    }
}
