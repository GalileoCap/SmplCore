use std::collections::HashMap;

#[allow(unused_imports)]
use common::prelude::*;
use crate::{parse, GroupDelim, Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Label(String),

    Nop,
    Mov(Token, Token),
}

impl Expr {
    pub fn to_instructions(&self, ctx : &CompileContext) -> Result<Vec<Instruction>> {
        use Expr::*;
        match self {
            Label(_) => Ok(vec![]), // TODO: Error, panic?
            Nop => Ok(vec![Instruction::Nop]),
            Mov(src, dest) => Self::mov(src, dest, ctx),
        }
    }
    
    fn mov(src : &Token, dest : &Token, ctx : &CompileContext) -> Result<Vec<Instruction>> {
        match src {
            Token::Number(value) => Self::movi2x(value, dest, ctx),

            Token::Group(GroupDelim::Brack, toks) if toks.len() == 1 && toks[0].is_number() => {
                let Token::Number(src) = &toks[0] else { unreachable!() };
                Self::movip2x(src, dest, ctx)
            }

            _ => Err(Error::UnexpectedToken("mov".to_string(), format!("{src:?}"))),
        }
    }

    fn movi2x(value : &u64, dest : &Token, _ctx : &CompileContext) -> Result<Vec<Instruction>> {
        match dest {
            Token::Ident(ident)
                => if let Some(reg) = Register::from(ident) {
                    Ok(vec![Instruction::MovI2R(Immediate::new(reg.width(), *value)?, reg)])
                } else {
                    todo!("{ident}")
                },

            Token::Group(GroupDelim::Brack, toks) if toks.len() == 1 && toks[0].is_ident() => {
                let Token::Ident(ident) = &toks[0] else { unreachable!() };
                if let Some(reg) = Register::from(&ident) {
                    Ok(vec![Instruction::MovI2RP(Immediate::new(Width::smallest_that_fits(*value), *value)?, reg)])
                } else {
                    todo!("{ident}")
                }
            }

            Token::Group(GroupDelim::Brack, toks) if toks.len() == 1 && toks[0].is_number() => {
                let Token::Number(dest) = &toks[0] else { unreachable!() };
                Ok(vec![Instruction::MovI2IP(
                    Immediate::new(Width::smallest_that_fits(*value), *value)?,
                    Immediate::new(Width::Word, *dest)?,
                )])
            },

            _ => Err(Error::UnexpectedToken("movi2x".to_string(), format!("{dest:?}"))),
        }
    }

    fn movip2x(value : &u64, dest : &Token, _ctx : &CompileContext) -> Result<Vec<Instruction>> {
        match dest {
            Token::Ident(ident) if Register::from(ident).is_some()
                => Ok(vec![Instruction::MovIP2R(Immediate::new(Width::Word, *value)?, Register::from(ident).unwrap())]),

            Token::Group(GroupDelim::Brack, toks) if toks.len() == 1 && toks[0].is_ident() => {
                let Token::Ident(ident) = &toks[0] else { unreachable!() };
                if let Some(reg) = Register::from(&ident) {
                    Ok(vec![Instruction::MovIP2RP(Immediate::new(Width::Word, *value)?, reg)])
                } else {
                    todo!("{ident}")
                }
            }

            Token::Group(GroupDelim::Brack, toks) if toks.len() == 1 && toks[0].is_number() => {
                let Token::Number(dest) = &toks[0] else { unreachable!() };
                Ok(vec![Instruction::MovIP2IP(
                    Immediate::new(Width::Word, *value)?,
                    Immediate::new(Width::Word, *dest)?,
                )])
            },

            _ => Err(Error::UnexpectedToken("movi2x".to_string(), format!("{dest:?}"))),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CompileContext {
    labels : HashMap<String, usize>,
}

pub fn compile_to_instructions(code : &str) -> Result<Vec<Instruction>> {
    let mut instructions = Vec::new();

    let mut ctx = CompileContext::default();
    for expr in parse(code)?.into_iter() {
        if let Expr::Label(label) = expr {
            ctx.labels.insert(label, instructions.len());
        } else {
            instructions.append(&mut expr.to_instructions(&ctx)?);
        }
    }

    Ok(instructions)
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
            vec![Instruction::Nop],
        ))
    }

    #[test]
    fn mov() {
        let cases = vec![
            ("mov 0x60, rb0", Ok(vec![Instruction::MovI2R(Immediate::byte(0x60), Register::Rb0)])),
            ("mov 0x600D, r0", Ok(vec![Instruction::MovI2R(Immediate::word(0x600D), Register::R0)])),
            // ("label: mov label, rb0", Ok(vec![Instruction::MovI2R(Immediate::byte(0x00), Register::Rb0)])),
            // ("label: mov label, r0", Ok(vec![Instruction::MovI2R(Immediate::word(0x0000), Register::R0)])),
            ("mov 0x60, [r0]", Ok(vec![Instruction::MovI2RP(Immediate::byte(0x60), Register::R0)])),
            ("mov 0x600D, [r0]", Ok(vec![Instruction::MovI2RP(Immediate::word(0x600D), Register::R0)])),
            ("mov 0x60, [0xF337]", Ok(vec![Instruction::MovI2IP(Immediate::byte(0x60), Immediate::word(0xF337))])),
            ("mov 0x600D, [0xF337]", Ok(vec![Instruction::MovI2IP(Immediate::word(0x600D), Immediate::word(0xF337))])),
            // ("label: mov 0x60, [label]", Ok(vec![Instruction::MovI2IP(Immediate::byte(0x60), Immediate::word(0x0000))])),
            // ("label: mov 0x600D, [label]", Ok(vec![Instruction::MovI2IP(Immediate::word(0x600D), Immediate::word(0x0000))])),
            ("mov [0x600D], rb0", Ok(vec![Instruction::MovIP2R(Immediate::word(0x600D), Register::Rb0)])),
            ("mov [0x600D], r0", Ok(vec![Instruction::MovIP2R(Immediate::word(0x600D), Register::R0)])),
            // ("label: mov [label], rb0", Ok(vec![Instruction::MovIP2R(Immediate::word(0x0000), Register::Rb0)])),
            // ("label: mov [label], r0", Ok(vec![Instruction::MovIP2R(Immediate::word(0x0000), Register::Rb0)])),
            ("mov [0x600D], [r0]", Ok(vec![Instruction::MovIP2RP(Immediate::word(0x600D), Register::R0)])),
            ("mov [0x600D], [0xF337]", Ok(vec![Instruction::MovIP2IP(Immediate::word(0x600D), Immediate::word(0xF337))])),
            // ("label: mov [0x600D], [label]", Ok(vec![Instruction::MovIP2IP(Immediate::word(0x600D), Immediate::word(0x0000))])),
            // ("label: mov [0x600D], [label]", Ok(vec![Instruction::MovIP2IP(Immediate::word(0x600D), Immediate::word(0x0000))])),
        ];

        for (code, expect) in cases.into_iter() {
            let instructions = compile_to_instructions(code);
            assert_eq!(instructions, expect);
        }
    }
}
