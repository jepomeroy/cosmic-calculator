#[derive(Copy, PartialEq, Clone, Debug)]
pub(crate) enum Token {
    Number(u32),
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
