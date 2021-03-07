#[derive(Copy, Clone, PartialEq)]
pub enum AddrMode {
    None, // For KIL operations
    IMP,  // Implied
    IMM,  // Immediate
    ZP0,  // Zero page
    ZPX,  // Zero page with X
    ZPY,  // Zero page with Y
    REL,  // Relative
    ABS,  // Absolute
    ABX,  // Absolute with X
    ABXW, // Absolute with X (Write)
    ABY,  // Absolute with Y
    ABYW, // Absolute with Y (Write)
    IND,  // Indirect
    IZX,  // Indirect with X
    IZY,  // Indirect with Y
    IZYW, // Indirect with Y (Write)
}

impl std::fmt::Display for AddrMode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            AddrMode::None => write!(f, "None"),
            AddrMode::IMP => write!(f, "IMP"),
            AddrMode::IMM => write!(f, "IMM"),
            AddrMode::ZP0 => write!(f, "ZP0"),
            AddrMode::ZPX => write!(f, "ZPX"),
            AddrMode::ZPY => write!(f, "ZPY"),
            AddrMode::REL => write!(f, "REL"),
            AddrMode::ABS => write!(f, "ABS"),
            AddrMode::ABX => write!(f, "ABX"),
            AddrMode::ABXW => write!(f, "ABXW"),
            AddrMode::ABY => write!(f, "ABY"),
            AddrMode::ABYW => write!(f, "ABYW"),
            AddrMode::IND => write!(f, "IND"),
            AddrMode::IZX => write!(f, "IZX"),
            AddrMode::IZY => write!(f, "IZY"),
            AddrMode::IZYW => write!(f, "IZYW"),
        }
    }
}
