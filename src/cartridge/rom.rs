use std::fmt::Display;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use super::MirrorMode;

const PRG_PAGE_SIZE: usize = 0x4000;
const CHR_PAGE_SIZE: usize = 0x2000;
const HEADER_SIZE: usize = 16;
const NES_TAG: [u8; 4] = [b'N', b'E', b'S', 0x1A];

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

    pub fn prg_count(&self) -> usize {
        self.bytes[4] as usize
    }

    pub fn chr_count(&self) -> usize {
        self.bytes[5] as usize
    }

    pub fn has_trainer(&self) -> bool {
        self.bytes[6] & 0x4 != 0
    }

    pub fn mirror_mode(&self) -> MirrorMode {
        match self.bytes[6] & 0x1 != 0 {
            true => MirrorMode::Vertical,
            false => MirrorMode::Horizontal,
        }
    }

    pub fn mapper_id(&self) -> u8 {
        (self.bytes[7] & 0xF0) | (self.bytes[6] >> 4)
    }
}

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

        let prg_size = PRG_PAGE_SIZE * header.prg_count();
        let prg_start = if header.has_trainer() { 512 } else { 0 };
        let chr_size = CHR_PAGE_SIZE * header.chr_count();
        let chr_start = prg_start + prg_size;

        println!(
            "PRG Size: {} * {:#06X} = {:#06X}",
            header.prg_count(),
            PRG_PAGE_SIZE,
            prg_size
        );
        println!(
            "CHR Size: {} * {:#06X} = {:#06X}",
            header.chr_count(),
            CHR_PAGE_SIZE,
            chr_size
        );
        println!("Mapper ID: {}", header.mapper_id());
        println!("Mirror mode: {:?}", header.mirror_mode());

        let mut rom_bytes = Vec::new();
        file.read_to_end(&mut rom_bytes)?;

        Ok(Self {
            header,
            prg: rom_bytes[prg_start..(prg_start + prg_size)].to_vec(),
            chr: rom_bytes[chr_start..(chr_start + chr_size)].to_vec(),
        })
    }
}
