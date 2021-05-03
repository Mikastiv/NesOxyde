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

/// Cartridge ROM Mapping
pub trait Mapper {
    /// Reads a byte from PRG ROM
    fn read_prg(&mut self, addr: u16) -> u8;
    
    /// Writes a byte to PRG ROM
    fn write_prg(&mut self, addr: u16, data: u8);
    
    /// Reads a byte from CHR ROM
    fn read_chr(&mut self, addr: u16) -> u8;
    
    /// Writes a byte to CHR ROM
    fn write_chr(&mut self, addr: u16, data: u8);

    /// Returns the current mirroring mode
    fn mirror_mode(&self) -> MirrorMode;

    /// Resets the mapper
    fn reset(&mut self);

    /// Tells the mapper a new scanline was rendered
    ///
    /// This is only used by a few mappers and only by Mapper4 in my emulator
    fn inc_scanline(&mut self) {}

    /// Returns if the mapper is requesting an interrupt or not
    ///
    /// This is only used by a few mappers and only by Mapper4 in my emulator
    fn poll_irq(&mut self) -> bool {
        false
    }
}
