use dmc::Dmc;
use noise::Noise;
use square::Square;
use triangle::Triangle;

use crate::decay::Decay;
use crate::filters::{Filter, HighPass, LowPass};

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

mod dmc;
mod noise;
mod square;
mod triangle;

pub struct Apu {
    cycles: u32,
    frame_counter: u32,
    irq_off: bool,
    pending_irq: Option<bool>,

    sq1: Square,
    sq2: Square,
    tri: Triangle,
    noise: Noise,
    dmc: Dmc,

    env: Decay,
    filters: Vec<Box<dyn Filter>>,
}

impl Apu {
    pub fn new() -> Self {
        let filters: Vec<Box<dyn Filter>> = vec![
            Box::new(LowPass::new(14000.0, 44100.0, 2.0f32.sqrt())),
            Box::new(HighPass::new(90.0, 44100.0, 2.0f32.sqrt())),
            Box::new(HighPass::new(440.0, 44100.0, 2.0f32.sqrt())),
        ];

        Self {
            cycles: 0,
            frame_counter: 0,
            irq_off: false,
            pending_irq: None,

            sq1: Square::new(),
            sq2: Square::new(),
            tri: Triangle::new(),
            noise: Noise::new(),
            dmc: Dmc::new(),

            env: Decay::new(0.001),
            filters,
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            SND_CHN => {
                let sq1 = (self.sq1.length_counter() > 0) as u8;
                let sq2 = (self.sq2.length_counter() > 0) as u8;
                let tri = (self.tri.length_counter() > 0) as u8;
                let noise = (self.noise.length_counter() > 0) as u8;

                self.pending_irq = None;

                noise << 3 | tri << 2 | sq2 << 1 | sq1
            }
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

            NOISE_VOL => self.noise.write_vol(data),
            NOISE_LO => self.noise.write_lo(data),
            NOISE_HI => self.noise.write_hi(data),

            DMC_FREQ => self.dmc.write_freq(data),
            DMC_RAW => self.dmc.write_raw(data),
            DMC_START => self.dmc.write_start(data),
            DMC_LEN => self.dmc.write_len(data),

            SND_CHN => {
                self.sq1.set_enabled(data & 0x1 != 0);
                self.sq2.set_enabled(data & 0x2 != 0);
                self.tri.set_enabled(data & 0x4 != 0);
                self.noise.set_enabled(data & 0x8 != 0);
                self.dmc.set_enabled(data & 0x10 != 0);
            }
            FRAME_COUNTER => {
                self.frame_counter = 0;
                if data & 0x80 == 0 {
                    self.tick_envelopes();
                    self.tick_lengths();
                }

                self.irq_off = data & 0x40 != 0;
            }
            _ => {}
        }
    }

    pub fn clock(&mut self) {
        self.cycles = self.cycles.wrapping_add(1);

        let mut quarter_frame = false;
        let mut half_frame = false;

        self.tri.tick_timer();
        if self.cycles % 2 == 0 {
            self.sq1.tick_timer();
            self.sq2.tick_timer();
            self.noise.tick_timer();

            self.frame_counter += 1;

            match self.frame_counter {
                3729 | 11186 => quarter_frame = true,
                7457 | 14916 => {
                    quarter_frame = true;
                    half_frame = true;
                    if self.frame_counter == 14916 {
                        self.frame_counter = 0;
                        self.pending_irq = if !self.irq_off { Some(true) } else { None };
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
    }

    pub fn poll_irq(&mut self) -> Option<bool> {
        self.pending_irq.take()
    }

    pub fn sample(&mut self) -> f32 {
        self.output()
    }

    pub fn reset(&mut self) {
        self.cycles = 0;
        self.frame_counter = 0;
        self.sq1 = Square::new();
        self.sq2 = Square::new();
        self.tri = Triangle::new();
        self.noise = Noise::new();
        self.dmc = Dmc::new();
    }

    fn output(&mut self) -> f32 {
        let sq1 = self.sq1.output();
        let sq2 = self.sq2.output();
        let pulse = 95.88 / (100.0 + (8128.0 / (sq1 as f32 + sq2 as f32)));

        let tri = self.env.decay(0.9 * self.tri.output() as f32);
        let noise = 0.8 * self.noise.output() as f32;
        let dmc = self.dmc.output() as f32;
        let tnd = 159.79
            / (100.0 + (1.0 / ((tri as f32 / 8227.0) + (noise / 12241.0) + (dmc / 22638.0))));

        let signal = pulse + tnd;

        self.filters
            .iter_mut()
            .fold(signal, |signal, filter| filter.filter(signal))
    }

    fn tick_envelopes(&mut self) {
        self.sq1.tick_envelope();
        self.sq2.tick_envelope();
        self.tri.tick_counter();
        self.noise.tick_envelope();
    }

    fn tick_lengths(&mut self) {
        self.sq1.tick_length();
        self.sq2.tick_length();
        self.tri.tick_length();
        self.noise.tick_length();
    }

    fn tick_sweep(&mut self) {
        self.sq1.tick_sweep(square::Channel::One);
        self.sq2.tick_sweep(square::Channel::Two);
    }
}
