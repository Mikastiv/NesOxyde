#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use]
extern crate bitflags;
extern crate lazy_static;

use sdl2::keyboard::Keycode;

use cartridge::Cartridge;
use joypad::Button;

mod bus;
mod cartridge;
mod joypad;
mod cpu;
mod nes;
mod ppu;
mod snake_game;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cartridge = match Cartridge::new(&args[1]) {
        Ok(cart) => cart,
        Err(e) => {
            eprintln!("Problem while loading ROM \"{}\" -> {}", args[1], e);
            std::process::exit(0);
        }
    };

    let map_key = |key: Keycode| match key {
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

    nes::run(cartridge, map_key);
}
