#[allow(unused_imports)]
use crate::prelude::*;
use super::*;

impl Instruction {
    pub fn compile(&self) -> Vec<u8> {
        use Instruction::*;
        match self {
            Nop => vec![self.opcode(), 0x00],

            MovI2IP(src, dest) | MovIP2IP(src, dest)
                => vec![self.opcode(), 0x00, src.get_byte(0), src.get_byte(1), dest.get_byte(0), dest.get_byte(1)],

            MovI2R(value, dest) | MovI2RP(value, dest) | MovIP2R(value, dest) | MovIP2RP(value, dest)
                => vec![self.opcode(), dest.as_dest(), value.get_byte(0), value.get_byte(1)],

            MovR2IP(src, dest) | MovRP2IP(src, dest)
                => vec![self.opcode(), src.as_src(), dest.get_byte(0), dest.get_byte(1)],

            MovR2R(src, dest) | MovR2RP(src, dest) | MovRP2R(src, dest) | MovRP2RP(src, dest)
                => vec![self.opcode(), src.as_src_with(dest)],
        }
    }

    pub fn opcode(&self) -> u8 {
        use Instruction::*;
        macro_rules! case {
            ($src:ident, $start:literal) => {
                match $src.width() {
                    Width::Byte => $start,
                    Width::Word => $start + 1,
                }
            };
        }

        match self {
            Nop => 0x00,
            MovI2R(value, _) => case!(value, 0x01),
            MovI2RP(value, _) => case!(value, 0x03),
            MovI2IP(value, _) => case!(value, 0x05),
            MovIP2R(_, dest) => case!(dest, 0x07),
            MovIP2RP(_, _) => 0x09, // TODO: Width of the data being sent?
            MovIP2IP(_, _) => 0x0A,
            MovR2R(src, _) => case!(src, 0x0B),
            MovR2RP(src, _) => case!(src, 0x0D),
            MovR2IP(src, _) => case!(src, 0x0F),
            MovRP2R(_, dest) => case!(dest, 0x11),
            MovRP2RP(_, _) => 0x13,
            MovRP2IP(_, _) => 0x14,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test_case {
        ($ident:ident, $bytes:expr) => {
            #[test]
            fn $ident() {
                let instr = Instruction::$ident().unwrap();
                let bytes = instr.compile();
                assert_eq!(bytes, $bytes);
                assert_eq!(Instruction::decompile(&bytes), Ok(instr));
            }
        };

        ($ident:ident, $($param:expr, $bytes:expr);+) => {
            #[test]
            fn $ident() {
                $(
                    let instr = Instruction::$ident($param).unwrap();
                    let bytes = instr.compile();
                    assert_eq!(bytes, $bytes);
                    assert_eq!(Instruction::decompile(&bytes), Ok(instr));
                )+
            }
        };

        ($ident:ident, $($left:expr, $right:expr, $bytes:expr);+) => {
            #[test]
            fn $ident() {
                $(
                    let instr = Instruction::$ident($left, $right).unwrap();
                    let bytes = instr.compile();
                    assert_eq!(bytes, $bytes);
                    assert_eq!(Instruction::decompile(&bytes), Ok(instr));
                )+
            }
        };
    }

    test_case!(nop, [0x00, 0x00]);
    test_case!(
        movi2r,
        Immediate::byte(0x60), Register::Rb0, [0x01, 0x00, 0x60, 0x00];
        Immediate::word(0x600D), Register::R0, [0x02, 0x00, 0x0D, 0x60]
    );
    test_case!(
        movi2rp,
        Immediate::byte(0x60), Register::R0, [0x03, 0x00, 0x60, 0x00];
        Immediate::word(0x600D), Register::R0, [0x04, 0x00, 0x0D, 0x60]
    );
    test_case!(
        movi2ip,
        Immediate::byte(0x60), Immediate::word(0xF337), [0x05, 0x00, 0x60, 0x00, 0x37, 0xF3];
        Immediate::word(0x600D), Immediate::word(0xF337), [0x06, 0x00, 0x0D, 0x60, 0x37, 0xF3]
    );
    test_case!(
        movip2r,
        Immediate::word(0x600D), Register::Rb0, [0x07, 0x00, 0x0D, 0x60];
        Immediate::word(0x600D), Register::R0, [0x08, 0x00, 0x0D, 0x60]
    );
    test_case!(
        movip2rp,
        Immediate::word(0x600D), Register::R0, [0x09, 0x00, 0x0D, 0x60]
    );
    test_case!(
        movip2ip,
        Immediate::word(0x600D), Immediate::word(0xF337), [0x0A, 0x00, 0x0D, 0x60, 0x37, 0xF3]
    );
    test_case!(
        movr2r,
        Register::Rb0, Register::Rb1, [0x0B, 0x10];
        Register::R0, Register::R1, [0x0C, 0x10]
    );
    test_case!(
        movr2rp,
        Register::Rb0, Register::R1, [0x0D, 0x10];
        Register::R0, Register::R1, [0x0E, 0x10]
    );
    test_case!(
        movr2ip,
        Register::Rb0, Immediate::word(0x600D), [0x0F, 0x00, 0x0D, 0x60];
        Register::R0, Immediate::word(0x600D), [0x10, 0x00, 0x0D, 0x60]
    );
    test_case!(
        movrp2r,
        Register::R0, Register::Rb1, [0x11, 0x10];
        Register::R0, Register::R1, [0x12, 0x10]
    );
    test_case!(
        movrp2rp,
        Register::R0, Register::R1, [0x13, 0x10]
    );
    test_case!(
        movrp2ip,
        Register::R0, Immediate::word(0x600D), [0x14, 0x00, 0x0D, 0x60]
    );
}
