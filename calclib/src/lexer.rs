use crate::error::CalcLibError;
use crate::numformat::NumberFormat;
use crate::token::{FunctionType, Token};
use crate::utils::{bin_to_dec, hex_to_dec};

pub(crate) struct Lexer {
    input: String,
    number_format: NumberFormat,
    position: usize,
    read_position: usize,
    ch: Option<char>,
}

impl Lexer {
    pub(crate) fn new(input: String, number_format: NumberFormat) -> Self {
        let mut lexer = Lexer {
            input,
            number_format,
            position: 0,
            read_position: 0,
            ch: None,
        };
        lexer.read_char();
        lexer
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
            '.' | '0'..='9' => {
                let num = self.read_number();

                match num {
                    Ok(value) => Ok(Token::Number(value)),
                    Err(e) => Err(e),
                }
            }
            'a'..='z' | 'A'..='Z' => {
                let ident = self.read_ident();
                match ident.as_str() {
                    "log" => Ok(Token::Function(FunctionType::Log)),
                    "logtwo" => Ok(Token::Function(FunctionType::LogTwo)),

                    "ln" => Ok(Token::Function(FunctionType::Ln)),
                    "sqrt" => Ok(Token::Function(FunctionType::SqRt)),
                    "cbrt" => Ok(Token::Function(FunctionType::CbRt)),
                    "abs" => Ok(Token::Function(FunctionType::Abs)),
                    _ => {
                        if self.is_hexadecimal(&ident) {
                            return Ok(Token::Number(hex_to_dec(&ident).unwrap()));
                        } else {
                            return Err(CalcLibError::SyntaxError(format!(
                                "Unknown identifier: {}",
                                ident
                            )));
                        }
                    }
                }
            }
            _ => Err(CalcLibError::SyntaxError(format!("Unknown type: {}", ch))),
        }
    }

    pub(crate) fn next_token(&mut self) -> Result<Token, CalcLibError> {
        if let Some(ch) = self.ch {
            let token = self.lookup_token(ch);
            self.read_char();

            token
        } else {
            Ok(Token::Eof)
        }
    }

    fn is_hexadecimal(&self, hexadecimal_str: &str) -> bool {
        for ch in hexadecimal_str.chars() {
            if !ch.is_ascii_hexdigit() {
                return false;
            }
        }

        true
    }

    fn peek_is_char(&self) -> bool {
        if self.read_position < self.input.len() {
            return self.input.as_bytes()[self.read_position].is_ascii_alphanumeric();
        }

        false
    }

    fn peek_is_decimal(&self) -> bool {
        if self.read_position < self.input.len() {
            return self.input.as_bytes()[self.read_position].is_ascii_digit();
        }

        false
    }

    fn peek_is_dot(&self) -> bool {
        if self.read_position < self.input.len() {
            return self.input.as_bytes()[self.read_position] == b'.';
        }

        false
    }

    fn peek_is_binary(&self) -> bool {
        if self.read_position < self.input.len() {
            match self.input.as_bytes()[self.read_position] {
                b'0' | b'1' => return true,
                _ => return false,
            }
        }

        false
    }

    fn peek_is_hexadecimal(&self) -> bool {
        if self.read_position < self.input.len() {
            match self.input.as_bytes()[self.read_position] {
                b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' => return true,
                _ => return false,
            }
        }

        false
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = None;
        } else {
            self.ch = Some(self.input.as_bytes()[self.read_position] as char);
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

        self.input[position..self.position + 1].to_string()
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

        let s = self.input[position..self.position + 1].to_string();

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

        let s = &self.input[position..self.position + 1];

        match hex_to_dec(s) {
            Some(value) => Ok(value),
            None => Err(CalcLibError::HexCoversionError(
                "Failed to parse hexadecimal number {s}".to_string(),
            )),
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

        let s = &self.input[position..self.position + 1].to_string();

        match bin_to_dec(s) {
            Some(value) => Ok(value),
            None => Err(CalcLibError::BinCoversionError(
                "Failed to parse binary number {s}".to_string(),
            )),
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

        for i in input {
            let mut l = Lexer::new(i.0.to_string(), NumberFormat::Decimal);
            let token = l.next_token().unwrap();
            let expected_value = i.1 as f64;
            assert_eq!(token, Token::Number(expected_value));
        }
    }

    #[test]
    fn test_lexer_hex_literal() {
        let input = vec![
            // ("f", 15),
            // ("F", 15),
            ("ff", 255),
            ("f8f8f8", 16316664),
            ("2A", 42),
            ("98967f", 9999999),
            ("10", 16),
            ("0", 0),
        ];

        for i in input {
            let mut l = Lexer::new(i.0.to_string(), NumberFormat::Hexadecimal);
            let token = l.next_token().unwrap();
            let expected_value = i.1 as f64;
            assert_eq!(token, Token::Number(expected_value));
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

        for i in input {
            let mut l = Lexer::new(i.0.to_string(), NumberFormat::Binary);
            let token = l.next_token().unwrap();
            let expected_value = i.1 as f64;
            assert_eq!(token, Token::Number(expected_value));
        }
    }

    #[test]
    fn test_lexer_operators() {
        let input = "+-*/()^!";
        let mut l = Lexer::new(input.to_string(), NumberFormat::Decimal);

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
        ];

        for (input, expected) in inputs {
            let mut l = Lexer::new(input.to_string(), NumberFormat::Decimal);
            let token = l.next_token();
            assert_eq!(token, Ok(expected));
        }
    }

    #[test]
    fn test_lexer_invalid_char() {
        let input = "@";
        let mut l = Lexer::new(input.to_string(), NumberFormat::Decimal);
        let result = l.next_token();
        assert!(result.is_err());
    }
}
