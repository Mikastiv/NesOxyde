// 0 - 0 1 0 0 0 0 0 0 (12.5%)
// 1 - 0 1 1 0 0 0 0 0 (25%)
// 2 - 0 1 1 1 1 0 0 0 (50%)
// 3 - 1 0 0 1 1 1 1 1 (25% negated)
/// Table of the different duty cycles
const DUTY_TABLE: [u8; 4] = [0b0100_0000, 0b0110_0000, 0b0111_1000, 0b1001_1111];

// http://wiki.nesdev.com/w/index.php/APU_Length_Counter
/// Length counter values table
const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

/// Channel 1 or 2
pub enum Channel {
    One,
    Two,
}

/// Audio square channel
pub struct Square {
    enabled: bool,

    duty: u8,
    duty_phase: u8,
    timer: u16,
    timer_period: u16,

    length_halt: bool,
    length_counter: u8,

    constant_volume: bool,
    volume: u8,

    sweep_enabled: bool,
    sweep_negate: bool,
    sweep_period: u8,
    sweep_shift: u8,
    sweep: u8,

    envelope_loop: bool,
    envelope_divider: u8,
    envelope_volume: u8,
}

impl Square {
    pub fn new() -> Self {
        Self {
            enabled: false,

            /// Selected duty cycle
            duty: 0,
            /// Duty cycle phase (bit position in the duty cycle)
            duty_phase: 0,
            /// Timer or clock divider
            timer: 0,
            /// Start value of the timer
            timer_period: 0,

            length_halt: false,
            length_counter: 0,

            constant_volume: false,
            volume: 0,

            sweep_enabled: false,
            sweep_negate: false,
            sweep_period: 0,
            sweep_shift: 0,
            sweep: 0,

            envelope_loop: false,
            envelope_divider: 0,
            envelope_volume: 0,
        }
    }

    pub fn reset(&mut self) {
        self.enabled = false;

        self.duty = 0;
        self.duty_phase = 0;
        self.timer = 0;
        self.timer_period = 0;

        self.length_halt = false;
        self.length_counter = 0;

        self.constant_volume = false;
        self.volume = 0;

        self.sweep_enabled = false;
        self.sweep_negate = false;
        self.sweep_period = 0;
        self.sweep_shift = 0;
        self.sweep = 0;

        self.envelope_loop = false;
        self.envelope_divider = 0;
        self.envelope_volume = 0;
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.enabled = v;
        if !v {
            self.length_counter = 0;
        }
    }

    pub fn write_vol(&mut self, data: u8) {
        self.duty = data >> 6;
        self.length_halt = data & 0x20 != 0;
        self.envelope_loop = self.length_halt;
        self.constant_volume = data & 0x10 != 0;
        self.volume = data & 0xF;
    }

    pub fn write_sweep(&mut self, data: u8) {
        self.sweep_enabled = data & 0x80 != 0;
        self.sweep_period = (data & 0x70) >> 4;
        self.sweep_negate = data & 0x8 != 0;
        self.sweep_shift = data & 0x7;
        self.sweep = self.sweep_period + 1;
    }

    pub fn write_lo(&mut self, data: u8) {
        self.timer_period = (self.timer_period & 0xFF00) | data as u16;
    }

    pub fn write_hi(&mut self, data: u8) {
        self.timer_period = ((data & 0x7) as u16) << 8 | (self.timer_period & 0xFF);
        self.length_counter = LENGTH_TABLE[(data >> 3) as usize];
        self.duty_phase = 0;
        self.envelope_volume = 15;
        self.envelope_divider = self.volume + 1;
    }

    pub fn tick_timer(&mut self) {
        match self.timer == 0 {
            true => {
                self.timer = self.timer_period + 1;
                self.duty_phase = (self.duty_phase + 1) & 0x7;
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
        match self.envelope_divider > 0 {
            true => self.envelope_divider -= 1,
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

    pub fn tick_sweep(&mut self, channel: Channel) {
        match self.sweep > 0 {
            true => self.sweep -= 1,
            false => self.sweep = self.sweep_period + 1,
        }

        if self.sweep_enabled && self.timer_period > 7 && self.sweep_shift > 0 {
            self.sweep(channel);
        }
    }

    fn sweep(&mut self, channel: Channel) {
        let delta = self.timer_period >> self.sweep_shift;

        let wl = match self.sweep_negate {
            true => match channel {
                Channel::One => self.timer_period + !delta,
                Channel::Two => self.timer_period - delta,
            } 
            false => self.timer_period +  delta
        };

        if wl < 0x800 {
            self.timer_period = wl;
        }
    }

    pub fn output(&self) -> u8 {
        let duty = (DUTY_TABLE[self.duty as usize] & (1 << self.duty_phase)) != 0;

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

    pub fn length_counter(&self) -> u8 {
        self.length_counter
    }
}
