use crate::token::Token;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Expression {
    Number {
        value: f64,
    },
    Infix {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    Prefix {
        operator: Token,
        right: Box<Expression>,
    },
    Unary {
        operator: Token,
        expression: Box<Expression>,
    },
}
