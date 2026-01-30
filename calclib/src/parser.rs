use crate::{lexer::Lexer, token::Token};

pub struct Parser {
    lexer: Lexer,
    // curr_token: Token,
    // peek_token: Token,
    errors: Vec<String>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            lexer: Lexer::new("".to_string()),
            errors: Vec::new(),
        }
    }

    pub fn parse(&mut self, input: &str) -> Result<(), String> {
        // Parsing logic will go here
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    #[ignore]
    fn test_parser_literal() {
        let input = "5";
        let mut p = Parser::new();
        let result = p.parse(input);
        assert_eq!(p.errors.len(), 0);
    }
}
