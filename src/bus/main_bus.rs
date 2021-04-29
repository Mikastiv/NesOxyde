use std::cell::RefCell;
use std::rc::Rc;

use super::PpuBus;
use crate::apu::Apu;
use crate::cartridge::Cartridge;
use crate::cpu::Interface;
use crate::joypad::{Button, JoyPad, JoyPort};
use crate::ppu::{Ppu, OAM_DATA};

const RAM_SIZE: usize = 0x800;
const RAM_MASK: u16 = 0x7FF;
const RAM_START: u16 = 0x0000;
const RAM_END: u16 = 0x1FFF;

const PPU_MASK: u16 = 0x7;
const PPU_REG_START: u16 = 0x2000;
const PPU_REG_END: u16 = 0x3FFF;

const ROM_START: u16 = 0x4020;
const ROM_END: u16 = 0xFFFF;

const JOY1: u16 = 0x4016;
const JOY2: u16 = 0x4017;

const OAM_DMA: u16 = 0x4014;

const APU_REG_START: u16 = 0x4000;
const APU_REG_END: u16 = 0x4013;
const APU_STATUS: u16 = 0x4015;
const APU_CH_ENABLE: u16 = 0x4015;
const APU_FRAME_COUNTER: u16 = 0x4017;

pub struct MainBus<'a> {
    ram: [u8; RAM_SIZE],
    cartridge: Rc<RefCell<Cartridge>>,
    apu: Apu,
    ppu: Ppu<'a>,
    joypads: [JoyPad; 2],

    audio_time: f64,
    time_per_clock: f64,
    time_per_sample: f64,
    samples: Vec<f32>,
}

impl Interface for MainBus<'_> {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM_START..=RAM_END => self.ram[(addr & RAM_MASK) as usize],
            PPU_REG_START..=PPU_REG_END => {
                let addr = addr & PPU_MASK;
                self.ppu.read(addr)
            }
            APU_REG_START..=APU_REG_END | APU_STATUS => self.apu.read(addr),
            JOY1 => self.joypads[0].read(),
            JOY2 => self.joypads[1].read(),
            ROM_START..=ROM_END => self.cartridge.borrow_mut().read_prg(addr),
            _ => 0,
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM_START..=RAM_END => self.ram[(addr & RAM_MASK) as usize] = data,
            PPU_REG_START..=PPU_REG_END => {
                let addr = addr & PPU_MASK;
                self.ppu.write(addr, data);
            }
            OAM_DMA => {
                let page = (data as u16) << 8;
                for byte in 0..256 {
                    let v = self.read(page + byte);
                    self.tick(1);
                    self.write(0x2000 + OAM_DATA, v);
                    self.tick(1);
                }
            }
            APU_REG_START..=APU_REG_END | APU_CH_ENABLE | APU_FRAME_COUNTER => {
                self.apu.write(addr, data)
            }
            JOY1 => {
                self.joypads[0].strobe(data);
                self.joypads[1].strobe(data);
            }
            ROM_START..=ROM_END => self.cartridge.borrow_mut().write_prg(addr, data),
            _ => {}
        }
    }

    fn poll_nmi(&mut self) -> bool {
        self.ppu.poll_nmi().is_some()
    }

    fn poll_irq(&mut self) -> bool {
        self.apu.poll_irq().is_some() | self.cartridge.borrow_mut().poll_irq()
    }

    fn tick(&mut self, cycles: u64) {
        for _ in 0..cycles {
            for _ in 0..3 {
                self.ppu.clock();
            }

            self.apu.clock();
            self.audio_time += self.time_per_clock;
            if self.audio_time >= self.time_per_sample {
                self.audio_time -= self.time_per_sample;
                let sample = self.apu.sample();
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

    fn samples(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.samples.as_mut())
    }

    fn sample_count(&self) -> usize {
        self.samples.len()
    }
}

impl<'a> MainBus<'a> {
    const APU_CLOCK_RATE: f64 = 1789773.0;

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
            time_per_clock: 1.0 / Self::APU_CLOCK_RATE,
            time_per_sample: 1.0 / sample_rate,
            samples: Vec::new(),
        }
    }
}
