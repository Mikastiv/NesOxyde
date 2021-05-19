const RATE_TABLE: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
];

/// Delta modulation channel
pub struct Dmc {
    enabled: bool,

    irq: bool,
    pending_irq: Option<bool>,
    loop_flag: bool,
    rate: u16,
    rate_counter: u16,

    pending_read: Option<bool>,
    address: u8,
    curr_address: u16,
    buffer: u8,
    phase: u8,

    output_level: u8,
    length_counter: u16,
    pcm_length: u16,
}

impl Dmc {
    pub fn new() -> Self {
        Self {
            enabled: false,

            irq: false,
            pending_irq: None,
            loop_flag: false,
            rate: 0,
            rate_counter: 0,

            pending_read: None,
            address: 0,
            curr_address: 0xC000,
            buffer: 0,
            phase: 0,

            output_level: 0,
            length_counter: 0,
            pcm_length: 0,
        }
    }

    /// Resets the channel state
    pub fn reset(&mut self) {
        self.enabled = false;

        self.irq = false;
        self.pending_irq = None;
        self.loop_flag = false;
        self.rate = 0;
        self.rate_counter = 0;

        self.pending_read = None;
        self.address = 0;
        self.curr_address = 0xC000;
        self.buffer = 0;
        self.phase = 0;

        self.output_level = 0;
        self.length_counter = 0;
        self.pcm_length = 0;
    }

    /// Enables or disables the channel
    pub fn set_enabled(&mut self, v: bool) {
        self.enabled = v;
        match v {
            true if self.length_counter == 0 => self.length_counter = self.pcm_length * 16 + 1,
            false => self.length_counter = 0,
            _ => {}
        }
    }

    /// Sets register 0x4010
    pub fn write_freq(&mut self, data: u8) {
        // IL-- RRRR
        // I: IRQ enable
        // L: Loop flag
        // R: Rate index (frequency)
        self.rate = RATE_TABLE[(data & 0xF) as usize];
        self.loop_flag = data & 0x40 != 0;
        self.irq = data & 0x80 != 0;
    }

    /// Sets register 0x4011
    pub fn write_raw(&mut self, data: u8) {
        // -DDD DDDD
        // D: Raw PCM sample
        self.output_level = data & 0x7F;
    }

    /// Sets register 0x4012
    pub fn write_start(&mut self, data: u8) {
        // AAAA AAAA
        // A: Sample start address
        self.address = data;
        // The address is always calculated like below
        self.curr_address = 0xC000 + (data as u16 * 64);
    }

    /// Sets register 0x4013
    pub fn write_len(&mut self, data: u8) {
        // LLLL LLLL
        // L: Sample length (how many samples to play)
        self.pcm_length = data as u16;
        // The number of samples to play is calculated like below
        self.length_counter = self.pcm_length * 16 + 1;
    }

    /// Clocks the DMC
    pub fn tick(&mut self) {
        match self.rate_counter == 0 {
            // If the counter is 0, clock the timer and reset counter
            true => {
                self.tick_timer();
                self.rate_counter = self.rate;
            }
            // Otherwise decrement
            false => self.rate_counter -= 1,
        }
    }

    /// Clocks the DMC timer
    fn tick_timer(&mut self) {
        // If the phase == 0, the PCM or DPCM sample has been played
        if self.phase == 0 {
            // If the length counter == 0 (all the samples have been played)
            // and the loop flag is set, we load the start address and
            // reset the length counter
            if self.length_counter == 0 && self.loop_flag {
                self.length_counter = self.pcm_length * 16 + 1;
                self.curr_address = 0xC000 + (self.address as u16 * 64);
            }
            match self.length_counter > 0 {
                // If if is greater than 0, load the next sample,
                // reset the phase to 8 (sample are only 7 bits) 
                // and decrement the counter
                true => {
                    self.pending_read = Some(true);
                    self.phase = 8;
                    self.length_counter -= 1;
                }
                // Otherwise, set the IRQ flag is enabled
                false => match self.irq {
                    true => self.pending_irq = Some(true),
                    // Disable the channel if IRQ is disabled
                    false => self.enabled = false,
                },
            }
        }
        // Here, the current sample is not done playing
        if self.phase != 0 {
            // Decrement the phase
            self.phase -= 1;
            // Check the bit of the current phase
            let delta = (self.buffer & (0x80 >> self.phase)) != 0;
            // The new output volume is simply incremented or decremented
            // by 2 based on the phase bit
            let v = match delta {
                // If the bit is set, add 2
                true => self.output_level.wrapping_add(2),
                // Otherwise, subtract 2
                false => self.output_level.wrapping_sub(2),
            };
            // The output is only changed if it stays between 0 and 127
            if (0..=0x7F).contains(&v) && self.enabled {
                self.output_level = v;
            }
        }
    }

    /// Returns the address of the next sample
    pub fn address(&self) -> u16 {
        self.curr_address
    }

    /// Sets the audio sample of the channel
    pub fn set_sample(&mut self, sample: u8) {
        self.buffer = sample;
        // Increments the address after updating the sample
        // Note that bit 15 is always set
        self.curr_address = self.curr_address.wrapping_add(1) | 0x8000;
    }

    /// Returns if the channel needs a sample or not
    pub fn need_sample(&mut self) -> bool {
        self.pending_read.take().is_some()
    }

    /// Polls the IRQ flag
    pub fn poll_irq(&mut self) -> bool {
        self.pending_irq.take().is_some()
    }

    /// Returns the length counter value
    pub fn length_counter(&self) -> u16 {
        self.length_counter
    }

    /// Returns the output volume of the channel
    pub fn output(&self) -> u8 {
        self.output_level
    }
}
