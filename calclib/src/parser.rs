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
    pub fn new() -> Self {
        Self {
            lexer: Lexer::new("".to_string()),
            curr_token: None,
            peek_token: None,
        }
    }

    fn next_token(&mut self) -> Result<(), String> {
        self.curr_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token()?;
        Ok(())
    }

    pub fn parse(&mut self, input: &str) -> Result<Option<f64>, String> {
        self.lexer = Lexer::new(input.to_string());
        self.next_token()?;
        self.next_token()?;

        // Parsing logic would go here

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_incomplete() {
        let input = vec!["5", "+", "(3", "*", "3-"];
        let mut p = Parser::new();
        for expr in input {
            let result = p.parse(expr);
            assert_eq!(result, Ok(None));
        }
    }
}
