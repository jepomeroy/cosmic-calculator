#[derive(Copy, PartialEq, Clone, Debug)]
pub(crate) enum FunctionType {
    Log,    // Log10
    Ln,     // Natural Log
    LogTwo, // Log base 2
    SqRt,   // Square Root
    CbRt,   // Cube Root
    Abs,    // absolute value
}

#[derive(Copy, PartialEq, Clone, Debug)]
pub(crate) enum Token {
    Number(f64),
    Plus,
    Minus,
    Multiply,
    Divide,
    LParen,
    RParen,
    Percent,
    Caret,
    Exclamation,
    Function(FunctionType),
    Eof, // equal sign or newline
}

pub(crate) const LOWEST: u8 = 0;
pub(crate) const EOF: u8 = 1;
pub(crate) const ADD: u8 = 10;
pub(crate) const MULTIPLY: u8 = 20;
pub(crate) const PREFIX: u8 = 30;
pub(crate) const EXPONENT: u8 = 40;
pub(crate) const PARENTHETICAL: u8 = 50;

impl Token {
    pub(crate) fn precedence(&self) -> u8 {
        match self {
            Token::Eof => EOF,
            Token::Plus | Token::Minus => ADD,
            Token::Multiply | Token::Divide | Token::Percent | Token::Exclamation => MULTIPLY,
            Token::Caret => EXPONENT,
            Token::LParen => PARENTHETICAL,
            _ => LOWEST,
        }
    }
}
