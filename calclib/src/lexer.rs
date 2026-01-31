use crate::token::Token;
use std::num::ParseIntError;

pub(crate) struct Lexer {
    input: String,
    position: usize,
    read_position: usize,
    ch: Option<char>,
}

impl Lexer {
    pub(crate) fn new(input: String) -> Self {
        let mut lexer = Lexer {
            input,
            position: 0,
            read_position: 0,
            ch: None,
        };
        lexer.read_char();
        lexer
    }

    fn lookup_token(&mut self, ch: char) -> Result<Option<Token>, String> {
        match ch {
            '(' => Ok(Some(Token::LParen)),
            ')' => Ok(Some(Token::RParen)),
            '+' => Ok(Some(Token::Plus)),
            '-' => Ok(Some(Token::Minus)),
            '*' => Ok(Some(Token::Multiply)),
            '/' => Ok(Some(Token::Divide)),
            '×' => Ok(Some(Token::Multiply)),
            '÷' => Ok(Some(Token::Divide)),
            '^' => Ok(Some(Token::Caret)),
            '%' => Ok(Some(Token::Percent)),
            '.' => Ok(Some(Token::Period)),
            '!' => Ok(Some(Token::Exclamation)),
            '=' | '\n' => Ok(Some(Token::Eof)),
            ' ' => Ok(Some(Token::Nop)),
            '0'..='9' => {
                let num = self.read_number();

                match num {
                    Ok(value) => Ok(Some(Token::Number(value))),
                    Err(_) => Err("Failed to parse number".to_string()),
                }
            }
            _ => Err(format!("Unknown type: {}", ch)),
        }
    }

    pub(crate) fn next_token(&mut self) -> Result<Option<Token>, String> {
        if let Some(ch) = self.ch {
            let token = self.lookup_token(ch);
            self.read_char();

            token
        } else {
            Ok(None)
        }
    }

    fn peek_is_digit(&self) -> bool {
        if self.read_position < self.input.len() {
            return self.input.as_bytes()[self.read_position].is_ascii_digit();
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

    fn read_number(&mut self) -> Result<u32, ParseIntError> {
        let position = self.position;
        while self.ch.is_some() {
            if self.peek_is_digit() {
                self.read_char();
            } else {
                break;
            }
        }

        let s = self.input[position..self.position + 1].to_string();

        s.parse::<u32>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_lexer_literal() {
        let input = vec![
            ("5", 5),
            ("42", 42),
            ("9999999", 9999999),
            ("100", 100),
            ("0", 0),
        ];

        for i in input {
            let mut l = Lexer::new(i.0.to_string());
            let token = l.next_token().unwrap().unwrap();
            let expected_value = i.1 as u32;
            assert_eq!(token, Token::Number(expected_value));
        }
    }

    #[test]
    fn test_lexer_operators() {
        let input = "+-*/()%^=!.\n ";
        let mut l = Lexer::new(input.to_string());

        let expected_tokens = vec![
            Token::Plus,
            Token::Minus,
            Token::Multiply,
            Token::Divide,
            Token::LParen,
            Token::RParen,
            Token::Percent,
            Token::Caret,
            Token::Eof,
            Token::Exclamation,
            Token::Period,
            Token::Eof,
            Token::Nop,
        ];

        for expected in expected_tokens {
            let token = l.next_token().unwrap().unwrap();
            assert_eq!(token, expected);
        }
    }

    #[test]
    fn test_lexer_invalid_char() {
        let input = "@";
        let mut l = Lexer::new(input.to_string());
        let result = l.next_token();
        assert!(result.is_err());
    }
}
