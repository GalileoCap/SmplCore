#[allow(unused_imports)]
use common::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegisterValue(u64);

impl RegisterValue {
    pub fn set_byte(&mut self, idx : u8, value : u8) {
        self.0 &= !(0xFF << (idx * 8)); // Set this byte to 0
        self.0 |= ((value as u64) << (idx * 8)); // Set the value
    }

    pub fn set_word(&mut self, idx : u8, value : u16) {
        self.0 &= !(0xFFFF << (idx * 16)); // Set this word to 0
        self.0 |= ((value as u64) << (idx * 16)); // Set the value
    }

    pub fn get_byte(&self, idx : u8) -> u8 {
        (self.0 >> (idx * 8)) as u8
    }

    pub fn get_word(&self, idx : u8) -> u16 {
        (self.0 >> (idx * 16)) as u16
    }
}

impl From<u64> for RegisterValue {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl Into<u64> for RegisterValue {
    fn into(self) -> u64 {
        self.0
    }
}

pub struct VM {
    regs : [RegisterValue; 16],
    rom : Vec<u8>,
    ram : Vec<u8>,
}

impl VM {
    pub fn new(rom : Vec<u8>, ram_size : usize) -> Self {
        Self {
            regs: [RegisterValue(0); 16],
            rom,
            ram: vec![0; ram_size],
        }
    }

    pub fn boot(&mut self) {
        self.set_reg_value(&Register::rip(), 0);
        self.ram = vec![0; self.ram.len()];
    }

    pub fn execute_next(&mut self) -> Result<()> {
        let rip = self.get_reg(&Register::rip()).get_word(0);

        let bytes : Vec<_> = (0..6).map(|offset| self.get_mem_byte(rip.wrapping_add(offset))).collect();
        let instr = Instruction::decompile(&bytes)?;

        // Move RIP
        self.set_reg_value(&Register::rip(), rip.wrapping_add(instr.len()) as u64);

        self.execute(&instr)?;

        Ok(())
    }

    pub fn execute(&mut self, instr : &Instruction) -> Result<()> {
        use Instruction::*;
        match instr {
            Nop => (),
            MovI2R(value, dest) => self.set_reg(dest, value),
            MovI2RP(value, dest) => self.set_mem(self.get_reg(dest).get_word(0), value),
            MovI2IP(value, dest) => self.set_mem(dest.get_word(0), value),
            MovR2R(src, dest) => self.set_reg(dest, &self.get_reg(src)),
            MovR2RP(src, dest) => self.set_mem(self.get_reg(dest).get_word(0), &self.get_reg(src)),

            _ => todo!("{instr:?}"),
        };
        Ok(())
    }

    pub fn set_reg(&mut self, reg : &Register, value : &Immediate) {
        let reg = &mut self.regs[reg.as_src() as usize];
        match value.width() {
            Width::Byte => reg.set_byte(0, value.get_byte(0)),
            Width::Word => reg.set_word(0, value.get_word(0)),
        };
    }

    pub fn set_reg_value(&mut self, reg : &Register, value : u64) {
        self.regs[reg.as_src() as usize] = value.into()
    }
    
    pub fn get_reg(&self, reg : &Register) -> Immediate {
        let value = self.regs[reg.as_src() as usize];
        Immediate::new_unchecked(reg.width(), value.into())
    }

    pub fn set_mem(&mut self, addr : u16, value : &Immediate) {
        match value.width() {
            Width::Byte => self.set_mem_byte(addr, value.get_byte(0)),
            Width::Word => for offset in 0..=1 {
                self.set_mem_byte(addr + offset, value.get_byte(offset as u8));
            },
        }
    }

    pub fn set_mem_byte(&mut self, addr : u16, value : u8) {
        if addr < 0x6000 {
            self.rom[addr as usize] = value
        } else if addr < 0x7800 {
            todo!("Display")
        } else if addr < 0x8000 {
            todo!("IO")
        } else {
            self.ram[(addr - 0x8000) as usize] = value
        }
    }

    pub fn get_mem_byte(&self, addr : u16) -> u8 {
        if addr < 0x6000 {
            self.rom.get(addr as usize).map_or(0, |b| *b)
        } else if addr < 0x7800 {
            todo!("Display")
        } else if addr < 0x8000 {
            todo!("IO")
        } else {
            self.ram.get((addr - 0x8000) as usize).map_or(0, |b| *b)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! case {
        ($ident:ident, $code:expr, $reps:literal, $regs:expr, $mem:expr) => {
            #[test]
            fn $ident() {
                let rom = $code.into_iter().flat_map(|instr| instr.unwrap().compile()).collect();
                let mut vm = VM::new(rom, 0x8000);

                vm.boot();
                for _ in 0..$reps {
                    vm.execute_next().unwrap();
                }

                assert_eq!(vm.regs.iter().map(|reg| (*reg).into()).collect::<Vec<u64>>(), $regs);
                for (addr, value) in $mem.into_iter() {
                    assert_eq!(vm.get_mem_byte(addr), value, "at {addr:#06X}");
                }
            }
        };

        ($ident:ident, $code:expr, $reps:literal, $regs:expr) => {
            case!($ident, $code, $reps, $regs, []);
        };
    }

    case!(nop, [Instruction::nop()], 1, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x02, 0]);
    case!(movi2r, [
        Instruction::movi2r(Immediate::byte(0x60), Register::rb0()),
        Instruction::movi2r(Immediate::word(0x600D), Register::r1()),
    ], 2, [0x60, 0x600D, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x08, 0]);
    case!(
        movi2rp,
        [
            Instruction::movi2r(Immediate::word(0x8000), Register::r0()),
            Instruction::movi2r(Immediate::word(0x8002), Register::r1()),
            Instruction::movi2rp(Immediate::byte(0x60), Register::r0()),
            Instruction::movi2rp(Immediate::word(0x600D), Register::r1()),
        ],
        4,
        [0x8000, 0x8002, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x10, 0],
        [(0x8000, 0x60), (0x8001, 0x00), (0x8002, 0x0D), (0x8003, 0x60)]
    );
    case!(
        movi2ip,
        [
            Instruction::movi2ip(Immediate::byte(0x60), Immediate::word(0xF337)),
            Instruction::movi2ip(Immediate::word(0x600D), Immediate::word(0xF335)),
        ],
        2,
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x0C, 0],
        [(0xF335, 0x0D), (0xF336, 0x60), (0xF337, 0x60)]
    );
    case!(movr2r, [
        Instruction::movi2r(Immediate::byte(0x60), Register::rb0()),
        Instruction::movi2r(Immediate::word(0x600D), Register::r1()),
        Instruction::movi2r(Immediate::word(0xF337), Register::r2()),
        Instruction::movi2r(Immediate::word(0xB007), Register::r3()),
        Instruction::movr2r(Register::rb0(), Register::rb0()),
        Instruction::movr2r(Register::r1(), Register::r1()),
        Instruction::movr2r(Register::rb0(), Register::rb2()),
        Instruction::movr2r(Register::r1(), Register::r3()),
    ], 8, [0x60, 0x600D, 0xF360, 0x600D, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x18, 0]);
    case!(
        movr2rp, [
            Instruction::movi2r(Immediate::word(0x600D), Register::r0()),
            Instruction::movi2r(Immediate::word(0xF337), Register::r1()),
            Instruction::movi2r(Immediate::word(0xF338), Register::r2()),
            Instruction::movr2rp(Register::rb0(), Register::r1()),
            Instruction::movr2rp(Register::r0(), Register::r2()),
        ],
        5,
        [0x600D, 0xF337, 0xF338, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x10, 0],
        [(0xF337, 0x0D), (0xF338, 0x0D), (0xF339, 0x60)]
    );
}
