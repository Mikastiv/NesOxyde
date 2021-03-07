use std::collections::HashMap;

use lazy_static::lazy_static;

use super::{AddrMode, Cpu};

#[derive(Clone, Copy)]
pub struct Instruction {
    pub opcode: u8,
    pub mnemonic: &'static str,
    pub cpu_fn: fn(&mut Cpu, AddrMode),
    pub mode: AddrMode,
    pub cycles: u32,
}

impl Instruction {
    pub fn new(opcode: u8, mnemonic: &'static str, cpu_fn: fn(&mut Cpu, AddrMode), mode: AddrMode, cycles: u32) -> Self {
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
    pub static ref INSTRUCTIONS: Vec<Instruction> = vec![
        Instruction::new(0xA9, LDA, |cpu, mode| cpu.lda(mode), AddrMode::IMM, 2),
        Instruction::new(0xA5, LDA, |cpu, mode| cpu.lda(mode), AddrMode::ZP0, 3),
        Instruction::new(0xB5, LDA, |cpu, mode| cpu.lda(mode), AddrMode::ZPX, 4),
        Instruction::new(0xAD, LDA, |cpu, mode| cpu.lda(mode), AddrMode::ABS, 4),
        Instruction::new(0xBD, LDA, |cpu, mode| cpu.lda(mode), AddrMode::ABX, 4),
        Instruction::new(0xB9, LDA, |cpu, mode| cpu.lda(mode), AddrMode::ABY, 4),
        Instruction::new(0xA1, LDA, |cpu, mode| cpu.lda(mode), AddrMode::IZX, 6),
        Instruction::new(0xB1, LDA, |cpu, mode| cpu.lda(mode), AddrMode::IZY, 5),

        Instruction::new(0xA2, LDX, |cpu, mode| cpu.ldx(mode), AddrMode::IMM, 2),
        Instruction::new(0xA6, LDX, |cpu, mode| cpu.ldx(mode), AddrMode::ZP0, 3),
        Instruction::new(0xB6, LDX, |cpu, mode| cpu.ldx(mode), AddrMode::ZPY, 4),
        Instruction::new(0xAE, LDX, |cpu, mode| cpu.ldx(mode), AddrMode::ABS, 4),
        Instruction::new(0xBE, LDX, |cpu, mode| cpu.ldx(mode), AddrMode::ABY, 4),

        Instruction::new(0xA0, LDY, |cpu, mode| cpu.ldy(mode), AddrMode::IMM, 2),
        Instruction::new(0xA4, LDY, |cpu, mode| cpu.ldy(mode), AddrMode::ZP0, 3),
        Instruction::new(0xB4, LDY, |cpu, mode| cpu.ldy(mode), AddrMode::ZPY, 4),
        Instruction::new(0xAC, LDY, |cpu, mode| cpu.ldy(mode), AddrMode::ABS, 4),
        Instruction::new(0xBC, LDY, |cpu, mode| cpu.ldy(mode), AddrMode::ABY, 4),

        Instruction::new(0x85, STA, |cpu, mode| cpu.sta(mode), AddrMode::ZP0, 3),
        Instruction::new(0x95, STA, |cpu, mode| cpu.sta(mode), AddrMode::ZPX, 4),
        Instruction::new(0x8D, STA, |cpu, mode| cpu.sta(mode), AddrMode::ABS, 4),
        Instruction::new(0x9D, STA, |cpu, mode| cpu.sta(mode), AddrMode::ABXW, 5),
        Instruction::new(0x99, STA, |cpu, mode| cpu.sta(mode), AddrMode::ABYW, 5),
        Instruction::new(0x81, STA, |cpu, mode| cpu.sta(mode), AddrMode::IZX, 6),
        Instruction::new(0x91, STA, |cpu, mode| cpu.sta(mode), AddrMode::IZYW, 6),

        Instruction::new(0x86, STX, |cpu, mode| cpu.stx(mode), AddrMode::ZP0, 3),
        Instruction::new(0x96, STX, |cpu, mode| cpu.stx(mode), AddrMode::ZPY, 4),
        Instruction::new(0x8E, STX, |cpu, mode| cpu.stx(mode), AddrMode::ABS, 4),

        Instruction::new(0x84, STY, |cpu, mode| cpu.sty(mode), AddrMode::ZP0, 3),
        Instruction::new(0x94, STY, |cpu, mode| cpu.sty(mode), AddrMode::ZPY, 4),
        Instruction::new(0x8C, STY, |cpu, mode| cpu.sty(mode), AddrMode::ABS, 4),

        Instruction::new(0xAA, TAX, |cpu, mode| cpu.tax(mode), AddrMode::IMP, 2),
        Instruction::new(0xA8, TAY, |cpu, mode| cpu.tay(mode), AddrMode::IMP, 2),
        Instruction::new(0xBA, TSX, |cpu, mode| cpu.tsx(mode), AddrMode::IMP, 2),
        Instruction::new(0x8A, TXA, |cpu, mode| cpu.txa(mode), AddrMode::IMP, 2),
        Instruction::new(0x9A, TXS, |cpu, mode| cpu.txs(mode), AddrMode::IMP, 2),
        Instruction::new(0x98, TYA, |cpu, mode| cpu.tya(mode), AddrMode::IMP, 2),
        
        Instruction::new(0x18, CLC, |cpu, mode| cpu.clc(mode), AddrMode::IMP, 2),
        Instruction::new(0xD8, CLD, |cpu, mode| cpu.cld(mode), AddrMode::IMP, 2),
        Instruction::new(0x58, CLI, |cpu, mode| cpu.cli(mode), AddrMode::IMP, 2),
        Instruction::new(0xB8, CLV, |cpu, mode| cpu.clv(mode), AddrMode::IMP, 2),
        Instruction::new(0x38, SEC, |cpu, mode| cpu.sec(mode), AddrMode::IMP, 2),
        Instruction::new(0xF8, SED, |cpu, mode| cpu.sed(mode), AddrMode::IMP, 2),
        Instruction::new(0x78, SEI, |cpu, mode| cpu.sei(mode), AddrMode::IMP, 2),

        Instruction::new(0xE6, INC, |cpu, mode| cpu.inc(mode), AddrMode::ZP0, 5),
        Instruction::new(0xF6, INC, |cpu, mode| cpu.inc(mode), AddrMode::ZPX, 6),
        Instruction::new(0xEE, INC, |cpu, mode| cpu.inc(mode), AddrMode::ABS, 6),
        Instruction::new(0xFE, INC, |cpu, mode| cpu.inc(mode), AddrMode::ABXW, 7),

        Instruction::new(0xE8, INX, |cpu, mode| cpu.inx(mode), AddrMode::IMP, 2),
        Instruction::new(0xC8, INY, |cpu, mode| cpu.iny(mode), AddrMode::IMP, 2),

        Instruction::new(0xC6, DEC, |cpu, mode| cpu.dec(mode), AddrMode::ZP0, 5),
        Instruction::new(0xD6, DEC, |cpu, mode| cpu.dec(mode), AddrMode::ZPX, 6),
        Instruction::new(0xCE, DEC, |cpu, mode| cpu.dec(mode), AddrMode::ABS, 6),
        Instruction::new(0xDE, DEC, |cpu, mode| cpu.dec(mode), AddrMode::ABXW, 7),

        Instruction::new(0xCA, DEX, |cpu, mode| cpu.dex(mode), AddrMode::IMP, 2),
        Instruction::new(0x88, DEY, |cpu, mode| cpu.dey(mode), AddrMode::IMP, 2),
        
        Instruction::new(0xC9, CMP, |cpu, mode| cpu.cpa(mode), AddrMode::IMM, 2),
        Instruction::new(0xC5, CMP, |cpu, mode| cpu.cpa(mode), AddrMode::ZP0, 3),
        Instruction::new(0xD5, CMP, |cpu, mode| cpu.cpa(mode), AddrMode::ZPX, 4),
        Instruction::new(0xCD, CMP, |cpu, mode| cpu.cpa(mode), AddrMode::ABS, 4),
        Instruction::new(0xDD, CMP, |cpu, mode| cpu.cpa(mode), AddrMode::ABX, 4),
        Instruction::new(0xD9, CMP, |cpu, mode| cpu.cpa(mode), AddrMode::ABY, 4),
        Instruction::new(0xC1, CMP, |cpu, mode| cpu.cpa(mode), AddrMode::IZX, 6),
        Instruction::new(0xD1, CMP, |cpu, mode| cpu.cpa(mode), AddrMode::IZY, 5),
        
        Instruction::new(0xE0, CPX, |cpu, mode| cpu.cpx(mode), AddrMode::IMM, 2),
        Instruction::new(0xE4, CPX, |cpu, mode| cpu.cpx(mode), AddrMode::ZP0, 3),
        Instruction::new(0xEC, CPX, |cpu, mode| cpu.cpx(mode), AddrMode::ABS, 4),

        Instruction::new(0xC0, CPY, |cpu, mode| cpu.cpy(mode), AddrMode::IMM, 2),
        Instruction::new(0xC4, CPY, |cpu, mode| cpu.cpy(mode), AddrMode::ZP0, 3),
        Instruction::new(0xCC, CPY, |cpu, mode| cpu.cpy(mode), AddrMode::ABS, 4),

        Instruction::new(0x90, BCC, |cpu, mode| cpu.bcc(mode), AddrMode::REL, 2),
        Instruction::new(0xB0, BCS, |cpu, mode| cpu.bcs(mode), AddrMode::REL, 2),
        Instruction::new(0xF0, BEQ, |cpu, mode| cpu.beq(mode), AddrMode::REL, 2),
        Instruction::new(0xD0, BNE, |cpu, mode| cpu.bne(mode), AddrMode::REL, 2),
        Instruction::new(0x30, BMI, |cpu, mode| cpu.bmi(mode), AddrMode::REL, 2),
        Instruction::new(0x10, BPL, |cpu, mode| cpu.bpl(mode), AddrMode::REL, 2),
        Instruction::new(0x50, BVC, |cpu, mode| cpu.bvc(mode), AddrMode::REL, 2),
        Instruction::new(0x70, BVS, |cpu, mode| cpu.bvs(mode), AddrMode::REL, 2),

        Instruction::new(0x4C, JMP, |cpu, mode| cpu.jmp_abs(mode), AddrMode::ABS, 3),
        Instruction::new(0x6C, JMP, |cpu, mode| cpu.jmp_ind(mode), AddrMode::IND, 5),

        Instruction::new(0x48, PHA, |cpu, mode| cpu.pha(mode), AddrMode::IMP, 3),
        Instruction::new(0x08, PHP, |cpu, mode| cpu.php(mode), AddrMode::IMP, 3),
        Instruction::new(0x68, PLA, |cpu, mode| cpu.pla(mode), AddrMode::IMP, 4),
        Instruction::new(0x28, PLP, |cpu, mode| cpu.plp(mode), AddrMode::IMP, 4),

        Instruction::new(0x20, JSR, |cpu, mode| cpu.jsr(mode), AddrMode::ABS, 6),
        Instruction::new(0x60, RTS, |cpu, mode| cpu.rts(mode), AddrMode::IMP, 6),
        Instruction::new(0x40, RTI, |cpu, mode| cpu.rti(mode), AddrMode::IMP, 6),

        Instruction::new(0xEA, NOP, |cpu, mode| cpu.nop(mode), AddrMode::IMP, 2),

        Instruction::new(0x24, BIT, |cpu, mode| cpu.bit(mode), AddrMode::ZP0, 3),
        Instruction::new(0x2C, BIT, |cpu, mode| cpu.bit(mode), AddrMode::ABS, 4),

        Instruction::new(0x29, AND, |cpu, mode| cpu.and(mode), AddrMode::IMM, 2),
        Instruction::new(0x25, AND, |cpu, mode| cpu.and(mode), AddrMode::ZP0, 3),
        Instruction::new(0x35, AND, |cpu, mode| cpu.and(mode), AddrMode::ZPX, 4),
        Instruction::new(0x2D, AND, |cpu, mode| cpu.and(mode), AddrMode::ABS, 4),
        Instruction::new(0x3D, AND, |cpu, mode| cpu.and(mode), AddrMode::ABX, 4),
        Instruction::new(0x39, AND, |cpu, mode| cpu.and(mode), AddrMode::ABY, 4),
        Instruction::new(0x21, AND, |cpu, mode| cpu.and(mode), AddrMode::IZX, 6),
        Instruction::new(0x31, AND, |cpu, mode| cpu.and(mode), AddrMode::IZY, 5),

        Instruction::new(0x49, EOR, |cpu, mode| cpu.eor(mode), AddrMode::IMM, 2),
        Instruction::new(0x45, EOR, |cpu, mode| cpu.eor(mode), AddrMode::ZP0, 3),
        Instruction::new(0x55, EOR, |cpu, mode| cpu.eor(mode), AddrMode::ZPX, 4),
        Instruction::new(0x4D, EOR, |cpu, mode| cpu.eor(mode), AddrMode::ABS, 4),
        Instruction::new(0x5D, EOR, |cpu, mode| cpu.eor(mode), AddrMode::ABX, 4),
        Instruction::new(0x59, EOR, |cpu, mode| cpu.eor(mode), AddrMode::ABY, 4),
        Instruction::new(0x41, EOR, |cpu, mode| cpu.eor(mode), AddrMode::IZX, 6),
        Instruction::new(0x51, EOR, |cpu, mode| cpu.eor(mode), AddrMode::IZY, 5),

        Instruction::new(0x09, ORA, |cpu, mode| cpu.ora(mode), AddrMode::IMM, 2),
        Instruction::new(0x05, ORA, |cpu, mode| cpu.ora(mode), AddrMode::ZP0, 3),
        Instruction::new(0x15, ORA, |cpu, mode| cpu.ora(mode), AddrMode::ZPX, 4),
        Instruction::new(0x0D, ORA, |cpu, mode| cpu.ora(mode), AddrMode::ABS, 4),
        Instruction::new(0x1D, ORA, |cpu, mode| cpu.ora(mode), AddrMode::ABX, 4),
        Instruction::new(0x19, ORA, |cpu, mode| cpu.ora(mode), AddrMode::ABY, 4),
        Instruction::new(0x01, ORA, |cpu, mode| cpu.ora(mode), AddrMode::IZX, 6),
        Instruction::new(0x11, ORA, |cpu, mode| cpu.ora(mode), AddrMode::IZY, 5),

        Instruction::new(0x0A, ASL, |cpu, mode| cpu.asl_acc(mode), AddrMode::IMP, 2),
        Instruction::new(0x06, ASL, |cpu, mode| cpu.asl_mem(mode), AddrMode::ZP0, 5),
        Instruction::new(0x16, ASL, |cpu, mode| cpu.asl_mem(mode), AddrMode::ZPX, 6),
        Instruction::new(0x0E, ASL, |cpu, mode| cpu.asl_mem(mode), AddrMode::ABS, 6),
        Instruction::new(0x1E, ASL, |cpu, mode| cpu.asl_mem(mode), AddrMode::ABXW, 7),

        Instruction::new(0x4A, LSR, |cpu, mode| cpu.lsr_acc(mode), AddrMode::IMP, 2),
        Instruction::new(0x46, LSR, |cpu, mode| cpu.lsr_mem(mode), AddrMode::ZP0, 5),
        Instruction::new(0x56, LSR, |cpu, mode| cpu.lsr_mem(mode), AddrMode::ZPX, 6),
        Instruction::new(0x4E, LSR, |cpu, mode| cpu.lsr_mem(mode), AddrMode::ABS, 6),
        Instruction::new(0x5E, LSR, |cpu, mode| cpu.lsr_mem(mode), AddrMode::ABXW, 7),

    ];

    pub static ref OPTABLE: HashMap<u8, &'static Instruction> = {
        let mut map = HashMap::<u8, &'static Instruction>::new();

        for i in &*INSTRUCTIONS {
            map.insert(i.opcode, i);
        }

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
static XXA: &str = "*XXA";
static TAS: &str = "*TAS";
static LAS: &str = "*LAS";
static AXS: &str = "*AXS";
static SHY: &str = "*SHY";
static SHX: &str = "*SHX";
static SBC_U: &str = "*SBC";
static KIL: &str = "KIL";
