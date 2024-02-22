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

impl Expr {
    pub fn to_instructions(&self, ctx : &mut CompileContext) -> Result<Vec<Instruction>> {
        use Expr::*;
        match self {
            Label(_) => Ok(vec![]), // TODO: Error, panic?
            Nop => Ok(vec![Instruction::nop()?]),
            Mov(src, dest) => Self::mov(src, dest, ctx),
        }
    }
    
    fn mov(src : &Token, dest : &Token, ctx : &mut CompileContext) -> Result<Vec<Instruction>> {
        match src {
            Token::Number(value) => Self::movi2x(value, dest, ctx),
            Token::Ident(ident) if Register::from(ident).is_none() => {
                ctx.label_refs.push((ident.to_owned(), ctx.instructions.len(), ParamIdx::FirstImm));
                Self::movi2x(&0, dest, ctx)
            }

            Token::Group(GroupDelim::Brack, toks) if toks.len() == 1 && toks[0].is_number() => {
                let Token::Number(src) = &toks[0] else { unreachable!() };
                Self::movip2x(src, dest, ctx)
            }

            Token::Ident(ident) if Register::from(ident).is_some()
                => Self::movr2x(Register::from(ident).unwrap(), dest, ctx),

            Token::Group(GroupDelim::Brack, toks) if toks.len() == 1 && toks[0].is_ident() => {
                let Token::Ident(ident) = &toks[0] else { unreachable!() };
                if let Some(src) = Register::from(ident) {
                    Self::movrp2x(src, dest, ctx)
                } else {
                    ctx.label_refs.push((ident.to_owned(), ctx.instructions.len(), ParamIdx::FirstImm));
                    Self::movip2x(&0, dest, ctx)
                }
            },

            _ => Err(Error::UnexpectedToken("mov".to_string(), format!("{src:?}"))),
        }
    }

    fn movi2x(value : &u64, dest : &Token, ctx : &mut CompileContext) -> Result<Vec<Instruction>> {
        match dest {
            Token::Ident(ident) if Register::from(ident).is_some() => {
                let reg = Register::from(ident).unwrap();
                Ok(vec![Instruction::movi2r(Immediate::new(reg.width(), *value)?, reg)?])
            },

            Token::Group(GroupDelim::Brack, toks) if toks.len() == 1 && toks[0].is_ident() => {
                let Token::Ident(ident) = &toks[0] else { unreachable!() };
                if let Some(reg) = Register::from(&ident) {
                    Ok(vec![Instruction::movi2rp(Immediate::new(Width::smallest_that_fits(*value), *value)?, reg)?])
                } else {
                    ctx.label_refs.push((ident.to_owned(), ctx.instructions.len(), ParamIdx::SecondImm));
                    Ok(vec![Instruction::movi2ip(
                        Immediate::new(Width::smallest_that_fits(*value), *value)?,
                        Immediate::new(Width::Word, 0)?,
                    )?])
                }
            }

            Token::Group(GroupDelim::Brack, toks) if toks.len() == 1 && toks[0].is_number() => {
                let Token::Number(dest) = &toks[0] else { unreachable!() };
                Ok(vec![Instruction::movi2ip(
                    Immediate::new(Width::smallest_that_fits(*value), *value)?,
                    Immediate::new(Width::Word, *dest)?,
                )?])
            },

            _ => Err(Error::UnexpectedToken("movi2x".to_string(), format!("{dest:?}"))),
        }
    }

    fn movip2x(value : &u64, dest : &Token, ctx : &mut CompileContext) -> Result<Vec<Instruction>> {
        match dest {
            Token::Ident(ident) if Register::from(ident).is_some()
                => Ok(vec![Instruction::movip2r(Immediate::new(Width::Word, *value)?, Register::from(ident).unwrap())?]),

            Token::Group(GroupDelim::Brack, toks) if toks.len() == 1 && toks[0].is_ident() => {
                let Token::Ident(ident) = &toks[0] else { unreachable!() };
                if let Some(reg) = Register::from(&ident) {
                    Ok(vec![Instruction::movip2rp(Immediate::new(Width::Word, *value)?, reg)?])
                } else {
                    ctx.label_refs.push((ident.to_owned(), ctx.instructions.len(), ParamIdx::SecondImm));
                    Ok(vec![Instruction::movip2ip(
                        Immediate::new(Width::Word, *value)?,
                        Immediate::new(Width::Word, 0)?,
                    )?])
                }
            }

            Token::Group(GroupDelim::Brack, toks) if toks.len() == 1 && toks[0].is_number() => {
                let Token::Number(dest) = &toks[0] else { unreachable!() };
                Ok(vec![Instruction::movip2ip(
                    Immediate::new(Width::Word, *value)?,
                    Immediate::new(Width::Word, *dest)?,
                )?])
            },

            _ => Err(Error::UnexpectedToken("movip2x".to_string(), format!("{dest:?}"))),
        }
    }

    fn movr2x(src : Register, dest : &Token, ctx : &mut CompileContext) -> Result<Vec<Instruction>> {
        match dest {
            Token::Ident(ident) if Register::from(ident).is_some()
                => Ok(vec![Instruction::movr2r(src, Register::from(ident).unwrap())?]),

            Token::Group(GroupDelim::Brack, toks) if toks.len() == 1 && toks[0].is_ident() => {
                let Token::Ident(ident) = &toks[0] else { unreachable!() };
                if let Some(reg) = Register::from(&ident) {
                    Ok(vec![Instruction::movr2rp(src, reg)?])
                } else {
                    ctx.label_refs.push((ident.to_owned(), ctx.instructions.len(), ParamIdx::FirstImm));
                    Ok(vec![Instruction::movr2ip(src, Immediate::new(Width::Word, 0)?)?])
                }
            }

            Token::Group(GroupDelim::Brack, toks) if toks.len() == 1 && toks[0].is_number() => {
                let Token::Number(dest) = &toks[0] else { unreachable!() };
                Ok(vec![Instruction::movr2ip(src, Immediate::new(Width::Word, *dest)?)?])
            },

            _ => Err(Error::UnexpectedToken("movr2x".to_string(), format!("{dest:?}"))),
        }
    }

    fn movrp2x(src : Register, dest : &Token, ctx : &mut CompileContext) -> Result<Vec<Instruction>> {
        match dest {
            Token::Ident(ident) if Register::from(ident).is_some()
                => Ok(vec![Instruction::movrp2r(src, Register::from(ident).unwrap())?]),

            Token::Group(GroupDelim::Brack, toks) if toks.len() == 1 && toks[0].is_ident() => {
                let Token::Ident(ident) = &toks[0] else { unreachable!() };
                if let Some(reg) = Register::from(&ident) {
                    Ok(vec![Instruction::movrp2rp(src, reg)?])
                } else {
                    ctx.label_refs.push((ident.to_owned(), ctx.instructions.len(), ParamIdx::FirstImm));
                    Ok(vec![Instruction::movrp2ip(src, Immediate::new(Width::Word, 0)?)?])
                }
            }

            Token::Group(GroupDelim::Brack, toks) if toks.len() == 1 && toks[0].is_number() => {
                let Token::Number(dest) = &toks[0] else { unreachable!() };
                Ok(vec![Instruction::movrp2ip(src, Immediate::new(Width::Word, *dest)?)?])
            },

            _ => Err(Error::UnexpectedToken("movrp2x".to_string(), format!("{dest:?}"))),
        }
    }
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
