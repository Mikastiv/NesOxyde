use registers::{Controller, Loopy, Mask, Status};

mod registers;

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
