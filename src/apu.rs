use dmc::Dmc;
use noise::Noise;
use square::Square;
use triangle::Triangle;

use crate::decay::Decay;
use crate::filters::{Filter, HighPass, LowPass};

/// Square channel 1 volume register
const SQ1_VOL: u16 = 0x4000;
/// Square channel 1 sweep register
const SQ1_SWEEP: u16 = 0x4001;
/// Square channel 1 timer low register
const SQ1_LO: u16 = 0x4002;
/// Square channel 1 timer high register
const SQ1_HI: u16 = 0x4003;

/// Square channel 2 volume register
const SQ2_VOL: u16 = 0x4004;
/// Square channel 2 sweep register
const SQ2_SWEEP: u16 = 0x4005;
/// Square channel 2 timer low register
const SQ2_LO: u16 = 0x4006;
/// Square channel 2 timer high register
const SQ2_HI: u16 = 0x4007;

/// Triangle channel linear counter register
const TRI_LINEAR: u16 = 0x4008;
/// Triangle channel timer low register
const TRI_LO: u16 = 0x400A;
/// Triangle channel timer high register
const TRI_HI: u16 = 0x400B;

/// Noise channel volume register
const NOISE_VOL: u16 = 0x400C;
/// Noise channel timer low register
const NOISE_LO: u16 = 0x400E;
/// Noise channel timer high register
const NOISE_HI: u16 = 0x400F;

/// DMC frequency register
const DMC_FREQ: u16 = 0x4010;
/// DMC raw sample register
const DMC_RAW: u16 = 0x4011;
/// DMC start (address) register
const DMC_START: u16 = 0x4012;
/// DMC length register
const DMC_LEN: u16 = 0x4013;

/// Sound status / enable register
const SND_CHN: u16 = 0x4015;
/// Frame counter register
const FRAME_COUNTER: u16 = 0x4017;

mod dmc;
mod noise;
mod square;
mod triangle;

/// Sequencer stepping mode
#[derive(PartialEq)]
enum SequencerMode {
    FourStep,
    FiveStep,
}

/// NES audio processing unit
pub struct Apu {
    cycles: u32,
    hz240_counter: u16,
    irq_off: bool,
    pending_irq: Option<bool>,

    sq1: Square,
    sq2: Square,
    tri: Triangle,
    noise: Noise,
    dmc: Dmc,
    sequencer: u8,
    mode: SequencerMode,

    tri_decay: Decay,
    filters: Vec<Box<dyn Filter>>,
}

impl Apu {
    pub fn new(sample_rate: f32) -> Self {
        let filters: Vec<Box<dyn Filter>> = vec![
            Box::new(HighPass::new(90.0, sample_rate, 2.0f32.sqrt())),
            // Box::new(HighPass::new(440.0, sample_rate, 2.0f32.sqrt())),
            Box::new(LowPass::new(14000.0, sample_rate, 2.0f32.sqrt())),
        ];

        Self {
            cycles: 0,
            hz240_counter: 0,
            irq_off: false,
            pending_irq: None,

            sq1: Square::new(),
            sq2: Square::new(),
            tri: Triangle::new(),
            noise: Noise::new(),
            dmc: Dmc::new(),
            sequencer: 0,
            mode: SequencerMode::FourStep,

            tri_decay: Decay::new(0.1),
            filters,
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        // The Apu can only be read from the status register
        match addr {
            SND_CHN => {
                // Returns IF-D NT21
                // I: DMC Interrupt requested and clears it if set
                // F: Apu interrupt flag and clears it if set
                // D: 1 if DMC length counter > 0
                // N: 1 if noise length counter > 0
                // T: 1 if triangle length counter > 0
                // 2: 1 if square 2 length counter > 0
                // 1: 1 if square 1 length counter > 0

                let sq1 = (self.sq1.length_counter() > 0) as u8;
                let sq2 = (self.sq2.length_counter() > 0) as u8;
                let tri = (self.tri.length_counter() > 0) as u8;
                let noise = (self.noise.length_counter() > 0) as u8;
                let dmc = (self.dmc.length_counter() > 0) as u8;
                let irq = self.pending_irq.take().is_some() as u8;
                let dmc_irq = self.dmc.poll_irq().is_some() as u8;

                dmc_irq << 7 | irq << 6 | dmc << 4 | noise << 3 | tri << 2 | sq2 << 1 | sq1
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
                // ---D NT21
                // Enables or disable a channel based on the bits of data
                self.sq1.set_enabled(data & 0x1 != 0);
                self.sq2.set_enabled(data & 0x2 != 0);
                self.tri.set_enabled(data & 0x4 != 0);
                self.noise.set_enabled(data & 0x8 != 0);
                self.dmc.set_enabled(data & 0x10 != 0);
            }
            FRAME_COUNTER => {
                // MI-- ---
                // Sets the stepping based on M
                self.mode = match data & 0x80 == 0 {
                    true => SequencerMode::FiveStep,
                    false => SequencerMode::FourStep,
                };

                self.hz240_counter = 0;

                // Sets the IRQ disable bit based on I
                self.irq_off = data & 0x40 != 0;
                // Clear the IRQ flag if set to disabled
                if self.irq_off {
                    self.dmc.poll_irq();
                    self.pending_irq = None;
                }
            }
            _ => {}
        }
    }

    /// Clocks the Apu once
    pub fn clock(&mut self) {
        // Count the cycles
        self.cycles = self.cycles.wrapping_add(1);
        
        // The triangle channel's timer is clocked at Cpu rate
        // The DMC rate counter is also clocked at Cpu rate
        self.tri.tick_timer();
        self.dmc.tick();
        if self.cycles % 2 == 0 {
            self.sq1.tick_timer();
            self.sq2.tick_timer();
            self.noise.tick_timer();
        }

        self.hz240_counter += 2;
        if self.hz240_counter >= 14915 {
            self.hz240_counter -= 14915;

            self.sequencer += 1;
            match self.mode {
                SequencerMode::FourStep => self.sequencer %= 4,
                SequencerMode::FiveStep => self.sequencer %= 5,
            }

            if !self.irq_off && self.mode == SequencerMode::FourStep && self.sequencer == 0 {
                self.pending_irq = Some(true);
            }

            let half_tick = (self.hz240_counter & 0x5) == 1;
            let full_tick = self.sequencer < 4;

            if half_tick {
                self.sq1.tick_length();
                self.sq2.tick_length();
                self.sq1.tick_sweep(square::Channel::One);
                self.sq2.tick_sweep(square::Channel::Two);
                self.tri.tick_length();
                self.noise.tick_length();
            }

            if full_tick {
                self.sq1.tick_envelope();
                self.sq2.tick_envelope();
                self.noise.tick_envelope();
                self.tri.tick_counter();
            }
        }
    }

    /// Polls the IRQ flag
    pub fn poll_irq(&mut self) -> bool {
        // IRQ can be requested by the Apu or the DMC
        self.pending_irq.take().is_some() | self.dmc.poll_irq().is_some()
    }

    /// Returns if the DMC needs a new audio sample or not
    pub fn need_dmc_sample(&mut self) -> bool {
        self.dmc.need_sample().is_some()
    }

    /// Sets the audio sample of the DMC
    pub fn set_dmc_sample(&mut self, sample: u8) {
        self.dmc.set_sample(sample);
    }

    /// Gets the address of the next DMC audio sample
    pub fn dmc_sample_address(&self) -> u16 {
        self.dmc.address()
    }

    /// Resets the Apu and its channels
    pub fn reset(&mut self) {
        self.cycles = 0;
        self.hz240_counter = 0;
        self.sequencer = 0;
        self.pending_irq = None;
        self.mode = SequencerMode::FourStep;
        self.sq1.reset();
        self.sq2.reset();
        self.tri.reset();
        self.noise.reset();
        self.dmc.reset();
    }

    /// Gets an audio sample
    pub fn output(&mut self) -> f32 {
        // Mix the audio according to NesDev
        // http://wiki.nesdev.com/w/index.php/APU_Mixer

        let sq1 = self.sq1.output();
        let sq2 = self.sq2.output();
        let pulse = 95.88 / (100.0 + (8128.0 / (sq1 as f32 + sq2 as f32)));

        // I apply a "decay" on the triangle channel to reduce audio pops
        // Is only applied if the volume goes from a high value to zero
        let tri = self.tri_decay.decay(self.tri.output() as f32);
        let noise = self.noise.output() as f32;
        let dmc = self.dmc.output() as f32;
        let tnd = 159.79
            / (100.0 + (1.0 / ((tri as f32 / 8227.0) + (noise / 12241.0) + (dmc / 22638.0))));

        let sample = pulse + tnd;

        // Apply filters
        // The has 3 filters applied
        // High-pass at 90Hz
        // High-pass at 440Hz (I removed this one because the bass sounds way better without it)
        // Low-pass at 14000Hz
        self.filters
            .iter_mut()
            .fold(sample, |sample, filter| filter.filter(sample))
    }
}
