use sdl2::keyboard::Keycode;

use cartridge::Cartridge;
use joypad::{Button, JoyPort};
use nes::Mode;

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
mod savable;
mod snake_game;
mod timer;

/// Parses program arguments
fn parse_args(args: &[String]) -> (Mode, &String) {
    if args.len() != 2 && args.len() != 3 {
        eprintln!("Usage: {} [-V] <iNES File>", args[0]);
        std::process::exit(0);
    }

    match args.len() {
        // Default to AudioSync
        2 => (nes::Mode::AudioSync, &args[1]),
        3 => match args[1].as_str() {
            "-A" => (nes::Mode::AudioSync, &args[2]),
            "-V" => (nes::Mode::VideoSync, &args[2]),
            flag => {
                eprintln!("Bad option flag: {}. Use -A or -V", flag);
                std::process::exit(0);
            }
        },
        count => {
            eprintln!("Bad argument count: {}", count);
            std::process::exit(0);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (mode, rom) = parse_args(&args);

    // Load the rom from iNES file
    let cartridge = match Cartridge::new(rom) {
        Ok(cart) => cart,
        Err(e) => {
            eprintln!("Problem while loading ROM \"{}\" -> {}", rom, e);
            std::process::exit(0);
        }
    };

    // Closure which maps keycodes to NES buttons
    let map_key = |key: Keycode, port: JoyPort| match port {
        // Controller 1
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
        // Controller 2
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

    // Run the game
    nes::run(cartridge, map_key, mode);
}
