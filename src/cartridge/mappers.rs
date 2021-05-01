pub use mapper0::Mapper0;
pub use mapper1::Mapper1;
pub use mapper2::Mapper2;
pub use mapper3::Mapper3;
pub use mapper4::Mapper4;
pub use mapper7::Mapper7;
pub use mapper9::Mapper9;
pub use mapper10::Mapper10;

use super::MirrorMode;

mod mapper0;
mod mapper1;
mod mapper2;
mod mapper3;
mod mapper4;
mod mapper7;
mod mapper9;
mod mapper10;

pub trait Mapper {
    fn read_prg(&mut self, addr: u16) -> u8;
    fn write_prg(&mut self, addr: u16, data: u8);
    fn read_chr(&mut self, addr: u16) -> u8;
    fn write_chr(&mut self, addr: u16, data: u8);
    fn mirror_mode(&self) -> MirrorMode;
    fn reset(&mut self);
    fn inc_scanline(&mut self) {}
    fn poll_irq(&mut self) -> bool {
        false
    }
}
