use square::Square;
use triangle::Triangle;

const SQ1_VOL: u16 = 0x4000;
const SQ1_SWEEP: u16 = 0x4001;
const SQ1_LO: u16 = 0x4002;
const SQ1_HI: u16 = 0x4003;

const SQ2_VOL: u16 = 0x4004;
const SQ2_SWEEP: u16 = 0x4005;
const SQ2_LO: u16 = 0x4006;
const SQ2_HI: u16 = 0x4007;

const TRI_LINEAR: u16 = 0x4008;
const TRI_LO: u16 = 0x400A;
const TRI_HI: u16 = 0x400B;

const NOISE_VOL: u16 = 0x400C;
const NOISE_LO: u16 = 0x400E;
const NOISE_HI: u16 = 0x400F;

const DMC_FREQ: u16 = 0x4010;
const DMC_RAW: u16 = 0x4011;
const DMC_START: u16 = 0x4012;
const DMC_LEN: u16 = 0x4013;

const SND_CHN: u16 = 0x4015;
const FRAME_COUNTER: u16 = 0x4017;

mod square;
mod triangle;

pub struct Apu {
    cycles: u32,
    frame_counter: u32,

    sq1: Square,
    sq2: Square,
    tri: Triangle,

    samples: Vec<f32>,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            cycles: 0,
            frame_counter: 0,

            sq1: Square::new(),
            sq2: Square::new(),
            tri: Triangle::new(),

            samples: Vec::new(),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            SND_CHN => 0,
            _ => 0,
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            SQ1_VOL => self.sq1.write_vol(data),
            SQ1_SWEEP => self.sq1.write_sweep(data),
            SQ1_LO => self.sq1.write_lo(data),
            SQ1_HI => self.sq1.write_hi(data),

            SQ2_VOL => self.sq2.write_vol(data),
            SQ2_SWEEP => self.sq2.write_sweep(data),
            SQ2_LO => self.sq2.write_lo(data),
            SQ2_HI => self.sq2.write_hi(data),

            TRI_LINEAR => self.tri.write_linear(data),
            TRI_LO => self.tri.write_lo(data),
            TRI_HI => self.tri.write_hi(data),

            NOISE_VOL => {}
            NOISE_LO => {}
            NOISE_HI => {}

            DMC_FREQ => {}
            DMC_RAW => {}
            DMC_START => {}
            DMC_LEN => {}

            SND_CHN => {
                self.sq1.set_enabled(data & 0x1 != 0);
                self.sq2.set_enabled(data & 0x2 != 0);
                self.tri.set_enabled(data & 0x4 != 0);
            }
            FRAME_COUNTER => {
                if data & 0x80 == 0 {
                    self.tick_envelopes();
                    self.tick_lengths();
                }
            }
            _ => {}
        }
    }

    const SAMPLE_RATE: f64 = 1789773.0 / 44100.0;

    pub fn clock(&mut self) {
        let c1 = self.cycles as f64;
        self.cycles = self.cycles.wrapping_add(1);
        let c2 = self.cycles as f64;

        let mut quarter_frame = false;
        let mut half_frame = false;

        self.tri.tick_timer();
        if self.cycles % 2 == 0 {
            self.sq1.tick_timer();
            self.sq2.tick_timer();

            self.frame_counter += 1;

            match self.frame_counter {
                3729 | 11186 => quarter_frame = true,
                7457 | 14916 => {
                    quarter_frame = true;
                    half_frame = true;
                    if self.frame_counter == 14916 {
                        self.frame_counter = 0;
                    }
                }
                _ => {}
            }

            if quarter_frame {
                self.tick_envelopes();
            }

            if half_frame {
                self.tick_lengths();
                self.tick_sweep();
            }
        }

        let s1 = (c1 / Self::SAMPLE_RATE) as u64;
        let s2 = (c2 / Self::SAMPLE_RATE) as u64;
        if s1 != s2 {
            self.samples.push(self.output());
        }
    }

    pub fn sample_ready(&self) -> bool {
        !self.samples.is_empty()
    }

    pub fn sample(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.samples)
    }

    pub fn reset(&mut self) {
        self.cycles = 0;
        self.frame_counter = 0;
        self.samples.clear();
        self.sq1 = Square::new();
        self.sq2 = Square::new();
        self.tri = Triangle::new();
    }

    fn output(&self) -> f32 {
        let sq1 = self.sq1.output();
        let sq2 = self.sq2.output();
        let pulse = 95.88 / (100.0 + (8128.0 / (sq1 as f32 + sq2 as f32)));

        let tri = 0.8 * self.tri.output() as f32;
        let tnd =
            159.79 / (100.0 + (1.0 / ((tri as f32 / 8227.0) + (0.0 / 12241.0) + (0.0 / 22638.0))));

        pulse + tnd
    }

    fn tick_envelopes(&mut self) {
        self.sq1.tick_envelope();
        self.sq2.tick_envelope();
        self.tri.tick_counter();
    }

    fn tick_lengths(&mut self) {
        self.sq1.tick_length();
        self.sq2.tick_length();
        self.tri.tick_length();
    }

    fn tick_sweep(&mut self) {
        self.sq1.tick_sweep(square::Channel::One);
        self.sq2.tick_sweep(square::Channel::Two);
    }
}
