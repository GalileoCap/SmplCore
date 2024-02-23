use std::collections::HashMap;

#[allow(unused_imports)]
use common::{prelude::*, ParamIdx};
use crate::{parse, Expr};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CompileContext {
    pub label_defs : HashMap<String, usize>,
    pub label_refs : Vec<(String, usize, ParamIdx)>,
    pub instructions : Vec<Instruction>,
}

pub fn compile_to_context(code : &str) -> Result<CompileContext> {
    let mut ctx = CompileContext::default();
    for expr in parse(code)?.into_iter() {
        if let Expr::Label(label) = expr {
            ctx.label_defs.insert(label, ctx.instructions.len());
        } else {
            let mut instructions = expr.to_instructions(&mut ctx)?;
            ctx.instructions.append(&mut instructions);
        }
    }
    Ok(ctx)
}

fn calc_label_offsets(ctx : &CompileContext) -> Vec<u16> {
    let mut offsets = Vec::new();
    let mut accum = 0;
    for instruction in ctx.instructions.iter() {
        offsets.push(accum);
        accum += instruction.len();
    }
    offsets
}

pub fn compile_to_instructions(code : &str) -> Result<Vec<Instruction>> {
    let mut ctx = compile_to_context(code)?;
    let offsets = calc_label_offsets(&ctx);
    
    // Replace labels
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

    #[test]
    fn comment() {
        let cases = vec![
            ("/* NOPE */ nop /* NOPE */\n/* NOPE */ label: /* NOPE */ mov /* NOPE */ 0x600D /* NOPE */, /* NOPE */[/* NOPE */ label /* NOPE */] /* NOPE */ // NOPE /* NOPE */ // NOPE\nnop", Ok(vec![Instruction::nop().unwrap(), Instruction::movi2ip(Immediate::word(0x600D), Immediate::word(0x0002)).unwrap(), Instruction::nop().unwrap()])),
        ];

        for (code, expect) in cases.into_iter() {
            let instructions = compile_to_instructions(code);
            assert_eq!(instructions, expect, "\"{code}\"");
        }
    }
}
