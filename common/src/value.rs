#[allow(unused_imports)]
use crate::prelude::*;

pub trait Value : std::fmt::Debug + std::fmt::Display + Clone + Copy + PartialEq {
   fn width(&self) -> Width; 
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Width {
    Byte, Word,
}

impl Width {
    pub fn smallest_that_fits(value : u64) -> Self {
        <u64 as TryInto<u8>>::try_into(value).map(|_| Self::Byte)
            .or(<u64 as TryInto<u16>>::try_into(value).map(|_| Self::Word))
            .unwrap()
    }

    pub fn fits(&self, value : u64) -> bool {
        use Width::*;
        match self {
            Byte => <u64 as TryInto<u8>>::try_into(value).is_ok(),
            Word => <u64 as TryInto<u16>>::try_into(value).is_ok(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register {
    RINFO,
    RIP,
    Flags,
    RSH,
    RSB,
    R(Width, u8),
}

impl Register {
    pub fn rinfo() -> Self { Self::RINFO }
    pub fn rip() -> Self { Self::RIP }
    pub fn flags() -> Self { Self::Flags }
    pub fn rsh() -> Self { Self::RSH }
    pub fn rsb() -> Self { Self::RSB }
    fn r(width : Width, idx : u8) -> Self { Self::R(width, idx) }
    pub fn rb0() -> Self { Self::r(Width::Byte, 0) }
    pub fn r0() -> Self { Self::r(Width::Word, 0) }
    pub fn r1() -> Self { Self::r(Width::Word, 1) }
    pub fn rb1() -> Self { Self::r(Width::Byte, 1) }
    pub fn r2() -> Self { Self::r(Width::Word, 2) }
    pub fn rb2() -> Self { Self::r(Width::Byte, 2) }
    pub fn r3() -> Self { Self::r(Width::Word, 3) }
    pub fn rb3() -> Self { Self::r(Width::Byte, 3) }
    pub fn r4() -> Self { Self::r(Width::Word, 4) }
    pub fn rb4() -> Self { Self::r(Width::Byte, 4) }
    pub fn r5() -> Self { Self::r(Width::Word, 5) }
    pub fn rb5() -> Self { Self::r(Width::Byte, 5) }
    pub fn r6() -> Self { Self::r(Width::Word, 6) }
    pub fn rb6() -> Self { Self::r(Width::Byte, 6) }
    pub fn r7() -> Self { Self::r(Width::Word, 7) }
    pub fn rb7() -> Self { Self::r(Width::Byte, 7) }
    pub fn r8() -> Self { Self::r(Width::Word, 8) }
    pub fn rb8() -> Self { Self::r(Width::Byte, 8) }
    pub fn r9() -> Self { Self::r(Width::Word, 9) }
    pub fn rb9() -> Self { Self::r(Width::Byte, 9) }
    pub fn r10() -> Self { Self::r(Width::Word, 10) }
    pub fn rb10() -> Self { Self::r(Width::Byte, 10) }

    pub fn from(s : &str) -> Option<Self> {
        match &*s.to_lowercase() {
            "rinfo" => Some(Self::rinfo()),
            "rip" => Some(Self::rip()),
            "flags" => Some(Self::flags()),
            "rsh" => Some(Self::rsh()),
            "rsb" => Some(Self::rsb()),
            "rb0" => Some(Self::rb0()),
            "r0" => Some(Self::r0()),
            "rb1" => Some(Self::rb1()),
            "r1" => Some(Self::r1()),
            "rb2" => Some(Self::rb2()),
            "r2" => Some(Self::r2()),
            "rb3" => Some(Self::rb3()),
            "r3" => Some(Self::r3()),
            "rb4" => Some(Self::rb4()),
            "r4" => Some(Self::r4()),
            "rb5" => Some(Self::rb5()),
            "r5" => Some(Self::r5()),
            "rb6" => Some(Self::rb6()),
            "r6" => Some(Self::r6()),
            "rb7" => Some(Self::rb7()),
            "r7" => Some(Self::r7()),
            "rb8" => Some(Self::rb8()),
            "r8" => Some(Self::r8()),
            "rb9" => Some(Self::rb9()),
            "r9" => Some(Self::r9()),
            "rb10" => Some(Self::rb10()),
            "r10" => Some(Self::r10()),
            _ => None,
        }
    }

    pub fn from_src(width : Width, byte : u8) -> Self {
        match byte & 0xF {
            0xB => Self::rsb(),
            0xC => Self::rsh(),
            0xD => Self::flags(),
            0xE => Self::rip(),
            0xF => Self::rinfo(),
            idx => Self::r(width, idx),
        }
    }

    pub fn from_dest(width : Width, byte : u8) -> Self {
        Self::from_src(width, byte >> 4)
    }

    pub fn as_src(&self) -> u8 {
        use Register::*;
        match self {
            R(_, idx) => *idx,
            RSB => 0xB,
            RSH => 0xC,
            Flags => 0xD,
            RIP => 0xE,
            RINFO => 0xF,
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
            RINFO | RIP | Flags | RSH | RSB
                => Width::Word,
            R(width, _) => *width,
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
    pub fn new(width : Width, value : u64) -> Result<Self> {
        if width.fits(value) {
            Ok(Self { width, value })
        } else {
            Err(Error::NumberOOB(value, width))
        }
    }

    pub fn new_unchecked(width : Width, value : u64) -> Self {
        Self { width, value }
    }

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

    pub fn get_value(&self) -> u64 {
        self.value
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
