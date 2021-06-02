use bitflags::bitflags;

bitflags! {
    /// Ppu control register
    pub struct Controller: u8 {
        const NMI_ENABLED    = 0b10000000;
        const MASTER_SLAVE   = 0b01000000;
        const SPRITE_SIZE    = 0b00100000;
        const BG_ADDRESS     = 0b00010000;
        const SP_ADDRESS     = 0b00001000;
        const VRAM_INCREMENT = 0b00000100;
        const NAMETABLE_V    = 0b00000010;
        const NAMETABLE_H    = 0b00000001;
    }
}

impl Controller {
    /// Set all the register bits at once
    pub fn set_raw(&mut self, v: u8) {
        self.bits = v;
    }

    /// Returns the address increment value
    pub fn increment(&self) -> u16 {
        if self.contains(Self::VRAM_INCREMENT) {
            32
        } else {
            1
        }
    }

    /// Return the backgrounds base address
    pub fn bg_base_addr(&self) -> u16 {
        match self.contains(Self::BG_ADDRESS) {
            true => 0x1000,
            false => 0x0000,
        }
    }

    /// Return the sprites base address
    pub fn sp_base_addr(&self) -> u16 {
        match self.contains(Self::SP_ADDRESS) {
            true => 0x1000,
            false => 0x0000,
        }
    }

    /// Returns the NMI enabled flag value
    pub fn nmi_enabled(&self) -> bool {
        self.contains(Self::NMI_ENABLED)
    }

    /// Returns the nametable H flag value
    pub fn nta_h(&self) -> bool {
        self.contains(Self::NAMETABLE_H)
    }

    /// Returns the nametable V flag value
    pub fn nta_v(&self) -> bool {
        self.contains(Self::NAMETABLE_V)
    }

    /// Returns the sprite size flag value
    pub fn sprite_size(&self) -> bool {
        self.contains(Self::SPRITE_SIZE)
    }
}

bitflags! {
    /// Ppu mask register
    pub struct Mask: u8 {
        const EMPH_BLUE  = 0b10000000;
        const EMPH_GREEN = 0b01000000;
        const EMPH_RED   = 0b00100000;
        const SHOW_SP    = 0b00010000;
        const SHOW_BG    = 0b00001000;
        const SHOW_SP8   = 0b00000100;
        const SHOW_BG8   = 0b00000010;
        const GREYSCALE  = 0b00000001;
    }
}

impl Mask {
    /// Set all the register bits at once
    pub fn set_raw(&mut self, v: u8) {
        self.bits = v;
    }

    /// Returns the render background flag value
    pub fn render_bg(&self) -> bool {
        self.contains(Self::SHOW_BG)
    }

    /// Returns the render sprites flag value
    pub fn render_sp(&self) -> bool {
        self.contains(Self::SHOW_SP)
    }

    /// Returns the render left 8 background pixels value
    pub fn render_bg8(&self) -> bool {
        self.contains(Self::SHOW_BG8)
    }

    /// Returns the render left 8 sprites pixels value
    pub fn render_sp8(&self) -> bool {
        self.contains(Self::SHOW_SP8)
    }

    /// Returns the greyscale mask value
    pub fn greyscale_mask(&self) -> u8 {
        match self.contains(Self::GREYSCALE) {
            true => 0x30,
            false => 0xFF,
        }
    }

    /// Return the color emphasis factors
    pub fn emph_factors(&self) -> (f64, f64, f64) {
        let mut r_factor = 1.0;
        let mut g_factor = 1.0;
        let mut b_factor = 1.0;

        if self.contains(Self::EMPH_RED) {
            g_factor = 0.75;
            b_factor = 0.75;
        }
        if self.contains(Self::EMPH_GREEN) {
            r_factor = 0.75;
            b_factor = 0.75;
        }
        if self.contains(Self::EMPH_BLUE) {
            r_factor = 0.75;
            g_factor = 0.75;
        }
        (r_factor, g_factor, b_factor)
    }

    /// Returns true if one of the color emphasis bits is set
    pub fn color_emph_enabled(&self) -> bool {
        self.intersects(Self::EMPH_RED | Self::EMPH_GREEN | Self::EMPH_BLUE)
    }
}

bitflags! {
    /// Ppu status register
    pub struct Status: u8 {
        const IN_VBLANK   = 0b10000000;
        const SP_0_HIT    = 0b01000000;
        const SP_OVERFLOW = 0b00100000;
        const UNUSED      = 0b00011111;
    }
}

impl Status {
    /// Set all the register bits at once
    pub fn set_raw(&mut self, v: u8) {
        self.bits = v;
    }

    /// Sets the vblank flag value
    pub fn set_vblank(&mut self, v: bool) {
        self.set(Self::IN_VBLANK, v);
    }

    /// Sets the sprite zero hit value
    pub fn set_sp_0_hit(&mut self, v: bool) {
        self.set(Self::SP_0_HIT, v);
    }

    /// Sets the sprite overflow value
    pub fn set_sp_overflow(&mut self, v: bool) {
        self.set(Self::SP_OVERFLOW, v);
    }
}

const XCOARSE_MASK: u16 = 0b11111;
const YCOARSE_MASK: u16 = 0b11111;
const NTA_H_MASK: u16 = 0b1;
const NTA_V_MASK: u16 = 0b1;
const YFINE_MASK: u16 = 0b111;

const XCOARSE_SHIFT: u16 = 0;
const YCOARSE_SHIFT: u16 = 5;
const NTA_H_SHIFT: u16 = 10;
const NTA_V_SHIFT: u16 = 11;
const YFINE_SHIFT: u16 = 12;

#[derive(Default, Clone, Copy)]
/// Loopy Ppu register
pub struct Loopy {
    xcoarse: u8,
    ycoarse: u8,
    nta_h: bool,
    nta_v: bool,
    yfine: u8,
}

impl Loopy {
    pub fn new() -> Self {
        Self::default()
    }

    /// X coarse value
    pub fn xcoarse(&self) -> u8 {
        self.xcoarse
    }

    /// Set x coarse value
    pub fn set_xcoarse(&mut self, v: u8) {
        self.xcoarse = v & XCOARSE_MASK as u8;
    }

    /// Y coarse value
    pub fn ycoarse(&self) -> u8 {
        self.ycoarse
    }

    /// Set y coarse value
    pub fn set_ycoarse(&mut self, v: u8) {
        self.ycoarse = v & YCOARSE_MASK as u8;
    }

    /// Y fine value
    pub fn yfine(&self) -> u8 {
        self.yfine
    }

    /// Set y fine value
    pub fn set_yfine(&mut self, v: u8) {
        self.yfine = v & YFINE_MASK as u8;
    }

    /// Nametable H value
    pub fn nta_h(&self) -> bool {
        self.nta_h
    }

    /// Set nametable H value
    pub fn set_nta_h(&mut self, v: bool) {
        self.nta_h = v;
    }

    /// Nametable V value
    pub fn nta_v(&self) -> bool {
        self.nta_v
    }

    /// Set nametable V value
    pub fn set_nta_v(&mut self, v: bool) {
        self.nta_v = v;
    }

    /// Nametable address
    pub fn nta_addr(&self) -> u16 {
        ((self.nta_v as u16) << NTA_V_SHIFT) | ((self.nta_h as u16) << NTA_H_SHIFT)
    }

    /// Set address low bits
    pub fn set_addr_lo(&mut self, v: u8) {
        self.xcoarse = v & 0b0001_1111;
        self.ycoarse &= 0b0001_1000;
        self.ycoarse |= v >> 5;
    }

    /// Set address high bits
    pub fn set_addr_hi(&mut self, v: u8) {
        self.ycoarse &= 0b0000_0111;
        self.ycoarse |= (v & 0b0000_0011) << 3;
        self.nta_h = v & 0b0000_0100 != 0;
        self.nta_v = v & 0b0000_1000 != 0;
        self.yfine = v >> 4;
        self.yfine &= 0b0000_0111;
    }

    /// Returns the raw address value
    ///
    /// -yyy VHYY YYYX XXXX
    ///
    /// X: X coarse
    ///
    /// Y: Y coarse
    ///
    /// H: Nametable H
    ///
    /// V: Nametable V
    ///
    /// y: Y fine
    pub fn raw(&self) -> u16 {
        (self.xcoarse as u16) << XCOARSE_SHIFT
            | (self.ycoarse as u16) << YCOARSE_SHIFT
            | (self.nta_h as u16) << NTA_H_SHIFT
            | (self.nta_v as u16) << NTA_V_SHIFT
            | (self.yfine as u16) << YFINE_SHIFT
    }

    /// Set all the register bits at once
    pub fn set_raw(&mut self, v: u16) {
        self.xcoarse = ((v & (XCOARSE_MASK << XCOARSE_SHIFT)) >> XCOARSE_SHIFT) as u8;
        self.ycoarse = ((v & (YCOARSE_MASK << YCOARSE_SHIFT)) >> YCOARSE_SHIFT) as u8;
        self.nta_h = (v & (NTA_H_MASK << NTA_H_SHIFT)) != 0;
        self.nta_v = (v & (NTA_V_MASK << NTA_V_SHIFT)) != 0;
        self.yfine = ((v & (YFINE_MASK << YFINE_SHIFT)) >> YFINE_SHIFT) as u8;
    }

    /// Address of the next tile
    ///
    // 0x2000 to offset in VRAM space
    // The lower 12 bits of the address register represent an index
    // in one of the four nametables
    //
    /// 0010 VHYY YYYX XXXX
    ///
    /// V: Nametable V
    ///
    /// H: Nametable H
    ///
    /// Y: Coarse Y
    ///
    /// X: Coarse X
    //
    //   0                1
    // 0 +----------------+----------------+
    //   |                |                |
    //   |                |                |
    //   |    (32x32)     |    (32x32)     |
    //   |                |                |
    //   |                |                |
    // 1 +----------------+----------------+
    //   |                |                |
    //   |                |                |
    //   |    (32x32)     |    (32x32)     |
    //   |                |                |
    //   |                |                |
    //   +----------------+----------------+
    pub fn tile_addr(&self) -> u16 {
        0x2000 | (self.raw() & 0xFFF)
    }

    /// Address of the next tile attribute byte
    ///
    /// 0010 0011 11YY YXXX
    ///
    /// Y: Higher 3 bits of Y coarse
    ///
    /// X: Higher 3 bits of X coarse
    //
    // The last 2 row (last 64 bytes) of each nametable columns are attribute bytes
    pub fn tile_attr_addr(&self) -> u16 {
        0x23C0
            | self.nta_addr()
            | ((self.ycoarse() & 0x1C) << 1) as u16
            | (self.xcoarse() >> 2) as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loopy_raw() {
        let mut loopy = Loopy::new();

        loopy.set_raw(0b0110_0000_0001_0001);
        assert_eq!(loopy.raw(), 0b0110_0000_0001_0001);
        assert_eq!(loopy.xcoarse(), 0b10001);
        assert_eq!(loopy.ycoarse(), 0);
        assert_eq!(loopy.nta_h, false);
        assert_eq!(loopy.nta_v, false);
        assert_eq!(loopy.yfine(), 0b110);

        loopy.set_raw(0b0101_1010_1101_0111);
        assert_eq!(loopy.raw(), 0b0101_1010_1101_0111);
        assert_eq!(loopy.xcoarse(), 0b10111);
        assert_eq!(loopy.ycoarse(), 0b10110);
        assert_eq!(loopy.nta_h, false);
        assert_eq!(loopy.nta_v, true);
        assert_eq!(loopy.yfine(), 0b101);
    }

    #[test]
    fn test_loopy_addr() {
        let mut loopy = Loopy::new();

        loopy.set_addr_lo(0b1001_1011);
        assert_eq!(loopy.raw(), 0b0000_0000_1001_1011);
        assert_eq!(loopy.xcoarse(), 0b11011);
        assert_eq!(loopy.ycoarse(), 0b00100);
        assert_eq!(loopy.nta_h, false);
        assert_eq!(loopy.nta_v, false);
        assert_eq!(loopy.yfine(), 0);

        loopy.set_addr_lo(0);
        loopy.set_addr_hi(0b1011_0111);
        assert_eq!(loopy.raw(), 0b0011_0111_0000_0000);
        assert_eq!(loopy.xcoarse(), 0);
        assert_eq!(loopy.ycoarse(), 0b11000);
        assert_eq!(loopy.nta_h, true);
        assert_eq!(loopy.nta_v, false);
        assert_eq!(loopy.yfine(), 0b011);
    }

    #[test]
    fn test_loopy_setters() {
        let mut loopy = Loopy::new();

        loopy.set_xcoarse(0b1111_1111);
        assert_eq!(loopy.xcoarse(), 0b11111);
        loopy.set_xcoarse(0);

        loopy.set_ycoarse(0b1111_1111);
        assert_eq!(loopy.ycoarse(), 0b11111);
        loopy.set_ycoarse(0);

        loopy.set_yfine(0b1111_1111);
        assert_eq!(loopy.yfine(), 0b111);
    }
}
