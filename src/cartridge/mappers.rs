pub use mapper_0::Mapper0;

use super::MirrorMode;

mod mapper_0;

pub trait Mapper {
    fn read_prg(&mut self, addr: u16) -> u8;
    fn write_prg(&mut self, addr: u16, data: u8);
    fn read_chr(&mut self, addr: u16) -> u8;
    fn write_chr(&mut self, addr: u16, data: u8);
    fn mirror_mode(&self) -> MirrorMode;
}