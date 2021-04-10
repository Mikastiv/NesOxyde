use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use std::cell::RefCell;
use std::rc::Rc;

use crate::bus::MainBus;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::ppu::frame::{HEIGHT, WIDTH};

mod trace;

pub fn run(cartridge: Cartridge) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("NesOxyde v0.1.0", WIDTH as u32, HEIGHT as u32)
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

    let bus = MainBus::new(Rc::new(RefCell::new(cartridge)), move |frame| {
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => std::process::exit(0),
                _ => {}
            }
        }
        texture.update(None, frame, 256 * 3).unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
    });

    let mut cpu = Cpu::new(bus);
    cpu.reset();
    cpu.run();
}
