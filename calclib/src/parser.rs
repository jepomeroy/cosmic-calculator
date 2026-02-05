use crate::{
    ast::Expression,
    lexer::Lexer,
    token::{LOWEST, PREFIX, Token},
};

pub struct Parser {
    lexer: Lexer,
    curr_token: Option<Token>,
    peek_token: Option<Token>,
    found_eof: bool,
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
            found_eof: false,
        }
    }

    fn next_token(&mut self) {
        self.curr_token = self.peek_token;
        self.peek_token = self.lexer.next_token().ok();
    }

    pub(crate) fn parse(&mut self, input: String) -> Result<Option<Expression>, String> {
        self.lexer = Lexer::new(input);
        self.found_eof = false;
        self.next_token();
        self.next_token();

        if self.curr_token.is_none() {
            return Ok(None);
        }

        let expression = self.parse_expression(LOWEST);

        if !self.found_eof {
            return Ok(None);
        }

        if expression.is_none() {
            return Ok(None);
        }

        Ok(expression)
    }

    fn parse_infix(&mut self, left: Option<Expression>) -> Option<Expression> {
        // Handle implicit multiplication: 5(3-1) -> 5 * (3-1)
        if self.curr_token == Some(Token::LParen) {
            self.next_token();
            let right = self.parse_expression(LOWEST);
            self.next_token();

            if !self.test_current_token(Token::RParen) {
                return None;
            }

            return Some(Expression::Infix {
                left: Box::new(left?),
                operator: Token::Multiply,
                right: Box::new(right?),
            });
        }

        let op = self.curr_token?;
        let precedense = op.precedence();
        self.next_token();
        let right = self.parse_expression(precedense);

        Some(Expression::Infix {
            left: Box::new(left?),
            operator: op,
            right: Box::new(right?),
        })
    }

    fn parse_prefix(&mut self) -> Option<Expression> {
        let op = self.curr_token?;
        self.next_token();
        let right = self.parse_expression(PREFIX);

        Some(Expression::Prefix {
            operator: op,
            right: Box::new(right?),
        })
    }

    fn parse_expression(&mut self, precedense: u8) -> Option<Expression> {
        let mut left = match &self.curr_token {
            Some(Token::Eof) => return None,
            Some(Token::Minus) => self.parse_prefix(),
            Some(Token::LParen) => {
                self.next_token();
                let expr = self.parse_expression(LOWEST);
                self.next_token();

                if !self.test_current_token(Token::RParen) {
                    return None;
                }

                expr
            }
            Some(Token::Number(value)) => Some(Expression::Integer { value: *value }),
            _ => return None,
        };

        while precedense < self.peek_precedence() {
            self.next_token();

            if self.curr_token == Some(Token::Eof) {
                self.found_eof = true;
                break;
            };

            left = self.parse_infix(left)
        }

        left
    }

    fn peek_precedence(&mut self) -> u8 {
        match &self.peek_token {
            Some(token) => token.precedence(),
            None => LOWEST,
        }
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
    fn test_parser_empty() {
        let input = vec![""];
        let mut p = Parser::new();
        for expr in input {
            let result = p.parse(expr.to_string());
            assert_eq!(result, Ok(None));
        }
    }

    #[test]
    fn test_simple_literals() {
        let input = vec![("5", 5), ("42", 42), ("0", 0), ("1234567890", 1234567890)];
        let mut p = Parser::new();
        for expr in input {
            let result = p.parse(expr.0.to_string());

            assert_eq!(result, Ok(Some(Expression::Integer { value: expr.1 })));
        }
    }

    #[test]
    fn test_simple_negative_literals() {
        let input = vec![("-5", 5), ("-42", 42), ("-1234567890", 1234567890)];
        let mut p = Parser::new();
        for expr in input {
            let result = p.parse(expr.0.to_string());

            // println!("Result for '{}': {:?}", expr.0, result);

            assert_eq!(
                result,
                Ok(Some(Expression::Prefix {
                    operator: Token::Minus,
                    right: Box::new(Expression::Integer { value: expr.1 })
                }))
            );
        }
    }

    #[test]
    fn test_parser_incomplete() {
        let input = vec!["-", "(399", "*", "3-", "-5+"];
        let mut p = Parser::new();
        for expr in input {
            let result = p.parse(expr.to_string());
            assert_eq!(result, Ok(None));
        }
    }

    #[test]
    fn test_parser_complete_simple_expression() {
        let input: Vec<(&str, Result<Option<Expression>, String>)> = vec![
            (
                "15+3",
                Ok(Some(Expression::Infix {
                    left: Box::new(Expression::Integer { value: 15 }),
                    operator: Token::Plus,
                    right: Box::new(Expression::Integer { value: 3 }),
                })),
            ),
            (
                "15-3",
                Ok(Some(Expression::Infix {
                    left: Box::new(Expression::Integer { value: 15 }),
                    operator: Token::Minus,
                    right: Box::new(Expression::Integer { value: 3 }),
                })),
            ),
            (
                "15*3",
                Ok(Some(Expression::Infix {
                    left: Box::new(Expression::Integer { value: 15 }),
                    operator: Token::Multiply,
                    right: Box::new(Expression::Integer { value: 3 }),
                })),
            ),
            (
                "15/3",
                Ok(Some(Expression::Infix {
                    left: Box::new(Expression::Integer { value: 15 }),
                    operator: Token::Divide,
                    right: Box::new(Expression::Integer { value: 3 }),
                })),
            ),
        ];

        let mut p = Parser::new();
        for (expr, expected_tokens) in input {
            let result = p.parse(expr.to_string());
            assert_eq!(result, expected_tokens);
        }
    }

    #[test]
    fn test_parser_complete_complex_expressions() {
        let input: Vec<(&str, Result<Option<Expression>, String>)> = vec![
            (
                "5*(3-1)",
                Ok(Some(Expression::Infix {
                    left: Box::new(Expression::Integer { value: 5 }),
                    operator: Token::Multiply,
                    right: Box::new(Expression::Infix {
                        left: Box::new(Expression::Integer { value: 3 }),
                        operator: Token::Minus,
                        right: Box::new(Expression::Integer { value: 1 }),
                    }),
                })),
            ),
            (
                "5(3-1)",
                Ok(Some(Expression::Infix {
                    left: Box::new(Expression::Integer { value: 5 }),
                    operator: Token::Multiply,
                    right: Box::new(Expression::Infix {
                        left: Box::new(Expression::Integer { value: 3 }),
                        operator: Token::Minus,
                        right: Box::new(Expression::Integer { value: 1 }),
                    }),
                })),
            ),
            (
                "5*(3-1*4+8)/2",
                Ok(Some(Expression::Infix {
                    left: Box::new(Expression::Infix {
                        left: Box::new(Expression::Integer { value: 5 }),
                        operator: Token::Multiply,
                        right: Box::new(Expression::Infix {
                            left: Box::new(Expression::Infix {
                                left: Box::new(Expression::Integer { value: 3 }),
                                operator: Token::Minus,
                                right: Box::new(Expression::Infix {
                                    left: Box::new(Expression::Integer { value: 1 }),
                                    operator: Token::Multiply,
                                    right: Box::new(Expression::Integer { value: 4 }),
                                }),
                            }),
                            operator: Token::Plus,
                            right: Box::new(Expression::Integer { value: 8 }),
                        }),
                    }),
                    operator: Token::Divide,
                    right: Box::new(Expression::Integer { value: 2 }),
                })),
            ),
            (
                "42-7*(2+3)",
                Ok(Some(Expression::Infix {
                    left: Box::new(Expression::Integer { value: 42 }),
                    operator: Token::Minus,
                    right: Box::new(Expression::Infix {
                        left: Box::new(Expression::Integer { value: 7 }),
                        operator: Token::Multiply,
                        right: Box::new(Expression::Infix {
                            left: Box::new(Expression::Integer { value: 2 }),
                            operator: Token::Plus,
                            right: Box::new(Expression::Integer { value: 3 }),
                        }),
                    }),
                })),
            ),
        ];

        let mut p = Parser::new();
        for (expr, expected_tokens) in input {
            let result = p.parse(expr.to_string());
            assert_eq!(result, expected_tokens);
        }
    }
}
