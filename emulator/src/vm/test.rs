#[allow(unused_imports)]
use common::prelude::*;

use super::*;

macro_rules! case {
    ($ident:ident, $code:expr, $reps:literal, $regs:expr, $mem:expr) => {
        #[test]
        fn $ident() {
            let rom = $code.into_iter().flat_map(|instr| instr.unwrap().compile()).collect();
            let mut vm = VM::new(rom, 0x8000);

            vm.boot();
            for _ in 0..$reps {
                vm.execute_next().unwrap();
            }

            assert_eq!(vm.regs.iter().map(|reg| (*reg).into()).collect::<Vec<u16>>(), $regs);
            for (addr, value) in $mem.into_iter() {
                assert_eq!(vm.get_mem_byte(addr), value, "at {addr:#06X}");
            }
        }
    };

    ($ident:ident, $code:expr, $reps:literal, $regs:expr) => {
        case!($ident, $code, $reps, $regs, []);
    };
}

case!(nop, [Instruction::nop()], 1, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x02, 0]);
case!(movi2r, [
    Instruction::movi2r(Immediate::byte(0x60), Register::rb0()),
    Instruction::movi2r(Immediate::word(0x600D), Register::r1()),
], 2, [0x60, 0x600D, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x08, 0]);
case!(
    movi2rp,
    [
        Instruction::movi2r(Immediate::word(0x8000), Register::r0()),
        Instruction::movi2r(Immediate::word(0x8002), Register::r1()),
        Instruction::movi2rp(Immediate::byte(0x60), Register::r0()),
        Instruction::movi2rp(Immediate::word(0x600D), Register::r1()),
    ],
    4,
    [0x8000, 0x8002, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x10, 0],
    [(0x8000, 0x60), (0x8001, 0x00), (0x8002, 0x0D), (0x8003, 0x60)]
);
case!(
    movi2ip,
    [
        Instruction::movi2ip(Immediate::byte(0x60), Immediate::word(0xF337)),
        Instruction::movi2ip(Immediate::word(0x600D), Immediate::word(0xF335)),
    ],
    2,
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x0C, 0],
    [(0xF335, 0x0D), (0xF336, 0x60), (0xF337, 0x60)]
);
case!(movip2r, [
    Instruction::movi2ip(Immediate::word(0x600D), Immediate::word(0xF337)),
    Instruction::movip2r(Immediate::word(0xF337), Register::rb0()),
    Instruction::movip2r(Immediate::word(0xF337), Register::r1()),
], 3, [0x0D, 0x600D, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xE, 0]);
case!(
    movip2rp, [
        Instruction::movi2ip(Immediate::word(0x600D), Immediate::word(0xF337)),
        Instruction::movi2r(Immediate::word(0xF338), Register::r0()),
        Instruction::movip2rp(Immediate::word(0xF337), Register::r0()),
    ],
    3,
    [0xF338, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xE, 0],
    [(0xF337, 0x0D), (0xF338, 0x0D)]
);
case!(
    movip2ip, [
        Instruction::movi2ip(Immediate::word(0x600D), Immediate::word(0xF337)),
        Instruction::movip2ip(Immediate::word(0xF337), Immediate::word(0xF338)),
    ],
    2,
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xC, 0],
    [(0xF337, 0x0D), (0xF338, 0x60)]
);
case!(movr2r, [
    Instruction::movi2r(Immediate::byte(0x60), Register::rb0()),
    Instruction::movi2r(Immediate::word(0x600D), Register::r1()),
    Instruction::movi2r(Immediate::word(0xF337), Register::r2()),
    Instruction::movi2r(Immediate::word(0xB007), Register::r3()),
    Instruction::movr2r(Register::rb0(), Register::rb0()),
    Instruction::movr2r(Register::r1(), Register::r1()),
    Instruction::movr2r(Register::rb0(), Register::rb2()),
    Instruction::movr2r(Register::r1(), Register::r3()),
], 8, [0x60, 0x600D, 0xF360, 0x600D, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x18, 0]);
case!(
    movr2rp, [
        Instruction::movi2r(Immediate::word(0x600D), Register::r0()),
        Instruction::movi2r(Immediate::word(0xF337), Register::r1()),
        Instruction::movi2r(Immediate::word(0xF338), Register::r2()),
        Instruction::movr2rp(Register::rb0(), Register::r1()),
        Instruction::movr2rp(Register::r0(), Register::r2()),
    ],
    5,
    [0x600D, 0xF337, 0xF338, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x10, 0],
    [(0xF337, 0x0D), (0xF338, 0x0D), (0xF339, 0x60)]
);
case!(
    movr2ip, [
        Instruction::movi2r(Immediate::word(0x600D), Register::r0()),
        Instruction::movr2ip(Register::rb0(), Immediate::word(0xF337)),
        Instruction::movr2ip(Register::r0(), Immediate::word(0xF338)),
    ],
    3,
    [0x600D, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xC, 0],
    [(0xF337, 0x0D), (0xF338, 0x0D), (0xF339, 0x60)]
);
case!(
    movrp2r,
    [
        Instruction::movi2r(Immediate::word(0xF337), Register::r0()),
        Instruction::movi2ip(Immediate::word(0x600D), Immediate::word(0xF337)),
        Instruction::movrp2r(Register::r0(), Register::rb1()),
        Instruction::movrp2r(Register::r0(), Register::r2()),
    ],
    4,
    [0xF337, 0x0D, 0x600D, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xE, 0],
    [(0xF337, 0x0D), (0xF338, 0x60)]
);
case!(
    movrp2rp,
    [
        Instruction::movi2r(Immediate::word(0xF337), Register::r0()),
        Instruction::movi2r(Immediate::word(0xF339), Register::r1()),
        Instruction::movi2ip(Immediate::word(0x600D), Immediate::word(0xF337)),
        Instruction::movrp2rp(Register::r0(), Register::r1()),
    ],
    4,
    [0xF337, 0xF339, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x10, 0],
    [(0xF337, 0x0D), (0xF338, 0x60), (0xF339, 0x0D), (0xF33A, 0)]
);
case!(
    movrp2ip,
    [
        Instruction::movi2r(Immediate::word(0xF337), Register::r0()),
        Instruction::movi2ip(Immediate::word(0xF339), Immediate::word(0xF337)),
        Instruction::movrp2ip(Register::r0(), Immediate::word(0xF337)),
    ],
    3,
    [0xF337, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x0E, 0],
    [(0xF337, 0x39), (0xF338, 0xF3), (0xF339, 0x39), (0xF33A, 0)]
);
