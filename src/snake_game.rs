// This module was used to test my cpu emulation in early stages
// Play a simple 6502 asm snake game

#![allow(dead_code)]

use rand::prelude::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::EventPump;
use std::time::Instant;

use crate::bus::SnakeBus;
use crate::cpu::Cpu;

const NS_BETWEEN_INS: u128 = 100_000;

fn handle_user_input(cpu: &mut Cpu, event_pump: &mut EventPump) {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => std::process::exit(0),
            Event::KeyDown {
                keycode: Some(Keycode::W),
                ..
            } => {
                cpu.mem_write(0xFF, 0x77);
            }
            Event::KeyDown {
                keycode: Some(Keycode::S),
                ..
            } => {
                cpu.mem_write(0xFF, 0x73);
            }
            Event::KeyDown {
                keycode: Some(Keycode::A),
                ..
            } => {
                cpu.mem_write(0xFF, 0x61);
            }
            Event::KeyDown {
                keycode: Some(Keycode::D),
                ..
            } => {
                cpu.mem_write(0xFF, 0x64);
            }
            _ => { /* do nothing */ }
        }
    }
}

fn color(byte: u8) -> Color {
    match byte {
        0 => sdl2::pixels::Color::BLACK,
        1 => sdl2::pixels::Color::WHITE,
        2 | 9 => sdl2::pixels::Color::GREY,
        3 | 10 => sdl2::pixels::Color::RED,
        4 | 11 => sdl2::pixels::Color::GREEN,
        5 | 12 => sdl2::pixels::Color::BLUE,
        6 | 13 => sdl2::pixels::Color::MAGENTA,
        7 | 14 => sdl2::pixels::Color::YELLOW,
        _ => sdl2::pixels::Color::CYAN,
    }
}

fn read_screen_state(cpu: &mut Cpu, frame: &mut [u8; 32 * 3 * 32]) -> bool {
    let mut frame_idx = 0;
    let mut update = false;
    for i in 0x0200..0x600 {
        let color_idx = cpu.mem_read(i as u16);
        let (b1, b2, b3) = color(color_idx).rgb();
        if frame[frame_idx] != b1 || frame[frame_idx + 1] != b2 || frame[frame_idx + 2] != b3 {
            frame[frame_idx] = b1;
            frame[frame_idx + 1] = b2;
            frame[frame_idx + 2] = b3;
            update = true;
        }
        frame_idx += 3;
    }
    update
}

/// Run a 6502 asm snake game
pub fn run() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Snake game", (32.0 * 10.0) as u32, (32.0 * 10.0) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(10.0, 10.0).unwrap();
    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 32, 32)
        .unwrap();

    let game_code = vec![
        0x20, 0x06, 0x06, 0x20, 0x38, 0x06, 0x20, 0x0D, 0x06, 0x20, 0x2A, 0x06, 0x60, 0xA9, 0x02,
        0x85, 0x02, 0xA9, 0x04, 0x85, 0x03, 0xA9, 0x11, 0x85, 0x10, 0xA9, 0x10, 0x85, 0x12, 0xA9,
        0x0F, 0x85, 0x14, 0xA9, 0x04, 0x85, 0x11, 0x85, 0x13, 0x85, 0x15, 0x60, 0xA5, 0xFE, 0x85,
        0x00, 0xA5, 0xFE, 0x29, 0x03, 0x18, 0x69, 0x02, 0x85, 0x01, 0x60, 0x20, 0x4D, 0x06, 0x20,
        0x8D, 0x06, 0x20, 0xC3, 0x06, 0x20, 0x19, 0x07, 0x20, 0x20, 0x07, 0x20, 0x2D, 0x07, 0x4C,
        0x38, 0x06, 0xA5, 0xFF, 0xC9, 0x77, 0xF0, 0x0D, 0xC9, 0x64, 0xF0, 0x14, 0xC9, 0x73, 0xF0,
        0x1B, 0xC9, 0x61, 0xF0, 0x22, 0x60, 0xA9, 0x04, 0x24, 0x02, 0xD0, 0x26, 0xA9, 0x01, 0x85,
        0x02, 0x60, 0xA9, 0x08, 0x24, 0x02, 0xD0, 0x1B, 0xA9, 0x02, 0x85, 0x02, 0x60, 0xA9, 0x01,
        0x24, 0x02, 0xD0, 0x10, 0xA9, 0x04, 0x85, 0x02, 0x60, 0xA9, 0x02, 0x24, 0x02, 0xD0, 0x05,
        0xA9, 0x08, 0x85, 0x02, 0x60, 0x60, 0x20, 0x94, 0x06, 0x20, 0xA8, 0x06, 0x60, 0xA5, 0x00,
        0xC5, 0x10, 0xD0, 0x0D, 0xA5, 0x01, 0xC5, 0x11, 0xD0, 0x07, 0xE6, 0x03, 0xE6, 0x03, 0x20,
        0x2A, 0x06, 0x60, 0xA2, 0x02, 0xB5, 0x10, 0xC5, 0x10, 0xD0, 0x06, 0xB5, 0x11, 0xC5, 0x11,
        0xF0, 0x09, 0xE8, 0xE8, 0xE4, 0x03, 0xF0, 0x06, 0x4C, 0xAA, 0x06, 0x4C, 0x35, 0x07, 0x60,
        0xA6, 0x03, 0xCA, 0x8A, 0xB5, 0x10, 0x95, 0x12, 0xCA, 0x10, 0xF9, 0xA5, 0x02, 0x4A, 0xB0,
        0x09, 0x4A, 0xB0, 0x19, 0x4A, 0xB0, 0x1F, 0x4A, 0xB0, 0x2F, 0xA5, 0x10, 0x38, 0xE9, 0x20,
        0x85, 0x10, 0x90, 0x01, 0x60, 0xC6, 0x11, 0xA9, 0x01, 0xC5, 0x11, 0xF0, 0x28, 0x60, 0xE6,
        0x10, 0xA9, 0x1F, 0x24, 0x10, 0xF0, 0x1F, 0x60, 0xA5, 0x10, 0x18, 0x69, 0x20, 0x85, 0x10,
        0xB0, 0x01, 0x60, 0xE6, 0x11, 0xA9, 0x06, 0xC5, 0x11, 0xF0, 0x0C, 0x60, 0xC6, 0x10, 0xA5,
        0x10, 0x29, 0x1F, 0xC9, 0x1F, 0xF0, 0x01, 0x60, 0x4C, 0x35, 0x07, 0xA0, 0x00, 0xA5, 0xFE,
        0x91, 0x00, 0x60, 0xA6, 0x03, 0xA9, 0x00, 0x81, 0x10, 0xA2, 0x00, 0xA9, 0x01, 0x81, 0x10,
        0x60, 0xA2, 0x00, 0xEA, 0xEA, 0xCA, 0xD0, 0xFB, 0x60,
    ];

    let mut bus = SnakeBus::new();
    bus.load(game_code);
    let mut cpu = Cpu::new(bus);
    let mut screen_state = [0; 32 * 3 * 32];
    let mut rng = rand::thread_rng();

    cpu.reset();
    cpu.run_with_callback(move |cpu| {
        handle_user_input(cpu, &mut event_pump);
        cpu.mem_write(0xFE, rng.gen_range(1..16));

        if read_screen_state(cpu, &mut screen_state) {
            texture.update(None, &screen_state, 32 * 3).unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
        }

        let start = Instant::now();
        let mut elapsed = 0;
        while elapsed < NS_BETWEEN_INS {
            elapsed = (Instant::now() - start).as_nanos();
        }
    });
}
