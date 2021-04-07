use crate::cpu::{AddrMode, Cpu, OPTABLE};

impl Cpu {
    fn operand_addr_peek(&mut self, mode: AddrMode, pc: u16) -> u16 {
        match mode {
            AddrMode::None | AddrMode::Imp => 0,
            AddrMode::Imm | AddrMode::Rel => pc,
            AddrMode::Zp0 => self.mem_read(pc) as u16,
            AddrMode::Zpx => {
                let base = self.mem_read(pc);
                base.wrapping_add(self.x()) as u16
            }
            AddrMode::Zpy => {
                let base = self.mem_read(pc);
                base.wrapping_add(self.y()) as u16
            }
            AddrMode::Abs | AddrMode::Ind => self.mem_read_word(pc),
            AddrMode::Abx => {
                let base = self.mem_read_word(pc);
                base.wrapping_add(self.x() as u16)
            }
            AddrMode::AbxW => {
                let base = self.mem_read_word(pc);
                base.wrapping_add(self.x() as u16)
            }
            AddrMode::Aby => {
                let base = self.mem_read_word(pc);
                base.wrapping_add(self.y() as u16)
            }
            AddrMode::AbyW => {
                let base = self.mem_read_word(pc);
                base.wrapping_add(self.y() as u16)
            }
            AddrMode::Izx => {
                let base = self.mem_read(pc);
                let ptr = base.wrapping_add(self.x());
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                u16::from_le_bytes([lo, hi])
            }
            AddrMode::Izy => {
                let ptr = self.mem_read(pc);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                u16::from_le_bytes([lo, hi]).wrapping_add(self.y() as u16)
            }
            AddrMode::IzyW => {
                let ptr = self.mem_read(pc);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                u16::from_le_bytes([lo, hi]).wrapping_add(self.y() as u16)
            }
        }
    }
}

pub fn trace(cpu: &mut Cpu) -> String {
    let code = cpu.mem_read(cpu.pc());
    let ops = *OPTABLE.get(&code).unwrap();

    let begin = cpu.pc();
    let mut hex_dump = vec![code];

    let (mem_addr, stored_value) = match ops.mode {
        AddrMode::Imm | AddrMode::None | AddrMode::Imp => (0, 0),
        _ => {
            let addr = cpu.operand_addr_peek(ops.mode, begin + 1);
            (addr, cpu.mem_read(addr))
        }
    };

    let tmp = match ops.mode {
        AddrMode::None | AddrMode::Imp => match ops.opcode {
            0x0a | 0x4a | 0x2a | 0x6a => "A ".to_string(),
            _ => String::from(""),
        },
        AddrMode::Imm
        | AddrMode::Zp0
        | AddrMode::Zpx
        | AddrMode::Zpy
        | AddrMode::Izx
        | AddrMode::Izy
        | AddrMode::IzyW
        | AddrMode::Rel => {
            let address: u8 = cpu.mem_read(begin + 1);
            hex_dump.push(address);

            match ops.mode {
                AddrMode::Imm => format!("#${:02x}", address),
                AddrMode::Zp0 => format!("${:02x} = {:02x}", mem_addr, stored_value),
                AddrMode::Zpx => format!(
                    "${:02x},X @ {:02x} = {:02x}",
                    address, mem_addr, stored_value
                ),
                AddrMode::Zpy => format!(
                    "${:02x},Y @ {:02x} = {:02x}",
                    address, mem_addr, stored_value
                ),
                AddrMode::Izx => format!(
                    "(${:02x},X) @ {:02x} = {:04x} = {:02x}",
                    address,
                    (address.wrapping_add(cpu.x())),
                    mem_addr,
                    stored_value
                ),
                AddrMode::Izy | AddrMode::IzyW => format!(
                    "(${:02x}),Y = {:04x} @ {:04x} = {:02x}",
                    address,
                    (mem_addr.wrapping_sub(cpu.y() as u16)),
                    mem_addr,
                    stored_value
                ),
                AddrMode::Rel => {
                    // assuming local jumps: BNE, BVS, etc....
                    let address: usize =
                        (begin as usize + 2).wrapping_add((address as i8) as usize);
                    format!("${:04x}", address)
                }

                _ => panic!(
                    "unexpected addressing mode {:?} has ops-len 2. code {:02x}",
                    ops.mode, ops.opcode
                ),
            }
        }
        AddrMode::Abs
        | AddrMode::Abx
        | AddrMode::AbxW
        | AddrMode::Aby
        | AddrMode::AbyW
        | AddrMode::Ind => {
            let address_lo = cpu.mem_read(begin + 1);
            let address_hi = cpu.mem_read(begin + 2);
            hex_dump.push(address_lo);
            hex_dump.push(address_hi);

            let address = cpu.mem_read_word(begin + 1);

            match ops.mode {
                AddrMode::Ind | AddrMode::Abs
                    if (ops.opcode == 0x4C) | (ops.opcode == 0x20) | (ops.opcode == 0x6C) =>
                {
                    if ops.opcode == 0x6C {
                        //jmp indirect
                        let jmp_addr = if address & 0x00FF == 0x00FF {
                            let lo = cpu.mem_read(address);
                            let hi = cpu.mem_read(address & 0xFF00);
                            (hi as u16) << 8 | (lo as u16)
                        } else {
                            cpu.mem_read_word(address)
                        };

                        format!("(${:04x}) = {:04x}", address, jmp_addr)
                    } else {
                        format!("${:04x}", address)
                    }
                }
                AddrMode::Abs => format!("${:04x} = {:02x}", mem_addr, stored_value),
                AddrMode::Abx | AddrMode::AbxW => format!(
                    "${:04x},X @ {:04x} = {:02x}",
                    address, mem_addr, stored_value
                ),
                AddrMode::Aby | AddrMode::AbyW => format!(
                    "${:04x},Y @ {:04x} = {:02x}",
                    address, mem_addr, stored_value
                ),
                _ => panic!(
                    "unexpected addressing mode {:?} has ops-len 3. code {:02x}",
                    ops.mode, ops.opcode
                ),
            }
        }
    };

    let hex_str = hex_dump
        .iter()
        .map(|z| format!("{:02x}", z))
        .collect::<Vec<String>>()
        .join(" ");
    let asm_str = format!("{:04x}  {:8} {: >4} {}", begin, hex_str, ops.mnemonic, tmp)
        .trim()
        .to_string();

    format!(
        "{:47} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x} CYC:{}",
        asm_str,
        cpu.a(),
        cpu.x(),
        cpu.y(),
        cpu.p(),
        cpu.s(),
        cpu.cycles()
    )
    .to_ascii_uppercase()
}
