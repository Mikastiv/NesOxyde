bitflags! {
    pub struct Controller: u8 {
        const NMI_ENABLED    = 0b10000000;
        const MASTER_SLAVE   = 0b01000000;
        const SPRITE_SIZE    = 0b00100000;
        const BG_ADDRESS     = 0b00010000;
        const SP_ADDRESS     = 0b00001000;
        const VRAM_INCREMENT = 0b00000100;
        const NAMETABLE2     = 0b00000010;
        const NAMETABLE1     = 0b00000001;
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
    pub fn set_raw(&mut self, v: u8) {
        self.bits = v;
    }
}
