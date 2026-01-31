use crate::{lexer::Lexer, token::Token};

pub struct Parser {
    lexer: Lexer,
    curr_token: Option<Token>,
    peek_token: Option<Token>,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}
impl Parser {
    pub(crate) fn new() -> Self {
        Self {
            lexer: Lexer::new("".to_string()),
            curr_token: None,
            peek_token: None,
        }
    }

    // fn next_token(&mut self) -> Result<(), String> {
    fn next_token(&mut self) {
        println!("{:?}", self.curr_token.unwrap_or(Token::Nop));
        self.curr_token = self.peek_token;
        self.peek_token = self.lexer.next_token().unwrap_or(None);
        // println!("curr token: {:?}", self.curr_token.clone().unwrap());
    }

    pub(crate) fn parse(&mut self, input: String) -> Result<Option<Vec<Token>>, String> {
        self.lexer = Lexer::new(input);
        let mut complete = false;
        let mut tokens = Vec::<Token>::new();

        self.next_token();
        self.next_token();

        while self.curr_token.is_some() {
            if self.test_current_token(Token::Nop) {
                self.next_token();
                continue;
            }
            if self.test_current_token(Token::Eof) {
                complete = true;
            }
            tokens.push(self.curr_token.unwrap());
            self.next_token();
        }

        if !complete || tokens.is_empty() {
            return Ok(None);
        }

        println!("Parsed tokens: {:?}", tokens);

        Ok(Some(tokens))
    }

    fn test_current_token(&self, expected: Token) -> bool {
        match &self.curr_token {
            Some(t) => t == &expected,
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_incomplete() {
        let input = vec!["5", "-", "(399", "*", "3-"];
        let mut p = Parser::new();
        for expr in input {
            let result = p.parse(expr.to_string());
            assert_eq!(result, Ok(None));
        }
    }

    #[test]
    fn test_parser_complete() {
        let input = vec![
            (
                "15 + 3 =",
                vec![Token::Number(15), Token::Plus, Token::Number(3), Token::Eof],
            ),
            (
                "42 - 7 * (2 + 3)=",
                vec![
                    Token::Number(42),
                    Token::Minus,
                    Token::Number(7),
                    Token::Multiply,
                    Token::LParen,
                    Token::Number(2),
                    Token::Plus,
                    Token::Number(3),
                    Token::RParen,
                    Token::Eof,
                ],
            ),
        ];

        let mut p = Parser::new();
        for (expr, expected_tokens) in input {
            let result = p.parse(expr.to_string());
            assert_eq!(result, Ok(Some(expected_tokens)));
        }
    }
}
