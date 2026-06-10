use crate::{
    ast::Expression,
    error::CalcLibError,
    lexer::Lexer,
    numformat::NumberFormat,
    token::{FunctionType, LOWEST, PREFIX, Token},
};

pub struct Parser {
    lexer: Lexer,
    curr_token: Option<Token>,
    peek_token: Option<Token>,
    found_eof: bool,
    lexer_error: Option<CalcLibError>,
}

impl Parser {
    pub(crate) fn new() -> Self {
        Self {
            lexer: Lexer::new(NumberFormat::Decimal),
            curr_token: None,
            peek_token: None,
            found_eof: false,
            lexer_error: None,
        }
    }

    fn next_token(&mut self) {
        self.curr_token = self.peek_token;
        match self.lexer.next_token() {
            Ok(token) => self.peek_token = Some(token),
            Err(e) => {
                self.peek_token = None;
                self.lexer_error = Some(e);
            }
        }
    }

    pub(crate) fn parse(
        &mut self,
        input: &str,
        number_format: NumberFormat,
    ) -> Result<Option<Expression>, CalcLibError> {
        self.lexer = Lexer::new(number_format);
        self.lexer.init(input);
        self.found_eof = false;
        self.next_token();
        self.next_token();

        if self.curr_token.is_none() {
            if let Some(e) = self.lexer_error.take() {
                return Err(e);
            }

            return Ok(None);
        }

        let expression = self.parse_expression(LOWEST);

        if let Some(e) = self.lexer_error.take() {
            return Err(e);
        }

        if !self.found_eof {
            return Ok(None);
        }

        if expression.is_none() {
            return Ok(None);
        }

        Ok(expression)
    }

    fn parse_function(&mut self, func_type: FunctionType) -> Option<Expression> {
        self.next_token();
        let right = self.parse_expression(LOWEST);

        Some(Expression::Function {
            function: func_type,
            argument: Box::new(right?),
        })
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
        let precedence = op.precedence();
        self.next_token();
        let right = self.parse_expression(precedence);

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

    fn parse_unary(&mut self, left: Option<Expression>) -> Option<Expression> {
        let op = self.curr_token?;

        Some(Expression::Unary {
            operator: op,
            expression: Box::new(left?),
        })
    }

    fn parse_expression(&mut self, precedence: u8) -> Option<Expression> {
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
            Some(Token::Number(value)) => Some(Expression::Number { value: *value }),
            Some(Token::Function(func_type)) => self.parse_function(*func_type),
            _ => return None,
        };

        while precedence < self.peek_precedence() {
            self.next_token();

            match &self.curr_token {
                Some(Token::Eof) => {
                    self.found_eof = true;
                    break;
                }
                Some(Token::Exclamation) => {
                    left = self.parse_unary(left);
                }
                _ => left = self.parse_infix(left),
            };
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
    use core::f64;

    use super::*;

    #[test]
    fn test_parser_empty() {
        let input = vec![""];
        let mut p = Parser::new();
        for expr in input {
            let result = p.parse(expr, NumberFormat::Decimal);
            assert_eq!(result, Ok(None));
        }
    }

    #[test]
    fn test_simple_literals() {
        let input = vec![
            ("5", 5.0),
            ("42", 42.0),
            ("0", 0.0),
            ("1234567890", 1234567890.0),
        ];
        let mut p = Parser::new();
        for (input, expected) in input {
            let result = p.parse(input, NumberFormat::Decimal);

            assert_eq!(result, Ok(Some(Expression::Number { value: expected })));
        }
    }

    #[test]
    fn test_simple_negative_literals() {
        let input = vec![("-5", 5.0), ("-42", 42.0), ("-1234567890", 1234567890.0)];
        let mut p = Parser::new();
        for (input, expected) in input {
            let result = p.parse(input, NumberFormat::Decimal);

            assert_eq!(
                result,
                Ok(Some(Expression::Prefix {
                    operator: Token::Minus,
                    right: Box::new(Expression::Number { value: expected })
                }))
            );
        }
    }

    #[test]
    fn test_parser_incomplete() {
        let input = vec!["-", "(399", "*", "3-", "-5+"];
        let mut p = Parser::new();
        for expr in input {
            let result = p.parse(expr, NumberFormat::Decimal);
            assert_eq!(result, Ok(None));
        }
    }

    #[test]
    fn test_parser_complete_simple_expression() {
        let input: Vec<(&str, Result<Option<Expression>, CalcLibError>)> = vec![
            (
                "15+3",
                Ok(Some(Expression::Infix {
                    left: Box::new(Expression::Number { value: 15.0 }),

                    operator: Token::Plus,
                    right: Box::new(Expression::Number { value: 3.0 }),
                })),
            ),
            (
                "15-3",
                Ok(Some(Expression::Infix {
                    left: Box::new(Expression::Number { value: 15.0 }),
                    operator: Token::Minus,
                    right: Box::new(Expression::Number { value: 3.0 }),
                })),
            ),
            (
                "15*3",
                Ok(Some(Expression::Infix {
                    left: Box::new(Expression::Number { value: 15.0 }),
                    operator: Token::Multiply,
                    right: Box::new(Expression::Number { value: 3.0 }),
                })),
            ),
            (
                "15/3",
                Ok(Some(Expression::Infix {
                    left: Box::new(Expression::Number { value: 15.0 }),
                    operator: Token::Divide,
                    right: Box::new(Expression::Number { value: 3.0 }),
                })),
            ),
        ];

        let mut p = Parser::new();
        for (input, expected) in input {
            let result = p.parse(input, NumberFormat::Decimal);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_parse_unary_expressions() {
        let input: Vec<(&str, Result<Option<Expression>, CalcLibError>)> = vec![(
            "3!",
            Ok(Some(Expression::Unary {
                operator: Token::Exclamation,
                expression: Box::new(Expression::Number { value: 3.0 }),
            })),
        )];

        let mut p = Parser::new();
        for (input, expected) in input {
            let result = p.parse(input, NumberFormat::Decimal);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_parser_complete_complex_expressions() {
        let input: Vec<(&str, Result<Option<Expression>, CalcLibError>)> = vec![
            (
                "5*(3-1)",
                Ok(Some(Expression::Infix {
                    left: Box::new(Expression::Number { value: 5.0 }),
                    operator: Token::Multiply,
                    right: Box::new(Expression::Infix {
                        left: Box::new(Expression::Number { value: 3.0 }),

                        operator: Token::Minus,
                        right: Box::new(Expression::Number { value: 1.0 }),
                    }),
                })),
            ),
            (
                "5(3-1)",
                Ok(Some(Expression::Infix {
                    left: Box::new(Expression::Number { value: 5.0 }),
                    operator: Token::Multiply,
                    right: Box::new(Expression::Infix {
                        left: Box::new(Expression::Number { value: 3.0 }),
                        operator: Token::Minus,
                        right: Box::new(Expression::Number { value: 1.0 }),
                    }),
                })),
            ),
            (
                "5*(3-1*4+8)/2",
                Ok(Some(Expression::Infix {
                    left: Box::new(Expression::Infix {
                        left: Box::new(Expression::Number { value: 5.0 }),
                        operator: Token::Multiply,
                        right: Box::new(Expression::Infix {
                            left: Box::new(Expression::Infix {
                                left: Box::new(Expression::Number { value: 3.0 }),
                                operator: Token::Minus,
                                right: Box::new(Expression::Infix {
                                    left: Box::new(Expression::Number { value: 1.0 }),
                                    operator: Token::Multiply,
                                    right: Box::new(Expression::Number { value: 4.0 }),
                                }),
                            }),
                            operator: Token::Plus,
                            right: Box::new(Expression::Number { value: 8.0 }),
                        }),
                    }),
                    operator: Token::Divide,
                    right: Box::new(Expression::Number { value: 2.0 }),
                })),
            ),
            (
                "42-7*(2+3)",
                Ok(Some(Expression::Infix {
                    left: Box::new(Expression::Number { value: 42.0 }),
                    operator: Token::Minus,
                    right: Box::new(Expression::Infix {
                        left: Box::new(Expression::Number { value: 7.0 }),
                        operator: Token::Multiply,
                        right: Box::new(Expression::Infix {
                            left: Box::new(Expression::Number { value: 2.0 }),
                            operator: Token::Plus,
                            right: Box::new(Expression::Number { value: 3.0 }),
                        }),
                    }),
                })),
            ),
        ];

        let mut p = Parser::new();
        for (input, expected) in input {
            let result = p.parse(input, NumberFormat::Decimal);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_functions() {
        let input: Vec<(&str, Result<Option<Expression>, CalcLibError>)> = vec![
            (
                "log(100)",
                Ok(Some(Expression::Function {
                    function: FunctionType::Log,
                    argument: Box::new(Expression::Number { value: 100.0 }),
                })),
            ),
            (
                "ln(2.718281828459045)",
                Ok(Some(Expression::Function {
                    function: FunctionType::Ln,
                    argument: Box::new(Expression::Number {
                        value: f64::consts::E,
                    }),
                })),
            ),
            (
                "sqrt(16)",
                Ok(Some(Expression::Function {
                    function: FunctionType::SqRt,
                    argument: Box::new(Expression::Number { value: 16.0 }),
                })),
            ),
            (
                "NOT(3)",
                Ok(Some(Expression::Function {
                    function: FunctionType::Not,
                    argument: Box::new(Expression::Number { value: 3.0 }),
                })),
            ),
        ];

        let mut p = Parser::new();
        for (input, expected) in input {
            let result = p.parse(input, NumberFormat::Decimal);
            assert_eq!(result, expected);
        }
    }
}
