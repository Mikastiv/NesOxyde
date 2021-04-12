use registers::{Controller, Loopy, Mask, Status};

use self::frame::Frame;
use self::tile::Tile;

pub mod frame;
mod registers;
mod tile;

#[derive(Clone, Copy)]
pub struct Pixel(u8, u8, u8);

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

pub struct Ppu<'a> {
    ctrl: Controller,
    mask: Mask,
    status: Status,

    bus: Box<dyn Interface>,
    pending_nmi: Option<bool>,
    open_bus: u8,
    oam: [u8; OAM_SIZE],

    addr_toggle: bool,
    read_buffer: u8,
    xfine: u8,
    v_addr: Loopy,
    scroll: Loopy,

    scanline: i32,
    cycle: usize,
    curr_tile: Tile,
    next_tile: Tile,
    bg_lo_shift: u16,
    bg_hi_shift: u16,
    bg_attr_lo_shift: u16,
    bg_attr_hi_shift: u16,

    frame: Frame,
    odd_frame: bool,
    render_fn: Box<dyn FnMut(&[u8]) + 'a>,
}

impl<'a> Ppu<'a> {
    pub fn new<F>(bus: Box<dyn Interface>, render_fn: Box<F>) -> Self
    where
        F: FnMut(&[u8]) + 'a,
    {
        Self {
            ctrl: Controller::from_bits_truncate(0),
            mask: Mask::from_bits_truncate(0),
            status: Status::from_bits_truncate(0),

            bus,
            pending_nmi: None,
            open_bus: 0,
            oam: [0; OAM_SIZE],

            addr_toggle: false,
            read_buffer: 0,
            xfine: 0,
            v_addr: Loopy::new(),
            scroll: Loopy::new(),

            scanline: 0,
            cycle: 0,
            curr_tile: Tile::new(),
            next_tile: Tile::new(),
            bg_lo_shift: 0,
            bg_hi_shift: 0,
            bg_attr_lo_shift: 0,
            bg_attr_hi_shift: 0,

            frame: Frame::new(),
            odd_frame: false,
            render_fn,
        }
    }

    fn render_chr_pattern(&mut self) {
        for tile_y in 0..16 {
            for tile_x in 0..16 {
                let offset = tile_y * 256 + tile_x * 16;

                for row in 0..8 {
                    let mut lo_sp = self.mem_read(offset + row);
                    let mut hi_sp = self.mem_read(offset + row + 0x8);
                    let mut lo_bg = self.mem_read(0x1000 + offset + row);
                    let mut hi_bg = self.mem_read(0x1000 + offset + row + 0x8);

                    for col in (0..8).rev() {
                        let pixel_sp = (hi_sp & 0x1) << 1 | (lo_sp & 0x1);
                        let pixel_bg = (hi_bg & 0x1) << 1 | (lo_bg & 0x1);
                        lo_sp >>= 1;
                        hi_sp >>= 1;
                        lo_bg >>= 1;
                        hi_bg >>= 1;

                        let rgb_sp = match pixel_sp {
                            0 => NES_PALETTE[0x01],
                            1 => NES_PALETTE[0x23],
                            2 => NES_PALETTE[0x27],
                            3 => NES_PALETTE[0x30],
                            _ => unreachable!(),
                        };
                        let rgb_bg = match pixel_bg {
                            0 => NES_PALETTE[0x01],
                            1 => NES_PALETTE[0x23],
                            2 => NES_PALETTE[0x27],
                            3 => NES_PALETTE[0x30],
                            _ => unreachable!(),
                        };

                        self.frame.set_pixel(
                            (tile_x * 8 + col) as usize,
                            (tile_y * 8 + row) as usize,
                            rgb_sp,
                        );
                        self.frame.set_pixel(
                            (tile_x * 8 + col + 128) as usize,
                            (tile_y * 8 + row) as usize,
                            rgb_bg,
                        );
                    }
                }
            }
        }
    }

    fn render_nametable_0(&mut self) {
        for addr in 0..0x3C0 {
            let tile_id = self.mem_read(0x2000 | addr);
            let tile_addr = self.ctrl.bg_base_addr() + (tile_id as u16) * 16;
            let tile_x = addr % 32;
            let tile_y = addr / 32;
            let palette = self.get_bg_palette(tile_x, tile_y);

            for row in 0..8 {
                let mut lo = self.mem_read(tile_addr + row);
                let mut hi = self.mem_read(tile_addr + row + 0x8);

                for col in (0..8).rev() {
                    let pixel = (hi & 0x1) << 1 | (lo & 0x1);
                    lo >>= 1;
                    hi >>= 1;

                    let rgb = match pixel {
                        0 => NES_PALETTE[palette[0] as usize],
                        1 => NES_PALETTE[palette[1] as usize],
                        2 => NES_PALETTE[palette[2] as usize],
                        3 => NES_PALETTE[palette[3] as usize],
                        _ => unreachable!(),
                    };

                    self.frame.set_pixel(
                        (tile_x * 8 + col) as usize,
                        (tile_y * 8 + row) as usize,
                        rgb,
                    );
                }
            }
        }
    }

    fn get_bg_palette(&mut self, tile_x: u16, tile_y: u16) -> [u8; 4] {
        let attr_index = tile_y / 4 * 8 + tile_x / 4;
        let attr_byte = self.mem_read(0x23C0 + attr_index);

        let palette_idx = match (tile_x % 4 / 2, tile_y % 4 / 2) {
            (0, 0) => attr_byte & 0b11,
            (1, 0) => (attr_byte >> 2) & 0b11,
            (0, 1) => (attr_byte >> 4) & 0b11,
            (1, 1) => (attr_byte >> 6) & 0b11,
            _ => unreachable!(),
        };

        let start = 1 + palette_idx as u16 * 4;
        [
            self.mem_read(0x3F00),
            self.mem_read(0x3F00 + start),
            self.mem_read(0x3F00 + start + 1),
            self.mem_read(0x3F00 + start + 2),
        ]
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
                self.increment_vaddr();
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
                self.scroll.set_nta_h(self.ctrl.nta_h());
                self.scroll.set_nta_v(self.ctrl.nta_v());
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
                        self.scroll.set_yfine(data & 0x7);
                        self.scroll.set_ycoarse(data >> 3);
                    }
                    false => {
                        self.xfine = data & 0x7;
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
                self.increment_vaddr();
            }
            _ => {}
        }
    }

    pub fn poll_nmi(&mut self) -> Option<bool> {
        self.pending_nmi.take()
    }

    pub fn clock(&mut self) {
        if self.scanline >= -1 && self.scanline < 240 {
            if self.odd_frame && self.scanline == 0 && self.cycle == 0 && self.rendering_enabled() {
                self.cycle = 1;
            }

            if self.scanline == -1 && self.cycle == 1 {
                self.pending_nmi = None;
                self.status.set_sp_0_hit(false);
                self.status.set_sp_overflow(false);
                self.status.set_vblank(false);
            }

            if (self.cycle >= 2 && self.cycle < 258) || (self.cycle >= 321 && self.cycle < 338) {
                self.shift_bg();

                match (self.cycle - 1) % 8 {
                    0 => {
                        self.load_tile();
                        self.next_tile.id = self.mem_read(0x2000 | (self.v_addr.raw() & 0xFFF));
                    }
                    2 => {
                        self.next_tile.attr = self.mem_read(
                            0x23C0
                                | self.v_addr.nta_addr()
                                | ((self.v_addr.ycoarse() >> 2) << 3) as u16
                                | (self.v_addr.xcoarse() >> 2) as u16,
                        );

                        if self.v_addr.ycoarse() & 0x2 != 0 {
                            self.next_tile.attr >>= 4;
                        }
                        if self.v_addr.xcoarse() & 0x2 != 0 {
                            self.next_tile.attr >>= 2;
                        }
                        self.next_tile.attr &= 0x3;
                    }
                    4 => {
                        self.next_tile.lo = self.mem_read(
                            self.ctrl.bg_base_addr()
                                + ((self.next_tile.id as u16) << 4)
                                + self.v_addr.yfine() as u16,
                        );
                    }
                    6 => {
                        self.next_tile.hi = self.mem_read(
                            self.ctrl.bg_base_addr()
                                + ((self.next_tile.id as u16) << 4)
                                + self.v_addr.yfine() as u16
                                + 8,
                        )
                    }
                    7 => self.increment_xscroll(),
                    _ => {}
                }
            }

            if self.cycle == 256 {
                self.increment_yscroll();
            }

            if self.cycle == 257 {
                self.load_tile();
                if self.rendering_enabled() {
                    self.v_addr.set_nta_h(self.scroll.nta_h());
                    self.v_addr.set_xcoarse(self.scroll.xcoarse());
                }
            }

            if self.cycle == 338 || self.cycle == 340 {
                self.next_tile.id = self.mem_read(0x2000 | (self.v_addr.raw() & 0xFFF));
            }

            if self.scanline == -1 && self.cycle == 304 && self.rendering_enabled() {
                self.v_addr = self.scroll;
            }
        }

        if self.scanline == 241 && self.cycle == 1 {
            self.status.set_vblank(true);
            if self.ctrl.nmi_enabled() {
                self.pending_nmi = Some(true)
            }
            // self.render_nametable_0();
            (self.render_fn)(self.frame.pixels());
        }

        if (self.scanline >= 0 && self.scanline < 240) && (self.cycle >= 1 && self.cycle <= 256) {
            let mut bg_pixel = 0;
            let mut bg_palette = 0;

            if self.mask.render_bg() && (self.mask.render_bg8() || self.cycle >= 9) {
                let mux = 0x8000 >> self.xfine;

                let p0_pixel = ((self.bg_lo_shift & mux) != 0) as u8;
                let p1_pixel = ((self.bg_hi_shift & mux) != 0) as u8;
                bg_pixel = (p1_pixel << 1) | p0_pixel;

                let p0_pal = ((self.bg_attr_lo_shift & mux) != 0) as u8;
                let p1_pal = ((self.bg_attr_hi_shift & mux) != 0) as u8;
                bg_palette = (p1_pal << 1) | p0_pal;
            }

            let color = self.get_color(bg_palette as u16, bg_pixel);
            self.frame
                .set_pixel(self.cycle - 1, self.scanline as usize, color);
        }

        self.cycle += 1;
        if self.cycle > 340 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline > 260 {
                self.scanline = -1;
                self.odd_frame = !self.odd_frame;
            }
        }
    }

    fn get_color(&mut self, palette: u16, pixel: u8) -> Pixel {
        let index = self.mem_read(0x3F00 + (palette << 2) + pixel as u16) as usize;
        NES_PALETTE[index & 0x3F]
    }

    fn increment_xscroll(&mut self) {
        if self.rendering_enabled() {
            let xcoarse = self.v_addr.xcoarse();
            let nta_h = self.v_addr.nta_h();
            if xcoarse == 31 {
                self.v_addr.set_xcoarse(0);
                self.v_addr.set_nta_h(!nta_h);
            } else {
                self.v_addr.set_xcoarse(xcoarse + 1);
            }
        }
    }

    fn increment_yscroll(&mut self) {
        if self.rendering_enabled() {
            let yfine = self.v_addr.yfine();
            let ycoarse = self.v_addr.ycoarse();
            let nta_v = self.v_addr.nta_v();
            if yfine < 7 {
                self.v_addr.set_yfine(yfine + 1);
            } else {
                self.v_addr.set_yfine(0);
                if ycoarse == 29 {
                    self.v_addr.set_ycoarse(0);
                    self.v_addr.set_nta_v(!nta_v);
                } else if ycoarse == 31 {
                    self.v_addr.set_ycoarse(0);
                } else {
                    self.v_addr.set_ycoarse(ycoarse + 1);
                }
            }
        }
    }

    fn load_tile(&mut self) {
        if self.rendering_enabled() {
            self.curr_tile = self.next_tile;

            self.bg_lo_shift |= self.next_tile.lo as u16;
            self.bg_hi_shift |= self.next_tile.hi as u16;

            let attr = self.next_tile.attr;
            self.bg_attr_lo_shift |= if attr & 0x1 != 0 { 0xFF } else { 0x00 };
            self.bg_attr_hi_shift |= if attr & 0x2 != 0 { 0xFF } else { 0x00 };
        }
    }

    fn shift_bg(&mut self) {
        if self.mask.render_bg() {
            self.bg_lo_shift <<= 1;
            self.bg_hi_shift <<= 1;
            self.bg_attr_lo_shift <<= 1;
            self.bg_attr_hi_shift <<= 1;
        }
    }

    fn rendering_enabled(&self) -> bool {
        self.mask.render_sp() | self.mask.render_bg()
    }

    fn increment_vaddr(&mut self) {
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
