use crate::token::{Token, lookup_token};

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

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = None;
        } else {
            self.ch = Some(self.input.as_bytes()[self.read_position] as char);
        }

        self.position = self.read_position;
        self.read_position += 1;
    }

    pub(crate) fn next_token(&mut self) -> Result<Option<Token>, String> {
        if let Some(ch) = self.ch {
            let token = lookup_token(ch);
            self.read_char();

            token
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_lexer_literal() {
        let input = "5";
        let mut l = Lexer::new(input.to_string());
        let token = l.next_token().unwrap().unwrap();
        assert_eq!(token, Token::Number(5));

        let input = "42";
        let mut l = Lexer::new(input.to_string());
        let token = l.next_token().unwrap().unwrap();
        assert_eq!(token, Token::Number(4));
        let token = l.next_token().unwrap().unwrap();
        assert_eq!(token, Token::Number(2));
    }

    #[test]
    fn test_lexer_operators() {
        let input = "+-*/()%^=!.";
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
            Token::Equal,
            Token::Exclamation,
            Token::Period,
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
