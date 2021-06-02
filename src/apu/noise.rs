// The noise channels produced white noise and was generally used for the
// percussions of the songs

use serde::{Deserialize, Serialize};

use super::LENGTH_TABLE;

/// Table of the different timer periods
const TIMER_TABLE: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

/// Audio noise channel
#[derive(Serialize, Deserialize)]
pub struct Noise {
    enabled: bool,
    mode: bool,

    timer_period: u16,
    timer: u16,

    length_halt: bool,
    length_counter: u8,

    constant_volume: bool,
    volume: u8,

    envelope_timer: u8,
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

            envelope_timer: 0,
            envelope_volume: 0,

            shift: 1,
        }
    }

    /// Resets the channel state
    pub fn reset(&mut self) {
        self.enabled = false;
        self.mode = false;

        self.timer_period = 0;
        self.timer = 0;

        self.length_halt = false;
        self.length_counter = 0;

        self.constant_volume = false;
        self.volume = 0;

        self.envelope_volume = 0;

        self.shift = 1;
    }

    /// Enables or disables the channel
    pub fn set_enabled(&mut self, v: bool) {
        self.enabled = v;
        // If disabled, set the length counter to zero
        if !v {
            self.length_counter = 0;
        }
    }

    /// Sets register 0x400C
    pub fn write_vol(&mut self, data: u8) {
        // --LC VVVV
        // L: Envelope loop / length counter halt
        // C: Output constant volume
        // V: Volume value / envelope period
        self.length_halt = data & 0x20 != 0;
        self.constant_volume = data & 0x10 != 0;
        self.volume = data & 0xF;
    }

    /// Sets register 0x400E
    pub fn write_lo(&mut self, data: u8) {
        // M--- PPPP
        // M: Mode flag
        // P: Timer period table index
        self.mode = data & 0x80 != 0;
        self.timer_period = TIMER_TABLE[(data & 0xF) as usize];
    }

    /// Sets register 0x400F
    pub fn write_hi(&mut self, data: u8) {
        // LLLL L---
        // L: Length counter table index
        self.length_counter = LENGTH_TABLE[(data >> 3) as usize];
        // Also restarts the envelope generator
        self.envelope_volume = 15;
        self.envelope_timer = self.volume + 1;
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

    /// Clocks the length counter
    pub fn tick_length(&mut self) {
        // The length counter is a simple gate which lets the channel output
        // a signal when it is not 0. It can only be reloaded by writing to
        // register 0x400F.
        // It is like "for how many clocks the channel can output a signal"

        // If the length halt flag is not set and the counter is greater than
        // 0, decrement.
        if !self.length_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    /// Clocks the envelope
    pub fn tick_envelope(&mut self) {
        match self.envelope_timer > 0 {
            // If the timer is not 0, decrement it.
            true => self.envelope_timer -= 1,
            // If it is 0...
            false => {
                // If the volume is not 0, decrement it
                if self.envelope_volume > 0 {
                    self.envelope_volume -= 1;
                // Otherwise if it is 0 and the loop flag is set,
                // reset it to 15
                } else if self.length_halt {
                    self.envelope_volume = 15;
                }

                // Reset the timer to its period value + 1
                self.envelope_timer = self.volume + 1;
            }
        }
    }

    /// Returns the output volume of the channel
    pub fn output(&mut self) -> u8 {
        // All the conditions below silence the channel.
        if !self.enabled || self.length_counter == 0 || self.shift & 0x1 != 0 {
            return 0;
        }

        // Check if we should output constant volume or the envelope volume
        match self.constant_volume {
            true => self.volume,
            false => self.envelope_volume,
        }
    }

    /// Returns the length counter value
    pub fn length_counter(&self) -> u8 {
        self.length_counter
    }
}
