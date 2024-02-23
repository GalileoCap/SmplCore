#![allow(unused_braces)]
use std::collections::HashMap;

#[allow(unused_imports)]
use common::{prelude::*, ParamIdx};
use crate::{parse, GroupDelim, Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Label(String),

    Nop,
    Mov(Token, Token),
}

macro_rules! to_instructions {
    (
        fn $ident:ident($left:ident : $left_type:ident, $right:ident : $right_type:ident, $ctx:ident)
            $match:ident, $first:ident
            {$on_i:expr} {$on_ip:expr} {$on_r:expr} {$on_rp:expr}
    ) => {
        fn $ident($left : &$left_type, $right : &$right_type, $ctx : &mut CompileContext) -> Result<Vec<Instruction>> {
            match $match {
                Token::Number($match) => { $on_i },
                Token::Ident(ident) => {
                    if let Some($match) = Register::from(ident) {
                        $on_r
                    } else {
                        $ctx.label_refs.push((ident.to_owned(), $ctx.instructions.len(), ParamIdx::$first));
                        let $match = &0;
                        $on_i
                    }
                },
                Token::Group(GroupDelim::Brack, toks) if toks.len() == 1 => {
                    match &toks[0] {
                        Token::Number($match) => { $on_ip },
                        Token::Ident(ident) => {
                            if let Some($match) = Register::from(ident) {
                                $on_rp
                            } else {
                                $ctx.label_refs.push((ident.to_owned(), $ctx.instructions.len(), ParamIdx::$first));
                                let $match = &0;
                                $on_ip
                            }
                        },
                        _ => todo!("{:?}", $match),
                    }
                }
                _ => todo!("{:?}", $match),
            }
        }
    };

    (
        fn $ident:ident($left:ident : $left_type:ident, $right:ident : $right_type:ident, $ctx:ident) FIRST
        {$on_i:expr} {$on_ip:expr} {$on_r:expr} {$on_rp:expr}
    ) => { to_instructions!(fn $ident($left : $left_type, $right : $right_type, $ctx) $left, FirstImm
            { $on_i } { $on_ip } { $on_r } { $on_rp }
    );};

    (
        fn $ident:ident($left:ident : $left_type:ident, $right:ident : $right_type:ident, $ctx:ident) SECOND
        {$on_i:expr} {$on_ip:expr} {$on_r:expr} {$on_rp:expr}
    ) => { to_instructions!(fn $ident($left : $left_type, $right : $right_type, $ctx) $right, SecondImm
            { $on_i } { $on_ip } { $on_r } { $on_rp }
    );};
}

impl Expr {
    pub fn to_instructions(&self, ctx : &mut CompileContext) -> Result<Vec<Instruction>> {
        use Expr::*;
        match self {
            Label(_) => Ok(vec![]), // TODO: Error, panic?
            Nop => Ok(vec![Instruction::nop()?]),
            Mov(src, dest) => Self::mov(src, dest, ctx),
        }
    }

    to_instructions!(
        fn mov(left : Token, right : Token, ctx) FIRST 
            { Self::movi2x(left, right, ctx) }
            { Self::movip2x(left, right, ctx) }
            { Self::movr2x(&left, right, ctx) }
            { Self::movrp2x(&left, right, ctx) }
    );
    
    to_instructions!(
        fn movi2x(left : u64, right : Token, ctx) SECOND
        { Err(Error::UnexpectedToken("movi2x".to_string(), format!("{right:?}"))) }
        { Ok(vec![Instruction::movi2ip(
            Immediate::new(Width::smallest_that_fits(*left), *left)?,
            Immediate::new(Width::Word, *right)?,
        )?]) }
        { Ok(vec![Instruction::movi2r(Immediate::new(right.width(), *left)?, right)?]) }
        { Ok(vec![Instruction::movi2rp(Immediate::new(Width::smallest_that_fits(*left), *left)?, right)?]) }
    );
 
    to_instructions!(
        fn movip2x(left : u64, right : Token, ctx) SECOND
        { Err(Error::UnexpectedToken("movip2x".to_string(), format!("{right:?}"))) }
        { Ok(vec![Instruction::movip2ip(
            Immediate::new(Width::Word, *left)?,
            Immediate::new(Width::Word, *right)?,
        )?]) }
        { Ok(vec![Instruction::movip2r(Immediate::new(Width::Word, *left)?, right)?]) }
        { Ok(vec![Instruction::movip2rp(Immediate::new(Width::Word, *left)?, right)?]) }
    );

    to_instructions!(
        fn movr2x(left : Register, right : Token, ctx) SECOND
        { Err(Error::UnexpectedToken("movr2x".to_string(), format!("{right:?}"))) }
        { Ok(vec![Instruction::movr2ip(*left, Immediate::new(Width::Word, *right)?)?]) }
        { Ok(vec![Instruction::movr2r(*left, right)?]) }
        { Ok(vec![Instruction::movr2rp(*left, right)?]) }
    );

    to_instructions!(
        fn movrp2x(left : Register, right : Token, ctx) SECOND
        { Err(Error::UnexpectedToken("movrp2x".to_string(), format!("{right:?}"))) }
        { Ok(vec![Instruction::movrp2ip(*left, Immediate::new(Width::Word, *right)?)?]) }
        { Ok(vec![Instruction::movrp2r(*left, right)?]) }
        { Ok(vec![Instruction::movrp2rp(*left, right)?]) }
    );
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CompileContext {
    label_defs : HashMap<String, usize>,
    label_refs : Vec<(String, usize, ParamIdx)>,
    instructions : Vec<Instruction>,
}

pub fn compile_to_instructions(code : &str) -> Result<Vec<Instruction>> {
    let mut ctx = CompileContext::default();
    for expr in parse(code)?.into_iter() {
        if let Expr::Label(label) = expr {
            ctx.label_defs.insert(label, ctx.instructions.len());
        } else {
            let mut instructions = expr.to_instructions(&mut ctx)?;
            ctx.instructions.append(&mut instructions);
        }
    }

    let mut offsets = Vec::new();
    let mut accum = 0;
    for instruction in ctx.instructions.iter() {
        offsets.push(accum);
        accum += instruction.len();
    }

    for (label, instruction_idx, param_idx) in ctx.label_refs {
        let Some(offset_idx) = ctx.label_defs.get(&label) else { return Err(Error::LabelNotDefined(label.to_owned())) };
        let offset = offsets[*offset_idx];
        ctx.instructions[instruction_idx] = ctx.instructions[instruction_idx].clone()
                                                .replace_imm(param_idx, offset as u64)?;
    }

    Ok(ctx.instructions)
}

pub fn compile(code : &str) -> Result<Vec<u8>> {
    compile_to_instructions(code)
        .map(|instructions|
            instructions.into_iter()
                .flat_map(|instruction| instruction.compile())
                .collect()
        )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn nop() {
        let code = "nop";
        let instructions = compile_to_instructions(code);
        assert_eq!(instructions, Ok(
            vec![Instruction::nop().unwrap()],
        ))
    }

    #[test]
    fn mov() {
        let cases = vec![
            ("mov 0x60, rb0", Ok(vec![Instruction::movi2r(Immediate::byte(0x60), Register::Rb0).unwrap()])),
            ("mov 0x600D, r0", Ok(vec![Instruction::movi2r(Immediate::word(0x600D), Register::R0).unwrap()])),
            ("nop\nlabel: mov label, r0", Ok(vec![Instruction::nop().unwrap(), Instruction::movi2r(Immediate::word(0x0002), Register::R0).unwrap()])),
            ("mov 0x600D, [r0]", Ok(vec![Instruction::movi2rp(Immediate::word(0x600D), Register::R0).unwrap()])),
            ("mov 0x600D, [0xF337]", Ok(vec![Instruction::movi2ip(Immediate::word(0x600D), Immediate::word(0xF337)).unwrap()])),
            ("nop\nlabel: mov 0x600D, [label]", Ok(vec![Instruction::nop().unwrap(), Instruction::movi2ip(Immediate::word(0x600D), Immediate::word(0x0002)).unwrap()])),

            ("mov [0x600D], rb0", Ok(vec![Instruction::movip2r(Immediate::word(0x600D), Register::Rb0).unwrap()])),
            ("mov [0x600D], r0", Ok(vec![Instruction::movip2r(Immediate::word(0x600D), Register::R0).unwrap()])),
            ("nop\nlabel: mov [label], r0", Ok(vec![Instruction::nop().unwrap(), Instruction::movip2r(Immediate::word(0x0002), Register::R0).unwrap()])),
            ("mov [0x600D], [r0]", Ok(vec![Instruction::movip2rp(Immediate::word(0x600D), Register::R0).unwrap()])),
            ("mov [0x600D], [0xF337]", Ok(vec![Instruction::movip2ip(Immediate::word(0x600D), Immediate::word(0xF337)).unwrap()])),
            ("nop\nlabel: mov [0x600D], [label]", Ok(vec![Instruction::nop().unwrap(), Instruction::movip2ip(Immediate::word(0x600D), Immediate::word(0x0002)).unwrap()])),
            
            ("mov rb0, rb1", Ok(vec![Instruction::movr2r(Register::Rb0, Register::Rb1).unwrap()])),
            ("mov r0, r1", Ok(vec![Instruction::movr2r(Register::R0, Register::R1).unwrap()])),
            ("mov r0, [r1]", Ok(vec![Instruction::movr2rp(Register::R0, Register::R1).unwrap()])),
            ("mov r0, [0x600D]", Ok(vec![Instruction::movr2ip(Register::R0, Immediate::word(0x600D)).unwrap()])),
            ("nop\nlabel: mov r0, [label]", Ok(vec![Instruction::nop().unwrap(), Instruction::movr2ip(Register::R0, Immediate::word(0x0002)).unwrap()])),

            ("mov [r0], rb1", Ok(vec![Instruction::movrp2r(Register::R0, Register::Rb1).unwrap()])),
            ("mov [r0], r1", Ok(vec![Instruction::movrp2r(Register::R0, Register::R1).unwrap()])),
            ("mov [r0], [r1]", Ok(vec![Instruction::movrp2rp(Register::R0, Register::R1).unwrap()])),
            ("mov [r0], [0x600D]", Ok(vec![Instruction::movrp2ip(Register::R0, Immediate::word(0x600D)).unwrap()])),
            ("nop\nlabel: mov [r0], [label]", Ok(vec![Instruction::nop().unwrap(), Instruction::movrp2ip(Register::R0, Immediate::word(0x0002)).unwrap()])),
        ];

        for (code, expect) in cases.into_iter() {
            let instructions = compile_to_instructions(code);
            assert_eq!(instructions, expect, "\"{code}\"");
        }
    }
}
