const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

const OUTPUT_TABLE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
    13, 14, 15,
];

pub struct Triangle {
    enabled: bool,
    duty: u8,

    timer_period: u16,
    timer: u16,

    length_halt: bool,
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

            length_halt: false,
            length_counter: 0,

            counter_reload: false,
            counter_period: 0,
            linear_counter: 0,
        }
    }

    pub fn reset(&mut self) {
        self.enabled = false;
        self.duty = 0;

        self.timer_period = 0;
        self.timer = 0;

        self.length_halt = false;
        self.length_counter = 0;

        self.counter_reload = false;
        self.counter_period = 0;
        self.linear_counter = 0;
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.enabled = v;
        if !v {
            self.length_counter = 0;
        }
    }

    pub fn write_linear(&mut self, data: u8) {
        self.length_halt = data & 0x80 != 0;
        self.counter_period = data & 0x7F;
    }

    pub fn write_lo(&mut self, data: u8) {
        self.timer_period = (self.timer_period & 0xFF00) | data as u16;
    }

    pub fn write_hi(&mut self, data: u8) {
        self.timer_period = ((data & 0x7) as u16) << 8 | (self.timer_period & 0xFF);
        self.length_counter = LENGTH_TABLE[(data >> 3) as usize];
        self.timer = self.timer_period + 1;
        self.counter_reload = true;
    }

    pub fn tick_timer(&mut self) {
        match self.timer == 0 {
            true => {
                self.timer = self.timer_period + 1;
                if self.length_counter > 0 && self.linear_counter > 0 {
                    self.duty = (self.duty + 1) % 32;
                }
            }
            false => self.timer -= 1,
        }
    }

    pub fn tick_length(&mut self) {
        if !self.length_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn tick_counter(&mut self) {
        match self.counter_reload {
            true => self.linear_counter = self.counter_period,
            false if self.linear_counter > 0 => self.linear_counter -= 1,
            _ => {}
        }

        if !self.length_halt {
            self.counter_reload = false;
        }
    }

    pub fn output(&self) -> u8 {
        if !self.enabled
            || self.length_counter == 0
            || self.linear_counter == 0
            // Remove screaming noise from megaman 1 and 2 (Emulator does not behave like real hardware)
            || self.timer_period < 2
        {
            return 0;
        }

        OUTPUT_TABLE[self.duty as usize]
    }

    pub fn length_counter(&self) -> u8 {
        self.length_counter
    }
}
