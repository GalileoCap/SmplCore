use crate::prelude::*;
mod compile;
mod decompile;

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Nop,
    MovI2R(Immediate, Register),
    MovI2RP(Immediate, Register),
    MovI2IP(Immediate, Immediate),
    MovIP2R(Immediate, Register),
    MovIP2RP(Immediate, Register),
    MovIP2IP(Immediate, Immediate),
    MovR2R(Register, Register),
    MovR2RP(Register, Register),
    MovR2IP(Register, Immediate),
    MovRP2R(Register, Register),
    MovRP2RP(Register, Register),
    MovRP2IP(Register, Immediate),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamIdx {
    SrcReg,
    DestReg,
    FirstImm,
    SecondImm,
}

macro_rules! instruction_constructor {
    ($ident:ident, $IDENT:ident) => {
        pub fn $ident() -> Result<Self> {
            Ok(Self::$IDENT)
        }
    };

    ($ident:ident, $IDENT:ident, $param:ident) => {
        pub fn $ident(param : $param) -> Result<Self> {
            let res = Self::$IDENT(param);
            res.check_valid()
        }
    };

    ($ident:ident, $IDENT:ident, $left:ident, $right:ident) => {
        pub fn $ident(left : $left, right : $right) -> Result<Self> {
            let res = Self::$IDENT(left, right);
            res.check_valid()
        }
    };
}

impl Instruction {
    instruction_constructor!(nop, Nop);

    instruction_constructor!(movi2r, MovI2R, Immediate, Register);
    instruction_constructor!(movi2rp, MovI2RP, Immediate, Register);
    instruction_constructor!(movi2ip, MovI2IP, Immediate, Immediate);
    instruction_constructor!(movip2r, MovIP2R, Immediate, Register);
    instruction_constructor!(movip2rp, MovIP2RP, Immediate, Register);
    instruction_constructor!(movip2ip, MovIP2IP, Immediate, Immediate);
    instruction_constructor!(movr2r, MovR2R, Register, Register);
    instruction_constructor!(movr2rp, MovR2RP, Register, Register);
    instruction_constructor!(movr2ip, MovR2IP, Register, Immediate);
    instruction_constructor!(movrp2r, MovRP2R, Register, Register);
    instruction_constructor!(movrp2rp, MovRP2RP, Register, Register);
    instruction_constructor!(movrp2ip, MovRP2IP, Register, Immediate);

    fn check_valid(self) -> Result<Self> {
        if self.is_valid() {
            Ok(self)
        } else {
            Err(Error::InvalidOperands(self))
        }
    }

    fn is_valid(&self) -> bool {
        use Instruction::*;
        match self {
            Nop => true,

            MovI2R(src, dest) => src.width() == dest.width(),
            MovI2RP(_, dest) => dest.width() == Width::Word,
            MovI2IP(_, dest) => dest.width() == Width::Word,
            MovIP2R(src, _) => src.width() == Width::Word,
            MovIP2RP(src, dest) => src.width() == Width::Word && dest.width() == Width::Word,
            MovIP2IP(src, dest) => src.width() == Width::Word && dest.width() == Width::Word,
            MovR2R(src, dest) => src.width() == dest.width(),
            MovR2RP(_, dest) => dest.width() == Width::Word,
            MovR2IP(_, dest) => dest.width() == Width::Word,
            MovRP2R(src, _) => src.width() == Width::Word,
            MovRP2RP(src, dest) => src.width() == Width::Word && dest.width() == Width::Word,
            MovRP2IP(src, dest) => src.width() == Width::Word && dest.width() == Width::Word,
        }
    }

    #[allow(dead_code, clippy::len_without_is_empty)]
    pub fn len(&self) -> u16 {
        self.compile().len() as u16
    }

    pub fn replace_imm(self, param_idx : ParamIdx, new_value : u64) -> Result<Self> {
        match param_idx {
            ParamIdx::FirstImm => self.replace_first_imm(new_value),
            ParamIdx::SecondImm => self.replace_second_imm(new_value),
            _ => unreachable!("{param_idx:?}"), // TODO: Error
        }
    }

    pub fn replace_first_imm(self, new_value : u64) -> Result<Self> {
        use Instruction::*;
        match self {
            MovI2R(src, dest) => Self::movi2r(Immediate::new_unchecked(src.width(), new_value), dest),
            MovI2RP(src, dest) => Self::movip2r(Immediate::new_unchecked(src.width(), new_value), dest),
            MovI2IP(src, dest) => Self::movi2ip(Immediate::new_unchecked(src.width(), new_value), dest),
            MovIP2R(src, dest) => Self::movip2r(Immediate::new_unchecked(src.width(), new_value), dest),
            MovIP2RP(src, dest) => Self::movip2rp(Immediate::new_unchecked(src.width(), new_value), dest),
            MovIP2IP(src, dest) => Self::movip2ip(Immediate::new_unchecked(src.width(), new_value), dest),
            MovR2IP(src, dest) => Self::movr2ip(src, Immediate::new_unchecked(dest.width(), new_value)),
            MovRP2IP(src, dest) => Self::movrp2ip(src, Immediate::new_unchecked(dest.width(), new_value)),

            _ => panic!(), // TODO: Error
        }
    }

    pub fn replace_second_imm(self, new_value : u64) -> Result<Self> {
        use Instruction::*;
        match self {
            MovI2IP(src, dest) => Self::movi2ip(src, Immediate::new_unchecked(dest.width(), new_value)),
            MovIP2IP(src, dest) => Self::movip2ip(src, Immediate::new_unchecked(dest.width(), new_value)),
            MovR2IP(src, dest) => Self::movr2ip(src, Immediate::new_unchecked(dest.width(), new_value)),
            MovRP2IP(src, dest) => Self::movrp2ip(src, Immediate::new_unchecked(dest.width(), new_value)),

            _ => panic!("{self:?}"), // TODO: Error
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test_case {
        (
            $name:ident,
            $($l_ok:expr, $r_ok:expr),* ;
            $($l_err:expr, $r_err:expr),*
        ) => {
            #[test]
            fn $name() {
                $(
                    let left = $l_ok; let right = $r_ok;
                    let instr = Instruction::$name(left, right);
                    assert!(instr.is_ok(), "{instr:?}");
                )*

                $(
                    let left = $l_err; let right = $r_err;
                    let instr = Instruction::$name(left, right);
                    assert!(instr.is_err(), "{instr:?}");
                )*
            }
        };
    }

    test_case!(nop,;);

    // Mov
    test_case!(
        movi2r,
        Immediate::byte(0x60), Register::rb0(),
        Immediate::word(0x600D), Register::r0()
        ;
        Immediate::byte(0x60), Register::r0(),
        Immediate::word(0x600D), Register::rb0()
    );
    test_case!(
        movi2rp,
        Immediate::byte(0x60), Register::r0(),
        Immediate::word(0x600D), Register::r0()
        ;
        Immediate::byte(0x60), Register::rb0(),
        Immediate::word(0x600D), Register::rb0()
    );
    test_case!(
        movi2ip,
        Immediate::byte(0x60), Immediate::word(0xF337),
        Immediate::word(0x600D), Immediate::word(0xF337)
        ;
        Immediate::byte(0x60), Immediate::byte(0xF3),
        Immediate::word(0x600D), Immediate::byte(0xF3)
    );
    test_case!(
        movip2ip,
        Immediate::word(0x600D), Immediate::word(0xF337)
        ;
        Immediate::byte(0x60), Immediate::word(0xF337),
        Immediate::word(0x600D), Immediate::byte(0xF3)
    );
    test_case!(
        movr2r,
        Register::rb0(), Register::rb1(),
        Register::r0(), Register::r1()
        ;
        Register::rb0(), Register::r1(),
        Register::r0(), Register::rb1()
    );
    test_case!(
        movr2rp,
        Register::rb0(), Register::r1(),
        Register::r0(), Register::r1()
        ;
        Register::rb0(), Register::rb1(),
        Register::r0(), Register::rb1()
    );
    test_case!(
        movr2ip,
        Register::rb0(), Immediate::word(0x600D),
        Register::r0(), Immediate::word(0x600D)
        ;
        Register::rb0(), Immediate::byte(0x60),
        Register::r0(), Immediate::byte(0x60)
    );
    test_case!(
        movrp2r,
        Register::r0(), Register::rb1(),
        Register::r0(), Register::r1()
        ;
        Register::rb0(), Register::r1(),
        Register::rb0(), Register::rb1()
    );
    test_case!(
        movrp2rp,
        Register::r0(), Register::r1()
        ;
        Register::rb0(), Register::r1(),
        Register::rb0(), Register::rb1()
    );
}
