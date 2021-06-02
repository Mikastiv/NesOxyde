use std::fs::File;

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

use crate::joypad::{Button, JoyPort};
use crate::savable::Savable;

pub use addr_modes::AddrMode;
pub use instructions::OPTABLE;

mod addr_modes;
mod instructions;

/// Memory page of the cpu stack
const STACK_PAGE: u16 = 0x0100;
/// Reset value of the stack pointer
const STACK_RESET: u8 = 0xFD;
/// Reset value of the status register
const STATUS_RESET: u8 = Flags::U.bits() | Flags::I.bits();
/// Non-maskable interrupt vector
const NMI_VECTOR: u16 = 0xFFFA;
/// Reset vector
const RESET_VECTOR: u16 = 0xFFFC;
/// Interrupt request vector
const IRQ_VECTOR: u16 = 0xFFFE;

pub trait CpuInterface: Interface + Savable {}

/// Cpu's interface to the rest of the components
pub trait Interface {
    /// Reads a byte from `addr`
    fn read(&mut self, addr: u16) -> u8;

    /// Writes a byte to `addr`
    fn write(&mut self, addr: u16, data: u8);

    /// Polls the state of NMI flag of Ppu
    ///
    /// `true`: Ppu is requesting NMI. `false`: Ppu is not requesting NMI
    fn poll_nmi(&mut self) -> bool {
        false
    }

    /// Polls the state of IRQ flag of Apu
    ///
    /// `true`: Apu is requesting IRQ. `false`: Apu is not requesting IRQ
    fn poll_irq(&mut self) -> bool {
        false
    }

    /// Performs one clock tick on the bus
    fn tick(&mut self, _cycles: u64) {}

    /// Updates a controller's state
    ///
    /// Used with SDL2 keyboard events
    fn update_joypad(&mut self, _button: Button, _pressed: bool, _port: JoyPort) {}

    /// Returns the number of frame rendered by the Ppu
    fn frame_count(&self) -> u128 {
        0
    }

    /// Resets the bus and its components
    fn reset(&mut self) {}

    /// Gets audio samples from the Apu
    fn samples(&mut self) -> Vec<f32> {
        vec![]
    }

    /// Returns how many samples are ready to be played
    fn sample_count(&self) -> usize {
        0
    }
}

bitflags! {
    /// Cpu Flags
    #[derive(Serialize, Deserialize)]
    struct Flags: u8 {
        /// Negative
        const N = 0b10000000;
        /// Overflow
        const V = 0b01000000;
        /// Unused
        const U = 0b00100000;
        /// Break flag
        const B = 0b00010000;
        /// Decimal flag (disabled on the NES)
        const D = 0b00001000;
        /// Disable interrupt
        const I = 0b00000100;
        /// Zero
        const Z = 0b00000010;
        /// Carry
        const C = 0b00000001;
    }
}

/// 2A03 Cpu
pub struct Cpu<'a> {
    /// Accumulator
    a: u8,
    /// Index X
    x: u8,
    /// Index Y
    y: u8,
    /// Stack pointer
    s: u8,
    /// Status register
    p: Flags,
    /// Program counter
    pc: u16,

    /// Memory bus
    bus: Box<dyn CpuInterface + 'a>,
    /// Current instruction duration in cycles
    ins_cycles: u64,
    /// Cycles elapsed
    cycles: u64,
}

impl Savable for Cpu<'_> {
    fn save(&self, output: &File) -> bincode::Result<()> {
        self.bus.save(output)?;
        bincode::serialize_into(output, &self.a)?;
        bincode::serialize_into(output, &self.x)?;
        bincode::serialize_into(output, &self.y)?;
        bincode::serialize_into(output, &self.s)?;
        bincode::serialize_into(output, &self.p)?;
        bincode::serialize_into(output, &self.pc)?;
        bincode::serialize_into(output, &self.ins_cycles)?;
        bincode::serialize_into(output, &self.cycles)?;
        Ok(())
    }

    fn load(&mut self, input: &File) -> bincode::Result<()> {
        self.bus.load(input)?;
        self.a = bincode::deserialize_from(input)?;
        self.x = bincode::deserialize_from(input)?;
        self.y = bincode::deserialize_from(input)?;
        self.s = bincode::deserialize_from(input)?;
        self.p = bincode::deserialize_from(input)?;
        self.pc = bincode::deserialize_from(input)?;
        self.ins_cycles = bincode::deserialize_from(input)?;
        self.cycles = bincode::deserialize_from(input)?;
        Ok(())
    }
}

impl<'a> Cpu<'a> {
    pub fn new<I>(bus: I) -> Self
    where
        I: CpuInterface + 'a,
    {
        Self {
            a: 0,
            x: 0,
            y: 0,
            s: STACK_RESET,
            p: Flags::from_bits_truncate(STATUS_RESET),
            pc: 0,

            bus: Box::new(bus),
            ins_cycles: 0,
            cycles: 0,
        }
    }

    pub fn pc(&self) -> u16 {
        self.pc
    }

    pub fn a(&self) -> u8 {
        self.a
    }

    pub fn x(&self) -> u8 {
        self.x
    }

    pub fn y(&self) -> u8 {
        self.y
    }

    pub fn s(&self) -> u8 {
        self.s
    }

    pub fn p(&self) -> u8 {
        self.p.bits()
    }

    /// Cpu cycles passed
    pub fn cycles(&self) -> u64 {
        self.cycles
    }

    /// Ppu frames rendered
    pub fn frame_count(&self) -> u128 {
        self.bus.frame_count()
    }

    /// Resets the NES
    pub fn reset(&mut self) {
        self.bus.reset();
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.s = STACK_RESET;
        self.p = Flags::from_bits_truncate(STATUS_RESET);
        // Set pc to value at reset vector
        self.pc = self.mem_read_word(RESET_VECTOR);
        self.ins_cycles = 0;
        // Reset takes 7 cycles
        self.bus.tick(7);
        self.cycles = 7;
    }

    /// Gets audio samples from the Apu
    pub fn samples(&mut self) -> Vec<f32> {
        self.bus.samples()
    }

    /// Returns how many samples are ready to be played
    pub fn sample_count(&self) -> usize {
        self.bus.sample_count()
    }

    /// Non-maskable interrupt
    fn nmi(&mut self) {
        // Push the program counter
        self.push_word(self.pc);
        // Push the status register without the Break flag
        self.push_byte((self.p & !Flags::B).bits());
        // Set disable interrupt
        self.p.insert(Flags::I);
        // Set pc to value at NMI vector
        self.pc = self.mem_read_word(NMI_VECTOR);
        // NMI takes 7 cycles
        self.cycles += 7;
        self.ins_cycles = 7;
    }

    /// Interrupt request
    fn irq(&mut self) {
        // Don't execute if disable interrupt is set
        if !self.p.contains(Flags::I) {
            // Push the program counter
            self.push_word(self.pc);
            // Push the status register without the Break flag
            self.push_byte((self.p & !Flags::B).bits());
            // Set disable interrupt
            self.p.insert(Flags::I);
            // Set pc to value at IRQ vector
            self.pc = self.mem_read_word(IRQ_VECTOR);
            // IRQ takes 7 cycles
            self.cycles += 7;
            self.ins_cycles = 7;
        } else {
            self.ins_cycles = 0;
        }
    }

    /// Executes an instruction and runs a callback function in a loop
    ///
    /// Used with the trace debug module
    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut Self),
    {
        loop {
            callback(self);
            self.execute();
        }
    }

    /// Executes a full instruction
    ///
    /// Returns how many cycles were executed
    pub fn execute(&mut self) -> u64 {
        let mut nmi_cycles = 0;
        // If Ppu has requested a NMI, do it
        if self.bus.poll_nmi() {
            self.nmi();
            // Clock the bus for the NMI cycles duration (7)
            self.bus.tick(self.ins_cycles);
            nmi_cycles = self.ins_cycles;
        }

        // Get next instruction opcode
        let opcode = self.read_byte();

        // Get the instruction from the instruction table
        let ins = *OPTABLE.get(&opcode).unwrap();
        // Set the current instruction cycle duration
        self.ins_cycles = ins.cycles;
        // Call the instruction function
        (ins.cpu_fn)(self, ins.mode);

        // Clock the bus for the instruction's cycles duration
        self.bus.tick(self.ins_cycles);

        let mut irq_cycles = 0;
        // If Apu has requested an interrupt, do it
        if self.bus.poll_irq() {
            self.irq();
            self.bus.tick(self.ins_cycles);
            irq_cycles = self.ins_cycles;
        }

        // Count cycles
        self.cycles = self
            .cycles
            .wrapping_add(nmi_cycles + irq_cycles + self.ins_cycles);

        nmi_cycles + irq_cycles + self.ins_cycles
    }

    /// Clocks the Cpu once
    ///
    /// This function is not cycle accurate. I execute the instruction in one cycle and then do nothing for the remaining cycles
    pub fn clock(&mut self) {
        // If current instruction is done and a NMI is requested, do it
        if self.ins_cycles == 0 && self.bus.poll_nmi() {
            self.nmi();
        }

        // If current instruction is done and a IRQ is requested, do it
        if self.ins_cycles == 0 && self.bus.poll_irq() {
            self.irq();
        }

        // If current instruction is done, do the next one
        if self.ins_cycles == 0 {
            // Read opcode
            let opcode = self.read_byte();

            // Get the instruction from the instruction table
            let ins = *OPTABLE.get(&opcode).unwrap();

            self.ins_cycles = ins.cycles;
            (ins.cpu_fn)(self, ins.mode);
        }

        // Tick once
        self.bus.tick(1);
        // Count cycles
        self.cycles = self.cycles.wrapping_add(1);
        // Once instruction cycle has passed
        self.ins_cycles -= 1;
    }

    /// Updates a controller's state
    ///
    /// Used with SDL2 keyboard events
    pub fn update_joypad(&mut self, button: Button, pressed: bool, port: JoyPort) {
        self.bus.update_joypad(button, pressed, port);
    }

    /// Reads a byte at addr
    pub fn mem_read(&mut self, addr: u16) -> u8 {
        self.bus.read(addr)
    }

    /// Reads a word (2 bytes) at addr
    ///
    /// This Cpu is little endian
    pub fn mem_read_word(&mut self, addr: u16) -> u16 {
        // Read low byte first
        let lo = self.mem_read(addr);
        // Then high byte
        let hi = self.mem_read(addr.wrapping_add(1));
        // Combine them
        u16::from_le_bytes([lo, hi])
    }

    /// Writes a byte to addr
    pub fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.write(addr, data);
    }

    /// Reads a byte at program counter, then increments it
    fn read_byte(&mut self) -> u8 {
        let b = self.mem_read(self.pc);
        self.increment_pc();
        b
    }

    /// Reads a word (2 bytes) at program counter, then increments it
    fn read_word(&mut self) -> u16 {
        // Read low byte first
        let lo = self.read_byte();
        // Then high byte
        let hi = self.read_byte();
        // Combine them
        u16::from_le_bytes([lo, hi])
    }

    /// Pushes a byte on the stack
    fn push_byte(&mut self, data: u8) {
        // Write byte at stack pointer location
        self.mem_write(STACK_PAGE + self.s() as u16, data);
        // Decrement stack pointer
        self.s = self.s().wrapping_sub(1);
    }

    /// Pushes a word (2 bytes) on the stack
    fn push_word(&mut self, data: u16) {
        // Get high byte from data
        let hi = (data >> 8) as u8;
        // Get low byte from data
        let lo = data as u8;
        // Push the high byte first because it will be popped off in reverse order (low, then high)
        self.push_byte(hi);
        self.push_byte(lo);
    }

    /// Pops a byte off the stack
    fn pop_byte(&mut self) -> u8 {
        // Increment stack pointer
        self.s = self.s().wrapping_add(1);
        // Read byte at stack pointer location
        self.mem_read(STACK_PAGE + self.s() as u16)
    }

    /// Pops a word (2 bytes) off the stack
    fn pop_word(&mut self) -> u16 {
        // Little endian, so pop low
        let lo = self.pop_byte();
        // Then high
        let hi = self.pop_byte();
        // Combine them
        u16::from_le_bytes([lo, hi])
    }

    /// Returns the address of the operand based on the addressing mode
    fn operand_addr(&mut self, mode: AddrMode) -> u16 {
        match mode {
            // None or implied don't have operand
            AddrMode::None | AddrMode::Imp => panic!("Not supported"),
            // Immediate or relative: the operand is the byte right after the opcode
            AddrMode::Imm | AddrMode::Rel => self.pc,
            // Zero page: the byte after the opcode is the operand address in page 0x00
            AddrMode::Zp0 => self.read_byte() as u16,
            // Zero page with X: the byte after the opcode plus the value in register X is the operand address in page 0x00
            AddrMode::Zpx => {
                let base = self.read_byte();
                base.wrapping_add(self.x()) as u16
            }
            // Zero page with Y: the byte after the opcode plus the value in register Y is the operand address in page 0x00
            AddrMode::Zpy => {
                let base = self.read_byte();
                base.wrapping_add(self.y()) as u16
            }
            // Absolute: the two bytes right after the opcode makes the operand address
            // Indirect: I cheat a bit here, the actual address is the operand. I use it as the operand later. Indirect is only used with JMP
            AddrMode::Abs | AddrMode::Ind => self.read_word(),
            // Absolute with X: the two bytes right after the opcode plus the value in register X makes the operand address
            AddrMode::Abx => {
                let base = self.read_word();
                let addr = base.wrapping_add(self.x() as u16);

                // If a page is crossed (e.g. when the first byte is at 0x04FF and the second at 0x0500) it takes an extra cycle
                if Self::page_crossed(base, addr) {
                    self.ins_cycles += 1;
                }

                addr
            }
            // Absolute with X for write instructions: the two bytes right after the opcode plus the value in register X makes the operand address
            AddrMode::AbxW => {
                let base = self.read_word();
                base.wrapping_add(self.x() as u16)
            }
            // Absolute with Y: the two bytes right after the opcode plus the value in register Y makes the operand address
            AddrMode::Aby => {
                let base = self.read_word();
                let addr = base.wrapping_add(self.y() as u16);

                // If a page is crossed (e.g. when the first byte is at 0x04FF and the second at 0x0500) it takes an extra cycle
                if Self::page_crossed(base, addr) {
                    self.ins_cycles += 1;
                }

                addr
            }
            // Absolute with Y for write instructions: the two bytes right after the opcode plus the value in register Y makes the operand address
            AddrMode::AbyW => {
                let base = self.read_word();
                base.wrapping_add(self.y() as u16)
            }
            // Indirect with X: the two bytes right after the opcode plus the value in register X make a pointer in page 0x00. The value at this
            // location is the address of the operand
            AddrMode::Izx => {
                // Construct pointer
                let base = self.read_byte();
                let ptr = base.wrapping_add(self.x());
                // Read values
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                u16::from_le_bytes([lo, hi])
            }
            // Indirect with Y: the two bytes right after the opcode make a pointer in page 0x00. The value at this
            // location plus the value in register Y is the address of the operand. Note that the pointer never
            // leaves page 0x00. 0x00FF wraps to 0x0000
            AddrMode::Izy => {
                // Construct pointer
                let ptr = self.read_byte();
                // Read values
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                // Add value in register Y to the result
                let addr = u16::from_le_bytes([lo, hi]).wrapping_add(self.y() as u16);

                // If a page is crossed (e.g. when the first byte is at 0x04FF and the second at 0x0500) it takes an extra cycle
                if Self::page_crossed(u16::from_le_bytes([lo, hi]), addr) {
                    self.ins_cycles += 1;
                }

                addr
            }
            // Indirect with Y: the two bytes right after the opcode make a pointer in page 0x00. The value at this
            // location plus the value in register Y is the address of the operand. Note that the pointer never
            // leaves page 0x00. 0x00FF wraps to 0x0000
            AddrMode::IzyW => {
                // Construct pointer
                let ptr = self.read_byte();
                // Read values
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                // Add value in register Y to the result
                u16::from_le_bytes([lo, hi]).wrapping_add(self.y() as u16)
            }
        }
    }

    /// Fetches the operand at the address based on the addressing mode
    fn fetch_operand(&mut self, addr: u16, mode: AddrMode) -> u8 {
        match mode {
            // Should not be called with these modes
            AddrMode::None | AddrMode::Imp | AddrMode::Ind => panic!("Not supported"),
            // The operand is next to the opcode for these 2
            AddrMode::Imm | AddrMode::Rel => self.read_byte(),
            // All the others reads at the provided address. Should be the address returned from `operand_addr`
            _ => self.mem_read(addr),
        }
    }

    /// Branched if the condition is met
    fn branch(&mut self, cond: bool) {
        // Get the operand (offset). Can be positive or negative
        let offset = self.read_byte() as i8;

        // If the condition is true, branch
        if cond {
            // Branching take one more cycle
            self.ins_cycles += 1;
            // Calculate new address
            let new_addr = self.pc.wrapping_add(offset as u16);

            // If a page is crossed, add one more cycle
            if Self::page_crossed(self.pc, new_addr) {
                self.ins_cycles += 1;
            }

            // Set program counter to new address
            self.pc = new_addr;
        }
    }

    /// Increments the program counter
    fn increment_pc(&mut self) {
        self.pc = self.pc.wrapping_add(1);
    }

    /// Sets flags Zero and Negative based on value
    fn set_z_n(&mut self, v: u8) {
        // If value equals 0
        self.p.set(Flags::Z, v == 0);
        // If value bit 7 is set
        self.p.set(Flags::N, v & 0x80 != 0);
    }

    /// Sets the accumulator to v
    fn set_a(&mut self, v: u8) {
        self.a = v;
        // Update flags Z and N
        self.set_z_n(v);
    }

    /// Sets the X register to v
    fn set_x(&mut self, v: u8) {
        self.x = v;
        // Update flags Z and N
        self.set_z_n(v);
    }

    /// Sets the X register to v
    fn set_y(&mut self, v: u8) {
        self.y = v;
        // Update flags Z and N
        self.set_z_n(v);
    }

    /// Sets the status register to v
    fn set_p(&mut self, v: u8) {
        // Always set U flag and never the B flag
        self.p.bits = (v | Flags::U.bits()) & !Flags::B.bits();
    }

    /// Returns if a page was crossed or not
    fn page_crossed(old: u16, new: u16) -> bool {
        old & 0xFF00 != new & 0xFF00
    }

    /// Wrap an address so it doesn't change page
    fn wrap(old: u16, new: u16) -> u16 {
        (old & 0xFF00) | (new & 0x00FF)
    }

    // List of all the 2A03 instructions

    /// Load accumulator
    fn lda(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);
        self.set_a(v);
    }

    /// Load X register
    fn ldx(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);
        self.set_x(v);
    }

    /// Load Y register
    fn ldy(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);
        self.set_y(v);
    }

    /// Store accumulator
    fn sta(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        self.mem_write(addr, self.a());
    }

    /// Store X register
    fn stx(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        self.mem_write(addr, self.x());
    }

    /// Store Y register
    fn sty(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        self.mem_write(addr, self.y());
    }

    /// Transfer accumulator to X register
    fn tax(&mut self, _mode: AddrMode) {
        self.set_x(self.a());
    }

    /// Transfer accumulator to Y register
    fn tay(&mut self, _mode: AddrMode) {
        self.set_y(self.a());
    }

    /// Transfer stack pointer to X register
    fn tsx(&mut self, _mode: AddrMode) {
        self.set_x(self.s());
    }

    /// Transfer X register to accumulator
    fn txa(&mut self, _mode: AddrMode) {
        self.set_a(self.x());
    }

    /// Transfer X register to stack pointer
    fn txs(&mut self, _mode: AddrMode) {
        self.s = self.x();
    }

    /// Transfer Y register to accumulator
    fn tya(&mut self, _mode: AddrMode) {
        self.set_a(self.y());
    }

    /// Clear carry
    fn clc(&mut self, _mode: AddrMode) {
        self.p.remove(Flags::C);
    }

    /// Clear decimal
    fn cld(&mut self, _mode: AddrMode) {
        self.p.remove(Flags::D);
    }

    /// Clear interrupt
    fn cli(&mut self, _mode: AddrMode) {
        self.p.remove(Flags::I);
    }

    /// Clear overflow
    fn clv(&mut self, _mode: AddrMode) {
        self.p.remove(Flags::V);
    }

    /// Set carry
    fn sec(&mut self, _mode: AddrMode) {
        self.p.insert(Flags::C);
    }

    /// Set decimal
    fn sed(&mut self, _mode: AddrMode) {
        self.p.insert(Flags::D);
    }

    /// Set interrupt
    fn sei(&mut self, _mode: AddrMode) {
        self.p.insert(Flags::I);
    }

    /// Increment memory
    fn inc(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode).wrapping_add(1);
        self.set_z_n(v);
        self.mem_write(addr, v);
    }

    /// Increment X register
    fn inx(&mut self, _mode: AddrMode) {
        self.set_x(self.x().wrapping_add(1));
    }

    /// Increment Y register
    fn iny(&mut self, _mode: AddrMode) {
        self.set_y(self.y().wrapping_add(1));
    }

    /// Decrement memory
    fn dec(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode).wrapping_sub(1);
        self.set_z_n(v);
        self.mem_write(addr, v);
    }

    /// Decrement X register
    fn dex(&mut self, _mode: AddrMode) {
        self.set_x(self.x().wrapping_sub(1));
    }

    /// Decrement Y register
    fn dey(&mut self, _mode: AddrMode) {
        self.set_y(self.y().wrapping_sub(1));
    }

    /// Compare
    fn cmp(&mut self, v1: u8, v2: u8) {
        let result = v1.wrapping_sub(v2);
        self.p.set(Flags::C, v1 >= v2);
        self.set_z_n(result);
    }

    /// Compare with accumulator
    fn cpa(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);
        self.cmp(self.a(), v);
    }

    /// Compare with X register
    fn cpx(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);
        self.cmp(self.x(), v);
    }

    /// Compare with Y register
    fn cpy(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);
        self.cmp(self.y(), v);
    }

    /// Branch if carry set
    fn bcs(&mut self, _mode: AddrMode) {
        self.branch(self.p.contains(Flags::C));
    }

    /// Branch if carry clear
    fn bcc(&mut self, _mode: AddrMode) {
        self.branch(!self.p.contains(Flags::C));
    }

    /// Branch if equal
    fn beq(&mut self, _mode: AddrMode) {
        self.branch(self.p.contains(Flags::Z));
    }

    /// Branch if not equal
    fn bne(&mut self, _mode: AddrMode) {
        self.branch(!self.p.contains(Flags::Z));
    }

    /// Branch if minus
    fn bmi(&mut self, _mode: AddrMode) {
        self.branch(self.p.contains(Flags::N));
    }

    /// Branch if positive
    fn bpl(&mut self, _mode: AddrMode) {
        self.branch(!self.p.contains(Flags::N));
    }

    /// Branch if overflow set
    fn bvs(&mut self, _mode: AddrMode) {
        self.branch(self.p.contains(Flags::V));
    }

    /// Branch if overflow clear
    fn bvc(&mut self, _mode: AddrMode) {
        self.branch(!self.p.contains(Flags::V));
    }

    /// Jump absolute
    fn jmp_abs(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        self.pc = addr;
    }

    /// Jump indirect
    fn jmp_ind(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let lo = self.mem_read(addr);
        let hi = self.mem_read(Self::wrap(addr, addr.wrapping_add(1)));
        self.pc = u16::from_le_bytes([lo, hi]);
    }

    /// Break
    fn brk(&mut self, _mode: AddrMode) {
        if !self.p.contains(Flags::I) {
            self.increment_pc();
            self.push_word(self.pc);
            self.push_byte((self.p | Flags::B).bits());
            self.p.insert(Flags::I);
            self.pc = self.mem_read_word(IRQ_VECTOR);
        }
    }

    /// Push accumulator
    fn pha(&mut self, _mode: AddrMode) {
        self.push_byte(self.a());
    }

    /// Push status
    fn php(&mut self, _mode: AddrMode) {
        self.push_byte((self.p | Flags::B).bits());
    }

    /// Pull accumulator
    fn pla(&mut self, _mode: AddrMode) {
        let v = self.pop_byte();
        self.set_a(v);
    }

    /// Pull status
    fn plp(&mut self, _mode: AddrMode) {
        let v = self.pop_byte();
        self.set_p(v);
    }

    /// Jump to subroutine
    fn jsr(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        self.push_word(self.pc.wrapping_sub(1));
        self.pc = addr;
    }

    /// Return from subroutine
    fn rts(&mut self, _mode: AddrMode) {
        let addr = self.pop_word();
        self.pc = addr.wrapping_add(1);
    }

    /// Return from interrupt
    fn rti(&mut self, _mode: AddrMode) {
        let v = self.pop_byte();
        let addr = self.pop_word();
        self.set_p(v);
        self.pc = addr;
    }

    /// No-op
    fn nop(&mut self, mode: AddrMode) {
        match mode {
            AddrMode::Imp => {}
            _ => {
                let addr = self.operand_addr(mode);
                self.fetch_operand(addr, mode);
            }
        }
    }

    /// Bit test
    fn bit(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);

        self.p.set(Flags::Z, self.a() & v == 0);
        self.p.set(Flags::V, v & 0x40 != 0);
        self.p.set(Flags::N, v & 0x80 != 0);
    }

    /// Bitwise AND
    fn and(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);
        self.set_a(self.a() & v);
    }

    /// Exclusive OR
    fn eor(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);
        self.set_a(self.a() ^ v);
    }

    /// Bitwise OR
    fn ora(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);
        self.set_a(self.a() | v);
    }

    /// Arithmetic shift left
    fn asl(&mut self, v: u8) -> u8 {
        self.p.set(Flags::C, v & 0x80 != 0);
        let result = v << 1;
        self.set_z_n(result);
        result
    }

    /// Arithmetic shift left on accumulator
    fn asl_acc(&mut self, _mode: AddrMode) {
        let v = self.asl(self.a());
        self.set_a(v);
    }

    /// Arithmetic shift left on memory
    fn asl_mem(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);
        let result = self.asl(v);
        self.mem_write(addr, result);
    }

    /// Logical shift right
    fn lsr(&mut self, v: u8) -> u8 {
        self.p.set(Flags::C, v & 0x01 != 0);
        let result = v >> 1;
        self.set_z_n(result);
        result
    }

    /// Logical shift right on accumulator
    fn lsr_acc(&mut self, _mode: AddrMode) {
        let v = self.lsr(self.a());
        self.set_a(v);
    }

    /// Logical shift right on memory
    fn lsr_mem(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);
        let result = self.lsr(v);
        self.mem_write(addr, result);
    }

    /// Rotate left
    fn rol(&mut self, v: u8) -> u8 {
        let c = self.p.contains(Flags::C) as u8;
        self.p.set(Flags::C, v & 0x80 != 0);

        let result = (v << 1) | c;
        self.set_z_n(result);
        result
    }

    /// Rotate left on accumulator
    fn rol_acc(&mut self, _mode: AddrMode) {
        let v = self.rol(self.a());
        self.set_a(v);
    }

    /// Rotate left on memory
    fn rol_mem(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);
        let result = self.rol(v);
        self.mem_write(addr, result);
    }

    /// Rotate right
    fn ror(&mut self, v: u8) -> u8 {
        let c = self.p.contains(Flags::C) as u8;
        self.p.set(Flags::C, v & 0x01 != 0);

        let result = (c << 7) | (v >> 1);
        self.set_z_n(result);
        result
    }

    /// Rotate right on accumulator
    fn ror_acc(&mut self, _mode: AddrMode) {
        let v = self.ror(self.a());
        self.set_a(v);
    }

    /// Rotate right on memory
    fn ror_mem(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);
        let result = self.ror(v);
        self.mem_write(addr, result);
    }

    /// Performs addition with on accumulator value
    fn add(&mut self, v: u8) {
        let c = self.p.contains(Flags::C);
        let sum = self.a() as u16 + v as u16 + c as u16;
        let result = sum as u8;

        self.p
            .set(Flags::V, (v ^ result) & (result ^ self.a()) & 0x80 != 0);
        self.p.set(Flags::C, sum > 0xFF);
        self.set_a(result);
    }

    /// Performs substraction on accumulator with value
    ///
    /// Substraction is adding with all the bits flipped
    fn sub(&mut self, v: u8) {
        self.add(!v);
    }

    /// Add with carry
    fn adc(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);
        self.add(v);
    }

    /// Sub with carry
    fn sbc(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);
        self.sub(v);
    }

    // ----------- Illegal opcodes -----------

    /// Illegal operation which halts the cpu
    fn kil(&mut self, _mode: AddrMode) {
        panic!("KIL opcode called");
    }

    /// ASL & ORA
    fn slo(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);

        let result = self.asl(v);
        self.set_a(self.a() | result);
        self.mem_write(addr, result);
    }

    /// ROL & AND
    fn rla(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);

        let result = self.rol(v);
        self.set_a(self.a() & result);
        self.mem_write(addr, result);
    }

    /// LSR & EOR
    fn sre(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);

        let result = self.lsr(v);
        self.set_a(self.a() ^ result);
        self.mem_write(addr, result);
    }

    /// ROR & ADC
    fn rra(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);

        let result = self.ror(v);
        self.add(result);
        self.mem_write(addr, result);
    }

    /// STA & STX
    fn sax(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        self.mem_write(addr, self.a() & self.x());
    }

    /// STA & STX & (High byte + 1)
    fn ahx(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let hi = ((addr >> 8) as u8).wrapping_add(1);
        self.mem_write(addr, hi & self.a() & self.x());
    }

    /// LDA & LDX
    fn lax(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);

        self.set_x(v);
        self.set_a(v);
    }

    /// DEC & CMP
    fn dcp(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode).wrapping_sub(1);

        self.cmp(self.a(), v);
        self.mem_write(addr, v);
    }

    /// INC & SBC
    fn isb(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode).wrapping_add(1);

        self.sub(v);
        self.mem_write(addr, v);
    }

    /// AND with Carry flag
    fn anc(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);

        self.set_a(self.a() & v);
        self.p.set(Flags::C, self.p.contains(Flags::N));
    }

    /// AND with Carry flag & LSR
    fn alr(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);

        self.set_a(self.a() & v);
        self.p.set(Flags::C, self.a() & 0x01 != 0);
        self.set_a(self.a() >> 1);
    }

    fn arr(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);

        let c = (self.p.contains(Flags::C) as u8) << 7;
        self.set_a(((self.a() & v) >> 1) | c);
        self.p.set(Flags::C, self.a() & 0x40 != 0);

        let c = self.p.contains(Flags::C) as u8;
        self.p.set(Flags::V, (c ^ ((self.a() >> 5) & 0x01)) != 0);
    }

    fn xxa(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);

        self.set_a(self.x() & v);
    }

    fn tas(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);

        self.s = self.x() & self.a();
        let hi = ((addr >> 8) as u8).wrapping_add(1);
        self.mem_write(addr, self.s() & hi);
    }

    fn shy(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let hi = ((addr >> 8) as u8).wrapping_add(1);
        let lo = addr as u8;
        let v = self.y() & hi;
        self.mem_write(u16::from_le_bytes([lo, self.y() & hi]), v);
    }

    fn shx(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let hi = ((addr >> 8) as u8).wrapping_add(1);
        let lo = addr as u8;
        let v = self.x() & hi;
        self.mem_write(u16::from_le_bytes([lo, self.x() & hi]), v);
    }

    fn las(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);
        self.set_a(v & self.s());
        self.set_x(self.a());
        self.s = self.a();
    }

    fn axs(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);

        self.p.set(Flags::C, (self.a() & self.x()) >= v);
        self.set_x(v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::bus::TestBus;

    fn get_test_cpu(program: Vec<u8>, ram: Vec<u8>) -> Cpu<'static> {
        let mut bus = TestBus::new(program);
        for (addr, data) in ram.iter().enumerate() {
            bus.set_ram(addr as u16, *data);
        }
        let mut cpu = Cpu::new(bus);
        cpu.pc = 0x2000;
        cpu
    }

    fn get_test_cpu_from_bus(bus: TestBus) -> Cpu<'static> {
        let mut cpu = Cpu::new(bus);
        cpu.pc = 0x2000;
        cpu
    }

    #[test]
    fn test_a9() {
        let mut cpu = get_test_cpu(vec![0xA9, 0x05], vec![0]);
        cpu.execute();

        assert_eq!(cpu.a, 0x05);
        assert!(!cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));

        let mut cpu = get_test_cpu(vec![0xA9, 0x00], vec![0]);
        cpu.execute();

        assert_eq!(cpu.a, 0x00);
        assert!(!cpu.p.contains(Flags::N));
        assert!(cpu.p.contains(Flags::Z));

        let mut cpu = get_test_cpu(vec![0xA9, 0x80], vec![0]);
        cpu.execute();

        assert_eq!(cpu.a, 0x80);
        assert!(cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));
    }

    #[test]
    fn test_a5() {
        let mut cpu = get_test_cpu(vec![0xA5, 0x02], vec![0x00, 0x00, 0x23]);
        cpu.execute();

        assert_eq!(cpu.a, 0x23);
        assert!(!cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));

        let mut cpu = get_test_cpu(vec![0xA5, 0x00], vec![0x00]);
        cpu.execute();

        assert_eq!(cpu.a, 0x00);
        assert!(!cpu.p.contains(Flags::N));
        assert!(cpu.p.contains(Flags::Z));

        let mut cpu = get_test_cpu(vec![0xA5, 0x00], vec![0x85]);
        cpu.execute();

        assert_eq!(cpu.a, 0x85);
        assert!(cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));
        assert_eq!(cpu.ins_cycles, 3);
    }

    #[test]
    fn test_b5() {
        let mut bus = TestBus::new(vec![0xB5, 0x00]);
        bus.set_ram(0xFF, 0x50);
        let mut cpu = get_test_cpu_from_bus(bus);

        cpu.x = 0xFF;
        cpu.execute();

        assert_eq!(cpu.a, 0x50);
        assert!(!cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));

        let mut cpu = get_test_cpu(vec![0xB5, 0x01], vec![0x50]);

        cpu.x = 0xFF;
        cpu.execute();

        assert_eq!(cpu.a, 0x50);
        assert!(!cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));
        assert_eq!(cpu.ins_cycles, 4);
    }

    #[test]
    fn test_ad() {
        let mut bus = TestBus::new(vec![0xAD, 0x05, 0x02]);
        bus.set_ram(0x0205, 0xFE);
        let mut cpu = get_test_cpu_from_bus(bus);

        cpu.execute();

        assert_eq!(cpu.a, 0xFE);
        assert_eq!(cpu.ins_cycles, 4);
    }

    #[test]
    fn test_bd() {
        let mut bus = TestBus::new(vec![0xBD, 0x05, 0x02]);
        bus.set_ram(0x020A, 0xFE);
        let mut cpu = get_test_cpu_from_bus(bus);

        cpu.x = 5;
        cpu.execute();

        assert_eq!(cpu.a, 0xFE);
        assert_eq!(cpu.ins_cycles, 4);

        let mut bus = TestBus::new(vec![0xBD, 0xFF, 0x05]);
        bus.set_ram(0x0604, 0xFE);
        let mut cpu = get_test_cpu_from_bus(bus);

        cpu.x = 5;
        cpu.execute();

        assert_eq!(cpu.a, 0xFE);
        assert_eq!(cpu.ins_cycles, 5);
    }

    #[test]
    fn test_b9() {
        let mut bus = TestBus::new(vec![0xB9, 0x05, 0x02]);
        bus.set_ram(0x020A, 0xFE);
        let mut cpu = get_test_cpu_from_bus(bus);

        cpu.y = 5;
        cpu.execute();

        assert_eq!(cpu.a, 0xFE);
        assert_eq!(cpu.ins_cycles, 4);

        let mut bus = TestBus::new(vec![0xB9, 0xFF, 0x05]);
        bus.set_ram(0x0604, 0xFE);
        let mut cpu = get_test_cpu_from_bus(bus);

        cpu.y = 5;
        cpu.execute();

        assert_eq!(cpu.a, 0xFE);
        assert_eq!(cpu.ins_cycles, 5);
    }

    #[test]
    fn test_a1() {
        let mut bus = TestBus::new(vec![0xA1, 0x05]);
        bus.set_ram(0x0A, 0x00);
        bus.set_ram(0x0B, 0x02);
        bus.set_ram(0x0200, 0xFE);
        let mut cpu = get_test_cpu_from_bus(bus);

        cpu.x = 5;
        cpu.execute();

        assert_eq!(cpu.a, 0xFE);
        assert_eq!(cpu.ins_cycles, 6);
    }

    #[test]
    fn test_b1() {
        let mut bus = TestBus::new(vec![0xB1, 0x05]);
        bus.set_ram(0x05, 0x00);
        bus.set_ram(0x06, 0x02);
        bus.set_ram(0x0205, 0xFE);
        let mut cpu = get_test_cpu_from_bus(bus);

        cpu.y = 5;
        cpu.execute();

        assert_eq!(cpu.a, 0xFE);
        assert_eq!(cpu.ins_cycles, 5);

        let mut bus = TestBus::new(vec![0xB1, 0x05]);
        bus.set_ram(0x05, 0xFF);
        bus.set_ram(0x06, 0x02);
        bus.set_ram(0x0300, 0xFE);
        let mut cpu = get_test_cpu_from_bus(bus);

        cpu.y = 1;
        cpu.execute();

        assert_eq!(cpu.a, 0xFE);
        assert_eq!(cpu.ins_cycles, 6);
    }

    #[test]
    fn test_a2() {
        let mut cpu = get_test_cpu(vec![0xA2, 0x05], vec![0]);
        cpu.execute();

        assert_eq!(cpu.x, 0x05);
        assert!(!cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));

        let mut cpu = get_test_cpu(vec![0xA2, 0x00], vec![0]);
        cpu.execute();

        assert_eq!(cpu.x, 0x00);
        assert!(!cpu.p.contains(Flags::N));
        assert!(cpu.p.contains(Flags::Z));

        let mut cpu = get_test_cpu(vec![0xA2, 0x80], vec![0]);
        cpu.execute();

        assert_eq!(cpu.x, 0x80);
        assert!(cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));
    }

    #[test]
    fn test_a6() {
        let mut cpu = get_test_cpu(vec![0xA6, 0x02], vec![0x00, 0x00, 0x23]);
        cpu.execute();

        assert_eq!(cpu.x, 0x23);
        assert!(!cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));

        let mut cpu = get_test_cpu(vec![0xA6, 0x00], vec![0x00]);
        cpu.execute();

        assert_eq!(cpu.x, 0x00);
        assert!(!cpu.p.contains(Flags::N));
        assert!(cpu.p.contains(Flags::Z));

        let mut cpu = get_test_cpu(vec![0xA6, 0x00], vec![0x85]);
        cpu.execute();

        assert_eq!(cpu.x, 0x85);
        assert!(cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));
        assert_eq!(cpu.ins_cycles, 3);
    }

    #[test]
    fn test_b6() {
        let mut bus = TestBus::new(vec![0xB6, 0x00]);
        bus.set_ram(0xFF, 0x50);
        let mut cpu = get_test_cpu_from_bus(bus);

        cpu.y = 0xFF;
        cpu.execute();

        assert_eq!(cpu.x, 0x50);
        assert!(!cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));

        let mut cpu = get_test_cpu(vec![0xB6, 0x01], vec![0x50]);

        cpu.y = 0xFF;
        cpu.execute();

        assert_eq!(cpu.x, 0x50);
        assert!(!cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));
        assert_eq!(cpu.ins_cycles, 4);
    }

    #[test]
    fn test_ae() {
        let mut bus = TestBus::new(vec![0xAE, 0x05, 0x02]);
        bus.set_ram(0x0205, 0xFE);
        let mut cpu = get_test_cpu_from_bus(bus);

        cpu.execute();

        assert_eq!(cpu.x, 0xFE);
        assert_eq!(cpu.ins_cycles, 4);
    }

    #[test]
    fn test_be() {
        let mut bus = TestBus::new(vec![0xBE, 0x05, 0x02]);
        bus.set_ram(0x020A, 0xFE);
        let mut cpu = get_test_cpu_from_bus(bus);

        cpu.y = 5;
        cpu.execute();

        assert_eq!(cpu.x, 0xFE);
        assert_eq!(cpu.ins_cycles, 4);

        let mut bus = TestBus::new(vec![0xBE, 0xFF, 0x05]);
        bus.set_ram(0x0604, 0xFE);
        let mut cpu = get_test_cpu_from_bus(bus);

        cpu.y = 5;
        cpu.execute();

        assert_eq!(cpu.x, 0xFE);
        assert_eq!(cpu.ins_cycles, 5);
    }

    #[test]
    fn test_a0() {
        let mut cpu = get_test_cpu(vec![0xA0, 0x05], vec![0]);
        cpu.execute();

        assert_eq!(cpu.y, 0x05);
        assert!(!cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));

        let mut cpu = get_test_cpu(vec![0xA0, 0x00], vec![0]);
        cpu.execute();

        assert_eq!(cpu.y, 0x00);
        assert!(!cpu.p.contains(Flags::N));
        assert!(cpu.p.contains(Flags::Z));

        let mut cpu = get_test_cpu(vec![0xA0, 0x80], vec![0]);
        cpu.execute();

        assert_eq!(cpu.y, 0x80);
        assert!(cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));
    }

    #[test]
    fn test_85() {
        let mut cpu = get_test_cpu(vec![0x85, 0x03], vec![]);
        cpu.a = 0xDE;
        cpu.execute();

        assert_eq!(cpu.mem_read(0x03), 0xDE);
        assert_eq!(cpu.ins_cycles, 3);
    }

    #[test]
    fn test_9d() {
        let mut cpu = get_test_cpu(vec![0x9D, 0x03, 0x04], vec![]);
        cpu.a = 0xDE;
        cpu.x = 0x0A;
        cpu.execute();

        assert_eq!(cpu.mem_read(0x040D), 0xDE);
        assert_eq!(cpu.ins_cycles, 5);
    }

    #[test]
    fn test_86() {
        let mut cpu = get_test_cpu(vec![0x86, 0x03], vec![]);
        cpu.x = 0xDE;
        cpu.execute();

        assert_eq!(cpu.mem_read(0x03), 0xDE);
        assert_eq!(cpu.ins_cycles, 3);
    }

    #[test]
    fn test_84() {
        let mut cpu = get_test_cpu(vec![0x84, 0x03], vec![]);
        cpu.y = 0xDE;
        cpu.execute();

        assert_eq!(cpu.mem_read(0x03), 0xDE);
        assert_eq!(cpu.ins_cycles, 3);
    }

    #[test]
    fn test_aa() {
        let mut cpu = get_test_cpu(vec![0xAA], vec![]);
        cpu.a = 0x20;
        cpu.execute();

        assert_eq!(cpu.x, cpu.a);
        assert_eq!(cpu.x, 0x20);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_a8() {
        let mut cpu = get_test_cpu(vec![0xA8], vec![]);
        cpu.a = 0x20;
        cpu.execute();

        assert_eq!(cpu.y, cpu.a);
        assert_eq!(cpu.y, 0x20);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_ba() {
        let mut cpu = get_test_cpu(vec![0xBA], vec![]);
        cpu.s = 0x20;
        cpu.execute();

        assert_eq!(cpu.x, cpu.s);
        assert_eq!(cpu.x, 0x20);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_8a() {
        let mut cpu = get_test_cpu(vec![0x8A], vec![]);
        cpu.x = 0x20;
        cpu.execute();

        assert_eq!(cpu.a, cpu.x);
        assert_eq!(cpu.a, 0x20);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_9a() {
        let mut cpu = get_test_cpu(vec![0x9A], vec![]);
        cpu.x = 0x20;
        cpu.execute();

        assert_eq!(cpu.s, cpu.x);
        assert_eq!(cpu.s, 0x20);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_98() {
        let mut cpu = get_test_cpu(vec![0x98], vec![]);
        cpu.y = 0x20;
        cpu.execute();

        assert_eq!(cpu.a, cpu.y);
        assert_eq!(cpu.a, 0x20);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_18() {
        let mut cpu = get_test_cpu(vec![0x18], vec![]);
        cpu.p.insert(Flags::C);
        cpu.execute();

        assert!(!cpu.p.contains(Flags::C));
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_d8() {
        let mut cpu = get_test_cpu(vec![0xD8], vec![]);
        cpu.p.insert(Flags::D);
        cpu.execute();

        assert!(!cpu.p.contains(Flags::D));
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_58() {
        let mut cpu = get_test_cpu(vec![0x58], vec![]);
        cpu.p.insert(Flags::I);
        cpu.execute();

        assert!(!cpu.p.contains(Flags::I));
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_b8() {
        let mut cpu = get_test_cpu(vec![0xB8], vec![]);
        cpu.p.insert(Flags::V);
        cpu.execute();

        assert!(!cpu.p.contains(Flags::V));
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_38() {
        let mut cpu = get_test_cpu(vec![0x38], vec![]);
        cpu.execute();

        assert!(cpu.p.contains(Flags::C));
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_f8() {
        let mut cpu = get_test_cpu(vec![0xF8], vec![]);
        cpu.execute();

        assert!(cpu.p.contains(Flags::D));
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_78() {
        let mut cpu = get_test_cpu(vec![0x78], vec![]);
        cpu.execute();

        assert!(cpu.p.contains(Flags::I));
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_e6() {
        let mut cpu = get_test_cpu(vec![0xE6, 0x01], vec![0x00, 0xFE]);
        cpu.execute();

        assert!(cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));
        assert_eq!(cpu.mem_read(0x01), 0xFF);
        assert_eq!(cpu.ins_cycles, 5);

        let mut cpu = get_test_cpu(vec![0xE6, 0x01], vec![0x00, 0xFF]);
        cpu.execute();

        assert!(!cpu.p.contains(Flags::N));
        assert!(cpu.p.contains(Flags::Z));
        assert_eq!(cpu.mem_read(0x01), 0x00);
        assert_eq!(cpu.ins_cycles, 5);
    }

    #[test]
    fn test_e8() {
        let mut cpu = get_test_cpu(vec![0xE8], vec![]);
        cpu.execute();

        assert!(!cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));
        assert_eq!(cpu.x, 0x01);
        assert_eq!(cpu.ins_cycles, 2);

        let mut cpu = get_test_cpu(vec![0xE8], vec![]);
        cpu.x = 0xFF;
        cpu.execute();

        assert!(!cpu.p.contains(Flags::N));
        assert!(cpu.p.contains(Flags::Z));
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_c8() {
        let mut cpu = get_test_cpu(vec![0xC8], vec![]);
        cpu.execute();

        assert!(!cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));
        assert_eq!(cpu.y, 0x01);
        assert_eq!(cpu.ins_cycles, 2);

        let mut cpu = get_test_cpu(vec![0xC8], vec![]);
        cpu.y = 0xFF;
        cpu.execute();

        assert!(!cpu.p.contains(Flags::N));
        assert!(cpu.p.contains(Flags::Z));
        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_c6() {
        let mut cpu = get_test_cpu(vec![0xC6, 0x01], vec![0x00, 0x00]);
        cpu.execute();

        assert!(cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));
        assert_eq!(cpu.mem_read(0x01), 0xFF);
        assert_eq!(cpu.ins_cycles, 5);

        let mut cpu = get_test_cpu(vec![0xC6, 0x01], vec![0x00, 0x01]);
        cpu.execute();

        assert!(!cpu.p.contains(Flags::N));
        assert!(cpu.p.contains(Flags::Z));
        assert_eq!(cpu.mem_read(0x01), 0x00);
        assert_eq!(cpu.ins_cycles, 5);
    }

    #[test]
    fn test_ca() {
        let mut cpu = get_test_cpu(vec![0xCA], vec![]);
        cpu.execute();

        assert!(cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));
        assert_eq!(cpu.x, 0xFF);
        assert_eq!(cpu.ins_cycles, 2);

        let mut cpu = get_test_cpu(vec![0xCA], vec![]);
        cpu.x = 0x01;
        cpu.execute();

        assert!(!cpu.p.contains(Flags::N));
        assert!(cpu.p.contains(Flags::Z));
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_88() {
        let mut cpu = get_test_cpu(vec![0x88], vec![]);
        cpu.execute();

        assert!(cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));
        assert_eq!(cpu.y, 0xFF);
        assert_eq!(cpu.ins_cycles, 2);

        let mut cpu = get_test_cpu(vec![0x88], vec![]);
        cpu.y = 0x01;
        cpu.execute();

        assert!(!cpu.p.contains(Flags::N));
        assert!(cpu.p.contains(Flags::Z));
        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_c9() {
        let mut cpu = get_test_cpu(vec![0xC9, 0x05], vec![]);
        cpu.a = 0x05;
        cpu.execute();

        assert!(cpu.p.contains(Flags::C));
        assert!(cpu.p.contains(Flags::Z));
        assert!(!cpu.p.contains(Flags::N));

        let mut cpu = get_test_cpu(vec![0xC9, 0x0A], vec![]);
        cpu.a = 0x05;
        cpu.execute();

        assert!(!cpu.p.contains(Flags::C));
        assert!(!cpu.p.contains(Flags::Z));
        assert!(cpu.p.contains(Flags::N));
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_e4() {
        let mut bus = TestBus::new(vec![0xE4, 0x05]);
        bus.set_ram(0x05, 0x0A);
        let mut cpu = get_test_cpu_from_bus(bus);
        cpu.x = 0x05;
        cpu.execute();

        assert!(!cpu.p.contains(Flags::C));
        assert!(!cpu.p.contains(Flags::Z));
        assert!(cpu.p.contains(Flags::N));
        assert_eq!(cpu.ins_cycles, 3);
    }

    #[test]
    fn test_cc() {
        let mut bus = TestBus::new(vec![0xCC, 0x05, 0x03]);
        bus.set_ram(0x0305, 0x0A);
        let mut cpu = get_test_cpu_from_bus(bus);
        cpu.y = 0x05;
        cpu.execute();

        assert!(!cpu.p.contains(Flags::C));
        assert!(!cpu.p.contains(Flags::Z));
        assert!(cpu.p.contains(Flags::N));
        assert_eq!(cpu.ins_cycles, 4);
    }

    #[test]
    fn test_90() {
        let mut cpu = get_test_cpu(vec![0x90, 0x05], vec![]);
        cpu.execute();

        assert_eq!(cpu.pc, 0x2002 + 0x05);
        assert_eq!(cpu.ins_cycles, 3);

        let mut cpu = get_test_cpu(vec![0x90, !0x05 + 1], vec![]);
        cpu.execute();

        assert_eq!(cpu.pc, 0x2002 - 0x05);
        assert_eq!(cpu.ins_cycles, 4);

        let mut cpu = get_test_cpu(vec![0x90, !0x05 + 1], vec![]);
        cpu.p.insert(Flags::C);
        cpu.execute();

        assert_eq!(cpu.pc, 0x2002);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_b0() {
        let mut cpu = get_test_cpu(vec![0xB0, 0x05], vec![]);
        cpu.p.insert(Flags::C);
        cpu.execute();

        assert_eq!(cpu.pc, 0x2002 + 0x05);
        assert_eq!(cpu.ins_cycles, 3);

        let mut cpu = get_test_cpu(vec![0xB0, !0x05 + 1], vec![]);
        cpu.execute();

        assert_eq!(cpu.pc, 0x2002);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_f0() {
        let mut cpu = get_test_cpu(vec![0xF0, 0x05], vec![]);
        cpu.p.insert(Flags::Z);
        cpu.execute();

        assert_eq!(cpu.pc, 0x2002 + 0x05);
        assert_eq!(cpu.ins_cycles, 3);

        let mut cpu = get_test_cpu(vec![0xF0, !0x05 + 1], vec![]);
        cpu.execute();

        assert_eq!(cpu.pc, 0x2002);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_d0() {
        let mut cpu = get_test_cpu(vec![0xD0, 0x05], vec![]);
        cpu.execute();

        assert_eq!(cpu.pc, 0x2002 + 0x05);
        assert_eq!(cpu.ins_cycles, 3);

        let mut cpu = get_test_cpu(vec![0xD0, !0x05 + 1], vec![]);
        cpu.p.insert(Flags::Z);
        cpu.execute();

        assert_eq!(cpu.pc, 0x2002);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_30() {
        let mut cpu = get_test_cpu(vec![0x30, 0x05], vec![]);
        cpu.p.insert(Flags::N);
        cpu.execute();

        assert_eq!(cpu.pc, 0x2002 + 0x05);
        assert_eq!(cpu.ins_cycles, 3);

        let mut cpu = get_test_cpu(vec![0x30, !0x05 + 1], vec![]);
        cpu.execute();

        assert_eq!(cpu.pc, 0x2002);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_10() {
        let mut cpu = get_test_cpu(vec![0x10, 0x05], vec![]);
        cpu.execute();

        assert_eq!(cpu.pc, 0x2002 + 0x05);
        assert_eq!(cpu.ins_cycles, 3);

        let mut cpu = get_test_cpu(vec![0x10, !0x05 + 1], vec![]);
        cpu.p.insert(Flags::N);
        cpu.execute();

        assert_eq!(cpu.pc, 0x2002);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_50() {
        let mut cpu = get_test_cpu(vec![0x50, 0x05], vec![]);
        cpu.execute();

        assert_eq!(cpu.pc, 0x2002 + 0x05);
        assert_eq!(cpu.ins_cycles, 3);

        let mut cpu = get_test_cpu(vec![0x50, !0x05 + 1], vec![]);
        cpu.p.insert(Flags::V);
        cpu.execute();

        assert_eq!(cpu.pc, 0x2002);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_70() {
        let mut cpu = get_test_cpu(vec![0x70, 0x05], vec![]);
        cpu.p.insert(Flags::V);
        cpu.execute();

        assert_eq!(cpu.pc, 0x2002 + 0x05);
        assert_eq!(cpu.ins_cycles, 3);

        let mut cpu = get_test_cpu(vec![0x70, !0x05 + 1], vec![]);
        cpu.execute();

        assert_eq!(cpu.pc, 0x2002);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_4c() {
        let mut cpu = get_test_cpu(vec![0x4C, 0x34, 0x02], vec![]);
        cpu.execute();

        assert_eq!(cpu.pc, 0x0234);
        assert_eq!(cpu.ins_cycles, 3);
    }

    #[test]
    fn test_6c() {
        let mut bus = TestBus::new(vec![0x6C, 0x05, 0x03]);
        bus.set_ram(0x0305, 0x0A);
        bus.set_ram(0x0306, 0x06);
        let mut cpu = get_test_cpu_from_bus(bus);
        cpu.execute();

        assert_eq!(cpu.pc, 0x060A);

        // wrap bug test
        let mut bus = TestBus::new(vec![0x6C, 0xFF, 0x10]);
        bus.set_ram(0x10FF, 0x0A);
        bus.set_ram(0x1000, 0x06);
        let mut cpu = get_test_cpu_from_bus(bus);
        cpu.execute();

        assert_eq!(cpu.pc, 0x060A);
        assert_eq!(cpu.ins_cycles, 5);
    }

    #[test]
    fn test_48() {
        let mut cpu = get_test_cpu(vec![0x48], vec![]);
        cpu.a = 0x93;
        cpu.execute();

        assert_eq!(
            cpu.mem_read(STACK_PAGE + cpu.s.wrapping_add(1) as u16),
            0x93
        );
        assert_eq!(cpu.ins_cycles, 3);
    }

    #[test]
    fn test_08() {
        let mut cpu = get_test_cpu(vec![0x08], vec![]);
        cpu.p.insert(Flags::N);
        cpu.p.insert(Flags::V);
        cpu.p.insert(Flags::C);
        cpu.execute();

        assert_eq!(
            cpu.mem_read(STACK_PAGE + cpu.s.wrapping_add(1) as u16),
            (Flags::N | Flags::V | Flags::U | Flags::B | Flags::C | Flags::I).bits()
        );
        assert_eq!(cpu.ins_cycles, 3);
    }

    #[test]
    fn test_68() {
        let mut bus = TestBus::new(vec![0x68]);
        bus.set_ram(STACK_PAGE + 0xA5, 0x0A);
        let mut cpu = get_test_cpu_from_bus(bus);
        cpu.s = 0xA4;
        cpu.execute();

        assert_eq!(cpu.a, 0x0A);
        assert_eq!(cpu.ins_cycles, 4);

        let mut cpu = get_test_cpu(vec![0x68], vec![]);
        cpu.execute();

        assert_eq!(cpu.a, 0x00);
        assert!(cpu.p.contains(Flags::Z));
        assert_eq!(cpu.ins_cycles, 4);
    }

    #[test]
    fn test_28() {
        let mut bus = TestBus::new(vec![0x28]);
        bus.set_ram(STACK_PAGE + 0xA5, (Flags::N | Flags::B | Flags::I).bits());
        let mut cpu = get_test_cpu_from_bus(bus);
        cpu.s = 0xA4;
        cpu.execute();

        assert_eq!(cpu.p, Flags::N | Flags::U | Flags::I);
        assert_eq!(cpu.ins_cycles, 4);
    }

    #[test]
    fn test_20() {
        let mut cpu = get_test_cpu(vec![0x20, 0x63, 0x05], vec![]);
        cpu.execute();

        assert_eq!(cpu.mem_read_word(STACK_PAGE + cpu.s as u16 + 1), 0x2002);
        assert_eq!(cpu.pc, 0x0563);
        assert_eq!(cpu.ins_cycles, 6);
    }

    #[test]
    fn test_60() {
        let mut bus = TestBus::new(vec![0x60]);
        bus.set_ram(STACK_PAGE + 0xFE, 0xEF);
        bus.set_ram(STACK_PAGE + 0xFF, 0xBE);
        let mut cpu = get_test_cpu_from_bus(bus);
        cpu.execute();

        assert_eq!(cpu.pc, 0xBEEF + 1);
        assert_eq!(cpu.ins_cycles, 6);
    }

    #[test]
    fn test_40() {
        let mut bus = TestBus::new(vec![0x40]);
        bus.set_ram(STACK_PAGE + 0xFE, (Flags::V | Flags::C).bits());
        bus.set_ram(STACK_PAGE + 0xFF, 0xEF);
        bus.set_ram(STACK_PAGE, 0xBE);
        let mut cpu = get_test_cpu_from_bus(bus);
        cpu.execute();

        assert_eq!(cpu.pc, 0xBEEF);
        assert_eq!(cpu.p, Flags::V | Flags::U | Flags::C);
        assert_eq!(cpu.ins_cycles, 6);
    }

    #[test]
    fn test_ea() {
        let mut cpu = get_test_cpu(vec![0xEA], vec![]);
        cpu.execute();

        assert_eq!(cpu.pc, 0x2001);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_24() {
        let mut bus = TestBus::new(vec![0x24, 0xFE]);
        bus.set_ram(0xFE, 0b0010_0110);
        let mut cpu = get_test_cpu_from_bus(bus);
        cpu.a = 0b1101_1001;
        cpu.execute();

        assert!(cpu.p.contains(Flags::Z));
        assert_eq!(cpu.ins_cycles, 3);

        let mut bus = TestBus::new(vec![0x24, 0xFE]);
        bus.set_ram(0xFE, 0b1100_0110);
        let mut cpu = get_test_cpu_from_bus(bus);
        cpu.a = 0b1101_1001;
        cpu.execute();

        assert!(!cpu.p.contains(Flags::Z));
        assert!(cpu.p.contains(Flags::V));
        assert!(cpu.p.contains(Flags::N));
        assert_eq!(cpu.ins_cycles, 3);
    }

    #[test]
    fn test_29() {
        let mut cpu = get_test_cpu(vec![0x29, 0x8E], vec![]);
        cpu.a = 0x3C;
        cpu.execute();

        assert!(!cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));
        assert_eq!(cpu.a, 0x3C & 0x8E);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_49() {
        let mut cpu = get_test_cpu(vec![0x49, 0x8E], vec![]);
        cpu.a = 0x3C;
        cpu.execute();

        assert!(cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));
        assert_eq!(cpu.a, 0x3C ^ 0x8E);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_09() {
        let mut cpu = get_test_cpu(vec![0x09, 0x8E], vec![]);
        cpu.a = 0x3C;
        cpu.execute();

        assert!(cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));
        assert_eq!(cpu.a, 0x3C | 0x8E);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_0a() {
        let mut cpu = get_test_cpu(vec![0x0A], vec![]);
        cpu.a = 0xC1;
        cpu.execute();

        assert!(cpu.p.contains(Flags::C));
        assert_eq!(cpu.a, 0xC1 << 1);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_06() {
        let mut cpu = get_test_cpu(vec![0x06, 0x00], vec![0x67]);
        cpu.execute();

        assert!(!cpu.p.contains(Flags::C));
        assert_eq!(cpu.mem_read(0x00), 0x67 << 1);
        assert_eq!(cpu.ins_cycles, 5);
    }

    #[test]
    fn test_4a() {
        let mut cpu = get_test_cpu(vec![0x4A], vec![]);
        cpu.a = 0xC0;
        cpu.execute();

        assert!(!cpu.p.contains(Flags::C));
        assert_eq!(cpu.a, 0xC1 >> 1);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_46() {
        let mut cpu = get_test_cpu(vec![0x46, 0x00], vec![0x67]);
        cpu.execute();

        assert!(cpu.p.contains(Flags::C));
        assert_eq!(cpu.mem_read(0x00), 0x67 >> 1);
        assert_eq!(cpu.ins_cycles, 5);
    }

    #[test]
    fn test_2a() {
        let mut cpu = get_test_cpu(vec![0x2A], vec![]);
        cpu.a = 0b0100_0010;
        cpu.p.insert(Flags::C);
        cpu.execute();

        assert!(cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z | Flags::C));
        assert_eq!(cpu.a, 0b1000_0101);
    }

    #[test]
    fn test_2e() {
        let mut bus = TestBus::new(vec![0x2E, 0xFE, 0x10]);
        bus.set_ram(0x10FE, 0b1001_0110);
        let mut cpu = get_test_cpu_from_bus(bus);
        cpu.p.insert(Flags::C);
        cpu.execute();

        assert!(cpu.p.contains(Flags::C));
        assert!(!cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));
        assert_eq!(cpu.mem_read(0x10FE), 0b0010_1101);

        let mut bus = TestBus::new(vec![0x2E, 0xFE, 0x10]);
        bus.set_ram(0x10FE, 0b0001_0110);
        let mut cpu = get_test_cpu_from_bus(bus);
        cpu.execute();

        assert!(!cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));
        assert!(!cpu.p.contains(Flags::C));
        assert_eq!(cpu.mem_read(0x10FE), 0b0010_1100);
    }

    #[test]
    fn test_6a() {
        let mut cpu = get_test_cpu(vec![0x6A], vec![]);
        cpu.a = 0b0100_0011;
        cpu.p.insert(Flags::C);
        cpu.execute();

        assert!(cpu.p.contains(Flags::N));
        assert!(cpu.p.contains(Flags::C));
        assert!(!cpu.p.contains(Flags::Z));
        assert_eq!(cpu.a, 0b1010_0001);
    }

    #[test]
    fn test_6e() {
        let mut bus = TestBus::new(vec![0x6E, 0xFE, 0x10]);
        bus.set_ram(0x10FE, 0b1001_0110);
        let mut cpu = get_test_cpu_from_bus(bus);
        cpu.p.insert(Flags::C);
        cpu.execute();

        assert!(cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::C));
        assert!(!cpu.p.contains(Flags::Z));
        assert_eq!(cpu.mem_read(0x10FE), 0b1100_1011);

        let mut bus = TestBus::new(vec![0x6E, 0xFE, 0x10]);
        bus.set_ram(0x10FE, 0b0001_0110);
        let mut cpu = get_test_cpu_from_bus(bus);
        cpu.execute();

        assert!(!cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));
        assert!(!cpu.p.contains(Flags::C));
        assert_eq!(cpu.mem_read(0x10FE), 0b0000_1011);
    }

    #[test]
    fn test_69() {
        let mut cpu = get_test_cpu(vec![0x69, 0x45], vec![]);
        cpu.a = 0xBA;
        cpu.p.insert(Flags::C);
        cpu.execute();

        assert!(cpu.p.contains(Flags::C));
        assert!(cpu.p.contains(Flags::Z));
        assert!(!cpu.p.contains(Flags::V));
        assert!(!cpu.p.contains(Flags::N));
        assert_eq!(cpu.a, 0x00);

        let mut cpu = get_test_cpu(vec![0x69, 0xB5], vec![]);
        cpu.a = 0xBA;
        cpu.execute();

        assert!(cpu.p.contains(Flags::C));
        assert!(cpu.p.contains(Flags::V));
        assert!(!cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));
        assert_eq!(cpu.a, 0x6F);
    }

    #[test]
    fn test_e9() {
        let mut cpu = get_test_cpu(vec![0xE9, 0x45], vec![]);
        cpu.a = 0xBA;
        cpu.p.insert(Flags::C);
        cpu.execute();

        assert!(!cpu.p.contains(Flags::Z));
        assert!(!cpu.p.contains(Flags::N));
        assert!(cpu.p.contains(Flags::C));
        assert!(cpu.p.contains(Flags::V));
        assert_eq!(cpu.a, 0xBA - 0x45);

        let mut cpu = get_test_cpu(vec![0xE9, 0x38], vec![]);
        cpu.a = 0xF7;
        cpu.p.insert(Flags::C);
        cpu.execute();

        assert!(cpu.p.contains(Flags::C));
        assert!(cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));
        assert!(!cpu.p.contains(Flags::V));
        assert_eq!(cpu.a, 0xF7 - 0x38);

        let mut cpu = get_test_cpu(vec![0xE9, 0x02], vec![]);
        cpu.a = 0xFF;
        cpu.p.insert(Flags::C);
        cpu.execute();

        assert!(cpu.p.contains(Flags::C));
        assert!(cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::Z));
        assert!(!cpu.p.contains(Flags::V));
        assert_eq!(cpu.a, 0xFF - 0x02);

        let mut cpu = get_test_cpu(vec![0xE9, 0x02], vec![]);
        cpu.a = 0x00;
        cpu.p.insert(Flags::C);
        cpu.execute();

        assert!(cpu.p.contains(Flags::N));
        assert!(!cpu.p.contains(Flags::C));
        assert!(!cpu.p.contains(Flags::Z));
        assert!(!cpu.p.contains(Flags::V));
        assert_eq!(cpu.a, 0x00u8.wrapping_sub(0x02));
    }
}
