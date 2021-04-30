const RATE_TABLE: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
];

pub struct Dmc {
    enabled: bool,

    irq: bool,
    pending_irq: Option<bool>,
    loop_flag: bool,
    rate: u16,
    rate_counter: u16,

    address: u8,
    curr_address: u16,
    buffer: u8,
    phase: u8,

    output_level: u8,
    length_counter: u16,
    pcm_length: u8,
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

            address: 0,
            curr_address: 0xC000,
            buffer: 0,
            phase: 0,

            output_level: 0,
            length_counter: 0,
            pcm_length: 0,
        }
    }

    pub fn reset(&mut self) {
        self.enabled = false;

        self.irq = false;
        self.pending_irq = None;
        self.loop_flag = false;
        self.rate = 0;
        self.rate_counter = 0;

        self.address = 0;
        self.curr_address = 0xC000;
        self.buffer = 0;
        self.phase = 0;

        self.output_level = 0;
        self.length_counter = 0;
        self.pcm_length = 0;
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.enabled = v;
    }

    pub fn write_freq(&mut self, data: u8) {
        self.rate = RATE_TABLE[(data & 0xF) as usize];
        self.loop_flag = data & 0x40 != 0;
        self.irq = data & 0x80 != 0;
    }

    pub fn write_raw(&mut self, data: u8) {
        self.output_level = data & 0x7F;
    }

    pub fn write_start(&mut self, data: u8) {
        self.address = data;
        self.curr_address = 0xC000 + (data as u16 * 64);
    }

    pub fn write_len(&mut self, data: u8) {
        self.pcm_length = data;
        self.length_counter = data as u16 * 16 + 1;
    }

    pub fn tick(&mut self) {
        match self.rate_counter == 0 {
            true => {
                self.tick_timer();
                self.rate_counter = self.rate;
            }
            false => self.rate_counter -= 1,
        }
    }

    fn tick_timer(&mut self) {
        match self.phase == 0 {
            true => {
                if self.length_counter == 0 && self.loop_flag {
                    self.length_counter = self.pcm_length as u16 * 16 + 1;
                    self.curr_address = 0xC000 + (self.address as u16 * 64);
                }
                match self.length_counter > 0 {
                    true => {
                        self.read_sample();
                        self.phase = 8;
                        self.length_counter -= 1;
                    }
                    false => match self.irq {
                        true => self.pending_irq = Some(true),
                        false => self.enabled = false,
                    },
                }
            }
            false if self.phase != 0 => {
                self.phase -= 1;
                let delta = (self.buffer & (0x80 >> self.phase)) != 0;
                let v = match delta {
                    true => self.output_level + 2,
                    false => self.output_level - 2,
                };
                if (0..=0x7F).contains(&v) {
                    self.output_level = v;
                }
            }
            _ => {}
        }
    }

    fn read_sample(&mut self) {}

    pub fn poll_irq(&mut self) -> Option<bool> {
        self.pending_irq.take()
    }

    pub fn output(&self) -> u8 {
        self.output_level
    }
}
