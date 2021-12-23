use std::cell::RefCell;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::rc::Rc;

use super::PpuBus;
use crate::apu::Apu;
use crate::cartridge::Cartridge;
use crate::cpu::CpuInterface;
use crate::cpu::Interface;
use crate::joypad::{Button, JoyPad, JoyPort};
use crate::ppu::{Ppu, OAM_DATA};
use crate::savable::Savable;

/// Size of the RAM
const RAM_SIZE: usize = 0x800;
/// Mask for the RAM addresses (same as RAM_SIZE - 1)
const RAM_MASK: u16 = 0x7FF;
/// First address of the RAM memory space
const RAM_START: u16 = 0x0000;
/// Last address of the RAM memory space
const RAM_END: u16 = 0x1FFF;

/// Mask for the Ppu addresses
const PPU_MASK: u16 = 0x7;
/// First address of the Ppu registers memory space
const PPU_REG_START: u16 = 0x2000;
/// Last address of the Ppu registers memory space
const PPU_REG_END: u16 = 0x3FFF;

/// First address of the ROM memory space
const ROM_START: u16 = 0x4020;
/// Last address of the ROM memory space
const ROM_END: u16 = 0xFFFF;

/// Address of controller in port 1
const JOY1: u16 = 0x4016;
/// Address of controller in port 2
const JOY2: u16 = 0x4017;

/// Address of the OAM direct memory access
const OAM_DMA: u16 = 0x4014;

/// First address of the Apu registers memory space
const APU_REG_START: u16 = 0x4000;
/// Last address of the Apu registers memory space (minus 0x4015 and 0x4017)
const APU_REG_END: u16 = 0x4013;
/// Address of the Apu status
const APU_STATUS: u16 = 0x4015;
/// Address of the Apu channel enable
const APU_CH_ENABLE: u16 = 0x4015;
/// Address of the Apu frame counter
const APU_FRAME_COUNTER: u16 = 0x4017;

/// How much time needs to pass between each audio samples (Apu is clocked at ~1.789 MHz)
const TIME_PER_CLOCK: f64 = 1.0 / 1789773.0;

pub struct MainBus<'a> {
    ram: [u8; RAM_SIZE],
    cartridge: Rc<RefCell<Cartridge>>,
    apu: Apu,
    ppu: Ppu<'a>,
    joypads: [JoyPad; 2],

    audio_time: f64,
    time_per_sample: f64,
    samples: Vec<f32>,
}

impl CpuInterface for MainBus<'_> {}

impl Savable for MainBus<'_> {
    fn save(&self, output: &mut BufWriter<File>) -> bincode::Result<()> {
        self.apu.save(output)?;
        self.ppu.save(output)?;
        self.cartridge.borrow().save(output)?;
        for i in 0..RAM_SIZE {
            bincode::serialize_into::<&mut BufWriter<File>, _>(output, &self.ram[i])?;
        }
        bincode::serialize_into::<&mut BufWriter<File>, _>(output, &self.audio_time)?;
        bincode::serialize_into::<&mut BufWriter<File>, _>(output, &self.samples)?;
        Ok(())
    }

    fn load(&mut self, input: &mut BufReader<File>) -> bincode::Result<()> {
        self.apu.load(input)?;
        self.ppu.load(input)?;
        self.cartridge.borrow_mut().load(input)?;
        for i in 0..RAM_SIZE {
            self.ram[i] = bincode::deserialize_from::<&mut BufReader<File>, _>(input)?;
        }
        self.audio_time = bincode::deserialize_from::<&mut BufReader<File>, _>(input)?;
        self.samples = bincode::deserialize_from::<&mut BufReader<File>, _>(input)?;
        Ok(())
    }
}

impl Interface for MainBus<'_> {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            // RAM memory space: mirror address and read from RAM
            RAM_START..=RAM_END => self.ram[(addr & RAM_MASK) as usize],
            // Ppu registers memory space: read from Ppu
            PPU_REG_START..=PPU_REG_END => {
                // Mirror address first
                let addr = addr & PPU_MASK;
                self.ppu.read(addr)
            }
            // Apu registers memory space: read from Apu
            APU_REG_START..=APU_REG_END | APU_STATUS => self.apu.read(addr),
            // Read controller port 1
            JOY1 => self.joypads[0].read(),
            // Read controller port 2
            JOY2 => self.joypads[1].read(),
            // ROM memory space: read from PRG ROM
            ROM_START..=ROM_END => self.cartridge.borrow_mut().read_prg(addr),
            _ => 0,
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            // RAM memory space: mirror address and write to RAM
            RAM_START..=RAM_END => self.ram[(addr & RAM_MASK) as usize] = data,
            // Ppu registers memory space: write to Ppu
            PPU_REG_START..=PPU_REG_END => {
                // Mirror address first
                let addr = addr & PPU_MASK;
                self.ppu.write(addr, data);
            }
            // Perform OAM DMA
            OAM_DMA => {
                // Data byte is the memory page to copy
                let page = (data as u16) << 8;
                // Copy a whole page to Ppu OAM memory
                for byte in 0..256 {
                    // Read byte
                    let v = self.read(page + byte);
                    // A read is one clock cycle
                    self.tick(1);
                    // Write byte
                    self.write(0x2000 + OAM_DATA, v);
                    // A write is one clock cycle
                    self.tick(1);
                    // Check if the DMC channel needs a new sample
                    // because this can occur during DMA
                    self.update_dmc_sample();
                }
            }
            // Apu registers memory space: write to Apu
            APU_REG_START..=APU_REG_END | APU_CH_ENABLE | APU_FRAME_COUNTER => {
                self.apu.write(addr, data)
            }
            // Write controller port 1 (Strobe both controllers at same address)
            JOY1 => {
                self.joypads[0].strobe(data);
                self.joypads[1].strobe(data);
            }
            // ROM memory space: write to PRG ROM
            ROM_START..=ROM_END => self.cartridge.borrow_mut().write_prg(addr, data),
            _ => {}
        }
    }

    fn poll_nmi(&mut self) -> bool {
        self.ppu.poll_nmi()
    }

    fn poll_irq(&mut self) -> bool {
        // IRQ are normally from the Apu, but some mappers also do
        self.apu.poll_irq() | self.cartridge.borrow_mut().poll_irq()
    }

    fn tick(&mut self, cycles: u64) {
        for _ in 0..cycles {
            // Ppu is clocked at 3 times the speed of the Cpu
            for _ in 0..3 {
                self.ppu.clock();
            }

            // Apu is clocked at the same speed as the Cpu
            self.apu.clock();
            // Check if DMC channel needs a new sample
            self.update_dmc_sample();

            // This next part is to keep the audio of the NES in sync
            // Add the time per clock everytime the bus clocks
            self.audio_time += TIME_PER_CLOCK;
            // If enough time has passed to generate a new audio sample...
            if self.audio_time >= self.time_per_sample {
                // Substract the time per sample to the audio time.
                // I do not reset it to 0 because it is possible that more than one sample
                // needs to be generated.
                self.audio_time -= self.time_per_sample;
                // Generate a new sample
                let sample = self.apu.output();
                // Add it to the vec of samples
                self.samples.push(sample);
            }
        }
    }

    fn update_joypad(&mut self, button: Button, pressed: bool, port: JoyPort) {
        match port {
            JoyPort::Port1 => self.joypads[0].update(button, pressed),
            JoyPort::Port2 => self.joypads[1].update(button, pressed),
        }
    }

    fn frame_count(&self) -> u128 {
        self.ppu.frame_count()
    }

    fn reset(&mut self) {
        self.ppu.reset();
        self.apu.reset();
        self.cartridge.borrow_mut().reset();
    }

    /// Returns the samples which are ready to be queued
    fn samples(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.samples.as_mut())
    }

    /// How many samples are ready to be queued
    fn sample_count(&self) -> usize {
        self.samples.len()
    }
}

impl<'a> MainBus<'a> {
    pub fn new<F>(cartridge: Rc<RefCell<Cartridge>>, sdl_render_fn: F, sample_rate: f64) -> Self
    where
        F: FnMut(&[u8]) + 'a,
    {
        let ppu_bus = PpuBus::new(Rc::clone(&cartridge));
        Self {
            ram: [0; RAM_SIZE],
            cartridge,
            apu: Apu::new(sample_rate as f32),
            ppu: Ppu::new(Box::new(ppu_bus), Box::new(sdl_render_fn)),
            joypads: [JoyPad::new(); 2],

            audio_time: 0.0,
            time_per_sample: 1.0 / sample_rate,
            samples: Vec::new(),
        }
    }

    fn update_dmc_sample(&mut self) {
        // If DMC channel needs a new sample
        if self.apu.need_dmc_sample() {
            // Read at the address
            let addr = self.apu.dmc_sample_address();
            let sample = self.read(addr);
            // Set the sample in the channel
            self.apu.set_dmc_sample(sample);
            // The Cpu is stalled for 1-4 cycles, but I always use 4
            self.tick(4);
        }
    }
}
