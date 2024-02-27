#[allow(unused_imports)]
use common::prelude::*;

pub struct VM {
    regs : [u64; 16],
    rom : Vec<u8>,
    ram : Vec<u8>,
}

impl VM {
    pub fn new(rom : Vec<u8>, ram_size : usize) -> Self {
        Self {
            regs: [0; 16],
            rom,
            ram: vec![0; ram_size],
        }
    }

    pub fn boot(&mut self) {
        self.set_reg(&Register::rip(), 0);
        self.ram = vec![0; self.ram.len()];
    }

    pub fn execute_next(&mut self) -> Result<()> {
        let rip = self.get_reg(&Register::rip()).get_word(0);

        let bytes : Vec<_> = (0..6).map(|offset| self.get_mem_byte(rip.wrapping_add(offset))).collect();
        let instr = Instruction::decompile(&bytes)?;

        // Move RIP
        self.set_reg(&Register::rip(), rip.wrapping_add(instr.len()) as u64);

        self.execute(&instr)?;

        Ok(())
    }

    pub fn execute(&mut self, instr : &Instruction) -> Result<()> {
        use Instruction::*;
        match instr {
            Nop => Ok(()),
            _ => todo!("{instr:?}"),
        }
    }

    pub fn set_reg(&mut self, reg : &Register, value : u64) {
        self.regs[reg.as_src() as usize] = value
    }

    pub fn get_reg(&self, reg : &Register) -> Immediate {
        let value = self.regs[reg.as_src() as usize];
        Immediate::new_unchecked(reg.width(), value)
    }

    pub fn set_mem(&mut self, addr : u16, value : &Immediate) {
        match value.width() {
            Width::Byte => self.set_mem_byte(addr, value.get_byte(0)),
            Width::Word => for offset in 0..=1 {
                self.set_mem_byte(addr + offset, value.get_byte(offset as u8));
            },
        }
    }

    fn set_mem_byte(&mut self, addr : u16, value : u8) {
        if addr < 0x6000 {
            self.rom[addr as usize] = value
        } else if addr < 0x7800 {
            todo!("Display")
        } else if addr < 0x8000 {
            todo!("IO")
        } else {
            self.ram[addr as usize] = value
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
            self.ram.get(addr as usize).map_or(0, |b| *b)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! case {
        ($ident:ident, $code:expr, $reps:literal, $regs:expr) => {
            #[test]
            fn nop() {
                let rom = $code.into_iter().flat_map(|instr| instr.unwrap().compile()).collect();
                let mut vm = VM::new(rom, 0x8000);

                vm.boot();
                for _ in 0..$reps {
                    vm.execute_next().unwrap();
                }

                assert_eq!(vm.regs, $regs);
            }
        };
    }

    case!(nop, [Instruction::nop()], 1, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x02, 0]);
}
