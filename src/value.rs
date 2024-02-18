#[allow(unused_imports)]
use crate::prelude::*;

pub trait Value : std::fmt::Debug + std::fmt::Display + Clone + Copy + PartialEq {
   fn width(&self) -> Width; 
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Width {
    Byte, Word,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register {
    R0, R1,
    Rb0, Rb1,
}

impl Register {
    pub fn as_src(&self) -> u8 {
        use Register::*;
        match self {
            R0 | Rb0 => 0x00,
            R1 | Rb1 => 0x01,
        }
    }
    
    pub fn as_dest(&self) -> u8 {
        self.as_src() << 4
    }

    pub fn as_src_with(&self, dest : &Self) -> u8 {
        self.as_src() | dest.as_dest()
    }
}

impl Value for Register {
    fn width(&self) -> Width {
        use Register::*;
        match self {
            Rb0 | Rb1 => Width::Byte,
            R0 | R1 => Width::Word,
        }
    }
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Immediate {
    width : Width,
    value : u64,
}

impl Immediate {
    pub fn byte(value : u8) -> Self {
        Self { width: Width::Byte, value: value as u64 }
    }

    pub fn word(value : u16) -> Self {
        Self { width: Width::Word, value: value as u64 }
    }

    pub fn get_byte(&self, idx : u8) -> u8 {
        (self.value >> (idx * 8)) as u8
    }

    pub fn get_word(&self, idx : u8) -> u16 {
        (self.value >> (idx * 16)) as u16
    }
}

impl Value for Immediate {
    fn width(&self) -> Width {
        self.width
    }
}

impl std::fmt::Display for Immediate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.width {
            Width::Byte => write!(f, "{:#04}", self.value as u8),
            Width::Word => write!(f, "{:#04}", self.value as u16),
        }
    }
}
