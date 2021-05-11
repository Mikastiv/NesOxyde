// http://wiki.nesdev.com/w/index.php/APU_Length_Counter
/// Length counter values table
const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

/// Table of the channel's output volume values
const OUTPUT_TABLE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
    13, 14, 15,
];

/// Audio triangle channel
pub struct Triangle {
    enabled: bool,
    duty: u8,

    timer_period: u16,
    timer: u16,

    counter_halt: bool,
    length_counter: u8,

    counter_reload: bool,
    counter_period: u8,
    linear_counter: u8,
}

impl Triangle {
    pub fn new() -> Self {
        Self {
            enabled: false,
            duty: 0,

            timer_period: 0,
            timer: 0,

            counter_halt: false,
            length_counter: 0,

            counter_reload: false,
            counter_period: 0,
            linear_counter: 0,
        }
    }

    /// Resets the channel state
    pub fn reset(&mut self) {
        self.enabled = false;
        self.duty = 0;

        self.timer_period = 0;
        self.timer = 0;

        self.counter_halt = false;
        self.length_counter = 0;

        self.counter_reload = false;
        self.counter_period = 0;
        self.linear_counter = 0;
    }

    /// Enables or disables the channel
    pub fn set_enabled(&mut self, v: bool) {
        self.enabled = v;
        if !v {
            self.length_counter = 0;
        }
    }

    /// Sets register 0x4008
    pub fn write_linear(&mut self, data: u8) {
        self.counter_halt = data & 0x80 != 0;
        self.counter_period = data & 0x7F;
        if self.counter_halt {
            self.linear_counter = self.counter_period;
        }
    }
    
    /// Sets register 0x400A
    pub fn write_lo(&mut self, data: u8) {
        self.timer_period = (self.timer_period & 0xFF00) | data as u16;
    }
    
    /// Sets register 0x400B
    pub fn write_hi(&mut self, data: u8) {
        self.timer_period = ((data & 0x7) as u16) << 8 | (self.timer_period & 0xFF);
        self.length_counter = LENGTH_TABLE[(data >> 3) as usize];
        self.timer = self.timer_period + 1;
        self.counter_reload = true;
    }

    /// Clocks the timer / divider
    pub fn tick_timer(&mut self) {
        match self.timer == 0 {
            true => {
                self.timer = self.timer_period + 1;
                if self.length_counter > 0 && self.linear_counter > 0 && self.timer_period > 1 {
                    self.duty = (self.duty + 1) % 32;
                }
            }
            false => self.timer -= 1,
        }
    }

    /// Clocks the length counter
    pub fn tick_length(&mut self) {
        if !self.counter_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    /// Clocks the linear counter
    pub fn tick_counter(&mut self) {
        match self.counter_reload {
            true => self.linear_counter = self.counter_period,
            false if self.linear_counter > 0 => self.linear_counter -= 1,
            _ => {}
        }

        if !self.counter_halt {
            self.counter_reload = false;
        }
    }

    /// Returns the output volume of the channel
    pub fn output(&self) -> u8 {
        if !self.enabled || self.length_counter == 0 || self.linear_counter == 0 {
            return 0;
        }

        OUTPUT_TABLE[self.duty as usize]
    }

    pub fn length_counter(&self) -> u8 {
        self.length_counter
    }
}
