bitflags! {
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
    pub fn set_raw(&mut self, v: u8) {
        self.bits = v;
    }

    pub fn increment(&self) -> u16 {
        if self.contains(Self::VRAM_INCREMENT) {
            32
        } else {
            1
        }
    }

    pub fn bg_base_addr(&self) -> u16 {
        match self.contains(Self::BG_ADDRESS) {
            true => 0x1000,
            false => 0x0000,
        }
    }

    pub fn nmi_enabled(&self) -> bool {
        self.contains(Self::NMI_ENABLED)
    }

    pub fn nta_h(&self) -> bool {
        self.contains(Self::NAMETABLE_H)
    }

    pub fn nta_v(&self) -> bool {
        self.contains(Self::NAMETABLE_V)
    }
}

bitflags! {
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
    pub fn set_raw(&mut self, v: u8) {
        self.bits = v;
    }

    pub fn render_bg(&self) -> bool {
        self.contains(Self::SHOW_BG)
    }

    pub fn render_sp(&self) -> bool {
        self.contains(Self::SHOW_SP)
    }

    pub fn render_bg8(&self) -> bool {
        self.contains(Self::SHOW_BG8)
    }
}

bitflags! {
    pub struct Status: u8 {
        const IN_VBLANK   = 0b10000000;
        const SP_0_HIT    = 0b01000000;
        const SP_OVERFLOW = 0b00100000;
        const UNUSED      = 0b00011111;
    }
}

impl Status {
    pub fn set_vblank(&mut self, v: bool) {
        self.set(Self::IN_VBLANK, v);
    }

    pub fn set_sp_0_hit(&mut self, v: bool) {
        self.set(Self::SP_0_HIT, v);
    }

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

    pub fn xcoarse(&self) -> u8 {
        self.xcoarse
    }

    pub fn set_xcoarse(&mut self, v: u8) {
        self.xcoarse = v & XCOARSE_MASK as u8;
    }

    pub fn ycoarse(&self) -> u8 {
        self.ycoarse
    }

    pub fn set_ycoarse(&mut self, v: u8) {
        self.ycoarse = v & YCOARSE_MASK as u8;
    }

    pub fn yfine(&self) -> u8 {
        self.yfine
    }

    pub fn set_yfine(&mut self, v: u8) {
        self.yfine = v & YFINE_MASK as u8;
    }

    pub fn nta_h(&self) -> bool {
        self.nta_h
    }

    pub fn set_nta_h(&mut self, v: bool) {
        self.nta_h = v;
    }

    pub fn nta_v(&self) -> bool {
        self.nta_v
    }

    pub fn set_nta_v(&mut self, v: bool) {
        self.nta_v = v;
    }

    pub fn nta_addr(&self) -> u16 {
        ((self.nta_v as u16) << NTA_V_SHIFT) | ((self.nta_h as u16) << NTA_H_SHIFT)
    }

    pub fn set_addr_lo(&mut self, v: u8) {
        self.xcoarse = v & 0b0001_1111;
        self.ycoarse &= 0b0001_1000;
        self.ycoarse |= v >> 5;
    }

    pub fn set_addr_hi(&mut self, v: u8) {
        self.ycoarse &= 0b0000_0111;
        self.ycoarse |= (v & 0b0000_0011) << 3;
        self.nta_h = v & 0b0000_0100 != 0;
        self.nta_v = v & 0b0000_1000 != 0;
        self.yfine = v >> 4;
        self.yfine &= 0b0000_0111;
    }

    pub fn raw(&self) -> u16 {
        (self.xcoarse as u16) << XCOARSE_SHIFT
            | (self.ycoarse as u16) << YCOARSE_SHIFT
            | (self.nta_h as u16) << NTA_H_SHIFT
            | (self.nta_v as u16) << NTA_V_SHIFT
            | (self.yfine as u16) << YFINE_SHIFT
    }

    pub fn set_raw(&mut self, v: u16) {
        self.xcoarse = ((v & (XCOARSE_MASK << XCOARSE_SHIFT)) >> XCOARSE_SHIFT) as u8;
        self.ycoarse = ((v & (YCOARSE_MASK << YCOARSE_SHIFT)) >> YCOARSE_SHIFT) as u8;
        self.nta_h = (v & (NTA_H_MASK << NTA_H_SHIFT)) != 0;
        self.nta_v = (v & (NTA_V_MASK << NTA_V_SHIFT)) != 0;
        self.yfine = ((v & (YFINE_MASK << YFINE_SHIFT)) >> YFINE_SHIFT) as u8;
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
