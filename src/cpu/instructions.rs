use lazy_static::lazy_static;
use std::collections::HashMap;

use super::{AddrMode, Cpu};

/// Cpu instruction information
#[derive(Clone, Copy)]
pub struct Instruction {
    pub opcode: u8,
    pub mnemonic: &'static str,
    pub cpu_fn: fn(&mut Cpu, AddrMode),
    pub mode: AddrMode,
    pub cycles: u64,
}

impl Instruction {
    pub fn new(
        opcode: u8,
        mnemonic: &'static str,
        cpu_fn: fn(&mut Cpu, AddrMode),
        mode: AddrMode,
        cycles: u64,
    ) -> Self {
        Self {
            opcode,
            mnemonic,
            cpu_fn,
            mode,
            cycles,
        }
    }
}

lazy_static! {
    /// List of all the possible instructions
    pub static ref INSTRUCTIONS: Vec<Instruction> = vec![
        Instruction::new(0xA9, LDA, |cpu, mode| cpu.lda(mode), AddrMode::Imm, 2),
        Instruction::new(0xA5, LDA, |cpu, mode| cpu.lda(mode), AddrMode::Zp0, 3),
        Instruction::new(0xB5, LDA, |cpu, mode| cpu.lda(mode), AddrMode::Zpx, 4),
        Instruction::new(0xAD, LDA, |cpu, mode| cpu.lda(mode), AddrMode::Abs, 4),
        Instruction::new(0xBD, LDA, |cpu, mode| cpu.lda(mode), AddrMode::Abx, 4),
        Instruction::new(0xB9, LDA, |cpu, mode| cpu.lda(mode), AddrMode::Aby, 4),
        Instruction::new(0xA1, LDA, |cpu, mode| cpu.lda(mode), AddrMode::Izx, 6),
        Instruction::new(0xB1, LDA, |cpu, mode| cpu.lda(mode), AddrMode::Izy, 5),

        Instruction::new(0xA2, LDX, |cpu, mode| cpu.ldx(mode), AddrMode::Imm, 2),
        Instruction::new(0xA6, LDX, |cpu, mode| cpu.ldx(mode), AddrMode::Zp0, 3),
        Instruction::new(0xB6, LDX, |cpu, mode| cpu.ldx(mode), AddrMode::Zpy, 4),
        Instruction::new(0xAE, LDX, |cpu, mode| cpu.ldx(mode), AddrMode::Abs, 4),
        Instruction::new(0xBE, LDX, |cpu, mode| cpu.ldx(mode), AddrMode::Aby, 4),

        Instruction::new(0xA0, LDY, |cpu, mode| cpu.ldy(mode), AddrMode::Imm, 2),
        Instruction::new(0xA4, LDY, |cpu, mode| cpu.ldy(mode), AddrMode::Zp0, 3),
        Instruction::new(0xB4, LDY, |cpu, mode| cpu.ldy(mode), AddrMode::Zpx, 4),
        Instruction::new(0xAC, LDY, |cpu, mode| cpu.ldy(mode), AddrMode::Abs, 4),
        Instruction::new(0xBC, LDY, |cpu, mode| cpu.ldy(mode), AddrMode::Abx, 4),

        Instruction::new(0x85, STA, |cpu, mode| cpu.sta(mode), AddrMode::Zp0, 3),
        Instruction::new(0x95, STA, |cpu, mode| cpu.sta(mode), AddrMode::Zpx, 4),
        Instruction::new(0x8D, STA, |cpu, mode| cpu.sta(mode), AddrMode::Abs, 4),
        Instruction::new(0x9D, STA, |cpu, mode| cpu.sta(mode), AddrMode::AbxW, 5),
        Instruction::new(0x99, STA, |cpu, mode| cpu.sta(mode), AddrMode::AbyW, 5),
        Instruction::new(0x81, STA, |cpu, mode| cpu.sta(mode), AddrMode::Izx, 6),
        Instruction::new(0x91, STA, |cpu, mode| cpu.sta(mode), AddrMode::IzyW, 6),

        Instruction::new(0x86, STX, |cpu, mode| cpu.stx(mode), AddrMode::Zp0, 3),
        Instruction::new(0x96, STX, |cpu, mode| cpu.stx(mode), AddrMode::Zpy, 4),
        Instruction::new(0x8E, STX, |cpu, mode| cpu.stx(mode), AddrMode::Abs, 4),

        Instruction::new(0x84, STY, |cpu, mode| cpu.sty(mode), AddrMode::Zp0, 3),
        Instruction::new(0x94, STY, |cpu, mode| cpu.sty(mode), AddrMode::Zpx, 4),
        Instruction::new(0x8C, STY, |cpu, mode| cpu.sty(mode), AddrMode::Abs, 4),

        Instruction::new(0xAA, TAX, |cpu, mode| cpu.tax(mode), AddrMode::Imp, 2),
        Instruction::new(0xA8, TAY, |cpu, mode| cpu.tay(mode), AddrMode::Imp, 2),
        Instruction::new(0xBA, TSX, |cpu, mode| cpu.tsx(mode), AddrMode::Imp, 2),
        Instruction::new(0x8A, TXA, |cpu, mode| cpu.txa(mode), AddrMode::Imp, 2),
        Instruction::new(0x9A, TXS, |cpu, mode| cpu.txs(mode), AddrMode::Imp, 2),
        Instruction::new(0x98, TYA, |cpu, mode| cpu.tya(mode), AddrMode::Imp, 2),

        Instruction::new(0x18, CLC, |cpu, mode| cpu.clc(mode), AddrMode::Imp, 2),
        Instruction::new(0xD8, CLD, |cpu, mode| cpu.cld(mode), AddrMode::Imp, 2),
        Instruction::new(0x58, CLI, |cpu, mode| cpu.cli(mode), AddrMode::Imp, 2),
        Instruction::new(0xB8, CLV, |cpu, mode| cpu.clv(mode), AddrMode::Imp, 2),
        Instruction::new(0x38, SEC, |cpu, mode| cpu.sec(mode), AddrMode::Imp, 2),
        Instruction::new(0xF8, SED, |cpu, mode| cpu.sed(mode), AddrMode::Imp, 2),
        Instruction::new(0x78, SEI, |cpu, mode| cpu.sei(mode), AddrMode::Imp, 2),

        Instruction::new(0xE6, INC, |cpu, mode| cpu.inc(mode), AddrMode::Zp0, 5),
        Instruction::new(0xF6, INC, |cpu, mode| cpu.inc(mode), AddrMode::Zpx, 6),
        Instruction::new(0xEE, INC, |cpu, mode| cpu.inc(mode), AddrMode::Abs, 6),
        Instruction::new(0xFE, INC, |cpu, mode| cpu.inc(mode), AddrMode::AbxW, 7),

        Instruction::new(0xE8, INX, |cpu, mode| cpu.inx(mode), AddrMode::Imp, 2),
        Instruction::new(0xC8, INY, |cpu, mode| cpu.iny(mode), AddrMode::Imp, 2),

        Instruction::new(0xC6, DEC, |cpu, mode| cpu.dec(mode), AddrMode::Zp0, 5),
        Instruction::new(0xD6, DEC, |cpu, mode| cpu.dec(mode), AddrMode::Zpx, 6),
        Instruction::new(0xCE, DEC, |cpu, mode| cpu.dec(mode), AddrMode::Abs, 6),
        Instruction::new(0xDE, DEC, |cpu, mode| cpu.dec(mode), AddrMode::AbxW, 7),

        Instruction::new(0xCA, DEX, |cpu, mode| cpu.dex(mode), AddrMode::Imp, 2),
        Instruction::new(0x88, DEY, |cpu, mode| cpu.dey(mode), AddrMode::Imp, 2),

        Instruction::new(0xC9, CMP, |cpu, mode| cpu.cpa(mode), AddrMode::Imm, 2),
        Instruction::new(0xC5, CMP, |cpu, mode| cpu.cpa(mode), AddrMode::Zp0, 3),
        Instruction::new(0xD5, CMP, |cpu, mode| cpu.cpa(mode), AddrMode::Zpx, 4),
        Instruction::new(0xCD, CMP, |cpu, mode| cpu.cpa(mode), AddrMode::Abs, 4),
        Instruction::new(0xDD, CMP, |cpu, mode| cpu.cpa(mode), AddrMode::Abx, 4),
        Instruction::new(0xD9, CMP, |cpu, mode| cpu.cpa(mode), AddrMode::Aby, 4),
        Instruction::new(0xC1, CMP, |cpu, mode| cpu.cpa(mode), AddrMode::Izx, 6),
        Instruction::new(0xD1, CMP, |cpu, mode| cpu.cpa(mode), AddrMode::Izy, 5),

        Instruction::new(0xE0, CPX, |cpu, mode| cpu.cpx(mode), AddrMode::Imm, 2),
        Instruction::new(0xE4, CPX, |cpu, mode| cpu.cpx(mode), AddrMode::Zp0, 3),
        Instruction::new(0xEC, CPX, |cpu, mode| cpu.cpx(mode), AddrMode::Abs, 4),

        Instruction::new(0xC0, CPY, |cpu, mode| cpu.cpy(mode), AddrMode::Imm, 2),
        Instruction::new(0xC4, CPY, |cpu, mode| cpu.cpy(mode), AddrMode::Zp0, 3),
        Instruction::new(0xCC, CPY, |cpu, mode| cpu.cpy(mode), AddrMode::Abs, 4),

        Instruction::new(0x90, BCC, |cpu, mode| cpu.bcc(mode), AddrMode::Rel, 2),
        Instruction::new(0xB0, BCS, |cpu, mode| cpu.bcs(mode), AddrMode::Rel, 2),
        Instruction::new(0xF0, BEQ, |cpu, mode| cpu.beq(mode), AddrMode::Rel, 2),
        Instruction::new(0xD0, BNE, |cpu, mode| cpu.bne(mode), AddrMode::Rel, 2),
        Instruction::new(0x30, BMI, |cpu, mode| cpu.bmi(mode), AddrMode::Rel, 2),
        Instruction::new(0x10, BPL, |cpu, mode| cpu.bpl(mode), AddrMode::Rel, 2),
        Instruction::new(0x50, BVC, |cpu, mode| cpu.bvc(mode), AddrMode::Rel, 2),
        Instruction::new(0x70, BVS, |cpu, mode| cpu.bvs(mode), AddrMode::Rel, 2),

        Instruction::new(0x4C, JMP, |cpu, mode| cpu.jmp_abs(mode), AddrMode::Abs, 3),
        Instruction::new(0x6C, JMP, |cpu, mode| cpu.jmp_ind(mode), AddrMode::Ind, 5),

        Instruction::new(0x00, BRK, |cpu, mode| cpu.brk(mode), AddrMode::Imp, 3),
        Instruction::new(0x48, PHA, |cpu, mode| cpu.pha(mode), AddrMode::Imp, 3),
        Instruction::new(0x08, PHP, |cpu, mode| cpu.php(mode), AddrMode::Imp, 3),
        Instruction::new(0x68, PLA, |cpu, mode| cpu.pla(mode), AddrMode::Imp, 4),
        Instruction::new(0x28, PLP, |cpu, mode| cpu.plp(mode), AddrMode::Imp, 4),

        Instruction::new(0x20, JSR, |cpu, mode| cpu.jsr(mode), AddrMode::Abs, 6),
        Instruction::new(0x60, RTS, |cpu, mode| cpu.rts(mode), AddrMode::Imp, 6),
        Instruction::new(0x40, RTI, |cpu, mode| cpu.rti(mode), AddrMode::Imp, 6),

        Instruction::new(0xEA, NOP, |cpu, mode| cpu.nop(mode), AddrMode::Imp, 2),

        Instruction::new(0x24, BIT, |cpu, mode| cpu.bit(mode), AddrMode::Zp0, 3),
        Instruction::new(0x2C, BIT, |cpu, mode| cpu.bit(mode), AddrMode::Abs, 4),

        Instruction::new(0x29, AND, |cpu, mode| cpu.and(mode), AddrMode::Imm, 2),
        Instruction::new(0x25, AND, |cpu, mode| cpu.and(mode), AddrMode::Zp0, 3),
        Instruction::new(0x35, AND, |cpu, mode| cpu.and(mode), AddrMode::Zpx, 4),
        Instruction::new(0x2D, AND, |cpu, mode| cpu.and(mode), AddrMode::Abs, 4),
        Instruction::new(0x3D, AND, |cpu, mode| cpu.and(mode), AddrMode::Abx, 4),
        Instruction::new(0x39, AND, |cpu, mode| cpu.and(mode), AddrMode::Aby, 4),
        Instruction::new(0x21, AND, |cpu, mode| cpu.and(mode), AddrMode::Izx, 6),
        Instruction::new(0x31, AND, |cpu, mode| cpu.and(mode), AddrMode::Izy, 5),

        Instruction::new(0x49, EOR, |cpu, mode| cpu.eor(mode), AddrMode::Imm, 2),
        Instruction::new(0x45, EOR, |cpu, mode| cpu.eor(mode), AddrMode::Zp0, 3),
        Instruction::new(0x55, EOR, |cpu, mode| cpu.eor(mode), AddrMode::Zpx, 4),
        Instruction::new(0x4D, EOR, |cpu, mode| cpu.eor(mode), AddrMode::Abs, 4),
        Instruction::new(0x5D, EOR, |cpu, mode| cpu.eor(mode), AddrMode::Abx, 4),
        Instruction::new(0x59, EOR, |cpu, mode| cpu.eor(mode), AddrMode::Aby, 4),
        Instruction::new(0x41, EOR, |cpu, mode| cpu.eor(mode), AddrMode::Izx, 6),
        Instruction::new(0x51, EOR, |cpu, mode| cpu.eor(mode), AddrMode::Izy, 5),

        Instruction::new(0x09, ORA, |cpu, mode| cpu.ora(mode), AddrMode::Imm, 2),
        Instruction::new(0x05, ORA, |cpu, mode| cpu.ora(mode), AddrMode::Zp0, 3),
        Instruction::new(0x15, ORA, |cpu, mode| cpu.ora(mode), AddrMode::Zpx, 4),
        Instruction::new(0x0D, ORA, |cpu, mode| cpu.ora(mode), AddrMode::Abs, 4),
        Instruction::new(0x1D, ORA, |cpu, mode| cpu.ora(mode), AddrMode::Abx, 4),
        Instruction::new(0x19, ORA, |cpu, mode| cpu.ora(mode), AddrMode::Aby, 4),
        Instruction::new(0x01, ORA, |cpu, mode| cpu.ora(mode), AddrMode::Izx, 6),
        Instruction::new(0x11, ORA, |cpu, mode| cpu.ora(mode), AddrMode::Izy, 5),

        Instruction::new(0x0A, ASL, |cpu, mode| cpu.asl_acc(mode), AddrMode::Imp, 2),
        Instruction::new(0x06, ASL, |cpu, mode| cpu.asl_mem(mode), AddrMode::Zp0, 5),
        Instruction::new(0x16, ASL, |cpu, mode| cpu.asl_mem(mode), AddrMode::Zpx, 6),
        Instruction::new(0x0E, ASL, |cpu, mode| cpu.asl_mem(mode), AddrMode::Abs, 6),
        Instruction::new(0x1E, ASL, |cpu, mode| cpu.asl_mem(mode), AddrMode::AbxW, 7),

        Instruction::new(0x4A, LSR, |cpu, mode| cpu.lsr_acc(mode), AddrMode::Imp, 2),
        Instruction::new(0x46, LSR, |cpu, mode| cpu.lsr_mem(mode), AddrMode::Zp0, 5),
        Instruction::new(0x56, LSR, |cpu, mode| cpu.lsr_mem(mode), AddrMode::Zpx, 6),
        Instruction::new(0x4E, LSR, |cpu, mode| cpu.lsr_mem(mode), AddrMode::Abs, 6),
        Instruction::new(0x5E, LSR, |cpu, mode| cpu.lsr_mem(mode), AddrMode::AbxW, 7),

        Instruction::new(0x2A, ROL, |cpu, mode| cpu.rol_acc(mode), AddrMode::Imp, 2),
        Instruction::new(0x26, ROL, |cpu, mode| cpu.rol_mem(mode), AddrMode::Zp0, 5),
        Instruction::new(0x36, ROL, |cpu, mode| cpu.rol_mem(mode), AddrMode::Zpx, 6),
        Instruction::new(0x2E, ROL, |cpu, mode| cpu.rol_mem(mode), AddrMode::Abs, 6),
        Instruction::new(0x3E, ROL, |cpu, mode| cpu.rol_mem(mode), AddrMode::AbxW, 7),

        Instruction::new(0x6A, ROR, |cpu, mode| cpu.ror_acc(mode), AddrMode::Imp, 2),
        Instruction::new(0x66, ROR, |cpu, mode| cpu.ror_mem(mode), AddrMode::Zp0, 5),
        Instruction::new(0x76, ROR, |cpu, mode| cpu.ror_mem(mode), AddrMode::Zpx, 6),
        Instruction::new(0x6E, ROR, |cpu, mode| cpu.ror_mem(mode), AddrMode::Abs, 6),
        Instruction::new(0x7E, ROR, |cpu, mode| cpu.ror_mem(mode), AddrMode::AbxW, 7),

        Instruction::new(0x69, ADC, |cpu, mode| cpu.adc(mode), AddrMode::Imm, 2),
        Instruction::new(0x65, ADC, |cpu, mode| cpu.adc(mode), AddrMode::Zp0, 3),
        Instruction::new(0x75, ADC, |cpu, mode| cpu.adc(mode), AddrMode::Zpx, 4),
        Instruction::new(0x6D, ADC, |cpu, mode| cpu.adc(mode), AddrMode::Abs, 4),
        Instruction::new(0x7D, ADC, |cpu, mode| cpu.adc(mode), AddrMode::Abx, 4),
        Instruction::new(0x79, ADC, |cpu, mode| cpu.adc(mode), AddrMode::Aby, 4),
        Instruction::new(0x61, ADC, |cpu, mode| cpu.adc(mode), AddrMode::Izx, 6),
        Instruction::new(0x71, ADC, |cpu, mode| cpu.adc(mode), AddrMode::Izy, 5),

        Instruction::new(0xE9, SBC, |cpu, mode| cpu.sbc(mode), AddrMode::Imm, 2),
        Instruction::new(0xE5, SBC, |cpu, mode| cpu.sbc(mode), AddrMode::Zp0, 3),
        Instruction::new(0xF5, SBC, |cpu, mode| cpu.sbc(mode), AddrMode::Zpx, 4),
        Instruction::new(0xED, SBC, |cpu, mode| cpu.sbc(mode), AddrMode::Abs, 4),
        Instruction::new(0xFD, SBC, |cpu, mode| cpu.sbc(mode), AddrMode::Abx, 4),
        Instruction::new(0xF9, SBC, |cpu, mode| cpu.sbc(mode), AddrMode::Aby, 4),
        Instruction::new(0xE1, SBC, |cpu, mode| cpu.sbc(mode), AddrMode::Izx, 6),
        Instruction::new(0xF1, SBC, |cpu, mode| cpu.sbc(mode), AddrMode::Izy, 5),

        Instruction::new(0x02, KIL, |cpu, mode| cpu.kil(mode), AddrMode::None, 0),
        Instruction::new(0x12, KIL, |cpu, mode| cpu.kil(mode), AddrMode::None, 0),
        Instruction::new(0x22, KIL, |cpu, mode| cpu.kil(mode), AddrMode::None, 0),
        Instruction::new(0x32, KIL, |cpu, mode| cpu.kil(mode), AddrMode::None, 0),
        Instruction::new(0x42, KIL, |cpu, mode| cpu.kil(mode), AddrMode::None, 0),
        Instruction::new(0x52, KIL, |cpu, mode| cpu.kil(mode), AddrMode::None, 0),
        Instruction::new(0x62, KIL, |cpu, mode| cpu.kil(mode), AddrMode::None, 0),
        Instruction::new(0x72, KIL, |cpu, mode| cpu.kil(mode), AddrMode::None, 0),
        Instruction::new(0x92, KIL, |cpu, mode| cpu.kil(mode), AddrMode::None, 0),
        Instruction::new(0xB2, KIL, |cpu, mode| cpu.kil(mode), AddrMode::None, 0),
        Instruction::new(0xD2, KIL, |cpu, mode| cpu.kil(mode), AddrMode::None, 0),
        Instruction::new(0xF2, KIL, |cpu, mode| cpu.kil(mode), AddrMode::None, 0),

        // --------------------------- Illegal opcodes ---------------------------

        Instruction::new(0x80, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Imm, 2),
        Instruction::new(0x82, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Imm, 2),
        Instruction::new(0xC2, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Imm, 2),
        Instruction::new(0xE2, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Imm, 2),
        Instruction::new(0x04, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Zp0, 3),
        Instruction::new(0x14, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Zpx, 4),
        Instruction::new(0x34, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Zpx, 4),
        Instruction::new(0x44, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Zp0, 3),
        Instruction::new(0x54, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Zpx, 4),
        Instruction::new(0x64, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Zp0, 3),
        Instruction::new(0x74, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Zpx, 4),
        Instruction::new(0xD4, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Zpx, 4),
        Instruction::new(0xF4, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Zpx, 4),
        Instruction::new(0x89, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Imm, 2),
        Instruction::new(0x1A, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Imp, 2),
        Instruction::new(0x3A, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Imp, 2),
        Instruction::new(0x5A, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Imp, 2),
        Instruction::new(0x7A, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Imp, 2),
        Instruction::new(0xDA, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Imp, 2),
        Instruction::new(0xFA, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Imp, 2),
        Instruction::new(0x0C, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Abs, 4),
        Instruction::new(0x1C, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Abx, 4),
        Instruction::new(0x3C, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Abx, 4),
        Instruction::new(0x5C, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Abx, 4),
        Instruction::new(0x7C, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Abx, 4),
        Instruction::new(0xDC, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Abx, 4),
        Instruction::new(0xFC, NOP_U, |cpu, mode| cpu.nop(mode), AddrMode::Abx, 4),

        Instruction::new(0x03, SLO, |cpu, mode| cpu.slo(mode), AddrMode::Izx, 8),
        Instruction::new(0x13, SLO, |cpu, mode| cpu.slo(mode), AddrMode::IzyW, 8),
        Instruction::new(0x07, SLO, |cpu, mode| cpu.slo(mode), AddrMode::Zp0, 5),
        Instruction::new(0x17, SLO, |cpu, mode| cpu.slo(mode), AddrMode::Zpx, 6),
        Instruction::new(0x1B, SLO, |cpu, mode| cpu.slo(mode), AddrMode::AbyW, 7),
        Instruction::new(0x0F, SLO, |cpu, mode| cpu.slo(mode), AddrMode::Abs, 6),
        Instruction::new(0x1F, SLO, |cpu, mode| cpu.slo(mode), AddrMode::AbxW, 7),

        Instruction::new(0x23, RLA, |cpu, mode| cpu.rla(mode), AddrMode::Izx, 8),
        Instruction::new(0x33, RLA, |cpu, mode| cpu.rla(mode), AddrMode::IzyW, 8),
        Instruction::new(0x27, RLA, |cpu, mode| cpu.rla(mode), AddrMode::Zp0, 5),
        Instruction::new(0x37, RLA, |cpu, mode| cpu.rla(mode), AddrMode::Zpx, 6),
        Instruction::new(0x3B, RLA, |cpu, mode| cpu.rla(mode), AddrMode::AbyW, 7),
        Instruction::new(0x2F, RLA, |cpu, mode| cpu.rla(mode), AddrMode::Abs, 6),
        Instruction::new(0x3F, RLA, |cpu, mode| cpu.rla(mode), AddrMode::AbxW, 7),

        Instruction::new(0x43, SRE, |cpu, mode| cpu.sre(mode), AddrMode::Izx, 8),
        Instruction::new(0x53, SRE, |cpu, mode| cpu.sre(mode), AddrMode::IzyW, 8),
        Instruction::new(0x47, SRE, |cpu, mode| cpu.sre(mode), AddrMode::Zp0, 5),
        Instruction::new(0x57, SRE, |cpu, mode| cpu.sre(mode), AddrMode::Zpx, 6),
        Instruction::new(0x5B, SRE, |cpu, mode| cpu.sre(mode), AddrMode::AbyW, 7),
        Instruction::new(0x4F, SRE, |cpu, mode| cpu.sre(mode), AddrMode::Abs, 6),
        Instruction::new(0x5F, SRE, |cpu, mode| cpu.sre(mode), AddrMode::AbxW, 7),

        Instruction::new(0x63, RRA, |cpu, mode| cpu.rra(mode), AddrMode::Izx, 8),
        Instruction::new(0x73, RRA, |cpu, mode| cpu.rra(mode), AddrMode::IzyW, 8),
        Instruction::new(0x67, RRA, |cpu, mode| cpu.rra(mode), AddrMode::Zp0, 5),
        Instruction::new(0x77, RRA, |cpu, mode| cpu.rra(mode), AddrMode::Zpx, 6),
        Instruction::new(0x7B, RRA, |cpu, mode| cpu.rra(mode), AddrMode::AbyW, 7),
        Instruction::new(0x6F, RRA, |cpu, mode| cpu.rra(mode), AddrMode::Abs, 6),
        Instruction::new(0x7F, RRA, |cpu, mode| cpu.rra(mode), AddrMode::AbxW, 7),

        Instruction::new(0x83, SAX, |cpu, mode| cpu.sax(mode), AddrMode::Izx, 6),
        Instruction::new(0x87, SAX, |cpu, mode| cpu.sax(mode), AddrMode::Zp0, 3),
        Instruction::new(0x97, SAX, |cpu, mode| cpu.sax(mode), AddrMode::Zpy, 4),
        Instruction::new(0x8F, SAX, |cpu, mode| cpu.sax(mode), AddrMode::Abs, 4),

        Instruction::new(0x93, AHX, |cpu, mode| cpu.ahx(mode), AddrMode::IzyW, 6),
        Instruction::new(0x9F, AHX, |cpu, mode| cpu.ahx(mode), AddrMode::AbyW, 5),

        Instruction::new(0xA3, LAX, |cpu, mode| cpu.lax(mode), AddrMode::Izx, 6),
        Instruction::new(0xB3, LAX, |cpu, mode| cpu.lax(mode), AddrMode::Izy, 5),
        Instruction::new(0xA7, LAX, |cpu, mode| cpu.lax(mode), AddrMode::Zp0, 3),
        Instruction::new(0xB7, LAX, |cpu, mode| cpu.lax(mode), AddrMode::Zpy, 4),
        Instruction::new(0xAB, LAX, |cpu, mode| cpu.lax(mode), AddrMode::Imm, 2),
        Instruction::new(0xAF, LAX, |cpu, mode| cpu.lax(mode), AddrMode::Abs, 4),
        Instruction::new(0xBF, LAX, |cpu, mode| cpu.lax(mode), AddrMode::Aby, 4),

        Instruction::new(0xC3, DCP, |cpu, mode| cpu.dcp(mode), AddrMode::Izx, 8),
        Instruction::new(0xD3, DCP, |cpu, mode| cpu.dcp(mode), AddrMode::IzyW, 8),
        Instruction::new(0xC7, DCP, |cpu, mode| cpu.dcp(mode), AddrMode::Zp0, 5),
        Instruction::new(0xD7, DCP, |cpu, mode| cpu.dcp(mode), AddrMode::Zpx, 6),
        Instruction::new(0xDB, DCP, |cpu, mode| cpu.dcp(mode), AddrMode::AbyW, 7),
        Instruction::new(0xCF, DCP, |cpu, mode| cpu.dcp(mode), AddrMode::Abs, 6),
        Instruction::new(0xDF, DCP, |cpu, mode| cpu.dcp(mode), AddrMode::AbxW, 7),

        Instruction::new(0xE3, ISB, |cpu, mode| cpu.isb(mode), AddrMode::Izx, 8),
        Instruction::new(0xF3, ISB, |cpu, mode| cpu.isb(mode), AddrMode::IzyW, 8),
        Instruction::new(0xE7, ISB, |cpu, mode| cpu.isb(mode), AddrMode::Zp0, 5),
        Instruction::new(0xF7, ISB, |cpu, mode| cpu.isb(mode), AddrMode::Zpx, 6),
        Instruction::new(0xFB, ISB, |cpu, mode| cpu.isb(mode), AddrMode::AbyW, 7),
        Instruction::new(0xEF, ISB, |cpu, mode| cpu.isb(mode), AddrMode::Abs, 6),
        Instruction::new(0xFF, ISB, |cpu, mode| cpu.isb(mode), AddrMode::AbxW, 7),

        Instruction::new(0x0B, ANC, |cpu, mode| cpu.anc(mode), AddrMode::Imm, 2),
        Instruction::new(0x2B, ANC, |cpu, mode| cpu.anc(mode), AddrMode::Imm, 2),

        Instruction::new(0x4B, ALR, |cpu, mode| cpu.alr(mode), AddrMode::Imm, 2),

        Instruction::new(0x6B, ARR, |cpu, mode| cpu.arr(mode), AddrMode::Imm, 2),

        Instruction::new(0x8B, XXA, |cpu, mode| cpu.xxa(mode), AddrMode::Imm, 2),

        Instruction::new(0x9B, TAS, |cpu, mode| cpu.tas(mode), AddrMode::AbyW, 5),

        Instruction::new(0x9C, SHY, |cpu, mode| cpu.shy(mode), AddrMode::AbxW, 5),

        Instruction::new(0xBB, LAS, |cpu, mode| cpu.las(mode), AddrMode::Aby, 4),

        Instruction::new(0xCB, AXS, |cpu, mode| cpu.axs(mode), AddrMode::Imm, 2),

        Instruction::new(0xEB, SBC_U, |cpu, mode| cpu.sbc(mode), AddrMode::Imm, 2),

        Instruction::new(0x9E, SHX, |cpu, mode| cpu.shx(mode), AddrMode::AbyW, 5),
    ];

    /// HashMap of the instructions
    pub static ref OPTABLE: HashMap<u8, &'static Instruction> = {
        let mut map = HashMap::<u8, &'static Instruction>::new();

        for i in &*INSTRUCTIONS {
            map.insert(i.opcode, i);
        }

        assert_eq!(map.len(), 256);

        map
    };
}

static ORA: &str = "ORA";
static AND: &str = "AND";
static EOR: &str = "EOR";
static ADC: &str = "ADC";
static SBC: &str = "SBC";
static CMP: &str = "CMP";
static CPX: &str = "CPX";
static CPY: &str = "CPY";
static DEC: &str = "DEC";
static DEX: &str = "DEX";
static DEY: &str = "DEY";
static INC: &str = "INC";
static INX: &str = "INX";
static INY: &str = "INY";
static ASL: &str = "ASL";
static ROL: &str = "ROL";
static LSR: &str = "LSR";
static ROR: &str = "ROR";
static LDA: &str = "LDA";
static STA: &str = "STA";
static LDX: &str = "LDX";
static STX: &str = "STX";
static LDY: &str = "LDY";
static STY: &str = "STY";
static TAX: &str = "TAX";
static TXA: &str = "TXA";
static TAY: &str = "TAY";
static TYA: &str = "TYA";
static TSX: &str = "TSX";
static TXS: &str = "TXS";
static PLA: &str = "PLA";
static PHA: &str = "PHA";
static PLP: &str = "PLP";
static PHP: &str = "PHP";
static BPL: &str = "BPL";
static BMI: &str = "BMI";
static BVC: &str = "BVC";
static BVS: &str = "BVS";
static BCC: &str = "BCC";
static BCS: &str = "BCS";
static BNE: &str = "BNE";
static BEQ: &str = "BEQ";
static BRK: &str = "BRK";
static RTI: &str = "RTI";
static JSR: &str = "JSR";
static RTS: &str = "RTS";
static JMP: &str = "JMP";
static BIT: &str = "BIT";
static CLC: &str = "CLC";
static SEC: &str = "SEC";
static CLD: &str = "CLD";
static SED: &str = "SED";
static CLI: &str = "CLI";
static SEI: &str = "SEI";
static CLV: &str = "CLV";
static NOP: &str = "NOP";
static NOP_U: &str = "*NOP";
static SLO: &str = "*SLO";
static RLA: &str = "*RLA";
static SRE: &str = "*SRE";
static RRA: &str = "*RRA";
static SAX: &str = "*SAX";
static AHX: &str = "*AHX";
static LAX: &str = "*LAX";
static DCP: &str = "*DCP";
static ISB: &str = "*ISB";
static ANC: &str = "*ANC";
static ALR: &str = "*ALR";
static ARR: &str = "*ARR";
static XXA: &str = "*XXA";
static TAS: &str = "*TAS";
static LAS: &str = "*LAS";
static AXS: &str = "*AXS";
static SHY: &str = "*SHY";
static SHX: &str = "*SHX";
static SBC_U: &str = "*SBC";
static KIL: &str = "KIL";
