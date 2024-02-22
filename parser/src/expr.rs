#[allow(unused_imports)]
use common::prelude::*;
use crate::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Label(String),

    Nop,
    Mov(Token, Token),
}

#[cfg(test)]
mod test {
    use super::*;
}
