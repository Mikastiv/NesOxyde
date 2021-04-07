#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use]
extern crate bitflags;
extern crate lazy_static;

use nes::Nes;
use cartridge::Cartridge;

mod bus;
mod cartridge;
mod cpu;
mod nes;
mod ppu;
mod snake_game;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cartridge = match Cartridge::new(&args[1]) {
        Ok(cart) => cart,
        Err(e) => {
            eprintln!("Problem while loading NES rom: {}", e);
            std::process::exit(-1);
        }
    };
    let nes = Nes::new(cartridge);
}
