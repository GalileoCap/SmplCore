#[allow(unused_imports)]
use common::{prelude::*, ParamIdx};
use crate::{CompileContext, GroupDelim, Token};

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
        fn movi2x(left : u16, right : Token, ctx) SECOND
        { Err(Error::UnexpectedToken("movi2x".to_string(), format!("{right:?}"))) }
        { Ok(vec![Instruction::movi2ip(
            Immediate::new(Width::smallest_that_fits(*left), *left)?,
            Immediate::new(Width::Word, *right)?,
        )?]) }
        { Ok(vec![Instruction::movi2r(Immediate::new(right.width(), *left)?, right)?]) }
        { Ok(vec![Instruction::movi2rp(Immediate::new(Width::smallest_that_fits(*left), *left)?, right)?]) }
    );
 
    to_instructions!(
        fn movip2x(left : u16, right : Token, ctx) SECOND
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
