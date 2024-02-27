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
            0x03 => Self::decompile_movi2rp(Width::Byte, *regs, bytes),
            0x04 => Self::decompile_movi2rp(Width::Word, *regs, bytes),
            0x05 => Self::decompile_movi2ip(Width::Byte, bytes),
            0x06 => Self::decompile_movi2ip(Width::Word, bytes),
            0x07 => Self::decompile_movip2r(Width::Byte, *regs, bytes),
            0x08 => Self::decompile_movip2r(Width::Word, *regs, bytes),
            0x09 => Self::decompile_movip2rp(*regs, bytes),
            0x0A => Self::decompile_movip2ip(bytes),
            0x0B => Self::decompile_movr2r(Width::Byte, *regs),
            0x0C => Self::decompile_movr2r(Width::Word, *regs),
            0x0D => Self::decompile_movr2rp(Width::Byte, *regs),
            0x0E => Self::decompile_movr2rp(Width::Word, *regs),
            0x0F => Self::decompile_movr2ip(Width::Byte, *regs, bytes),
            0x10 => Self::decompile_movr2ip(Width::Word, *regs, bytes),
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

    fn decompile_movi2rp(width : Width, regs : u8, bytes : &[u8]) -> Result<Self> {
        let Some(value_lower) = bytes.get(2) else { return Err(Error::NoValue(0)) };
        let Some(value_higher) = bytes.get(3) else { return Err(Error::NoValue(1)) };
        let value = (*value_lower as u64) | ((*value_higher as u64) << 8);
        let value = Immediate::new(width, value)?;
        let dest = Register::from_dest(Width::Word, regs);
        Instruction::movi2rp(value, dest)
    }

    fn decompile_movi2ip(width : Width, bytes : &[u8]) -> Result<Self> {
        let Some(value_lower) = bytes.get(2) else { return Err(Error::NoValue(0)) };
        let Some(value_higher) = bytes.get(3) else { return Err(Error::NoValue(1)) };
        let value = (*value_lower as u64) | ((*value_higher as u64) << 8);
        let value = Immediate::new(width, value)?;

        let Some(dest_lower) = bytes.get(4) else { return Err(Error::NoValue(0)) };
        let Some(dest_higher) = bytes.get(5) else { return Err(Error::NoValue(1)) };
        let dest = (*dest_lower as u64) | ((*dest_higher as u64) << 8);
        let dest = Immediate::new(Width::Word, dest)?;
        Instruction::movi2ip(value, dest)
    }

    fn decompile_movip2r(width : Width, regs : u8, bytes : &[u8]) -> Result<Self> {
        let Some(value_lower) = bytes.get(2) else { return Err(Error::NoValue(0)) };
        let Some(value_higher) = bytes.get(3) else { return Err(Error::NoValue(1)) };
        let value = (*value_lower as u64) | ((*value_higher as u64) << 8);
        let value = Immediate::new(Width::Word, value)?;
        let dest = Register::from_dest(width, regs);
        Instruction::movip2r(value, dest)
    }

    fn decompile_movip2rp(regs : u8, bytes : &[u8]) -> Result<Self> {
        let Some(value_lower) = bytes.get(2) else { return Err(Error::NoValue(0)) };
        let Some(value_higher) = bytes.get(3) else { return Err(Error::NoValue(1)) };
        let value = (*value_lower as u64) | ((*value_higher as u64) << 8);
        let value = Immediate::new(Width::Word, value)?;
        let dest = Register::from_dest(Width::Word, regs);
        Instruction::movip2rp(value, dest)
    }

    fn decompile_movip2ip(bytes : &[u8]) -> Result<Self> {
        let Some(value_lower) = bytes.get(2) else { return Err(Error::NoValue(0)) };
        let Some(value_higher) = bytes.get(3) else { return Err(Error::NoValue(1)) };
        let value = (*value_lower as u64) | ((*value_higher as u64) << 8);
        let value = Immediate::new(Width::Word, value)?;

        let Some(dest_lower) = bytes.get(4) else { return Err(Error::NoValue(0)) };
        let Some(dest_higher) = bytes.get(5) else { return Err(Error::NoValue(1)) };
        let dest = (*dest_lower as u64) | ((*dest_higher as u64) << 8);
        let dest = Immediate::new(Width::Word, dest)?;
        Instruction::movip2ip(value, dest)
    }

    fn decompile_movr2r(width : Width, regs : u8) -> Result<Self> {
        let src = Register::from_src(width, regs);
        let dest = Register::from_dest(width, regs);
        Instruction::movr2r(src, dest)
    }

    fn decompile_movr2rp(width : Width, regs : u8) -> Result<Self> {
        let src = Register::from_src(width, regs);
        let dest = Register::from_dest(Width::Word, regs);
        Instruction::movr2rp(src, dest)
    }

    fn decompile_movr2ip(width : Width, regs : u8, bytes : &[u8]) -> Result<Self> {
        let src = Register::from_src(width, regs);
        let Some(value_lower) = bytes.get(2) else { return Err(Error::NoValue(0)) };
        let Some(value_higher) = bytes.get(3) else { return Err(Error::NoValue(1)) };
        let value = (*value_lower as u64) | ((*value_higher as u64) << 8);
        let dest = Immediate::new(Width::Word, value)?;

        Instruction::movr2ip(src, dest)
    }
}
