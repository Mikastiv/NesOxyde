#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use]
extern crate bitflags;
extern crate lazy_static;

mod bus;
mod cpu;
mod ppu;
mod cartridge;
mod snake_game;

fn main() {
    snake_game::run();
}
