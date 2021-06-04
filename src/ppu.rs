use std::fs::File;

use serde::{Deserialize, Serialize};

use registers::{Controller, Loopy, Mask, Status};

use crate::savable::Savable;

use self::frame::Frame;

pub mod frame;
mod registers;

#[derive(Clone, Copy)]
pub struct Rgb(u8, u8, u8);

/// NES color palette
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

/// Background tile
#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize)]
struct Tile {
    pub lo: u8,
    pub hi: u8,
    pub attr: u8,
    pub id: u8,
}

/// Sprite attributes
#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize)]
struct SpriteInfo {
    y: u8,
    id: u8,
    attr: u8,
    x: u8,
    index: u8,
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

/// Ppu memory interface
pub trait Interface {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
    fn inc_scanline(&mut self);
}

pub trait PpuInterface: Interface + Savable {}

/// 2C02 Ppu
pub struct Ppu<'a> {
    ctrl: Controller,
    mask: Mask,
    status: Status,

    bus: Box<dyn PpuInterface>,
    pending_nmi: Option<bool>,
    open_bus: u8,
    open_bus_timer: u32,

    oam_data: [u8; OAM_SIZE],
    oam2_data: [SpriteInfo; OAM2_SIZE],
    oam_addr: u8,
    clearing_oam: bool,
    sprite_0_rendering: bool,
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

impl Savable for Ppu<'_> {
    fn save(&self, output: &File) -> bincode::Result<()> {
        self.bus.save(output)?;
        bincode::serialize_into(output, &self.ctrl.bits())?;
        bincode::serialize_into(output, &self.mask.bits())?;
        bincode::serialize_into(output, &self.status.bits())?;
        bincode::serialize_into(output, &self.pending_nmi)?;
        bincode::serialize_into(output, &self.open_bus)?;
        bincode::serialize_into(output, &self.open_bus_timer)?;
        bincode::serialize_into(output, &self.oam_addr)?;
        bincode::serialize_into(output, &self.clearing_oam)?;
        bincode::serialize_into(output, &self.sprite_0_rendering)?;
        bincode::serialize_into(output, &self.sprite_count)?;
        bincode::serialize_into(output, &self.fg_lo_shift)?;
        bincode::serialize_into(output, &self.fg_hi_shift)?;
        bincode::serialize_into(output, &self.addr_toggle)?;
        bincode::serialize_into(output, &self.read_buffer)?;
        bincode::serialize_into(output, &self.xfine)?;
        bincode::serialize_into(output, &self.v_addr.raw())?;
        bincode::serialize_into(output, &self.scroll.raw())?;
        bincode::serialize_into(output, &self.scanline)?;
        bincode::serialize_into(output, &self.cycle)?;
        bincode::serialize_into(output, &self.next_tile)?;
        bincode::serialize_into(output, &self.bg_lo_shift)?;
        bincode::serialize_into(output, &self.bg_hi_shift)?;
        bincode::serialize_into(output, &self.bg_attr_lo_shift)?;
        bincode::serialize_into(output, &self.bg_attr_hi_shift)?;
        for i in 0..OAM_SIZE {
            bincode::serialize_into(output, &self.oam_data[i])?;
        }
        for i in 0..OAM2_SIZE {
            bincode::serialize_into(output, &self.oam2_data[i])?;
        }
        bincode::serialize_into(output, &self.frame_count)?;
        bincode::serialize_into(output, &self.odd_frame)?;
        Ok(())
    }

    fn load(&mut self, input: &File) -> bincode::Result<()> {
        self.bus.load(input)?;
        let byte: u8 = bincode::deserialize_from(input)?;
        self.ctrl.set_raw(byte);
        let byte: u8 = bincode::deserialize_from(input)?;
        self.mask.set_raw(byte);
        let byte: u8 = bincode::deserialize_from(input)?;
        self.status.set_raw(byte);
        self.pending_nmi = bincode::deserialize_from(input)?;
        self.open_bus = bincode::deserialize_from(input)?;
        self.open_bus_timer = bincode::deserialize_from(input)?;
        self.oam_addr = bincode::deserialize_from(input)?;
        self.clearing_oam = bincode::deserialize_from(input)?;
        self.sprite_0_rendering = bincode::deserialize_from(input)?;
        self.sprite_count = bincode::deserialize_from(input)?;
        self.fg_lo_shift = bincode::deserialize_from(input)?;
        self.fg_hi_shift = bincode::deserialize_from(input)?;
        self.addr_toggle = bincode::deserialize_from(input)?;
        self.read_buffer = bincode::deserialize_from(input)?;
        self.xfine = bincode::deserialize_from(input)?;
        let word: u16 = bincode::deserialize_from(input)?;
        self.v_addr.set_raw(word);
        let word: u16 = bincode::deserialize_from(input)?;
        self.scroll.set_raw(word);
        self.scanline = bincode::deserialize_from(input)?;
        self.cycle = bincode::deserialize_from(input)?;
        self.next_tile = bincode::deserialize_from(input)?;
        self.bg_lo_shift = bincode::deserialize_from(input)?;
        self.bg_hi_shift = bincode::deserialize_from(input)?;
        self.bg_attr_lo_shift = bincode::deserialize_from(input)?;
        self.bg_attr_hi_shift = bincode::deserialize_from(input)?;
        for i in 0..OAM_SIZE {
            self.oam_data[i] = bincode::deserialize_from(input)?;
        }
        for i in 0..OAM2_SIZE {
            self.oam2_data[i] = bincode::deserialize_from(input)?;
        }
        self.frame_count = bincode::deserialize_from(input)?;
        self.odd_frame = bincode::deserialize_from(input)?;
        Ok(())
    }
}

impl<'a> Ppu<'a> {
    pub fn new<F>(bus: Box<dyn PpuInterface>, render_fn: Box<F>) -> Self
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
            open_bus_timer: 0,

            oam_data: [0; OAM_SIZE],
            oam2_data: [SpriteInfo::default(); OAM2_SIZE],
            oam_addr: 0,
            clearing_oam: false,
            sprite_0_rendering: false,
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
            next_tile: Tile::default(),
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

    /// Resets the state of the Ppu
    pub fn reset(&mut self) {
        self.ctrl = Controller::from_bits_truncate(0);
        self.mask = Mask::from_bits_truncate(0);
        self.status = Status::from_bits_truncate(0);

        self.pending_nmi = None;
        self.open_bus = 0;
        self.open_bus_timer = 0;

        self.oam_data = [0; OAM_SIZE];
        self.oam2_data = [SpriteInfo::default(); OAM2_SIZE];
        self.oam_addr = 0;
        self.clearing_oam = false;
        self.sprite_0_rendering = false;
        self.sprite_count = 0;
        self.fg_lo_shift = [0; OAM2_SIZE];
        self.fg_hi_shift = [0; OAM2_SIZE];

        self.addr_toggle = false;
        self.read_buffer = 0;
        self.xfine = 0;
        self.v_addr.set_raw(0);
        self.scroll.set_raw(0);

        self.scanline = 0;
        self.cycle = 0;
        self.next_tile = Tile::default();
        self.bg_lo_shift = 0;
        self.bg_hi_shift = 0;
        self.bg_attr_lo_shift = 0;
        self.bg_attr_hi_shift = 0;

        self.frame.clear();
        self.frame_count = 0;
        self.odd_frame = false;
    }

    /// Debug function to show the cartridge CHR Patterns
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

    /// Debug function to show the nametable 0
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

    /// Returns how many frames have been rendered
    pub fn frame_count(&self) -> u128 {
        self.frame_count
    }

    /// Ppu register read
    pub fn read(&mut self, addr: u16) -> u8 {
        // The ppu bus would latch data for a few cycles, so there might
        // be data on the bus
        let mut data = self.open_bus;
        match addr {
            PPU_CTRL => {}
            PPU_MASK => {}
            PPU_STATUS => {
                // Only the upper 3 bits are the status data
                // The rest is set to what was on the open bus
                data = self.status.bits() | (self.open_bus & 0x1F);
                // Reading status removes the vblank flag
                self.status.remove(Status::IN_VBLANK);
                self.pending_nmi = None;
                // Also resets the address toggle
                self.addr_toggle = false;
            }
            OAM_ADDR => {}
            OAM_DATA => match self.clearing_oam {
                // Always returns 0xFF when clearing secondary OAM
                true => data = 0xFF,
                false => {
                    // Bits 2, 3 and 4 do not exist in the Ppu if reading byte 2
                    let mask = match self.oam_addr & 0x3 {
                        2 => 0xE3,
                        _ => 0xFF,
                    };
                    // Read from OAM and refresh open bus
                    data = self.refresh_open_bus(self.oam_data[self.oam_addr as usize] & mask);
                }
            },
            PPU_SCROLL => {}
            PPU_ADDR => {}
            PPU_DATA => {
                // Reading data takes 2 reads to get the data. The first read put the data
                // in a buffer amd the second read puts the buffer data on the bus

                // Put the buffer data on the bus
                data = self.read_buffer;
                // Read new data into the buffer
                self.read_buffer = self.mem_read(self.v_addr.raw());

                // If the data read in from palette RAM, it only takes 1 read
                if (self.v_addr.raw() & 0x3F00) == 0x3F00 {
                    // Put the buffer data which was just read on the bus
                    data = (self.open_bus & 0xC0) | (self.read_buffer & 0x3F);
                    // Add the geryscale mask if enabled
                    data &= self.mask.greyscale_mask();
                }
                // Refresh the open bus value
                self.refresh_open_bus(data);
                // Every read automatically increments the Ppu address register
                self.increment_vaddr();
            }
            _ => {}
        }
        data
    }

    /// Ppu register write
    pub fn write(&mut self, addr: u16, data: u8) {
        // Refresh the open bus value
        self.refresh_open_bus(data);
        match addr {
            PPU_CTRL => {
                // Set the register to data
                self.ctrl.set_raw(data);
                // Update scroll nametable
                self.scroll.set_nta_h(self.ctrl.nta_h());
                self.scroll.set_nta_v(self.ctrl.nta_v());
            }
            PPU_MASK => {
                // Set the register to data
                self.mask.set_raw(data);
            }
            PPU_STATUS => {}
            OAM_ADDR => {
                // Set the OAM address to data
                self.oam_addr = data;
            }
            OAM_DATA => {
                // Write the data into OAM
                self.oam_data[self.oam_addr as usize] = data;
                // Write to OAM auto increments the address
                self.oam_addr = self.oam_addr.wrapping_add(1);
            }
            PPU_SCROLL => {
                // Writing to the scroll register uses the same latch as the address register
                match self.addr_toggle {
                    // If it is set, write Y scroll values
                    true => {
                        self.scroll.set_yfine(data & 0x7);
                        self.scroll.set_ycoarse(data >> 3);
                    }
                    // Otherwise, write X scroll values
                    false => {
                        self.xfine = data & 0x7;
                        self.scroll.set_xcoarse(data >> 3);
                    }
                }
                // Update the toggle
                self.addr_toggle = !self.addr_toggle;
            }
            PPU_ADDR => {
                // Because the Ppu address is a 14 bit address and the Cpu uses
                // an 8 bit bus, we have to write in two steps. The Ppu uses a toggle
                // to choose which part of the address to write. I am using Loopy's
                // implementation of the scroll and address register
                match self.addr_toggle {
                    // If it is set, set the lower bits of the address in the scroll (loopy's t register)
                    // and then set the address register (v register) to the scroll
                    true => {
                        self.scroll.set_addr_lo(data);
                        self.v_addr = self.scroll;
                    }
                    // Otherwise, set the high bits of the scroll
                    false => self.scroll.set_addr_hi(data & 0x3F),
                }
                // Update the toggle
                self.addr_toggle = !self.addr_toggle;
            }
            PPU_DATA => {
                // Write on the memory bus at the current address
                self.mem_write(self.v_addr.raw(), data);
                // Refresh open bus
                self.refresh_open_bus(data);
                // Writing to Ppu data also auto increments the address
                self.increment_vaddr();
            }
            _ => {}
        }
    }

    /// Poll the NMI flag set by the Ppu
    pub fn poll_nmi(&mut self) -> bool {
        self.pending_nmi.take().is_some()
    }

    /// Clock the Ppu once
    pub fn clock(&mut self) {
        // Update the open bus timer
        self.update_open_bus();

        // Every odd frame on the first scanline, the first cycle is skipped if background rendering is enabled
        // A flag is updated every frame
        if self.odd_frame && self.scanline == 0 && self.cycle == 0 && self.rendering_enabled() {
            self.cycle = 1;
        }

        // To not have to write self. every time
        let cycle = self.cycle;
        let scanline = self.scanline;

        // Pre render scanline
        if scanline == -1 && cycle == 1 {
            // Clear NMI and reset status register
            self.pending_nmi = None;
            self.status.set_sp_0_hit(false);
            self.status.set_sp_overflow(false);
            self.status.set_vblank(false);
            // Clear sprite shifters
            self.fg_lo_shift.fill(0);
            self.fg_hi_shift.fill(0);
        }

        // 0..=240 -> rendering scanline
        if scanline < 240 && self.rendering_enabled() {
            self.process_rendering_scanline();
        }

        // Set NMI if enabled on cycle 241
        if scanline == 241 && cycle == 1 {
            self.status.set_vblank(true);
            if self.ctrl.nmi_enabled() {
                self.pending_nmi = Some(true)
            }

            // A new frame is done rendering
            self.frame_count = self.frame_count.wrapping_add(1);
            // Render in window (in this case, using SDL2)
            (self.render_fn)(self.frame.pixels());
        }

        // Calculate the pixel color
        if (0..240).contains(&scanline) && (1..257).contains(&cycle) {
            let (bg_pixel, bg_palette) = self.get_bg_pixel_info();
            // little hack to fix random sprite colors on left of first scanline
            let (fg_pixel, fg_palette, fg_priority) = match scanline != 0 {
                true => self.get_fg_pixel_info(),
                false => (0, 0, 0),
            };

            // Pixel priority logic
            let (pixel, palette) = match bg_pixel {
                // Both foreground and background are 0, result is 0
                0 if fg_pixel == 0 => (0, 0),
                // Only background is 0, output foreground
                0 if fg_pixel > 0 => (fg_pixel, fg_palette),
                // Only foreground is 0, output background
                1..=3 if fg_pixel == 0 => (bg_pixel, bg_palette),
                // Both are non zero
                _ => {
                    // Collision is possible
                    self.update_sprite_zero_hit();
                    // The result is choosen based on the sprite priority attribute
                    // If it is 0, output foreground
                    if fg_priority != 0 {
                        (fg_pixel, fg_palette)
                    // If it is 1, output background
                    } else {
                        (bg_pixel, bg_palette)
                    }
                }
            };

            // Get the color from palette RAM
            let color = self.get_color(palette, pixel);
            // Set the pixel
            self.frame.set_pixel(cycle - 1, scanline as usize, color);
        }

        // Update cycle count
        self.cycle += 1;

        // Signal the cartridge a new scanline was done (this is not how it worked on the NES).
        // The mapper 4 (MMC3) uses this
        if self.rendering_enabled() && self.cycle == 260 && scanline < 240 {
            self.bus.inc_scanline();
        }

        // Last cycle
        if self.cycle > 340 {
            // Reset back to 0
            self.cycle = 0;
            // Increment scanline
            self.scanline += 1;
            // Last scanline
            if self.scanline > 260 {
                // Reset back to -1 (pre render scanline)
                self.scanline = -1;
                // Toggle odd frame flag
                self.odd_frame = !self.odd_frame;
            }
        }
    }

    /// Refresh open bus latch value
    fn refresh_open_bus(&mut self, data: u8) -> u8 {
        self.open_bus = data;
        self.open_bus_timer = 7777;
        data
    }

    /// Refresh open bus latch timer
    fn update_open_bus(&mut self) {
        match self.open_bus_timer > 0 {
            true => self.open_bus_timer -= 1,
            false => self.open_bus = 0,
        }
    }

    /// Update the sprite 0 hit flag
    fn update_sprite_zero_hit(&mut self) {
        // Sprite 0 hit is a collision between a non 0 sprite pixel and bg pixel
        // To be possible, we have to be drawing a sprite 0 pixel and both
        // sprite and background rendering has to be enable
        if self.sprite_0_rendering && self.mask.render_bg() && self.mask.render_sp() {
            // If either bg or sprite left most pixels are disabled, don't check
            // first 8 pixels
            if !(self.mask.render_bg8() | self.mask.render_sp8()) {
                if (9..256).contains(&self.cycle) {
                    self.status.set_sp_0_hit(true);
                }
            } else if (1..256).contains(&self.cycle) {
                self.status.set_sp_0_hit(true);
            }
        }
    }

    /// Process the current cycle of a rendering scanline
    fn process_rendering_scanline(&mut self) {
        // To not have to write self. every time
        let cycle = self.cycle;
        let scanline = self.scanline;

        // Update scroll on prerender scanline
        if scanline == -1 && cycle == 304 && self.mask.render_bg() {
            self.v_addr = self.scroll;
        }

        // Background
        if (2..258).contains(&cycle) || (321..338).contains(&cycle) {
            // Update bg shifters
            self.shift_bg();

            // Background operations repeat every 8 cycles
            match (cycle - 1) % 8 {
                0 => {
                    // Load next tile in the shifters
                    self.load_next_tile();
                    // Get the address of the next tile
                    let vaddr = self.v_addr.tile_addr();
                    // At the address is the id of the pattern to draw
                    self.next_tile.id = self.mem_read(vaddr);
                }
                2 => {
                    // The attribute byte is one of the hardest thing to
                    // understand. It is well explained here:
                    // https://bugzmanov.github.io/nes_ebook/chapter_6_4.html
                    // and here:
                    // https://youtu.be/-THeUXqR3zY?t=2439

                    // Get the address of the tile attribute
                    let vaddr = self.v_addr.tile_attr_addr();
                    // Get the attribute byte
                    self.next_tile.attr = self.mem_read(vaddr);

                    // Attribute byte: BRBL TRTL
                    // BR: Bottom right metatile
                    // BL: Bottom left metatile
                    // TR: Top right metatile
                    // TL: Top left metatile

                    // Bottom part of the nametable?
                    if self.v_addr.ycoarse() & 0x2 != 0 {
                        // If so shift by 4
                        self.next_tile.attr >>= 4;
                    }
                    // Right part of the nametable?
                    if self.v_addr.xcoarse() & 0x2 != 0 {
                        // If so shift by 2
                        self.next_tile.attr >>= 2;
                    }
                    // Attribute is only two bits
                    self.next_tile.attr &= 0x3;
                }
                4 => {
                    // The pixel value are divided in two bitplanes.
                    // The bitplanes are 8 consecutive bytes in memory.
                    // So, the high and low bitplanes are 8 bytes apart
                    //
                    // Two bitplanes represent one background tile
                    // 0 1 1 0 0 1 2 0  =  0 1 1 0 0 1 1 0  +  0 0 0 0 0 0 1 0
                    // 0 0 0 0 0 1 2 0  =  0 0 0 0 0 1 1 0  +  0 0 0 0 0 0 1 0
                    // 0 0 0 0 0 1 2 0  =  0 0 0 0 0 1 1 0  +  0 0 0 0 0 0 1 0
                    // 0 1 0 0 0 0 2 0  =  0 1 0 0 0 0 1 0  +  0 0 0 0 0 0 1 0
                    // 0 1 1 0 0 0 1 0  =  0 1 1 0 0 0 0 0  +  0 0 0 0 0 0 1 0
                    // 0 0 0 0 0 1 2 0  =  0 0 0 0 0 1 1 0  +  0 0 0 0 0 0 1 0
                    // 0 1 1 0 0 0 1 0  =  0 1 1 0 0 0 0 0  +  0 0 0 0 0 0 1 0
                    // 0 1 1 0 0 1 2 0  =  0 1 1 0 0 1 1 0  +  0 0 0 0 0 0 1 0

                    let vaddr = self.ctrl.bg_base_addr()
                        + ((self.next_tile.id as u16) << 4)
                        + self.v_addr.yfine() as u16;

                    self.next_tile.lo = self.mem_read(vaddr);
                }
                6 => {
                    // Same thing but + 8 for the high bitplane
                    let vaddr = self.ctrl.bg_base_addr()
                        + ((self.next_tile.id as u16) << 4)
                        + self.v_addr.yfine() as u16
                        + 8;

                    self.next_tile.hi = self.mem_read(vaddr);
                }
                // Increment horizontal scroll
                7 => self.increment_xscroll(),
                _ => {}
            }
        }

        // Increment vertical scrolling
        if cycle == 256 {
            self.increment_yscroll();
        }

        // End of the scanline
        if cycle == 257 {
            // Load the next tile into the shifters
            self.load_next_tile();
            // Update x coarse and nametable x if background rendering is enabled
            if self.mask.render_bg() {
                self.v_addr.set_nta_h(self.scroll.nta_h());
                self.v_addr.set_xcoarse(self.scroll.xcoarse());
            }
        }

        // Sprites

        if cycle == 1 {
            self.clearing_oam = true;
        } else if cycle == 64 {
            self.clearing_oam = false;
        }

        // The sprite evaluation is done the same way as Javidx9 did
        // in his emulator tutorial youtube videos
        // https://www.youtube.com/playlist?list=PLrOv9FMX8xJHqMvSGB_9G9nZZ_4IgteYf

        // Update foreground shifters
        self.shift_fg();

        // All the sprite evaluation is done in 1 cycle (this is NOT how it is done on the real hardware)
        if cycle == 257 && scanline >= 0 {
            // Set all the values
            self.oam2_data[..].fill(SpriteInfo {
                y: 0xFF,
                id: 0xFF,
                attr: 0xFF,
                x: 0xFF,
                index: 0xFF,
            });

            // Reset the shifters
            self.fg_lo_shift.fill(0);
            self.fg_hi_shift.fill(0);

            let mut sprite_count = 0;
            let sprite_size = if self.ctrl.sprite_size() { 16 } else { 8 };

            // Every sprite attributes in OAM is 4 bytes, thus step by 4
            // 0: Y pos
            // 1: Sprite tile ID
            // 2: Attribute byte
            // 3: X pos
            for index in (0..OAM_SIZE).step_by(4) {
                // Calculate the difference between the scanline and the sprite y value
                let diff = (scanline as u16).wrapping_sub(self.oam_data[index] as u16);

                // Starting from sprite 0, check every sprite if they hit the scanline
                if (0..sprite_size).contains(&diff) {
                    // If the sprite is visible and there is less than 8 sprite already visible,
                    // add it to secondary OAM
                    if sprite_count < 8 {
                        self.oam2_data[sprite_count].y = self.oam_data[index];
                        self.oam2_data[sprite_count].id = self.oam_data[index + 1];
                        self.oam2_data[sprite_count].attr = self.oam_data[index + 2];
                        self.oam2_data[sprite_count].x = self.oam_data[index + 3];
                        self.oam2_data[sprite_count].index = index as u8;
                    }
                    // Total number of sprite on the scanline (including discarded ones)
                    sprite_count += 1;
                }
            }

            // If more than 8 sprites, set the sprite overflow bit
            self.status.set_sp_overflow(sprite_count > 8);
            // Visible sprite count
            self.sprite_count = if sprite_count > 8 { 8 } else { sprite_count };
        }

        if cycle == 321 {
            self.load_sprites();
        }
    }

    /// Load sprites from secondary OAM into the shifters
    fn load_sprites(&mut self) {
        let scanline = self.scanline as u8;
        // For each visible sprites
        for i in 0..self.sprite_count {
            // Check height (8 or 16)
            let sprite_addr = match !self.ctrl.sprite_size() {
                // 8
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
                // 16
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

            // Flip horizontal closure
            let flip_h = |mut v: u8| {
                v = (v & 0xF0) >> 4 | (v & 0x0F) << 4;
                v = (v & 0xCC) >> 2 | (v & 0x33) << 2;
                v = (v & 0xAA) >> 1 | (v & 0x55) << 1;
                v
            };

            self.fg_lo_shift[i] = match self.oam2_data[i].attr & 0x40 != 0 {
                true => flip_h(sprite_lo),
                false => sprite_lo,
            };

            self.fg_hi_shift[i] = match self.oam2_data[i].attr & 0x40 != 0 {
                true => flip_h(sprite_hi),
                false => sprite_hi,
            };
        }
    }

    /// Returns pixel value and palette index of current background pixel
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

    /// Returns pixel value, palette index and attribute byte of current foreground pixel
    fn get_fg_pixel_info(&mut self) -> (u8, u8, u8) {
        if self.mask.render_sp() && (self.mask.render_sp8() || self.cycle >= 9) {
            self.sprite_0_rendering = false;
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
                    // Set a flag if it is sprite 0
                    if self.oam2_data[i].index == 0 {
                        self.sprite_0_rendering = true;
                    }
                    return (fg_pixel, fg_palette, fg_priority);
                }
            }
        }

        (0, 0, 0)
    }

    /// Returns the RBG value of the pixel with greyscale and color emphasis applied
    fn get_color(&mut self, palette: u8, pixel: u8) -> Rgb {
        let index = self.mem_read(0x3F00 + ((palette as u16) << 2) + pixel as u16)
            & self.mask.greyscale_mask();
        let c = NES_PALETTE[(index as usize) & 0x3F];

        match self.mask.color_emph_enabled() {
            false => c,
            true => {
                let (r_factor, g_factor, b_factor) = self.mask.emph_factors();
                Rgb(
                    (c.0 as f64 * r_factor) as u8,
                    (c.1 as f64 * g_factor) as u8,
                    (c.2 as f64 * b_factor) as u8,
                )
            }
        }
    }

    /// Increment horizontal scroll
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

    /// Increment vertical scroll
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

    /// Loads the next background tile into the shifters
    fn load_next_tile(&mut self) {
        if self.rendering_enabled() {
            self.bg_lo_shift |= self.next_tile.lo as u16;
            self.bg_hi_shift |= self.next_tile.hi as u16;

            let attr = self.next_tile.attr;
            self.bg_attr_lo_shift |= if attr & 0x1 != 0 { 0xFF } else { 0x00 };
            self.bg_attr_hi_shift |= if attr & 0x2 != 0 { 0xFF } else { 0x00 };
        }
    }

    /// Shifts the backgroung shifters
    fn shift_bg(&mut self) {
        if self.mask.render_bg() {
            self.bg_lo_shift <<= 1;
            self.bg_hi_shift <<= 1;
            self.bg_attr_lo_shift <<= 1;
            self.bg_attr_hi_shift <<= 1;
        }
    }

    /// Shifts the foreground shifters
    fn shift_fg(&mut self) {
        if self.mask.render_sp() && (2..258).contains(&self.cycle) {
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

    /// Returns if the rendering is enabled or not
    fn rendering_enabled(&self) -> bool {
        self.mask.render_sp() | self.mask.render_bg()
    }

    /// Increments the VRAM address by 1 or 32 (based on the control register bit)
    fn increment_vaddr(&mut self) {
        let new_addr = self.v_addr.raw().wrapping_add(self.ctrl.increment());
        self.v_addr.set_raw(new_addr);
    }

    /// Reads from the Ppu bus
    fn mem_read(&mut self, addr: u16) -> u8 {
        self.bus.read(addr)
    }

    /// Writes to the Ppu bus
    fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.write(addr, data);
    }
}
