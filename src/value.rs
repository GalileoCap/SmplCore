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
