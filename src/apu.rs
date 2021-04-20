const PULSE1_TIMER: u16 = 0x4000;
const PULSE1_LENGTH: u16 = 0x4001;
const PULSE1_ENV: u16 = 0x4002;
const PULSE1_SWEEP: u16 = 0x4003;
const PULSE2_TIMER: u16 = 0x4000;
const PULSE2_LENGTH: u16 = 0x4001;
const PULSE2_ENV: u16 = 0x4002;
const PULSE2_SWEEP: u16 = 0x4003;

pub struct Apu {}

impl Apu {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read(&self, addr: u16) -> u8 {
        0
    }

    pub fn write(&self, addr: u16, data: u8) {}
}
