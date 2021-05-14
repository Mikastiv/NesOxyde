// The square channels produced square waves and were generally used for the
// melody of the songs

use super::LENGTH_TABLE;

// 0 - 0 1 0 0 0 0 0 0 (12.5%)
// 1 - 0 1 1 0 0 0 0 0 (25%)
// 2 - 0 1 1 1 1 0 0 0 (50%)
// 3 - 1 0 0 1 1 1 1 1 (25% negated)
/// Table of the different duty cycles
const DUTY_TABLE: [u8; 4] = [0b0100_0000, 0b0110_0000, 0b0111_1000, 0b1001_1111];

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
    sweep_timer: u8,

    envelope_loop: bool,
    envelope_period: u8,
    envelope_timer: u8,
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
            sweep_timer: 0,

            envelope_loop: false,
            envelope_period: 0,
            envelope_timer: 0,
            envelope_volume: 0,
        }
    }

    /// Resets the channel state
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
        self.sweep_timer = 0;

        self.envelope_loop = false;
        self.envelope_period = 0;
        self.envelope_timer = 0;
        self.envelope_volume = 0;
    }

    /// Enables or disables the channel
    pub fn set_enabled(&mut self, v: bool) {
        self.enabled = v;
        // If disabled, set the length counter to zero
        if !v {
            self.length_counter = 0;
        }
    }

    /// Sets register 0x4000 / 0x4004
    pub fn write_vol(&mut self, data: u8) {
        // DDLC VVVV
        // D: Duty cycle
        // L: Envelope loop / length counter halt
        // C: Output constant volume
        // V: Volume value / envelope period
        self.duty = data >> 6;
        self.length_halt = data & 0x20 != 0;
        self.envelope_loop = self.length_halt;
        self.constant_volume = data & 0x10 != 0;
        self.volume = data & 0xF;
        self.envelope_period = self.volume;
    }

    /// Sets register 0x4001 / 0x4005
    pub fn write_sweep(&mut self, data: u8) {
        // Sweep unit
        // EPPP NSSS
        // E: Enabled
        // P: Period
        // N: Negate
        // S: Shift
        self.sweep_enabled = data & 0x80 != 0;
        self.sweep_period = (data & 0x70) >> 4;
        self.sweep_negate = data & 0x8 != 0;
        self.sweep_shift = data & 0x7;
        // A write to this register reloads the sweep
        self.sweep_timer = self.sweep_period + 1;
    }

    /// Sets register 0x4002 / 0x4006
    pub fn write_lo(&mut self, data: u8) {
        // TTTT TTTT
        // T: Timer period low
        self.timer_period = (self.timer_period & 0xFF00) | data as u16;
    }

    /// Sets register 0x4003 / 0x4007
    pub fn write_hi(&mut self, data: u8) {
        // LLLL LTTT
        // L: Length counter table index
        // T: Timer period high
        self.timer_period = ((data & 0x7) as u16) << 8 | (self.timer_period & 0xFF);
        self.length_counter = LENGTH_TABLE[(data >> 3) as usize];
        // A write to this register resets the duty phase and the envelope volume + timer
        self.duty_phase = 0;
        self.envelope_volume = 15;
        self.envelope_timer = self.envelope_period + 1;
    }

    /// Clocks the timer / divider
    pub fn tick_timer(&mut self) {
        // The timer counts down clocks and makes something happen
        // when it its 0. For the square channel, it increments the duty phase.
        // We can think of it as the timer period like
        // "how many cpu clocks between each phase increment"
        // A smaller period produces a higher frequency audio wave
        match self.timer == 0 {
            // When the timer hits 0, we reset it to the timer period + 1
            // and we advance the duty phase by 1, wrapping to 0 if over 7
            true => {
                self.timer = self.timer_period + 1;
                self.duty_phase = (self.duty_phase + 1) % 8;
            }
            // Otherwise, decrement the timer
            false => self.timer -= 1,
        }
    }

    /// Clocks the length counter
    pub fn tick_length(&mut self) {
        // The length counter is a simple gate which lets the channel output
        // a signal when it is not 0. It can only be reloaded by writing to
        // register 0x4003 / 0x4007.
        // It is like "for how many clocks the channel can output a signal"

        // If the length halt flag is not set and the counter is greater than
        // 0, decrement.
        if !self.length_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    /// Clocks the envelope
    pub fn tick_envelope(&mut self) {
        // The envelope generator controls the volume of the channel.
        // It can generate constant volume (square wave)
        // _________       _________
        // |       |       |       |
        // |       |       |       |
        // |       |_______|       |
        //
        // Or a decreasing saw envelope (saw wave)
        //
        // |\      |\      |\
        // |  \    |  \    |  \
        // |    \  |    \  |    \
        //
        // Similar to the timer, the envelope timer makes something
        // happen when it hits 0.
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
                } else if self.envelope_loop {
                    self.envelope_volume = 15;
                }

                // Reset the timer to its period value + 1
                self.envelope_timer = self.envelope_period + 1;
            }
        }
    }

    /// Clocks the sweep unit
    pub fn tick_sweep(&mut self, channel: Channel) {
        // The sweep unit can make the audio frequency of the channel
        // increase rapidly or decrease rapidly. This causes a sweeping sound
        // to be output. The easiest example to understand this is the sound
        // when Mario slides down the flag pole.

        // Again, similar to the timer, the sweep timer makes something
        // happen when it hits 0.
        match self.sweep_timer > 0 {
            // If the timer is not 0, decrement it.
            true => self.sweep_timer -= 1,
            // If it is 0...
            false => {
                // If all the conditions below are met, we apply the sweep
                if self.sweep_enabled && self.timer_period > 7 && self.sweep_shift > 0 {
                    self.sweep(channel);
                }

                // Reset the timer to its period + 1
                self.sweep_timer = self.sweep_period + 1;
            }
        }
    }

    /// Applies a sweep to the timer period
    fn sweep(&mut self, channel: Channel) {
        // Sweeping affects the channel's timer period

        // We first calculate the delta with the sweep shift value
        let delta = self.timer_period >> self.sweep_shift;

        // The delta is then applied to the channel's timer
        // based on the negate flag
        self.timer_period = match self.sweep_negate {
            // If it is set, we substract the delta from the timer
            true => match channel {
                // Channel 1 substracts by adding the one's complement
                Channel::One => self.timer_period + !delta,
                // Channel 2 substracts by adding the two's complement
                Channel::Two => self.timer_period - delta,
            },
            // If it isn't set, add the delta
            false => self.timer_period + delta,
        };
    }

    /// Returns the output volume of the channel
    pub fn output(&self) -> u8 {
        // Check the phase of the duty cycle, 1 outputs a signal and 0 doesn't (see duty table at the top)
        let duty = (DUTY_TABLE[self.duty as usize] & (1 << self.duty_phase)) != 0;

        // All the conditions below silence the channel.
        // Disabled ?
        if !self.enabled
        // Timer period overflowed as a result of the sweep add operation
            || self.timer_period > 0x7FF
            // The length counter is 0
            || self.length_counter == 0
            // The timer period is smaller than 8
            || self.timer_period < 8
            // The duty is false (on a 0 bit in the duty cycle)
            || !duty
        {
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
