#[allow(unused_imports)]
use crate::prelude::*;
use super::*;

impl Instruction {
    pub fn decompile(bytes : &[u8]) -> crate::prelude::Result<Self> {
        let Some(opcode) = bytes.get(0) else { return Err(Error::NoOpcode) };
        let Some(regs) = bytes.get(1) else { return Err(Error::NoRegs) };

        match opcode {
            // TODO: Mix with Instruction::opcode() to update both at the same time
            0x00 => Self::nop(),
            0x01 => Self::decompile_movi2r(Width::Byte, *regs, bytes),
            0x02 => Self::decompile_movi2r(Width::Word, *regs, bytes),
            _ => Err(Error::NoSuchOpcode(*opcode)),
        }
    }

    fn decompile_movi2r(width : Width, regs : u8, bytes : &[u8]) -> Result<Self> {
        let Some(value_lower) = bytes.get(2) else { return Err(Error::NoValue(0)) };
        let Some(value_higher) = bytes.get(3) else { return Err(Error::NoValue(1)) };
        let value = (*value_lower as u64) | ((*value_higher as u64) << 8);
        let value = Immediate::new(width, value)?;
        let dest = Register::from_dest(width, regs);
        Instruction::movi2r(value, dest)
    }
}
