#[derive(Copy, PartialEq, Clone, Debug)]
pub(crate) enum Token {
    Number(i64),
    Plus,
    Minus,
    Multiply,
    Divide,
    LParen,
    RParen,
    Percent,
    Period,
    Caret,
    Exclamation,
    Eof, // equal sign or newline
    Nop, // whitespace
}

pub(crate) const LOWEST: u8 = 0;
pub(crate) const NOP: u8 = 1;
pub(crate) const EOF: u8 = 2;
pub(crate) const ADD: u8 = 10;
pub(crate) const MULTIPLY: u8 = 20;
pub(crate) const PREFIX: u8 = 30;
pub(crate) const EXPONENT: u8 = 40;

impl Token {
    pub(crate) fn precedence(&self) -> u8 {
        match self {
            Token::Nop => NOP,
            Token::Eof => EOF,
            Token::Plus | Token::Minus => ADD,
            Token::Multiply | Token::Divide | Token::Percent => MULTIPLY,
            Token::Caret => EXPONENT,
            _ => LOWEST,
        }
    }
}
