#[allow(unused_imports)]
use common::prelude::*;
use crate::{tokenize, Token, Expr, Scanner};

fn parse_two_params(cb : impl FnOnce(Token, Token) -> Expr, toks : &mut Scanner<Token>, ctx : String) -> Result<Expr> {
    let Some(t1) = toks.pop() else { return Err(Error::MissingToken(ctx)) };
    let Some(comma) = toks.pop() else { return Err(Error::MissingToken(ctx)) };
    if comma != Token::Punct(',') { return Err(Error::UnexpectedToken(ctx, format!("{comma:?}"))); }
    let Some(t2) = toks.pop() else { return Err(Error::MissingToken(ctx)) };

    Ok(cb(t1, t2))
}

fn parse_instruction(ident : String, toks : &mut Scanner<Token>) -> Result<Expr> {
    match &*ident {
        "nop" => Ok(Expr::Nop),
        "mov" => parse_two_params(Expr::Mov, toks, ident),

        _ => Err(Error::UnknownInstruction(ident)),
    }
}

fn parse_toks(t : Token, toks : &mut Scanner<Token>) -> Result<Expr> {
    match t {
        Token::Ident(ident) => {
            if toks.take(|c| *c == Token::Punct(':')).is_some() {
                Ok(Expr::Label(ident))
            } else {
                parse_instruction(ident, toks)
            }
        }

        Token::Comment(_) => unreachable!(),
        _ => Err(Error::UnexpectedToken("parse_toks".to_string(), format!("{t:?}"))),
    }
}

pub fn parse(code : &str) -> Result<Vec<Expr>> {
    let toks = tokenize(code);
    let mut toks = Scanner::new(toks);

    let mut exprs = Vec::new();
    while let Some(t) = toks.pop() {
        let expr = parse_toks(t, &mut toks)?;
        exprs.push(expr);
    }
    Ok(exprs)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn label() {
        let code = "a_label: nop";
        let exprs = parse(code);
        assert_eq!(exprs, Ok(vec![
            Expr::Label("a_label".to_string()),
            Expr::Nop,
        ]));
    }

    #[test]
    fn nop() {
        let code = "nop";
        let exprs = parse(code);
        assert_eq!(exprs, Ok(vec![
            Expr::Nop,
        ]));
    }

    #[test]
    fn mov() {
        let code = "mov 0x600D, r0";
        let exprs = parse(code);
        assert_eq!(exprs, Ok(vec![
            Expr::Mov(Token::Number(0x600D), Token::Ident("r0".to_string())),
        ]));
    }
}
