mod scanner;
pub use scanner::*;

#[allow(unused_imports)]
use common::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
}

fn skip_whitespace(scanner : &mut Scanner<char>) -> usize {
    scanner.take_while(|c| c.is_whitespace()).len()
}

fn match_identifier(scanner : &mut Scanner<char>) -> Option<Token> {
    scanner.test(|c| c.is_alphabetic() || *c == '_')
        .then(||
            scanner.take_while(|c| c.is_alphanumeric() || *c == '_')
                .into_iter().collect()
        ).map(Token::Ident)
}

fn get_token(scanner : &mut Scanner<char>) -> Option<Token> {
    skip_whitespace(scanner);

    match_identifier(scanner)
}

pub fn tokenize(code : &str) -> Vec<Token> {
    let mut scanner = Scanner::new(code.chars().collect());
    let mut toks = Vec::new();
    while let Some(t) = get_token(&mut scanner) {
        toks.push(t)
    }
    toks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ident() {
        let code = " a ab  ab1   a1b   ab_    a_b     _ab ";
        let toks = tokenize(code);
        let res : Vec<_> = code.split_whitespace().map(|s| Token::Ident(s.to_string())).collect();
        assert_eq!(toks, res);
    }
}
