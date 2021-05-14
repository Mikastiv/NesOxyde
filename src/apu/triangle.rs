// The triangle channel produced triangle waves and was generally used for
// the baseline of the songs

use super::LENGTH_TABLE;

/// Table of the channel's output volume values
const OUTPUT_TABLE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
    13, 14, 15,
];

/// Audio triangle channel
pub struct Triangle {
    enabled: bool,
    phase: u8,

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
            phase: 0,

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
        self.phase = 0;

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
        // If disabled, set the length counter to zero
        if !v {
            self.length_counter = 0;
        }
    }

    /// Sets register 0x4008
    pub fn write_linear(&mut self, data: u8) {
        // CRRR RRRR
        // C: Control flag (linear counter halt and length counter halt)
        // R: Linear counter period
        self.counter_halt = data & 0x80 != 0;
        self.counter_period = data & 0x7F;
        if self.counter_halt {
            self.linear_counter = self.counter_period;
        }
    }

    /// Sets register 0x400A
    pub fn write_lo(&mut self, data: u8) {
        // TTTT TTTT
        // T: Timer period low
        self.timer_period = (self.timer_period & 0xFF00) | data as u16;
    }

    /// Sets register 0x400B
    pub fn write_hi(&mut self, data: u8) {
        // LLLL LTTT
        // L: Length counter table index
        // T: Timer period high
        self.timer_period = ((data & 0x7) as u16) << 8 | (self.timer_period & 0xFF);
        self.length_counter = LENGTH_TABLE[(data >> 3) as usize];
        // A write to this register sets the linear reload flag
        self.counter_reload = true;
    }

    /// Clocks the timer / divider
    pub fn tick_timer(&mut self) {
        // The timer counts down clocks and makes something happen
        // when it its 0. For the triangle channel, it increments the phase.
        // We can think of it as the timer period like
        // "how many cpu clocks between each phase increment"
        match self.timer == 0 {
            true => {
                // When the timer hits 0, we reset it to the timer period + 1
                self.timer = self.timer_period + 1;
                // If the conditions below are met, we advance the phase by 1,
                // wrapping to 0 if over 32
                if self.length_counter > 0 && self.linear_counter > 0 && self.timer_period > 1 {
                    self.phase = (self.phase + 1) % 32;
                }
            }
            false => self.timer -= 1,
        }
    }

    /// Clocks the length counter
    pub fn tick_length(&mut self) {
        // The length counter is a simple gate which lets the channel output
        // a signal when it is not 0. It can only be reloaded by writing to
        // register 0x400B.
        // It is like "for how many clocks the channel can output a signal"

        // If the length halt flag is not set and the counter is greater than
        // 0, decrement.
        if !self.counter_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    /// Clocks the linear counter
    pub fn tick_counter(&mut self) {
        // The triangle channel's linear counter is similar to the length counter
        // but is more precise because it is clocked at a faster rate

        // Check the reload flag
        match self.counter_reload {
            // If set, reload the linear counter
            true => self.linear_counter = self.counter_period,
            // Otherwise if the counter is > 0, decrement
            false if self.linear_counter > 0 => self.linear_counter -= 1,
            // Do nothing if it is 0
            _ => {}
        }

        // If the counter halt flag is clear, clear the counter reload flag
        if !self.counter_halt {
            self.counter_reload = false;
        }
    }

    /// Returns the output volume of the channel
    pub fn output(&self) -> u8 {
        // All the conditions below silence the channel.
        if !self.enabled || self.length_counter == 0 || self.linear_counter == 0 {
            return 0;
        }

        // The output signal is based on fixed values, changing with the phase
        OUTPUT_TABLE[self.phase as usize]
    }

    /// Returns the length counter value
    pub fn length_counter(&self) -> u8 {
        self.length_counter
    }
}
