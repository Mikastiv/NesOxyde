use self::addr_modes::AddrMode;
use self::instructions::OPTABLE;

mod addr_modes;
mod instructions;

pub trait Interface {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
}

bitflags! {
    struct Flags: u8 {
        const N = 0b10000000;
        const V = 0b01000000;
        const U = 0b00100000;
        const B = 0b00010000;
        const D = 0b00001000;
        const I = 0b00000100;
        const Z = 0b00000010;
        const C = 0b00000001;
    }
}

pub struct Cpu {
    a: u8,
    x: u8,
    y: u8,
    s: u8,
    p: Flags,
    pc: u16,

    bus: Box<dyn Interface>,
    ins_cycles: u32,
}

impl Cpu {
    pub fn new(bus: Box<dyn Interface>) -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            s: 0,
            p: Flags::U,
            pc: 0,

            bus,

            ins_cycles: 0,
        }
    }

    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.s = 0xFD;
        self.p = Flags::from_bits_truncate(0);
        self.pc = self.mem_read_word(0xFFFC);
    }

    pub fn execute(&mut self) -> u32 {
        let opcode = self.read_byte();

        let ins = *OPTABLE.get(&opcode).unwrap();
        self.ins_cycles = ins.cycles;
        (ins.cpu_fn)(self, ins.mode);

        self.ins_cycles
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        self.bus.read(addr)
    }

    fn mem_read_word(&mut self, addr: u16) -> u16 {
        let lo = self.mem_read(addr);
        let hi = self.mem_read(addr.wrapping_add(1));
        u16::from_le_bytes([lo, hi])
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.write(addr, data);
    }

    fn read_byte(&mut self) -> u8 {
        let b = self.mem_read(self.pc);
        self.increment_pc();
        b
    }

    fn read_word(&mut self) -> u16 {
        let lo = self.read_byte();
        let hi = self.read_byte();
        u16::from_le_bytes([lo, hi])
    }

    fn operand_addr(&mut self, mode: AddrMode) -> u16 {
        match mode {
            AddrMode::None | AddrMode::IMP => panic!("Not supported"),
            AddrMode::IMM | AddrMode::REL => self.pc,
            AddrMode::ZP0 => self.read_byte() as u16,
            AddrMode::ZPX => {
                let base = self.read_byte();
                base.wrapping_add(self.x) as u16
            }
            AddrMode::ZPY => {
                let base = self.read_byte();
                base.wrapping_add(self.y) as u16
            }
            AddrMode::ABS | AddrMode::IND => self.read_word(),
            AddrMode::ABX => {
                let base = self.read_word();
                let addr = base.wrapping_add(self.x as u16);

                if Self::page_crossed(base, addr) {
                    self.ins_cycles += 1;
                }

                addr
            }
            AddrMode::ABXW => {
                let base = self.read_word();
                base.wrapping_add(self.x as u16)
            }
            AddrMode::ABY => {
                let base = self.read_word();
                let addr = base.wrapping_add(self.y as u16);

                if Self::page_crossed(base, addr) {
                    self.ins_cycles += 1;
                }

                addr
            }
            AddrMode::ABYW => {
                let base = self.read_word();
                base.wrapping_add(self.y as u16)
            }
            AddrMode::IZX => {
                let base = self.read_byte();
                let ptr = base.wrapping_add(self.x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                u16::from_le_bytes([lo, hi])
            }
            AddrMode::IZY => {
                let ptr = self.read_byte();
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                let addr = u16::from_le_bytes([lo, hi]).wrapping_add(self.y as u16);

                if Self::page_crossed(u16::from_le_bytes([lo, hi]), addr) {
                    self.ins_cycles += 1;
                }

                addr
            }
            AddrMode::IZYW => {
                let ptr = self.read_byte();
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                u16::from_le_bytes([lo, hi]).wrapping_add(self.y as u16)
            }
        }
    }

    fn fetch_operand(&mut self, addr: u16, mode: AddrMode) -> u8 {
        match mode {
            AddrMode::None | AddrMode::IMP | AddrMode::IND => panic!("Not supported"),
            AddrMode::IMM | AddrMode::REL => self.read_byte(),
            _ => self.mem_read(addr),
        }
    }

    fn increment_pc(&mut self) {
        self.pc = self.pc.wrapping_add(1);
    }

    fn set_z_n(&mut self, v: u8) {
        self.p.set(Flags::Z, v == 0);
        self.p.set(Flags::N, v & 0x80 != 0);
    }

    fn set_a(&mut self, v: u8) {
        self.a = v;
        self.set_z_n(v);
    }

    fn page_crossed(old: u16, new: u16) -> bool {
        old & 0xFF00 != new & 0xFF00
    }

    fn lda(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        let v = self.fetch_operand(addr, mode);
        self.set_a(v);
    }

    fn sta(&mut self, mode: AddrMode) {
        let addr = self.operand_addr(mode);
        self.mem_write(addr, self.a);
    }

    fn tax(&mut self, mode: AddrMode) {
        self.x = self.a;
        self.set_z_n(self.x);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::bus::TestBus;

    fn get_test_cpu(program: Vec<u8>, ram: Vec<u8>) -> Cpu {
        let mut bus = Box::new(TestBus::new(program));
        for (addr, data) in ram.iter().enumerate() {
            bus.set_ram(addr as u16, *data);
        }
        let mut cpu = Cpu::new(bus);
        cpu.pc = 0x2000;
        cpu
    }

    fn get_test_cpu_from_bus(bus: TestBus) -> Cpu {
        let mut cpu = Cpu::new(Box::new(bus));
        cpu.pc = 0x2000;
        cpu
    }

    #[test]
    fn test_aa() {
        let mut cpu = get_test_cpu(vec![0xAA], vec![0]);
        cpu.a = 0x20;
        cpu.execute();

        assert_eq!(cpu.x, cpu.a);
        assert_eq!(cpu.x, 0x20);
        assert_eq!(cpu.ins_cycles, 2);
    }

    #[test]
    fn test_a9() {
        let mut cpu = get_test_cpu(vec![0xA9, 0x05], vec![0]);
        cpu.execute();

        assert_eq!(cpu.a, 0x05);
        assert_eq!(cpu.p.contains(Flags::N), false);
        assert_eq!(cpu.p.contains(Flags::Z), false);

        let mut cpu = get_test_cpu(vec![0xA9, 0x00], vec![0]);
        cpu.execute();

        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.p.contains(Flags::N), false);
        assert_eq!(cpu.p.contains(Flags::Z), true);

        let mut cpu = get_test_cpu(vec![0xA9, 0x80], vec![0]);
        cpu.execute();

        assert_eq!(cpu.a, 0x80);
        assert_eq!(cpu.p.contains(Flags::N), true);
        assert_eq!(cpu.p.contains(Flags::Z), false);
    }

    #[test]
    fn test_a5() {
        let mut cpu = get_test_cpu(vec![0xA5, 0x02], vec![0x00, 0x00, 0x23]);
        cpu.execute();

        assert_eq!(cpu.a, 0x23);
        assert_eq!(cpu.p.contains(Flags::N), false);
        assert_eq!(cpu.p.contains(Flags::Z), false);

        let mut cpu = get_test_cpu(vec![0xA5, 0x00], vec![0x00]);
        cpu.execute();

        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.p.contains(Flags::N), false);
        assert_eq!(cpu.p.contains(Flags::Z), true);

        let mut cpu = get_test_cpu(vec![0xA5, 0x00], vec![0x85]);
        cpu.execute();

        assert_eq!(cpu.a, 0x85);
        assert_eq!(cpu.p.contains(Flags::N), true);
        assert_eq!(cpu.p.contains(Flags::Z), false);
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
        assert_eq!(cpu.p.contains(Flags::N), false);
        assert_eq!(cpu.p.contains(Flags::Z), false);

        let mut cpu = get_test_cpu(vec![0xB5, 0x01], vec![0x50]);

        cpu.x = 0xFF;
        cpu.execute();

        assert_eq!(cpu.a, 0x50);
        assert_eq!(cpu.p.contains(Flags::N), false);
        assert_eq!(cpu.p.contains(Flags::Z), false);
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
}
