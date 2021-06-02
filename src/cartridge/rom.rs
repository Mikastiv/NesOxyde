use std::fmt::Display;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

use super::MirrorMode;

/// Size of one PRG bank
pub const PRG_PAGE_SIZE: usize = 0x4000;
/// Size of one CHR bank
pub const CHR_PAGE_SIZE: usize = 0x2000;
/// Size of the iNES header
const HEADER_SIZE: usize = 16;
/// iNES header tag. Must be at the start of the file
const NES_TAG: [u8; 4] = [b'N', b'E', b'S', 0x1A];

/// Header of the iNES file format
#[derive(Clone, Copy)]
pub struct INesHeader {
    bytes: [u8; 16],
}

impl INesHeader {
    pub fn new(bytes: [u8; HEADER_SIZE]) -> Self {
        Self { bytes }
    }

    pub fn is_valid(&self) -> bool {
        self.bytes[..4] == NES_TAG
    }

    /// PRG bank count
    pub fn prg_count(&self) -> usize {
        self.bytes[4] as usize
    }

    /// CHR bank count
    pub fn chr_count(&self) -> usize {
        self.bytes[5] as usize
    }

    /// Contains trainer data or not
    pub fn has_trainer(&self) -> bool {
        self.bytes[6] & 0x4 != 0
    }

    /// Hardware mirror mode
    pub fn mirror_mode(&self) -> MirrorMode {
        match self.bytes[6] & 0x1 != 0 {
            true => MirrorMode::Vertical,
            false => MirrorMode::Horizontal,
        }
    }

    /// Uses 4 screen VRAM
    pub fn four_screen(&self) -> bool {
        self.bytes[6] & 0x8 != 0
    }

    /// ID of the iNES mapper
    pub fn mapper_id(&self) -> u8 {
        (self.bytes[7] & 0xF0) | (self.bytes[6] >> 4)
    }
}

/// Game ROM data
pub struct Rom {
    pub header: INesHeader,
    pub prg: Vec<u8>,
    pub chr: Vec<u8>,
}

impl Rom {
    pub fn new<P: AsRef<Path> + Display>(romfile: P) -> io::Result<Self> {
        let mut file = File::open(&romfile)?;

        let mut buf = [0; HEADER_SIZE];
        file.read_exact(&mut buf)?;
        let header = INesHeader::new(buf);

        if !header.is_valid() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Not iNES file format",
            ));
        }

        if !(header.bytes[7] & 0xC == 0 && header.bytes[12..].iter().all(|byte| *byte == 0)) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "This emulator does not support iNES2.0 nor archaic iNES",
            ));
        }

        if header.has_trainer() {
            file.seek(SeekFrom::Current(512))?;
        }

        let prg_size = PRG_PAGE_SIZE * header.prg_count();
        let prg_start = 0;
        let chr_size = CHR_PAGE_SIZE * header.chr_count();
        let chr_start = prg_start + prg_size;

        println!(
            "PRG Size: {} * {:#06X} = {:#06X} ({} KB)",
            header.prg_count(),
            PRG_PAGE_SIZE,
            prg_size,
            header.prg_count() * 16,
        );
        if header.chr_count() == 0 {
            println!(
                "CHR Size (RAM): 1 * {:#06X} = {:#06X} (8 KB)",
                CHR_PAGE_SIZE, CHR_PAGE_SIZE,
            );
        } else {
            println!(
                "CHR Size: {} * {:#06X} = {:#06X} ({} KB)",
                header.chr_count(),
                CHR_PAGE_SIZE,
                chr_size,
                header.chr_count() * 8
            );
        }
        println!("Mapper ID: {}", header.mapper_id());

        let mut rom_bytes = Vec::new();
        file.read_to_end(&mut rom_bytes)?;

        let prg = rom_bytes[prg_start..(prg_start + prg_size)].to_vec();
        let chr = if header.chr_count() == 0 {
            vec![0; CHR_PAGE_SIZE]
        } else {
            rom_bytes[chr_start..(chr_start + chr_size)].to_vec()
        };

        Ok(Self { header, prg, chr })
    }
}
