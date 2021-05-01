const TIMER_TABLE: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];
const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

pub struct Noise {
    enabled: bool,
    mode: bool,

    timer_period: u16,
    timer: u16,

    length_halt: bool,
    length_counter: u8,

    constant_volume: bool,
    volume: u8,

    envelope_reload: bool,
    envelope_divider: u8,
    envelope_volume: u8,

    shift: u16,
}

impl Noise {
    pub fn new() -> Self {
        Self {
            enabled: false,
            mode: false,

            timer_period: 0,
            timer: 0,

            length_halt: false,
            length_counter: 0,

            constant_volume: false,
            volume: 0,

            envelope_reload: false,
            envelope_divider: 0,
            envelope_volume: 0,

            shift: 1,
        }
    }

    pub fn reset(&mut self) {
        self.enabled = false;
        self.mode = false;

        self.timer_period = 0;
        self.timer = 0;

        self.length_halt = false;
        self.length_counter = 0;

        self.constant_volume = false;
        self.volume = 0;

        self.envelope_reload = false;
        self.envelope_divider = 0;
        self.envelope_volume = 0;

        self.shift = 1;
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.enabled = v;
        if !v {
            self.length_counter = 0;
        }
    }

    pub fn write_vol(&mut self, data: u8) {
        self.length_halt = data & 0x20 != 0;
        self.constant_volume = data & 0x10 != 0;
        self.volume = data & 0xF;
    }

    pub fn write_lo(&mut self, data: u8) {
        self.mode = data & 0x80 != 0;
        self.timer_period = TIMER_TABLE[(data & 0xF) as usize];
    }

    pub fn write_hi(&mut self, data: u8) {
        self.envelope_reload = true;
        self.length_counter = LENGTH_TABLE[(data >> 3) as usize];
    }

    pub fn tick_timer(&mut self) {
        match self.timer == 0 {
            true => {
                self.timer = self.timer_period;

                let bit = match self.mode {
                    true => 6,
                    false => 1,
                };

                let feedback = (self.shift ^ (self.shift >> bit)) & 0x1;
                self.shift = (self.shift >> 1) | (feedback << 14);
            }
            false => self.timer -= 1,
        }
    }

    pub fn tick_length(&mut self) {
        if !self.length_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn tick_envelope(&mut self) {
        match self.envelope_reload {
            true => {
                self.envelope_volume = 15;
                self.envelope_divider = self.volume + 1;
                self.envelope_reload = false;
            }
            false if self.envelope_divider > 0 => self.envelope_divider -= 1,
            false => {
                if self.envelope_volume > 0 {
                    self.envelope_volume -= 1;
                } else if self.length_halt {
                    self.envelope_volume = 15;
                }

                self.envelope_divider = self.volume + 1;
            }
        }
    }

    pub fn output(&mut self) -> u8 {
        if !self.enabled || self.length_counter == 0 || self.shift & 0x1 != 0 {
            return 0;
        }

        match self.constant_volume {
            true => self.volume,
            false => self.envelope_volume,
        }
    }

    pub fn length_counter(&self) -> u8 {
        self.length_counter
    }
}
