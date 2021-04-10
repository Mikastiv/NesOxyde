use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use std::cell::RefCell;
use std::rc::Rc;

use crate::bus::MainBus;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::joypad::{Button, JoyPort};
use crate::ppu::frame::{HEIGHT, WIDTH};

mod trace;

pub fn run(cartridge: Cartridge) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("NesOxyde v0.1.0", (WIDTH * 2) as u32, (HEIGHT * 2) as u32)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_scale(2.0, 2.0).unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, WIDTH as u32, HEIGHT as u32)
        .unwrap();

    let bus = MainBus::new(Rc::new(RefCell::new(cartridge)), move |frame| {
        texture.update(None, frame, WIDTH * 3).unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
    });

    let convert_key = |key: Keycode| match key {
        Keycode::A => Some(Button::A),
        Keycode::S => Some(Button::B),
        Keycode::Z => Some(Button::Select),
        Keycode::X => Some(Button::Start),
        Keycode::Up => Some(Button::Up),
        Keycode::Down => Some(Button::Down),
        Keycode::Left => Some(Button::Left),
        Keycode::Right => Some(Button::Right),
        _ => None,
    };

    let mut cpu = Cpu::new(bus);
    cpu.reset();

    'nes: loop {
        cpu.execute();
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
                    if let Some(button) = convert_key(key) {
                        cpu.update_joypad(button, true, JoyPort::Port1)
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(button) = convert_key(key) {
                        cpu.update_joypad(button, false, JoyPort::Port1)
                    }
                }
                _ => {}
            }
        }
    }
}
