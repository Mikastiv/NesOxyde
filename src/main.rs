use sdl2::keyboard::Keycode;

use cartridge::Cartridge;
use joypad::{Button, JoyPort};

mod apu;
mod bus;
mod cartridge;
mod cpu;
mod decay;
mod filters;
mod joypad;
mod nes;
mod ppu;
mod reverb;
mod snake_game;
mod timer;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 && args.len() != 3 {
        eprintln!(
            "Usage: ./{} [SyncMode: Audio (default) or Video (-A or -V)] <iNES File>",
            args[0]
        );
        std::process::exit(0);
    }

    let (mode, rom) = match args.len() {
        2 => (nes::Mode::AudioSync, &args[1]),

        3 => (nes::Mode::VideoSync, &args[2]),
        count => panic!("Bad argument count: {}", count),
    };

    let cartridge = match Cartridge::new(rom) {
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

    nes::run(cartridge, map_key, mode);
}
