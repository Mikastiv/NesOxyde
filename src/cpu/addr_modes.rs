#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AddrMode {
    None, // For KIL operations
    Imp,  // Implied
    Imm,  // Immediate
    Zp0,  // Zero page
    Zpx,  // Zero page with X
    Zpy,  // Zero page with Y
    Rel,  // Relative
    Abs,  // Absolute
    Abx,  // Absolute with X
    AbxW, // Absolute with X (Write)
    Aby,  // Absolute with Y
    AbyW, // Absolute with Y (Write)
    Ind,  // Indirect
    Izx,  // Indirect with X
    Izy,  // Indirect with Y
    IzyW, // Indirect with Y (Write)
}

impl std::fmt::Display for AddrMode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            AddrMode::None => write!(f, "None"),
            AddrMode::Imp => write!(f, "IMP"),
            AddrMode::Imm => write!(f, "IMM"),
            AddrMode::Zp0 => write!(f, "ZP0"),
            AddrMode::Zpx => write!(f, "ZPX"),
            AddrMode::Zpy => write!(f, "ZPY"),
            AddrMode::Rel => write!(f, "REL"),
            AddrMode::Abs => write!(f, "ABS"),
            AddrMode::Abx => write!(f, "ABX"),
            AddrMode::AbxW => write!(f, "ABXW"),
            AddrMode::Aby => write!(f, "ABY"),
            AddrMode::AbyW => write!(f, "ABYW"),
            AddrMode::Ind => write!(f, "IND"),
            AddrMode::Izx => write!(f, "IZX"),
            AddrMode::Izy => write!(f, "IZY"),
            AddrMode::IzyW => write!(f, "IZYW"),
        }
    }
}
