const SQ1_VOL: u16 = 0x4000;
const SQ1_SWEEP: u16 = 0x4001;
const SQ1_LO: u16 = 0x4002;
const SQ1_HI: u16 = 0x4003;

const SQ2_VOL: u16 = 0x4004;
const SQ2_SWEEP: u16 = 0x4005;
const SQ2_LO: u16 = 0x4006;
const SQ2_HI: u16 = 0x4007;

const TRI_LINEAR: u16 = 0x4008;
const TRI_LO: u16 = 0x400A;
const TRI_HI: u16 = 0x400B;

const NOISE_VOL: u16 = 0x400C;
const NOISE_LO: u16 = 0x400E;
const NOISE_HI: u16 = 0x400F;

const DMC_FREQ: u16 = 0x4010;
const DMC_RAW: u16 = 0x4011;
const DMC_START: u16 = 0x4012;
const DMC_LEN: u16 = 0x4013;

const SND_CHN: u16 = 0x4015;
const FRAME_COUNTER: u16 = 0x4017;

mod square;

pub struct Apu {
    
}

impl Apu {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            SND_CHN => 0,
            _ => 0,
        }
    }

    pub fn write(&self, addr: u16, data: u8) {
        match addr {
            SQ1_VOL => {}
            SQ1_SWEEP => {}
            SQ1_LO => {}
            SQ1_HI => {}

            SQ2_VOL => {}
            SQ2_SWEEP => {}
            SQ2_LO => {}
            SQ2_HI => {}

            TRI_LINEAR => {}
            TRI_LO => {}
            TRI_HI => {}

            NOISE_VOL => {}
            NOISE_LO => {}
            NOISE_HI => {}

            DMC_FREQ => {}
            DMC_RAW => {}
            DMC_START => {}
            DMC_LEN => {}

            SND_CHN => {}
            FRAME_COUNTER => {}

            _ => {}
        }
    }
}
