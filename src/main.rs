#[macro_use]
extern crate bitflags;
extern crate lazy_static;

use sdl2::keyboard::Keycode;

use cartridge::Cartridge;
use joypad::{Button, JoyPort};

mod bus;
mod cartridge;
mod cpu;
mod joypad;
mod nes;
mod ppu;
mod snake_game;
mod timer;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cartridge = match Cartridge::new(&args[1]) {
        Ok(cart) => cart,
        Err(e) => {
            eprintln!("Problem while loading ROM \"{}\" -> {}", args[1], e);
            std::process::exit(0);
        }
    };

    let map_key = |key: Keycode, port: JoyPort| match port {
        JoyPort::Port1 => match key {
            Keycode::S => Some(Button::A),
            Keycode::A => Some(Button::B),
            Keycode::Z => Some(Button::Select),
            Keycode::X => Some(Button::Start),
            Keycode::Up => Some(Button::Up),
            Keycode::Down => Some(Button::Down),
            Keycode::Left => Some(Button::Left),
            Keycode::Right => Some(Button::Right),
            _ => None,
        },
        JoyPort::Port2 => match key {
            Keycode::J => Some(Button::A),
            Keycode::K => Some(Button::B),
            Keycode::N => Some(Button::Select),
            Keycode::M => Some(Button::Start),
            Keycode::Kp5 => Some(Button::Up),
            Keycode::Kp2 => Some(Button::Down),
            Keycode::Kp1 => Some(Button::Left),
            Keycode::Kp3 => Some(Button::Right),
            _ => None,
        },
    };

    nes::run(cartridge, map_key);
}
