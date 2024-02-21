mod scanner;
pub use scanner::*;

#[allow(unused_imports)]
use common::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Number(u64),
    Punct(char),
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

fn match_number(scanner : &mut Scanner<char>) -> Option<Token> {
    if scanner.test(|c| c.is_ascii_digit() || *c == '-') { // TODO: is_numeric?
        scanner.scan(|chars| match chars {
            ['-'] => ScannerAction::Request(Token::Punct('-')),
            ['-', digits @ ..] if digits.iter().all(|c| c.is_ascii_digit())
                => ScannerAction::Request(Token::Number(
                    -digits.iter().collect::<String>().parse::<i64>().unwrap() as u64
                    // TODO: Don't lose sign
                )),
            ['0'] => ScannerAction::Request(Token::Number(0)),

            ['0', 'x'] => ScannerAction::Require,
            ['0', 'x', digits @ ..] if digits.iter().all(|c| c.is_ascii_hexdigit())
                => ScannerAction::Request(Token::Number(
                    u64::from_str_radix(&digits.iter().collect::<String>(), 16).unwrap()
                )),

            ['0', 'o'] => ScannerAction::Require,
            ['0', 'o', digits @ ..] if digits.iter().all(|c| c.is_digit(8))
                => ScannerAction::Request(Token::Number(
                    u64::from_str_radix(&digits.iter().collect::<String>(), 8).unwrap()
                )),

            ['0', 'b'] => ScannerAction::Require,
            ['0', 'b', digits @ ..] if digits.iter().all(|c| c.is_digit(2))
                => ScannerAction::Request(Token::Number(
                    u64::from_str_radix(&digits.iter().collect::<String>(), 2).unwrap()
                )),

            _ if chars.iter().all(|c| c.is_ascii_digit())
                => ScannerAction::Request(Token::Number(chars.iter().collect::<String>().parse().unwrap())),

            _ => ScannerAction::None,
        }).unwrap() // TODO: Handle
    } else { None }
}

fn match_punct(scanner : &mut Scanner<char>) -> Option<Token> {
    scanner.take(|c| c.is_ascii_punctuation())
        .map(Token::Punct)
}

fn get_token(scanner : &mut Scanner<char>) -> Option<Token> {
    skip_whitespace(scanner);

    match_identifier(scanner)
    .or_else(|| match_number(scanner))
    .or_else(|| match_punct(scanner))
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

    #[test]
    fn number() {
        let code = "0 0x0 0o0 0b0 62263 -62263 0xF337 0o171467 0b1111001100110111";
        let toks = tokenize(code);
        assert_eq!(toks, vec![
            Token::Number(0),
            Token::Number(0x0),
            Token::Number(0o0),
            Token::Number(0b0),
            Token::Number(62263),
            Token::Number(-62263i64 as u64),
            Token::Number(0xF337),
            Token::Number(0o171467),
            Token::Number(0b1111001100110111),
        ]);
    }

    #[test]
    fn punct() {
        let code = "+ - * / % , ' \\";
        let toks = tokenize(code);
        assert_eq!(toks, vec![
            Token::Punct('+'),
            Token::Punct('-'),
            Token::Punct('*'),
            Token::Punct('/'),
            Token::Punct('%'),
            Token::Punct(','),
            Token::Punct('\''),
            Token::Punct('\\'),
        ]);
    }
}
