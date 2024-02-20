#[allow(unused_imports)]
use crate::prelude::*;
use super::*;

impl Instruction {
    pub fn compile(&self) -> Vec<u8> {
        use Instruction::*;
        match self {
            Nop => vec![self.opcode(), 0x00],

            MovI2R(value, dest) | MovI2RP(value, dest)
                => vec![self.opcode(), dest.as_dest(), value.get_byte(0), value.get_byte(1)],

            MovR2R(src, dest) | MovR2RP(src, dest) | MovRP2R(src, dest)
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
            MovR2R(src, _) => case!(src, 0x05),
            MovR2RP(src, _) => case!(src, 0x07),
            MovRP2R(_, dest) => case!(dest, 0x09),
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
            }
        };

        ($ident:ident, $($param:expr, $bytes:expr);+) => {
            #[test]
            fn $ident() {
                $(
                    let instr = Instruction::$ident($param).unwrap();
                    let bytes = instr.compile();
                    assert_eq!(bytes, $bytes);
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
        movr2r,
        Register::Rb0, Register::Rb1, [0x05, 0x10];
        Register::R0, Register::R1, [0x06, 0x10]
    );
    test_case!(
        movr2rp,
        Register::Rb0, Register::R1, [0x07, 0x10];
        Register::R0, Register::R1, [0x08, 0x10]
    );
    test_case!(
        movrp2r,
        Register::R0, Register::Rb1, [0x09, 0x10];
        Register::R0, Register::R1, [0x0A, 0x10]
    );
}
