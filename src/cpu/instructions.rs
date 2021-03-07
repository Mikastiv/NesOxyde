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

        Instruction::new(0x85, STA, |cpu, mode| cpu.sta(mode), AddrMode::ZP0, 3),
        Instruction::new(0x95, STA, |cpu, mode| cpu.sta(mode), AddrMode::ZPX, 4),
        Instruction::new(0x8D, STA, |cpu, mode| cpu.sta(mode), AddrMode::ABS, 4),
        Instruction::new(0x9D, STA, |cpu, mode| cpu.sta(mode), AddrMode::ABXW, 5),
        Instruction::new(0x99, STA, |cpu, mode| cpu.sta(mode), AddrMode::ABYW, 5),
        Instruction::new(0x81, STA, |cpu, mode| cpu.sta(mode), AddrMode::IZX, 6),
        Instruction::new(0x91, STA, |cpu, mode| cpu.sta(mode), AddrMode::IZYW, 6),

        Instruction::new(0xAA, TAX, |cpu, mode| cpu.tax(mode), AddrMode::IMP, 2),
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
