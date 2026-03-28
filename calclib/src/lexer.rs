use crate::error::CalcLibError;
use crate::numformat::NumberFormat;
use crate::token::{FunctionType, Token};
use crate::utils::{bin_to_dec, hex_to_dec};

pub(crate) struct Lexer {
    chars: Vec<char>,
    number_format: NumberFormat,
    position: usize,
    read_position: usize,
    ch: Option<char>,
}

impl Lexer {
    pub(crate) fn new(number_format: NumberFormat) -> Self {
        Self {
            chars: vec![],
            number_format,
            position: 0,
            read_position: 0,
            ch: None,
        }
    }

    pub(crate) fn init(&mut self, input: &str) {
        self.chars = input.chars().collect();
        self.position = 0;
        self.read_position = 0;
        self.ch = None;
        self.read_char();
    }

    fn lookup_token(&mut self, ch: char) -> Result<Token, CalcLibError> {
        match ch {
            '(' => Ok(Token::LParen),
            ')' => Ok(Token::RParen),
            '+' => Ok(Token::Plus),
            '-' => Ok(Token::Minus),
            '*' => Ok(Token::Multiply),
            '/' => Ok(Token::Divide),
            '×' => Ok(Token::Multiply),
            '÷' => Ok(Token::Divide),
            '^' => Ok(Token::Caret),
            '!' => Ok(Token::Exclamation),
            '«' => Ok(Token::Lshift),
            '»' => Ok(Token::Rshift),
            '.' | '0'..='9' => Ok(Token::Number(self.read_number()?)),
            'a'..='z' | 'A'..='Z' => {
                let ident = self.read_ident();
                match ident.as_str() {
                    "log" => Ok(Token::Function(FunctionType::Log)),
                    "logtwo" => Ok(Token::Function(FunctionType::LogTwo)),

                    "ln" => Ok(Token::Function(FunctionType::Ln)),
                    "sqrt" => Ok(Token::Function(FunctionType::SqRt)),
                    "cbrt" => Ok(Token::Function(FunctionType::CbRt)),
                    "abs" => Ok(Token::Function(FunctionType::Abs)),
                    "NOT" => Ok(Token::Function(FunctionType::Not)),
                    "AND" => Ok(Token::And),
                    "OR" => Ok(Token::Or),
                    "NAND" => Ok(Token::Nand),
                    "NOR" => Ok(Token::Nor),
                    "XNOR" => Ok(Token::Xnor),
                    "XOR" => Ok(Token::Xor),
                    "MOD" => Ok(Token::Mod),
                    _ => {
                        if self.is_hexadecimal(&ident) {
                            hex_to_dec(&ident)
                                .map(Token::Number)
                                .map(Ok)
                                .unwrap_or_else(|| {
                                    Err(CalcLibError::HexConversionError(format!(
                                        "Invalid hex identifier: '{}'",
                                        ident.as_str()
                                    )))
                                })
                        } else {
                            Err(CalcLibError::SyntaxError(format!(
                                "Unknown identifier: '{}'",
                                ident.as_str()
                            )))
                        }
                    }
                }
            }
            _ => Err(CalcLibError::SyntaxError(format!("Unknown type: '{}'", ch))),
        }
    }

    pub(crate) fn next_token(&mut self) -> Result<Token, CalcLibError> {
        while matches!(self.ch, Some(' ') | Some('\t')) {
            self.read_char();
        }

        if let Some(ch) = self.ch {
            let token = self.lookup_token(ch);
            self.read_char();

            token
        } else {
            Ok(Token::Eof)
        }
    }

    fn get_string(&self, start: usize, end: usize) -> String {
        let slice: &[char] = &self.chars[start..end];
        slice.iter().collect()
    }

    fn is_hexadecimal(&self, hexadecimal_str: &str) -> bool {
        hexadecimal_str.chars().all(|c| c.is_ascii_hexdigit())
    }

    fn peek_matches(&self, f: impl Fn(char) -> bool) -> bool {
        self.chars.get(self.read_position).copied().is_some_and(f)
    }

    fn peek_is_char(&self) -> bool {
        self.peek_matches(|c| c.is_ascii_alphanumeric())
    }

    fn peek_is_decimal(&self) -> bool {
        self.peek_matches(|c| c.is_ascii_digit())
    }

    fn peek_is_dot(&self) -> bool {
        self.peek_matches(|c| c == '.')
    }

    fn peek_is_binary(&self) -> bool {
        self.peek_matches(|c| c == '0' || c == '1')
    }

    fn peek_is_hexadecimal(&self) -> bool {
        self.peek_matches(|c| c.is_ascii_hexdigit())
    }

    fn read_char(&mut self) {
        if self.read_position >= self.chars.len() {
            self.ch = None;
        } else {
            self.ch = self.chars.get(self.read_position).copied();
        }

        self.position = self.read_position;
        self.read_position += 1;
    }

    fn read_ident(&mut self) -> String {
        let position = self.position;
        while self.ch.is_some() {
            if self.peek_is_char() {
                self.read_char();
            } else {
                break;
            }
        }

        self.get_string(position, self.position + 1)
    }

    fn read_number(&mut self) -> Result<f64, CalcLibError> {
        match self.number_format {
            NumberFormat::Decimal => self.read_decimal(),
            NumberFormat::Hexadecimal => self.read_hexadecimal(),
            NumberFormat::Binary => self.read_binary(),
        }
    }

    fn read_decimal(&mut self) -> Result<f64, CalcLibError> {
        let position = self.position;
        while self.ch.is_some() {
            if self.peek_is_decimal() || self.peek_is_dot() {
                self.read_char();
            } else {
                break;
            }
        }

        let s = self.get_string(position, self.position + 1);

        Ok(s.parse::<f64>()?)
    }

    fn read_hexadecimal(&mut self) -> Result<f64, CalcLibError> {
        let position = self.position;
        while self.ch.is_some() {
            if self.peek_is_hexadecimal() || self.peek_is_dot() {
                self.read_char();
            } else {
                break;
            }
        }

        let s = self.get_string(position, self.position + 1);

        match hex_to_dec(&s) {
            Some(value) => Ok(value),
            None => Err(CalcLibError::HexConversionError(format!(
                "Bad Hex format {}",
                s
            ))),
        }
    }

    fn read_binary(&mut self) -> Result<f64, CalcLibError> {
        let position = self.position;
        while self.ch.is_some() {
            if self.peek_is_binary() || self.peek_is_dot() {
                self.read_char();
            } else {
                break;
            }
        }

        let s = self.get_string(position, self.position + 1);

        match bin_to_dec(&s) {
            Some(value) => Ok(value),
            None => Err(CalcLibError::BinConversionError(format!(
                "Bad Bin format {}",
                s
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::token::FunctionType;

    use super::*;
    #[test]
    fn test_lexer_dec_literal() {
        let input = vec![
            ("5", 5),
            ("42", 42),
            ("9999999", 9999999),
            ("100", 100),
            ("0", 0),
        ];

        for (input, expected) in input {
            let mut l = Lexer::new(NumberFormat::Decimal);
            l.init(input);
            let token = l.next_token().unwrap();
            assert_eq!(token, Token::Number(expected as f64));
        }
    }

    #[test]
    fn test_lexer_hex_literal() {
        let input = vec![
            ("f", 15),
            ("F", 15),
            ("ff", 255),
            ("f8f8f8", 16316664),
            ("2A", 42),
            ("98967f", 9999999),
            ("10", 16),
            ("0", 0),
        ];

        for (input, expected) in input {
            let mut l = Lexer::new(NumberFormat::Hexadecimal);
            l.init(input);
            let token = l.next_token().unwrap();
            assert_eq!(token, Token::Number(expected as f64));
        }
    }

    #[test]
    fn test_lexer_bin_literal() {
        let input = vec![
            ("101", 5),
            ("101010", 42),
            ("11110100001000111111", 999999),
            ("1100100", 100),
            ("0", 0),
        ];

        for (input, expected) in input {
            let mut l = Lexer::new(NumberFormat::Binary);
            l.init(input);
            let token = l.next_token().unwrap();
            assert_eq!(token, Token::Number(expected as f64));
        }
    }

    #[test]
    fn test_lexer_operators() {
        let input = "+-*/()^!";
        let mut l = Lexer::new(NumberFormat::Decimal);
        l.init(input);

        let expected_tokens = vec![
            Token::Plus,
            Token::Minus,
            Token::Multiply,
            Token::Divide,
            Token::LParen,
            Token::RParen,
            Token::Caret,
            Token::Exclamation,
        ];

        for expected in expected_tokens {
            let token = l.next_token().unwrap();
            assert_eq!(token, expected);
        }
    }

    #[test]
    fn test_function_strings() {
        let inputs = vec![
            ("log", Token::Function(FunctionType::Log)),
            ("ln", Token::Function(FunctionType::Ln)),
            ("logtwo", Token::Function(FunctionType::LogTwo)),
            ("sqrt", Token::Function(FunctionType::SqRt)),
            ("cbrt", Token::Function(FunctionType::CbRt)),
            ("abs", Token::Function(FunctionType::Abs)),
            ("NOT", Token::Function(FunctionType::Not)),
        ];

        for (input, expected) in inputs {
            let mut l = Lexer::new(NumberFormat::Decimal);
            l.init(input);
            let token = l.next_token();
            assert_eq!(token, Ok(expected));
        }
    }

    #[test]
    fn test_lexer_invalid_char() {
        let input = "@";
        let mut l = Lexer::new(NumberFormat::Decimal);
        l.init(input);
        let result = l.next_token();
        assert!(result.is_err());
    }

    #[test]
    fn test_lexer_bitwise_keywords() {
        let inputs = vec![
            ("AND", Token::And),
            ("OR", Token::Or),
            ("NAND", Token::Nand),
            ("NOR", Token::Nor),
            ("XOR", Token::Xor),
            ("XNOR", Token::Xnor),
            ("MOD", Token::Mod),
        ];

        for (input, expected) in inputs {
            let mut l = Lexer::new(NumberFormat::Decimal);
            l.init(input);
            assert_eq!(l.next_token(), Ok(expected), "Failed to lex '{}'", input);
        }
    }

    #[test]
    fn test_lexer_shift_operators() {
        let mut l = Lexer::new(NumberFormat::Decimal);
        l.init("«»");
        assert_eq!(l.next_token(), Ok(Token::Lshift));
        assert_eq!(l.next_token(), Ok(Token::Rshift));
    }
}
