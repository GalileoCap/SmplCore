#[allow(unused_imports)]
use crate::prelude::*;
use super::*;

macro_rules! get_regs {
    ($src_width:expr, $dest_width:expr, $bytes:ident) => {
        $bytes.get(1)
            .map(|regs| (Register::from_src($src_width, *regs), Register::from_dest($dest_width, *regs)))
            .ok_or(Error::NoRegs)
    };

    ($width:expr, $bytes:ident) => {
        get_regs!($width, $width, $bytes)
    };
}

macro_rules! get_imm {
    (src, $width:expr, $bytes:ident) => {
        if let Some(b0) = $bytes.get(2) {
            if let Some(b1) = $bytes.get(3) {
                Immediate::new($width, (*b0 as u64) | ((*b1 as u64) << 8))
            } else { Err(Error::NoValue(1)) }
        } else { Err(Error::NoValue(0)) }
    };

    (dest, $width:expr, $bytes:ident) => {
        if let Some(b0) = $bytes.get(4) {
            if let Some(b1) = $bytes.get(5) {
                Immediate::new($width, (*b0 as u64) | ((*b1 as u64) << 8))
            } else { Err(Error::NoValue(1)) }
        } else { Err(Error::NoValue(0)) }
    };
}

impl Instruction {
    pub fn decompile(bytes : &[u8]) -> Result<Self> {
        let opcode = bytes.get(0).ok_or(Error::NoOpcode)?;
        match opcode {
            // TODO: Mix with Instruction::opcode() to update both at the same time
            0x00 => Self::nop(),
            0x01 => Self::decompile_movi2r(Width::Byte, bytes),
            0x02 => Self::decompile_movi2r(Width::Word, bytes),
            0x03 => Self::decompile_movi2rp(Width::Byte, bytes),
            0x04 => Self::decompile_movi2rp(Width::Word, bytes),
            0x05 => Self::decompile_movi2ip(Width::Byte, bytes),
            0x06 => Self::decompile_movi2ip(Width::Word, bytes),
            0x07 => Self::decompile_movip2r(Width::Byte, bytes),
            0x08 => Self::decompile_movip2r(Width::Word, bytes),
            0x09 => Self::decompile_movip2rp(bytes),
            0x0A => Self::decompile_movip2ip(bytes),
            0x0B => Self::decompile_movr2r(Width::Byte, bytes),
            0x0C => Self::decompile_movr2r(Width::Word, bytes),
            0x0D => Self::decompile_movr2rp(Width::Byte, bytes),
            0x0E => Self::decompile_movr2rp(Width::Word, bytes),
            0x0F => Self::decompile_movr2ip(Width::Byte, bytes),
            0x10 => Self::decompile_movr2ip(Width::Word, bytes),
            0x11 => Self::decompile_movrp2r(Width::Byte, bytes),
            0x12 => Self::decompile_movrp2r(Width::Word, bytes),
            0x13 => Self::decompile_movrp2rp(bytes),
            0x14 => Self::decompile_movrp2ip(bytes),
            _ => Err(Error::NoSuchOpcode(*opcode)),
        }
    }

    fn decompile_movi2r(width : Width, bytes : &[u8]) -> Result<Self> {
        let value = get_imm!(src, width, bytes)?;
        let (_, dest) = get_regs!(width, bytes)?;
        Instruction::movi2r(value, dest)
    }

    fn decompile_movi2rp(width : Width, bytes : &[u8]) -> Result<Self> {
        let value = get_imm!(src, width, bytes)?;
        let (_, dest) = get_regs!(Width::Word, bytes)?;
        Instruction::movi2rp(value, dest)
    }

    fn decompile_movi2ip(width : Width, bytes : &[u8]) -> Result<Self> {
        let value = get_imm!(src, width, bytes)?;
        let dest = get_imm!(dest, Width::Word, bytes)?;
        Instruction::movi2ip(value, dest)
    }

    fn decompile_movip2r(width : Width, bytes : &[u8]) -> Result<Self> {
        let src = get_imm!(src, Width::Word, bytes)?;
        let (_, dest) = get_regs!(width, bytes)?;
        Instruction::movip2r(src, dest)
    }

    fn decompile_movip2rp(bytes : &[u8]) -> Result<Self> {
        let src = get_imm!(src, Width::Word, bytes)?;
        let (_, dest) = get_regs!(Width::Word, bytes)?;
        Instruction::movip2rp(src, dest)
    }

    fn decompile_movip2ip(bytes : &[u8]) -> Result<Self> {
        let src = get_imm!(src, Width::Word, bytes)?;
        let dest = get_imm!(dest, Width::Word, bytes)?;
        Instruction::movip2ip(src, dest)
    }

    fn decompile_movr2r(width : Width, bytes : &[u8]) -> Result<Self> {
        let (src, dest) = get_regs!(width, bytes)?;
        Instruction::movr2r(src, dest)
    }

    fn decompile_movr2rp(width : Width, bytes : &[u8]) -> Result<Self> {
        let (src, dest) = get_regs!(width, Width::Word, bytes)?;
        Instruction::movr2rp(src, dest)
    }

    fn decompile_movr2ip(width : Width, bytes : &[u8]) -> Result<Self> {
        let (src, _) = get_regs!(width, bytes)?;
        let dest = get_imm!(src, Width::Word, bytes)?;

        Instruction::movr2ip(src, dest)
    }

    fn decompile_movrp2r(width : Width, bytes : &[u8]) -> Result<Self> {
        let (src, dest) = get_regs!(Width::Word, width, bytes)?;
        Instruction::movrp2r(src, dest)
    }

    fn decompile_movrp2rp(bytes : &[u8]) -> Result<Self> {
        let (src, dest) = get_regs!(Width::Word, bytes)?;
        Instruction::movrp2rp(src, dest)
    }

    fn decompile_movrp2ip(bytes : &[u8]) -> Result<Self> {
        let (src, _) = get_regs!(Width::Word, bytes)?;
        let dest = get_imm!(src, Width::Word, bytes)?;
        Instruction::movrp2ip(src, dest)
    }
}
