use std::cell::RefCell;
use std::rc::Rc;

use super::PpuBus;
use crate::cartridge::Cartridge;
use crate::cpu::Interface;
use crate::joypad::{Button, JoyPad, JoyPort};
use crate::ppu::Ppu;

const RAM_SIZE: usize = 0x800;
const RAM_MASK: u16 = 0x7FF;
const RAM_START: u16 = 0x0000;
const RAM_END: u16 = 0x1FFF;

const PPU_MASK: u16 = 0x7;
const PPU_REG_START: u16 = 0x2000;
const PPU_REG_END: u16 = 0x3FFF;

const ROM_START: u16 = 0x8000;
const ROM_END: u16 = 0xFFFF;

const JOY1: u16 = 0x4016;
const JOY2: u16 = 0x4017;

pub struct MainBus<'a> {
    ram: [u8; RAM_SIZE],
    cartridge: Rc<RefCell<Cartridge>>,
    ppu: Ppu<'a>,
    joypads: [JoyPad; 2],
}

impl Interface for MainBus<'_> {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM_START..=RAM_END => self.ram[(addr & RAM_MASK) as usize],
            PPU_REG_START..=PPU_REG_END => {
                let addr = addr & PPU_MASK;
                self.ppu.read(addr)
            }
            JOY1 => self.joypads[0].read(),
            JOY2 => self.joypads[1].read(),
            ROM_START..=ROM_END => self.cartridge.borrow_mut().read_prg(addr),
            _ => {
                //println!("Ignored read at {:#04X}", addr);
                0
            }
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM_START..=RAM_END => self.ram[(addr & RAM_MASK) as usize] = data,
            PPU_REG_START..=PPU_REG_END => {
                let addr = addr & PPU_MASK;
                self.ppu.write(addr, data);
            }
            JOY1 => {
                self.joypads[0].strobe(data);
                self.joypads[1].strobe(data);
            }
            ROM_START..=ROM_END => self.cartridge.borrow_mut().write_prg(addr, data),
            _ => {
                //println!("Ignored write at 0x{:04X}", addr);
            }
        }
    }

    fn poll_nmi(&mut self) -> Option<bool> {
        self.ppu.poll_nmi()
    }

    fn tick(&mut self, cycles: u64) {
        for _ in 0..(cycles * 3) {
            self.ppu.clock();
        }
    }

    fn update_joypad(&mut self, button: Button, pressed: bool, port: JoyPort) {
        match port {
            JoyPort::Port1 => self.joypads[0].update(button, pressed),
            JoyPort::Port2 => self.joypads[1].update(button, pressed),
        }
    }
}

impl<'a> MainBus<'a> {
    pub fn new<F>(cartridge: Rc<RefCell<Cartridge>>, sdl_render_fn: F) -> Self
    where
        F: FnMut(&[u8]) + 'a,
    {
        let ppu_bus = PpuBus::new(Rc::clone(&cartridge));
        Self {
            ram: [0; RAM_SIZE],
            cartridge,
            ppu: Ppu::new(Box::new(ppu_bus), Box::new(sdl_render_fn)),
            joypads: [JoyPad::new(); 2],
        }
    }
}
