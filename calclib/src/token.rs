#[derive(Copy, PartialEq, Clone, Debug)]
pub(crate) enum FunctionType {
    Log,    // Log10
    Ln,     // Natural Log
    LogTwo, // Log base 2
    SqRt,   // Square Root
    CbRt,   // Cube Root
    Abs,    // absolute value
    Not,    // bitwise not
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
    Caret,
    Exclamation,
    And,    // AND
    Or,     // OR
    Nand,   // NAND
    Nor,    // Nor
    Xor,    // XOR
    Xnor,   // XNOR
    Lshift, // LSHIFT
    Rshift, // RSHIFT
    Mod,    // MOD
    Function(FunctionType),
    Eof, // equal sign or newline
}

pub(crate) const LOWEST: u8 = 0;
pub(crate) const EOF: u8 = 1;
pub(crate) const BIT: u8 = 5;
pub(crate) const ADD: u8 = 10;
pub(crate) const MULTIPLY: u8 = 20;
pub(crate) const PREFIX: u8 = 30;
pub(crate) const EXPONENT: u8 = 40;
pub(crate) const PARENTHETICAL: u8 = 50;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitwise_tokens_have_bit_precedence() {
        let tokens = vec![
            Token::And,
            Token::Or,
            Token::Nand,
            Token::Nor,
            Token::Xor,
            Token::Xnor,
            Token::Lshift,
            Token::Rshift,
            Token::Mod,
        ];
        for token in tokens {
            assert_eq!(
                token.precedence(),
                BIT,
                "{:?} should have BIT precedence",
                token
            );
        }
    }

    #[test]
    fn test_arithmetic_tokens_have_correct_precedence() {
        assert_eq!(Token::Plus.precedence(), ADD);
        assert_eq!(Token::Minus.precedence(), ADD);
        assert_eq!(Token::Multiply.precedence(), MULTIPLY);
        assert_eq!(Token::Divide.precedence(), MULTIPLY);
        assert_eq!(Token::Caret.precedence(), EXPONENT);
        assert_eq!(Token::LParen.precedence(), PARENTHETICAL);
        assert_eq!(Token::Eof.precedence(), EOF);
    }

    #[test]
    fn test_bitwise_precedence_is_lower_than_arithmetic() {
        assert!(BIT < ADD);
        assert!(BIT < MULTIPLY);
        assert!(BIT < EXPONENT);
    }
}

impl Token {
    pub(crate) fn precedence(&self) -> u8 {
        match self {
            Token::Eof => EOF,
            Token::And
            | Token::Or
            | Token::Nand
            | Token::Nor
            | Token::Xor
            | Token::Xnor
            | Token::Lshift
            | Token::Rshift
            | Token::Mod => BIT,
            Token::Plus | Token::Minus => ADD,
            Token::Multiply | Token::Divide | Token::Exclamation => MULTIPLY,
            Token::Caret => EXPONENT,
            Token::LParen => PARENTHETICAL,
            _ => LOWEST,
        }
    }
}
