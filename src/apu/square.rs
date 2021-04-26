const DUTY_TABLE: [u8; 4] = [0b0100_0000, 0b0110_0000, 0b0111_1000, 0b1001_1111];
const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

pub struct Square {
    enabled: bool,

    duty: u8,
    duty_index: u8,
    timer: u16,
    timer_period: u16,

    length_halt: bool,
    length_counter: u8,

    constant_volume: bool,
    volume: u8,

    envelope_start: bool,
    envelope_divider: u8,
    envelope_volume: u8,
}

impl Square {
    pub fn new() -> Self {
        Self {
            enabled: false,

            duty: 0,
            duty_index: 0,
            timer: 0,
            timer_period: 0,

            length_halt: false,
            length_counter: 0,

            constant_volume: false,
            volume: 0,

            envelope_start: false,
            envelope_divider: 0,
            envelope_volume: 0,
        }
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.enabled = v;
        if !v {
            self.length_counter = 0;
        }
    }

    pub fn write_vol(&mut self, data: u8) {
        self.duty = data >> 6;
        self.length_halt = (data & 0x20) != 0;
        self.constant_volume = (data & 0x10) != 0;
        self.volume = data & 0xF;
        self.envelope_start = true;
    }

    pub fn write_lo(&mut self, data: u8) {
        self.timer_period = (self.timer_period & 0xFF00) | data as u16;
    }

    pub fn write_hi(&mut self, data: u8) {
        self.timer_period = ((data & 0x7) as u16) << 8 | (self.timer_period & 0xFF);
        self.length_counter = LENGTH_TABLE[(data >> 3) as usize];
        self.duty_index = 0;
        self.envelope_start = true;
    }

    pub fn tick_timer(&mut self) {
        match self.timer == 0 {
            true => {
                self.timer = self.timer_period + 1;
                self.duty_index = (self.duty_index + 1) % 8;
            }
            false => self.timer -= 1,
        }
    }

    pub fn tick_counter(&mut self) {
        if !self.length_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn tick_envelope(&mut self) {
        match self.envelope_start {
            true => {
                self.envelope_volume = 15;
                self.envelope_divider = self.volume + 1;
                self.envelope_start = false;
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

    pub fn output(&self) -> u8 {
        let duty = (DUTY_TABLE[self.duty as usize] & (1 << self.duty_index)) != 0;

        if !self.enabled
            || self.timer_period > 0x7FF
            || self.length_counter == 0
            || self.timer_period < 8
            || !duty
        {
            return 0;
        }

        match self.constant_volume {
            true => self.volume,
            false => self.envelope_volume,
        }
    }
}
