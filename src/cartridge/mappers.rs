pub use mapper0::Mapper0;
pub use mapper2::Mapper2;
pub use mapper3::Mapper3;

use super::MirrorMode;

mod mapper0;
mod mapper2;
mod mapper3;

pub trait Mapper {
    fn read_prg(&mut self, addr: u16) -> u8;
    fn write_prg(&mut self, addr: u16, data: u8);
    fn read_chr(&mut self, addr: u16) -> u8;
    fn write_chr(&mut self, addr: u16, data: u8);
    fn mirror_mode(&self) -> MirrorMode;
    fn reset(&mut self);
}