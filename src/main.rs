#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use]
extern crate bitflags;
extern crate lazy_static;

use cartridge::Cartridge;

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
    nes::run(cartridge);
}
