use registers::{Controller, Loopy, Mask, Status};

use self::frame::Frame;
use self::tile::Tile;

pub mod frame;
mod registers;
mod tile;

#[derive(Clone, Copy)]
pub struct Rgb(u8, u8, u8);

#[rustfmt::skip]
static NES_PALETTE: [Rgb; 0x40] = [
    Rgb(84, 84, 84),    Rgb(0, 30, 116),    Rgb(8, 16, 144),    Rgb(48, 0, 136),    Rgb(68, 0, 100),    Rgb(92, 0, 48),     Rgb(84, 4, 0),      Rgb(60, 24, 0),
    Rgb(32, 42, 0),     Rgb(8, 58, 0),      Rgb(0, 64, 0),      Rgb(0, 60, 0),      Rgb(0, 50, 60),     Rgb(0, 0, 0),       Rgb(0, 0, 0),       Rgb(0, 0, 0),

    Rgb(152, 150, 152), Rgb(8, 76, 196),    Rgb(48, 50, 236),   Rgb(92, 30, 228),   Rgb(136, 20, 176),  Rgb(160, 20, 100),  Rgb(152, 34, 32),   Rgb(120, 60, 0),
    Rgb(84, 90, 0),     Rgb(40, 114, 0),    Rgb(8, 124, 0),     Rgb(0, 118, 40),    Rgb(0, 102, 120),   Rgb(0, 0, 0),       Rgb(0, 0, 0),       Rgb(0, 0, 0),

    Rgb(236, 238, 236), Rgb(76, 154, 236),  Rgb(120, 124, 236), Rgb(176, 98, 236),  Rgb(228, 84, 236),  Rgb(236, 88, 180),  Rgb(236, 106, 100), Rgb(212, 136, 32),
    Rgb(160, 170, 0),   Rgb(116, 196, 0),   Rgb(76, 208, 32),   Rgb(56, 204, 108),  Rgb(56, 180, 204),  Rgb(60, 60, 60),    Rgb(0, 0, 0),       Rgb(0, 0, 0),

    Rgb(236, 238, 236), Rgb(168, 204, 236), Rgb(188, 188, 236), Rgb(212, 178, 236), Rgb(236, 174, 236), Rgb(236, 174, 212), Rgb(236, 180, 176), Rgb(228, 196, 144),
    Rgb(204, 210, 120), Rgb(180, 222, 120), Rgb(168, 226, 144), Rgb(152, 226, 180), Rgb(160, 214, 228), Rgb(160, 162, 160), Rgb(0, 0, 0),       Rgb(0, 0, 0),
];

#[derive(Clone, Copy, Default, Debug)]
struct SpriteInfo {
    y: u8,
    id: u8,
    attr: u8,
    x: u8,
}

const PPU_CTRL: u16 = 0x0;
const PPU_MASK: u16 = 0x1;
const PPU_STATUS: u16 = 0x2;
const OAM_ADDR: u16 = 0x3;
pub const OAM_DATA: u16 = 0x4;
const PPU_SCROLL: u16 = 0x5;
const PPU_ADDR: u16 = 0x6;
const PPU_DATA: u16 = 0x7;

const OAM_SIZE: usize = 0x100;
const OAM2_SIZE: usize = 0x8;

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

    oam_data: [u8; OAM_SIZE],
    oam2_data: [SpriteInfo; OAM2_SIZE],
    oam_addr: u8,
    clearing_oam: bool,
    sprite_count: usize,
    fg_lo_shift: [u8; OAM2_SIZE],
    fg_hi_shift: [u8; OAM2_SIZE],

    addr_toggle: bool,
    read_buffer: u8,
    xfine: u8,
    v_addr: Loopy,
    scroll: Loopy,

    scanline: i32,
    cycle: usize,
    next_tile: Tile,
    bg_lo_shift: u16,
    bg_hi_shift: u16,
    bg_attr_lo_shift: u16,
    bg_attr_hi_shift: u16,

    frame: Frame,
    frame_count: u128,
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

            oam_data: [0; OAM_SIZE],
            oam2_data: [SpriteInfo::default(); OAM2_SIZE],
            oam_addr: 0,
            clearing_oam: false,
            sprite_count: 0,
            fg_lo_shift: [0; OAM2_SIZE],
            fg_hi_shift: [0; OAM2_SIZE],

            addr_toggle: false,
            read_buffer: 0,
            xfine: 0,
            v_addr: Loopy::new(),
            scroll: Loopy::new(),

            scanline: 0,
            cycle: 0,
            next_tile: Tile::new(),
            bg_lo_shift: 0,
            bg_hi_shift: 0,
            bg_attr_lo_shift: 0,
            bg_attr_hi_shift: 0,

            frame: Frame::new(),
            frame_count: 0,
            odd_frame: false,
            render_fn,
        }
    }

    #[allow(dead_code)]
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
                            0 => NES_PALETTE[0x05],
                            1 => NES_PALETTE[0x2A],
                            2 => NES_PALETTE[0x27],
                            3 => NES_PALETTE[0x3B],
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

    #[allow(dead_code)]
    fn render_nametable_0(&mut self) {
        for addr in 0..0x3C0 {
            let tile_id = self.mem_read(0x2000 | addr);
            let tile_addr = self.ctrl.bg_base_addr() + (tile_id as u16) * 16;
            let tile_x = addr % 32;
            let tile_y = addr / 32;

            let attr_index = tile_y / 4 * 8 + tile_x / 4;
            let attr_byte = self.mem_read(0x23C0 + attr_index);
            let palette = match (tile_x % 4 / 2, tile_y % 4 / 2) {
                (0, 0) => attr_byte & 0b11,
                (1, 0) => (attr_byte >> 2) & 0b11,
                (0, 1) => (attr_byte >> 4) & 0b11,
                (1, 1) => (attr_byte >> 6) & 0b11,
                _ => unreachable!(),
            };

            for row in 0..8 {
                let mut lo = self.mem_read(tile_addr + row);
                let mut hi = self.mem_read(tile_addr + row + 0x8);

                for col in (0..8).rev() {
                    let pixel = (hi & 0x1) << 1 | (lo & 0x1);
                    lo >>= 1;
                    hi >>= 1;

                    let rgb = self.get_color(palette, pixel);

                    self.frame.set_pixel(
                        (tile_x * 8 + col) as usize,
                        (tile_y * 8 + row) as usize,
                        rgb,
                    );
                }
            }
        }
    }

    pub fn frame_count(&self) -> u128 {
        self.frame_count
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
            OAM_DATA => match self.clearing_oam {
                true => data = 0xFF,
                false => data = self.oam_data[self.oam_addr as usize],
            },
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
            OAM_ADDR => {
                self.oam_addr = data;
            }
            OAM_DATA => {
                self.oam_data[self.oam_addr as usize] = data;
                self.oam_addr = self.oam_addr.wrapping_add(1);
            }
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
        if self.odd_frame && self.scanline == 0 && self.cycle == 0 && self.mask.render_bg() {
            self.cycle = 1;
        }

        let cycle = self.cycle;
        let scanline = self.scanline;

        if scanline == -1 && cycle == 1 {
            self.pending_nmi = None;
            self.status.set_sp_0_hit(false);
            self.status.set_sp_overflow(false);
            self.status.set_vblank(false);
            self.fg_lo_shift.fill(0);
            self.fg_hi_shift.fill(0);
        }

        if scanline < 240 && self.rendering_enabled() {
            self.process_rendering_scanline();
        }

        if scanline == 241 && cycle == 1 {
            self.status.set_vblank(true);
            if self.ctrl.nmi_enabled() {
                self.pending_nmi = Some(true)
            }

            self.frame_count = self.frame_count.wrapping_add(1);
            // Render in window (in this case, using SDL2)
            (self.render_fn)(self.frame.pixels());
        }

        if (0..240).contains(&scanline) && (1..=256).contains(&cycle) {
            let (bg_pixel, bg_palette) = self.get_bg_pixel_info();
            let (fg_pixel, fg_palette, fg_priority) = self.get_fg_pixel_info();

            let (pixel, palette) = match bg_pixel {
                0 if fg_pixel == 0 => (0, 0),
                0 if fg_pixel > 0 => (fg_pixel, fg_palette),
                1..=3 if fg_pixel == 0 => (bg_pixel, bg_palette),
                _ => {
                    if fg_priority != 0 {
                        (fg_pixel, fg_palette)
                    } else {
                        (bg_pixel, bg_palette)
                    }
                }
            };

            let color = self.get_color(palette, pixel);
            self.frame.set_pixel(cycle - 1, scanline as usize, color);
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

    fn process_rendering_scanline(&mut self) {
        let cycle = self.cycle;
        let scanline = self.scanline;

        if scanline == -1 && cycle == 304 && self.mask.render_bg() {
            self.v_addr = self.scroll;
        }

        // Background
        if (2..258).contains(&cycle) || (321..338).contains(&cycle) {
            self.shift_bg();

            match (cycle - 1) % 8 {
                0 => {
                    self.load_next_tile();
                    let vaddr = 0x2000 | (self.v_addr.raw() & 0xFFF);
                    self.next_tile.id = self.mem_read(vaddr);
                }
                2 => {
                    let vaddr = 0x23C0
                        | self.v_addr.nta_addr()
                        | ((self.v_addr.ycoarse() >> 2) << 3) as u16
                        | (self.v_addr.xcoarse() >> 2) as u16;

                    self.next_tile.attr = self.mem_read(vaddr);

                    if self.v_addr.ycoarse() & 0x2 != 0 {
                        self.next_tile.attr >>= 4;
                    }
                    if self.v_addr.xcoarse() & 0x2 != 0 {
                        self.next_tile.attr >>= 2;
                    }
                    self.next_tile.attr &= 0x3;
                }
                4 => {
                    let vaddr = self.ctrl.bg_base_addr()
                        + ((self.next_tile.id as u16) << 4)
                        + self.v_addr.yfine() as u16;

                    self.next_tile.lo = self.mem_read(vaddr);
                }
                6 => {
                    let vaddr = self.ctrl.bg_base_addr()
                        + ((self.next_tile.id as u16) << 4)
                        + self.v_addr.yfine() as u16
                        + 8;

                    self.next_tile.hi = self.mem_read(vaddr);
                }
                7 => self.increment_xscroll(),
                _ => {}
            }
        }

        if cycle == 256 {
            self.increment_yscroll();
        }

        if cycle == 257 {
            self.load_next_tile();
            if self.mask.render_bg() {
                self.v_addr.set_nta_h(self.scroll.nta_h());
                self.v_addr.set_xcoarse(self.scroll.xcoarse());
            }
        }

        if cycle == 337 || cycle == 339 {
            self.next_tile.id = self.mem_read(0x2000 | (self.v_addr.raw() & 0xFFF));
        }

        // Sprites
        if cycle == 1 {
            self.clearing_oam = true;
        } else if cycle == 64 {
            self.clearing_oam = false;
        }

        self.shift_fg();
        if cycle == 257 && scanline >= 0 {
            self.oam2_data[..].fill(SpriteInfo {
                y: 0xFF,
                id: 0xFF,
                attr: 0xFF,
                x: 0xFF,
            });

            self.fg_lo_shift.fill(0);
            self.fg_hi_shift.fill(0);

            let mut sprite_count = 0;
            let sprite_size = if self.ctrl.sprite_size() { 16 } else { 8 };

            for index in (0..OAM_SIZE).step_by(4) {
                let diff = scanline as u16 - self.oam_data[index] as u16;

                if (0..sprite_size).contains(&diff) {
                    if sprite_count < 8 {
                        self.oam2_data[sprite_count].y = self.oam_data[index];
                        self.oam2_data[sprite_count].id = self.oam_data[index + 1];
                        self.oam2_data[sprite_count].attr = self.oam_data[index + 2];
                        self.oam2_data[sprite_count].x = self.oam_data[index + 3];
                    }
                    sprite_count += 1;
                }
            }

            self.status.set_sp_overflow(sprite_count > 8);
            self.sprite_count = if sprite_count > 8 { 8 } else { sprite_count };
        }

        if cycle == 340 {
            self.load_sprites();
        }
    }

    fn load_sprites(&mut self) {
        let scanline = self.scanline as u8;
        for i in 0..self.sprite_count {
            let sprite_addr = match !self.ctrl.sprite_size() {
                true => {
                    let offset = self.ctrl.sp_base_addr();
                    let flipped_v = self.oam2_data[i].attr & 0x80 != 0;
                    let tile_id = self.oam2_data[i].id;
                    let row = match flipped_v {
                        true => (7 - (scanline - self.oam2_data[i].y)) as u16,
                        false => (scanline - self.oam2_data[i].y) as u16,
                    };

                    offset | (tile_id as u16) << 4 | row
                }
                false => {
                    let offset = ((self.oam2_data[i].id & 0x01) as u16) << 12;
                    let flipped_v = self.oam2_data[i].attr & 0x80 != 0;
                    let top_half = scanline - self.oam2_data[i].y < 8;
                    let tile_id = match (flipped_v, top_half) {
                        (false, true) | (true, false) => self.oam2_data[i].id & 0xFE,
                        (false, false) | (true, true) => (self.oam2_data[i].id & 0xFE) + 1,
                    };
                    let row = match flipped_v {
                        true => ((7 - (scanline - self.oam2_data[i].y)) & 0x7) as u16,
                        false => ((scanline - self.oam2_data[i].y) & 0x7) as u16,
                    };

                    offset | (tile_id as u16) << 4 | row
                }
            };

            let sprite_lo = self.mem_read(sprite_addr);
            let sprite_hi = self.mem_read(sprite_addr.wrapping_add(8));

            let flip = |mut v: u8| {
                v = (v & 0xF0) >> 4 | (v & 0x0F) << 4;
                v = (v & 0xCC) >> 2 | (v & 0x33) << 2;
                v = (v & 0xAA) >> 1 | (v & 0x55) << 1;
                v
            };

            self.fg_lo_shift[i] = match self.oam2_data[i].attr & 0x40 != 0 {
                true => flip(sprite_lo),
                false => sprite_lo,
            };

            self.fg_hi_shift[i] = match self.oam2_data[i].attr & 0x40 != 0 {
                true => flip(sprite_hi),
                false => sprite_hi,
            };
        }
    }

    fn get_bg_pixel_info(&self) -> (u8, u8) {
        if self.mask.render_bg() && (self.mask.render_bg8() || self.cycle >= 9) {
            let mux = 0x8000 >> self.xfine;

            let lo_pixel = ((self.bg_lo_shift & mux) != 0) as u8;
            let hi_pixel = ((self.bg_hi_shift & mux) != 0) as u8;
            let bg_pixel = (hi_pixel << 1) | lo_pixel;

            let lo_pal = ((self.bg_attr_lo_shift & mux) != 0) as u8;
            let hi_pal = ((self.bg_attr_hi_shift & mux) != 0) as u8;
            let bg_palette = (hi_pal << 1) | lo_pal;

            return (bg_pixel, bg_palette);
        }

        (0, 0)
    }

    fn get_fg_pixel_info(&self) -> (u8, u8, u8) {
        if self.mask.render_sp() && (self.mask.render_sp8() || self.cycle >= 9) {
            for i in 0..self.sprite_count {
                if self.oam2_data[i].x != 0 {
                    continue;
                }

                let lo_pixel = ((self.fg_lo_shift[i] & 0x80) != 0) as u8;
                let hi_pixel = ((self.fg_hi_shift[i] & 0x80) != 0) as u8;
                let fg_pixel = (hi_pixel << 1) | lo_pixel;

                let fg_palette = (self.oam2_data[i].attr & 0x3) + 0x4;
                let fg_priority = ((self.oam2_data[i].attr & 0x20) == 0) as u8;

                if fg_pixel != 0 {
                    return (fg_pixel, fg_palette, fg_priority);
                }
            }
        }

        (0, 0, 0)
    }

    fn get_color(&mut self, palette: u8, pixel: u8) -> Rgb {
        let index = self.mem_read(0x3F00 + ((palette as u16) << 2) + pixel as u16) as usize;
        NES_PALETTE[index & 0x3F]
    }

    fn increment_xscroll(&mut self) {
        if self.mask.render_bg() {
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
        if self.mask.render_bg() {
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

    fn load_next_tile(&mut self) {
        if self.rendering_enabled() {
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

    fn shift_fg(&mut self) {
        if self.mask.render_sp() && (1..258).contains(&self.cycle) {
            for (i, sprite) in self
                .oam2_data
                .iter_mut()
                .take(self.sprite_count)
                .enumerate()
            {
                if sprite.x > 0 {
                    sprite.x -= 1;
                } else {
                    self.fg_lo_shift[i] <<= 1;
                    self.fg_hi_shift[i] <<= 1;
                }
            }
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
