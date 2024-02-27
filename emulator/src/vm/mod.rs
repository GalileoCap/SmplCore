#[allow(unused_imports)]
use common::prelude::*;

mod utils;
use utils::RegisterValue;

#[cfg(test)]
mod test;

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
            MovIP2R(src, dest) => {
                let value = self.get_mem(src.get_word(0), dest.width());
                self.set_reg(dest, &value)
            },
            MovIP2RP(src, dest) => {
                let value = self.get_mem(src.get_word(0), Width::Byte);
                self.set_mem(self.get_reg(dest).get_word(0), &value)
            },
            MovIP2IP(src, dest) => {
                let value = self.get_mem(src.get_word(0), Width::Byte);
                let addr = self.get_mem(dest.get_word(0), dest.width()).get_word(0);
                self.set_mem(addr, &value)
            },
            MovR2R(src, dest) => self.set_reg(dest, &self.get_reg(src)),
            MovR2RP(src, dest) => self.set_mem(self.get_reg(dest).get_word(0), &self.get_reg(src)),
            MovR2IP(src, dest) => self.set_mem(dest.get_word(0), &self.get_reg(src)),
            MovRP2R(src, dest) => {
                let value = self.get_mem(self.get_reg(src).get_word(0), dest.width());
                self.set_reg(dest, &value)
            },
            MovRP2RP(src, dest) => {
                let value = self.get_mem(self.get_reg(src).get_word(0), Width::Byte);
                self.set_mem(self.get_reg(dest).get_word(0), &value)
            },
            MovRP2IP(src, dest) => {
                let value = self.get_mem(self.get_reg(src).get_word(0), Width::Byte);
                let addr = self.get_mem(dest.get_word(0), Width::Word).get_word(0);
                dbg!(src, dest, value, addr);
                self.set_mem(addr, &value)
            },
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
            if let Some(b) = self.rom.get_mut(addr as usize) {
                *b = value;
            }
        } else if addr < 0x7800 {
            todo!("Display")
        } else if addr < 0x8000 {
            todo!("IO")
        } else {
            #[allow(clippy::collapsible_else_if)]
            if let Some(b) = self.ram.get_mut((addr - 0x8000) as usize) {
                *b = value;
            }
        }
    }

    pub fn get_mem(&mut self, addr : u16, width : Width) -> Immediate {
        let mut value = 0;
        for idx in 0..width.len() {
            let byte = self.get_mem_byte(addr.wrapping_add(idx as u16));
            value |= (byte as u64) << (8 * idx);
        }
        Immediate::new_unchecked(width, value)
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
