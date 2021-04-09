use registers::{Controller, Mask, Status};

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
    open_bus: u8,
    addr_toggle: bool,
    oam: [u8; OAM_SIZE],
}

impl Ppu {
    pub fn new(bus: Box<dyn Interface>) -> Self {
        Self {
            ctrl: Controller::from_bits_truncate(0),
            mask: Mask::from_bits_truncate(0),
            status: Status::from_bits_truncate(0),

            bus,
            open_bus: 0,
            addr_toggle: false,
            oam: [0; OAM_SIZE],
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        let mut data = self.open_bus;
        match addr {
            PPU_CTRL => {},
            PPU_MASK => {},
            PPU_STATUS => {
                data = self.status.bits() | (self.open_bus & 0x1F);
                self.status.remove(Status::IN_VBLANK);
                self.addr_toggle = false;
            },
            OAM_ADDR => {},
            OAM_DATA => {},
            PPU_SCROLL => {},
            PPU_ADDR => {},
            PPU_DATA => {},
            _ => {},
        }
        data
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.open_bus = data;
        match addr {
            PPU_CTRL => {
                self.ctrl.set_raw(data);
            },
            PPU_MASK => {
                self.mask.set_raw(data);
            },
            PPU_STATUS => {},
            OAM_ADDR => {},
            OAM_DATA => {},
            PPU_SCROLL => {},
            PPU_ADDR => {},
            PPU_DATA => {},
            _ => {},
        }
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        self.bus.read(addr)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.write(addr, data);
    }
}
