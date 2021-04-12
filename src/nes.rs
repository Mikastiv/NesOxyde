use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use crate::bus::MainBus;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::joypad::{Button, JoyPort};

// http://wiki.nesdev.com/w/index.php/CPU
const NS_PER_CPU_CLOCK: u128 = 559;

const WINDOW_TITLE: &str = "NesOxyde v0.1.0";
pub const WIDTH: u32 = 256;
pub const HEIGHT: u32 = 240;

mod trace;

pub fn run<KeyMap>(cartridge: Cartridge, map_key: KeyMap)
where
    KeyMap: Fn(Keycode) -> Option<Button>,
{
    // SDL2 init ----------------->
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(WINDOW_TITLE, WIDTH * 2, HEIGHT * 2)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, WIDTH as u32, HEIGHT as u32)
        .unwrap();
    // >----------------- SDL2 init

    let bus = MainBus::new(Rc::new(RefCell::new(cartridge)), move |frame| {
        texture.update(None, frame, (WIDTH * 3) as usize).unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
    });

    let mut cpu = Cpu::new(bus);
    cpu.reset();

    'nes: loop {
        let start_time = Instant::now();
        let cycles_passed = cpu.execute();
        
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'nes,
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(button) = map_key(key) {
                        cpu.update_joypad(button, true, JoyPort::Port1)
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(button) = map_key(key) {
                        cpu.update_joypad(button, false, JoyPort::Port1)
                    }
                }
                _ => {}
            }
        }

        let expected_time = cycles_passed as u128 * NS_PER_CPU_CLOCK;
        let time_passed = (Instant::now() - start_time).as_nanos();
        if expected_time > time_passed {
            while expected_time > (Instant::now() - start_time).as_nanos() {}
        }
    }
}
